//! Substrate Finality Verification Example
//! 
//! This example demonstrates how to use FROST Protocol's finality verification
//! for Substrate-based chains using GRANDPA consensus and parachain validation.
//! 
//! Key concepts demonstrated:
//! - GRANDPA finality verification
//! - Parachain block verification
//! - Authority set management
//! - Voting thresholds
//! - Metrics collection

use std::time::Duration;
use std::collections::HashMap;
use frost_protocol::{
    finality::{
        FinalityConfig,
        SubstrateVerifier,
        FinalityVerifier,
        FinalitySignal,
        SubstrateMetadata,
    },
    state::{BlockRef, ChainId},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create Substrate chain configuration with GRANDPA parameters
    let mut params = HashMap::new();
    params.insert("authority_set_id".to_string(), serde_json::json!(5));
    params.insert("voting_threshold".to_string(), serde_json::json!(0.67)); // 2/3 majority

    let config = FinalityConfig {
        min_confirmations: 1,  // GRANDPA provides instant finality
        finality_timeout: Duration::from_secs(30),
        basic_params: params,
    };

    // Create Substrate finality verifier
    let verifier = SubstrateVerifier::new(config);

    // Create a block reference for verification
    let chain_id = ChainId::new("polkadot");
    let block_ref = BlockRef::new(
        chain_id,
        1_000_000,  // Block number
        [0u8; 32],  // Block hash (simplified for example)
    );

    // Example 1: Basic GRANDPA Finality
    println!("\nVerifying GRANDPA finality:");
    let grandpa_signal = create_grandpa_signal(1_000_000, true);
    match verifier.verify_finality(&block_ref, &grandpa_signal).await {
        Ok(true) => println!("✓ Block is final (GRANDPA)"),
        Ok(false) => println!("✗ Block is not final (GRANDPA)"),
        Err(e) => println!("! Error verifying GRANDPA finality: {}", e),
    }

    // Example 2: Parachain Finality
    println!("\nVerifying parachain finality:");
    let parachain_signal = create_parachain_signal(1_000_000);
    match verifier.verify_finality(&block_ref, &parachain_signal).await {
        Ok(true) => println!("✓ Block is final (Parachain)"),
        Ok(false) => println!("✗ Block is not final (Parachain)"),
        Err(e) => println!("! Error verifying parachain finality: {}", e),
    }

    // Example 3: Invalid Authority Set
    println!("\nVerifying with invalid authority set:");
    let invalid_signal = create_grandpa_signal(1_000_000, false);
    match verifier.verify_finality(&block_ref, &invalid_signal).await {
        Ok(true) => println!("✓ Block is final (unexpected)"),
        Ok(false) => println!("✗ Block is not final (as expected)"),
        Err(e) => println!("! Error verifying finality: {}", e),
    }

    // Example 4: Check Metrics
    let metrics = verifier.get_metrics().await;
    println!("\nFinality Verification Metrics:");
    println!("Total blocks verified: {}", metrics.total_blocks_verified);
    println!("Failed verifications: {}", metrics.failed_verifications);

    Ok(())
}

/// Create a GRANDPA finality signal
fn create_grandpa_signal(block_number: u64, valid_set: bool) -> FinalitySignal {
    let (voting_power, total_power) = if valid_set {
        (800, 1000)  // Valid 80% voting power
    } else {
        (500, 1000)  // Invalid 50% voting power
    };

    FinalitySignal::Substrate {
        block_number,
        block_hash: [0u8; 32],  // Simplified for example
        metadata: Some(SubstrateMetadata {
            voting_power: Some(voting_power),
            total_power: Some(total_power),
            active_validators: Some(150),
            total_validators: Some(200),
        }),
    }
}

/// Create a parachain finality signal
fn create_parachain_signal(block_number: u64) -> FinalitySignal {
    FinalitySignal::Substrate {
        block_number,
        block_hash: [0u8; 32],
        metadata: Some(SubstrateMetadata {
            voting_power: Some(900),      // 90% voting power
            total_power: Some(1000),
            active_validators: Some(180),  // High validator participation
            total_validators: Some(200),
        }),
    }
} 