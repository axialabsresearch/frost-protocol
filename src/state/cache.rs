use std::time::{Duration, SystemTime};
use std::sync::Arc;
use dashmap::DashMap;
use metrics::{counter, gauge, histogram};
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{
    proof::{StateProof, VerificationResult},
    error::{ProofError, ProofErrorCategory},
};

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// When the entry was created
    pub created_at: SystemTime,
    /// When the entry was last accessed
    pub last_accessed: SystemTime,
    /// Number of times the entry was accessed
    pub access_count: u64,
    /// Size of the cached data in bytes
    pub size_bytes: usize,
}

/// Cache entry with value and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// Entry metadata
    pub metadata: CacheMetadata,
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least recently used
    LRU,
    /// Least frequently used
    LFU,
    /// Time-to-live based
    TTL(Duration),
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum total size in bytes
    pub max_size_bytes: usize,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Whether to collect metrics
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            eviction_policy: EvictionPolicy::LRU,
            enable_metrics: true,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in cache
    pub total_entries: usize,
    /// Total size of cached data
    pub total_size_bytes: usize,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
    /// Number of evicted entries
    pub eviction_count: u64,
    /// Cache configuration
    pub config: CacheConfig,
}

/// Advanced cache implementation with metrics
pub struct ProofCache {
    /// Memory cache
    memory_cache: DashMap<String, CacheEntry<VerificationResult>>,
    /// Cache configuration
    config: CacheConfig,
    /// Total size of cached data
    total_size: Arc<std::sync::atomic::AtomicUsize>,
    /// Cache hit counter
    hit_count: Arc<AtomicU64>,
    /// Cache miss counter
    miss_count: Arc<AtomicU64>,
    /// Cache eviction counter
    eviction_count: Arc<AtomicU64>,
}

impl ProofCache {
    /// Create new proof cache
    pub fn new(config: CacheConfig) -> Self {
        let cache = Self {
            memory_cache: DashMap::new(),
            config: config.clone(),
            total_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
            eviction_count: Arc::new(AtomicU64::new(0)),
        };

        // Initialize metrics if enabled
        if config.enable_metrics {
            gauge!("proof_cache.total_entries", 0.0);
            gauge!("proof_cache.total_size_bytes", 0.0);
        }

        cache
    }

    /// Get cache key for proof
    fn cache_key(proof: &StateProof) -> String {
        format!("{:?}:{:?}", proof.transition, proof.proof)
    }

    /// Get cached verification result
    pub fn get(&self, proof: &StateProof) -> Option<VerificationResult> {
        let key = Self::cache_key(proof);
        
        // First check if entry exists and get a clone if it does
        let entry = self.memory_cache.get(&key)?.value().clone();
        
        // Update metrics
        if self.config.enable_metrics {
            self.hit_count.fetch_add(1, Ordering::SeqCst);
        }

        // Update access metadata in a separate operation
        let mut updated = entry.clone();
        updated.metadata.last_accessed = SystemTime::now();
        updated.metadata.access_count += 1;
        self.memory_cache.insert(key, updated);

        Some(entry.value)
    }

    /// Cache verification result
    pub fn put(
        &self,
        proof: &StateProof,
        result: VerificationResult,
    ) -> Result<(), ProofError> {
        let key = Self::cache_key(proof);
        let size = std::mem::size_of_val(&result);

        // Check size limits
        if size > self.config.max_size_bytes {
            return Err(ProofError::new(
                ProofErrorCategory::Cache,
                super::error::ErrorSeverity::Warning,
                "Cache entry too large",
            ));
        }

        // Create cache entry first
        let entry = CacheEntry {
            value: result,
            metadata: CacheMetadata {
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
                access_count: 0,
                size_bytes: size,
            },
        };

        // Evict entries if needed
        self.evict_if_needed(size);

        // Update cache
        self.memory_cache.insert(key, entry);
        self.total_size.fetch_add(size, std::sync::atomic::Ordering::SeqCst);

        // Update metrics
        if self.config.enable_metrics {
            gauge!("proof_cache.total_entries", self.memory_cache.len() as f64);
            gauge!("proof_cache.total_size_bytes", self.total_size.load(std::sync::atomic::Ordering::SeqCst) as f64);
        }

        Ok(())
    }

    /// Evict entries based on policy
    fn evict_if_needed(&self, new_entry_size: usize) {
        let current_size = self.total_size.load(std::sync::atomic::Ordering::SeqCst);
        let current_entries = self.memory_cache.len();

        // Check if eviction needed
        if current_entries >= self.config.max_entries ||
           current_size + new_entry_size > self.config.max_size_bytes {
            match self.config.eviction_policy {
                EvictionPolicy::LRU => self.evict_lru(),
                EvictionPolicy::LFU => self.evict_lfu(),
                EvictionPolicy::TTL(ttl) => self.evict_expired(ttl),
            }
        }
    }

