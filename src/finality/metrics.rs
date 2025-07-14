use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;

/// Core metrics for finality verification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinalityMetrics {
    /// Block verification metrics
    pub verification: VerificationMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Resource metrics
    pub resources: ResourceMetrics,
    /// Chain-specific metrics
    pub chain_metrics: HashMap<String, ChainMetrics>,
}

/// Verification-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerificationMetrics {
    /// Total blocks verified
    pub total_blocks_verified: u64,
    /// Failed verifications
    pub failed_verifications: u64,
    /// Average verification time
    pub avg_verification_time: f64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Rate limit hits
    pub rate_limit_hits: u64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Message processing latency
    pub message_latency: LatencyMetrics,
    /// State sync performance
    pub state_sync: StateSyncMetrics,
}

/// Latency metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyMetrics {
    /// Average latency
    pub average_ms: f64,
    /// Percentile latencies (key = percentile)
    pub percentiles: HashMap<u8, f64>,
    /// Maximum latency
    pub max_ms: f64,
    /// Minimum latency
    pub min_ms: f64,
}

/// State sync metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateSyncMetrics {
    /// Sync duration
    pub sync_duration_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Conflicts resolved
    pub conflicts_resolved: u64,
    /// Bytes synced
    pub bytes_synced: u64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage MB
    pub memory_usage: f64,
    /// Network bandwidth Mbps
    pub network_bandwidth: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
}

/// Chain-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChainMetrics {
    /// Average block time
    pub avg_block_time: f64,
    /// Average finality time
    pub avg_finality_time: f64,
    /// Total blocks processed
    pub total_blocks: u64,
    /// Total messages processed
    pub total_messages: u64,
    /// Failed message count
    pub failed_messages: u64,
    /// Average message size
    pub avg_message_size: f64,
    /// Chain-specific data
    pub chain_data: serde_json::Value,
}

impl FinalityMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record verification attempt
    pub fn record_verification(&mut self, chain_id: &str, success: bool, duration: Duration) {
        self.verification.total_blocks_verified += 1;
        if !success {
            self.verification.failed_verifications += 1;
        }

        // Update average verification time
        let duration_ms = duration.as_secs_f64() * 1000.0;
        self.verification.avg_verification_time = (
            self.verification.avg_verification_time * (self.verification.total_blocks_verified - 1) as f64
            + duration_ms
        ) / self.verification.total_blocks_verified as f64;

        // Update chain-specific metrics
        let chain_metrics = self.chain_metrics.entry(chain_id.to_string()).or_default();
        chain_metrics.total_blocks += 1;
        chain_metrics.avg_finality_time = (
            chain_metrics.avg_finality_time * (chain_metrics.total_blocks - 1) as f64
            + duration_ms
        ) / chain_metrics.total_blocks as f64;
    }

    /// Record cache operation
    pub fn record_cache_operation(&mut self, hit: bool) {
        if hit {
            self.verification.cache_hits += 1;
        } else {
            self.verification.cache_misses += 1;
        }
    }

    /// Record rate limit hit
    pub fn record_rate_limit_hit(&mut self) {
        self.verification.rate_limit_hits += 1;
    }

    /// Record circuit breaker trip
    pub fn record_circuit_breaker_trip(&mut self) {
        self.verification.circuit_breaker_trips += 1;
    }

    /// Record message metrics
    pub fn record_message(
        &mut self,
        chain_id: &str,
        size: usize,
        success: bool,
        latency: Duration,
    ) {
        let chain_metrics = self.chain_metrics.entry(chain_id.to_string()).or_default();
        chain_metrics.total_messages += 1;
        if !success {
            chain_metrics.failed_messages += 1;
        }

        // Update average message size
        chain_metrics.avg_message_size = (
            chain_metrics.avg_message_size * (chain_metrics.total_messages - 1) as f64
            + size as f64
        ) / chain_metrics.total_messages as f64;

        // Update latency metrics
        let latency_ms = latency.as_secs_f64() * 1000.0;
        let latency = &mut self.performance.message_latency;
        latency.average_ms = (
            latency.average_ms * (chain_metrics.total_messages - 1) as f64
            + latency_ms
        ) / chain_metrics.total_messages as f64;
        latency.max_ms = latency.max_ms.max(latency_ms);
        latency.min_ms = if latency.min_ms == 0.0 {
            latency_ms
        } else {
            latency.min_ms.min(latency_ms)
        };
    }

    /// Update resource metrics
    pub fn update_resource_metrics(
        &mut self,
        cpu: f64,
        memory: f64,
        bandwidth: f64,
        disk: f64,
    ) {
        let resources = &mut self.resources;
        resources.cpu_usage = cpu;
        resources.memory_usage = memory;
        resources.network_bandwidth = bandwidth;
        resources.disk_usage = disk;
    }
} 