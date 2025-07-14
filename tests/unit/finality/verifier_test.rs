#![allow(unused_imports)]

use frost_protocol::{
    finality::{
        FinalityVerifier,
        FinalitySignal,
        error::FinalityError,
        verifier::{BasicMetrics, FinalityConfig},
    },
    state::{BlockRef, ChainId},
};

use std::time::Duration;
use async_trait::async_trait;
use std::collections::HashMap;

// Basic verifier implementation for testing
struct BasicVerifier {
    config: FinalityConfig,
    metrics: BasicMetrics,
}

impl BasicVerifier {
    fn new(config: FinalityConfig) -> Self {
        Self {
            config,
            metrics: BasicMetrics::default(),
        }
    }
}

#[async_trait]
impl FinalityVerifier for BasicVerifier {
    async fn verify_finality(&self, block_ref: &BlockRef, signal: &FinalitySignal) -> Result<bool, FinalityError> {
        // Basic verification checks
        if signal.chain_id != block_ref.chain_id.to_string() {
            return Err(FinalityError::InvalidSignal("Chain ID mismatch".to_string()));
        }

        if signal.block_number != block_ref.number {
            return Err(FinalityError::InvalidSignal("Block number mismatch".to_string()));
        }

        if signal.block_hash != block_ref.hash {
            return Err(FinalityError::InvalidSignal("Block hash mismatch".to_string()));
        }

        // Check for timeout if timestamp is present in metadata
        if let Some(timestamp) = signal.metadata.get("timestamp") {
            if let Some(ts) = timestamp.as_u64() {
                let age = Duration::from_secs(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .saturating_sub(ts)
                );
                if age > self.config.finality_timeout {
                    return Err(FinalityError::Timeout {
                        block_ref: block_ref.clone(),
                        timeout_secs: self.config.finality_timeout,
                        retry_count: 0,
                    });
                }
            }
        }

        Ok(true)
    }

    async fn get_metrics(&self) -> BasicMetrics {
        self.metrics.clone()
    }

    async fn update_config(&mut self, config: FinalityConfig) -> Result<(), FinalityError> {
        self.config = config;
        Ok(())
    }
}

#[tokio::test]
async fn test_basic_finality_verification() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(
        chain_id.clone(),
        1000,
        [0u8; 32],
    );
    
    let signal = FinalitySignal {
        chain_id: chain_id.to_string(),
        block_number: 1000,
        block_hash: [0u8; 32],
        proof_data: vec![1, 2, 3],
        metadata: serde_json::json!({}),
    };
    
    // Test basic finality verification
    let mut config = FinalityConfig::default();
    config.finality_timeout = Duration::from_secs(30);
    config.min_confirmations = 6;
    let verifier = BasicVerifier::new(config);
    
    let result = verifier.verify_finality(&block_ref, &signal).await;
    assert!(result.is_ok(), "Basic finality verification failed");
}

#[tokio::test]
async fn test_invalid_finality_signal() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(
        chain_id.clone(),
        1000,
        [0u8; 32],
    );
    
    // Create invalid signal (wrong block number)
    let signal = FinalitySignal {
        chain_id: chain_id.to_string(),
        block_number: 999, // Mismatch
        block_hash: [0u8; 32],
        proof_data: vec![],
        metadata: serde_json::json!({}),
    };
    
    let config = FinalityConfig::default();
    let verifier = BasicVerifier::new(config);
    
    let result = verifier.verify_finality(&block_ref, &signal).await;
    assert!(result.is_err(), "Invalid finality signal should fail verification");
}

#[tokio::test]
async fn test_finality_timeout() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(
        chain_id.clone(),
        1000,
        [0u8; 32],
    );
    
    let signal = FinalitySignal {
        chain_id: chain_id.to_string(),
        block_number: 1000,
        block_hash: [0u8; 32],
        proof_data: vec![1, 2, 3],
        metadata: serde_json::json!({
            "timestamp": 0, // Very old timestamp
        }),
    };
    
    let mut config = FinalityConfig::default();
    config.finality_timeout = Duration::from_secs(1);
    let verifier = BasicVerifier::new(config);
    
    let result = verifier.verify_finality(&block_ref, &signal).await;
    assert!(matches!(result, Err(FinalityError::Timeout { .. })));
}

#[tokio::test]
async fn test_chain_specific_verification() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(
        chain_id.clone(),
        1000,
        [0u8; 32],
    );
    
    // Create signal with chain-specific proof data
    let signal = FinalitySignal {
        chain_id: chain_id.to_string(),
        block_number: 1000,
        block_hash: [0u8; 32],
        proof_data: vec![1, 2, 3], // Chain-specific proof
        metadata: serde_json::json!({
            "validators": ["0x1", "0x2", "0x3"],
            "signatures": ["sig1", "sig2"],
        }),
    };
    
    let config = FinalityConfig::default();
    let verifier = BasicVerifier::new(config);
    
    let result = verifier.verify_finality(&block_ref, &signal).await;
    assert!(result.is_ok(), "Chain-specific verification failed");
} 
