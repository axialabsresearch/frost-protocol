use frost_protocol::{
    state::{
        cache::{ProofCache, CacheConfig, EvictionPolicy},
        proof::{StateProof, VerificationResult, ProofData, ProofType},
        transition::StateTransition,
        types::BlockId,
        ChainId,
    },
};

use std::time::{SystemTime, Duration};
use serde_json::json;
use tokio::time::timeout;

fn dummy_state_proof(chain: &str, block: u64) -> StateProof {
    // Create transition with proper BlockId parameters
    let chain_id = ChainId::new(chain);
    let transition = StateTransition::new(
        chain_id.clone(),
        BlockId::Number(block),
        BlockId::Number(block + 1),
        vec![1, 2, 3, 4],
    );

    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: vec![1, 2, 3, 4],
        metadata: Some(json!({
            "chain_id": chain,
            "block": block,
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    StateProof::new(transition, proof_data)
}

fn dummy_verification_result() -> VerificationResult {
    VerificationResult {
        success: true,
        verified_at: SystemTime::now(),
        params: Default::default(),
        error: None,
    }
}

#[tokio::test]
async fn test_cache_initialization() {
    let config = CacheConfig::default();
    let cache = ProofCache::new(config);
    
    // Verify initial cache state
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 0, "Cache should start empty");
    assert_eq!(stats.total_size_bytes, 0, "Cache should start with zero size");
    assert_eq!(stats.hit_count, 0, "Cache should have no hits initially");
    assert_eq!(stats.miss_count, 0, "Cache should have no misses initially");
    assert_eq!(stats.eviction_count, 0, "Cache should have no evictions initially");
    
    // Verify cache configuration
    assert!(cache.max_entries() > 0, "Cache should have positive capacity");
    assert!(!cache.is_full(), "New cache should not be full");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_metrics() {
    timeout(Duration::from_secs(5), async {
        let mut config = CacheConfig::default();
        config.max_entries = 2; // Small cache for testing eviction
        config.enable_metrics = false; // Disable metrics to avoid race conditions
        let cache = ProofCache::new(config);

        // Add some entries
        let proof1 = dummy_state_proof("ethereum", 1000);
        let proof2 = dummy_state_proof("ethereum", 1001);
        let result = dummy_verification_result();

        // Test cache hits and misses
        assert!(cache.get(&proof1).is_none(), "Should miss on empty cache");
        cache.put(&proof1, result.clone()).unwrap();
        
        // Verify first entry
        assert!(cache.get(&proof1).is_some(), "Should hit after insertion");
        assert!(cache.get(&proof2).is_none(), "Should miss on non-existent entry");

        // Test eviction
        let proof3 = dummy_state_proof("ethereum", 1002);
        cache.put(&proof2, result.clone()).unwrap();
        
        // Verify before final insertion
        assert!(cache.get(&proof1).is_some(), "First entry should still exist");
        assert!(cache.get(&proof2).is_some(), "Second entry should exist");
        
        // Add third entry to trigger eviction
        cache.put(&proof3, result).unwrap(); // Should evict proof1

        // Verify final state
        assert!(cache.get(&proof1).is_none(), "First entry should be evicted");
        assert!(cache.get(&proof2).is_some(), "Second entry should remain");
        assert!(cache.get(&proof3).is_some(), "Third entry should be present");

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2, "Should maintain max entries limit");
    }).await.expect("Test timed out");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_eviction_policies() {
    timeout(Duration::from_secs(5), async {
        // Test LRU eviction
        let mut config = CacheConfig::default();
        config.max_entries = 2;
        config.eviction_policy = EvictionPolicy::LRU;
        config.enable_metrics = false; // Disable metrics to avoid race conditions
        let cache = ProofCache::new(config);

        let proof1 = dummy_state_proof("ethereum", 1000);
        let proof2 = dummy_state_proof("ethereum", 1001);
        let proof3 = dummy_state_proof("ethereum", 1002);
        let result = dummy_verification_result();

        // Add initial entries
        cache.put(&proof1, result.clone()).unwrap();
        cache.put(&proof2, result.clone()).unwrap();
        
        // Verify initial state
        assert!(cache.get(&proof1).is_some(), "First entry should exist");
        assert!(cache.get(&proof2).is_some(), "Second entry should exist");
        
        // Access proof1 to make it more recently used
        cache.get(&proof1);
        
        // Add proof3, should evict proof2 (least recently used)
        cache.put(&proof3, result).unwrap();
        
        // Verify final state
        assert!(cache.get(&proof1).is_some(), "Most recently used entry should remain");
        assert!(cache.get(&proof2).is_none(), "Least recently used entry should be evicted");
        assert!(cache.get(&proof3).is_some(), "New entry should be present");

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2, "Should maintain max entries limit");
    }).await.expect("Test timed out");
}

#[tokio::test]
async fn test_cache_size_limits() {
    let mut config = CacheConfig::default();
    config.max_size_bytes = 1024; // Small size limit
    config.enable_metrics = false; // Disable metrics to avoid race conditions
    let cache = ProofCache::new(config);

    let proof = dummy_state_proof("ethereum", 1000);
    let result = dummy_verification_result();

    cache.put(&proof, result.clone()).unwrap();
    assert!(!cache.is_full(), "Cache should not be full with small entry");
    
    let stats = cache.stats();
    assert!(stats.total_size_bytes > 0, "Cache should track entry size");
    assert!(stats.total_size_bytes < cache.max_size(), "Entry should fit within limit");
}