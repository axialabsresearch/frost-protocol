//! Cosmos Finality Verification Example
//! 
//! This example demonstrates how to use FROST Protocol's finality verification
//! for Cosmos-based chains using Tendermint consensus.
//! 
//! Key concepts demonstrated:
//! - Tendermint consensus verification
//! - Validator signature verification
//! - Voting power thresholds
//! - Custom finality rules
//! - Metrics collection

use std::time::Duration;
use std::collections::HashMap;
use frost_protocol::{
    finality::{
        FinalityConfig,
        CosmosVerifier,
        FinalityVerifier,
        FinalitySignal,
        CosmosMetadata,
    },
    state::{BlockRef, ChainId},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create Cosmos chain configuration with custom parameters
    let mut params = HashMap::new();
    params.insert("min_signatures".to_string(), serde_json::json!(8));
    params.insert("voting_power_threshold".to_string(), serde_json::json!(0.67)); // 2/3 majority

    let config = FinalityConfig {
        min_confirmations: 2,  // Tendermint typically needs 2 blocks
        finality_timeout: Duration::from_secs(30),
        basic_params: params,
    };

    // Create Cosmos finality verifier
    let verifier = CosmosVerifier::new(config);

    // Create a block reference for verification
    let chain_id = ChainId::new("cosmoshub-4");
    let block_ref = BlockRef::new(
        chain_id,
        1_000_000,  // Block height
        [0u8; 32],  // Block hash (simplified for example)
    );

    // Example 1: Basic Tendermint Finality
    println!("\nVerifying Tendermint finality:");
    let basic_signal = create_tendermint_signal(1_000_000, 700, 1000);
    match verifier.verify_finality(&block_ref, &basic_signal).await {
        Ok(true) => println!("✓ Block is final (sufficient voting power)"),
        Ok(false) => println!("✗ Block is not final (insufficient voting power)"),
        Err(e) => println!("! Error verifying finality: {}", e),
    }

    // Example 2: Insufficient Voting Power
    println!("\nVerifying insufficient voting power:");
    let weak_signal = create_tendermint_signal(1_000_000, 500, 1000);
    match verifier.verify_finality(&block_ref, &weak_signal).await {
        Ok(true) => println!("✓ Block is final (unexpected)"),
        Ok(false) => println!("✗ Block is not final (as expected)"),
        Err(e) => println!("! Error verifying finality: {}", e),
    }

    // Example 3: With Validator Signatures
    println!("\nVerifying with validator signatures:");
    let signed_signal = create_signed_signal(1_000_000, 800, 1000);
    match verifier.verify_finality(&block_ref, &signed_signal).await {
        Ok(true) => println!("✓ Block is final (valid signatures)"),
        Ok(false) => println!("✗ Block is not final (invalid signatures)"),
        Err(e) => println!("! Error verifying signatures: {}", e),
    }

    // Example 4: Check Metrics
    let metrics = verifier.get_metrics().await;
    println!("\nFinality Verification Metrics:");
    println!("Total blocks verified: {}", metrics.total_blocks_verified);
    println!("Failed verifications: {}", metrics.failed_verifications);

    Ok(())
}

/// Create a basic Tendermint finality signal
fn create_tendermint_signal(height: u64, voting_power: u64, total_power: u64) -> FinalitySignal {
    FinalitySignal::Cosmos {
        height,
        block_hash: [0u8; 32],  // Simplified for example
        validator_signatures: vec![],  // No signatures in basic signal
        metadata: Some(CosmosMetadata {
            voting_power: Some(voting_power),
            total_power: Some(total_power),
        }),
    }
}

/// Create a Tendermint signal with validator signatures
fn create_signed_signal(height: u64, voting_power: u64, total_power: u64) -> FinalitySignal {
    // In a real implementation, these would be actual validator signatures
    let mock_signatures = vec![
        vec![1u8; 64],  // Mock signature 1
        vec![2u8; 64],  // Mock signature 2
        vec![3u8; 64],  // Mock signature 3
    ];

    FinalitySignal::Cosmos {
        height,
        block_hash: [0u8; 32],
        validator_signatures: mock_signatures,
        metadata: Some(CosmosMetadata {
            voting_power: Some(voting_power),
            total_power: Some(total_power),
        }),
    }
} 