    /// Evict least recently used entries
    fn evict_lru(&self) {
        // Collect all entries info first to avoid iterator deadlock
        let entries: Vec<_> = self.memory_cache
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().metadata.last_accessed,
                    entry.value().metadata.size_bytes
                )
            })
            .collect();

        // Find the least recently used entry
        if let Some((key, _, size)) = entries
            .iter()
            .min_by_key(|(_, last_accessed, _)| last_accessed)
            .cloned()
        {
            // Remove entry and update size
            self.memory_cache.remove(&key);
            self.total_size.fetch_sub(size, std::sync::atomic::Ordering::SeqCst);
            
            if self.config.enable_metrics {
                self.eviction_count.fetch_add(1, Ordering::SeqCst);
                gauge!("proof_cache.total_entries", self.memory_cache.len() as f64);
                gauge!("proof_cache.total_size_bytes", self.total_size.load(std::sync::atomic::Ordering::SeqCst) as f64);
            }
        }
    }

    fn evict_lfu(&self) {
        // Collect all entries info first to avoid iterator deadlock
        let entries: Vec<_> = self.memory_cache
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().metadata.access_count,
                    entry.value().metadata.size_bytes
                )
            })
            .collect();

        // Find the least frequently used entry
        if let Some((key, _, size)) = entries
            .iter()
            .min_by_key(|(_, access_count, _)| access_count)
            .cloned()
        {
            // Remove entry and update size
            self.memory_cache.remove(&key);
            self.total_size.fetch_sub(size, std::sync::atomic::Ordering::SeqCst);
            
            if self.config.enable_metrics {
                self.eviction_count.fetch_add(1, Ordering::SeqCst);
                gauge!("proof_cache.total_entries", self.memory_cache.len() as f64);
                gauge!("proof_cache.total_size_bytes", self.total_size.load(std::sync::atomic::Ordering::SeqCst) as f64);
            }
        }
    }

    /// Evict expired entries
    fn evict_expired(&self, ttl: Duration) {
        let now = SystemTime::now();
        
        // Collect all expired entries info first to avoid iterator deadlock
        let expired: Vec<_> = self.memory_cache
            .iter()
            .filter_map(|entry| {
                let is_expired = now.duration_since(entry.value().metadata.created_at)
                    .map(|age| age > ttl)
                    .unwrap_or(true);
                
                if is_expired {
                    Some((entry.key().clone(), entry.value().metadata.size_bytes))
                } else {
                    None
                }
            })
            .collect();

        // Check if we have any expired entries
        let has_expired = !expired.is_empty();

        // Remove expired entries
        for (key, size) in expired {
            self.memory_cache.remove(&key);
            self.total_size.fetch_sub(size, std::sync::atomic::Ordering::SeqCst);
            
            if self.config.enable_metrics {
                self.eviction_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Update metrics once after all evictions
        if self.config.enable_metrics && has_expired {
            gauge!("proof_cache.total_entries", self.memory_cache.len() as f64);
            gauge!("proof_cache.total_size_bytes", self.total_size.load(std::sync::atomic::Ordering::SeqCst) as f64);
        }
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.memory_cache.clear();
        self.total_size.store(0, std::sync::atomic::Ordering::SeqCst);

        if self.config.enable_metrics {
            gauge!("proof_cache.total_entries", 0.0);
            gauge!("proof_cache.total_size_bytes", 0.0);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.memory_cache.len(),
            total_size_bytes: self.total_size.load(std::sync::atomic::Ordering::SeqCst),
            hit_count: self.hit_count.load(Ordering::SeqCst),
            miss_count: self.miss_count.load(Ordering::SeqCst),
            eviction_count: self.eviction_count.load(Ordering::SeqCst),
            config: self.config.clone(),
        }
    }

    /// Check if cache is full (reached max entries or size)
    pub fn is_full(&self) -> bool {
        let stats = self.stats();
        stats.total_entries >= stats.config.max_entries ||
        stats.total_size_bytes >= stats.config.max_size_bytes
    }

    /// Get maximum number of entries allowed
    pub fn max_entries(&self) -> usize {
        self.config.max_entries
    }

    /// Get maximum size in bytes allowed
    pub fn max_size(&self) -> usize {
        self.config.max_size_bytes
    }

    /// Get cache hit rate (between 0.0 and 1.0)
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats();
        let total = stats.hit_count + stats.miss_count;
        if total == 0 { 0.0 } else { stats.hit_count as f64 / total as f64 }
    }
} 