//! Ethereum Finality Verification Example
//! 
//! This example demonstrates how to use FROST Protocol's finality verification
//! for Ethereum chains, including both PoW and Beacon Chain finality.
//! 
//! Key concepts demonstrated:
//! - Setting up finality configuration
//! - Creating and configuring verifiers
//! - Handling different finality types
//! - Processing finality signals
//! - Error handling and metrics

use std::time::Duration;
use frost_protocol::{
    finality::{
        FinalityConfig,
        EthereumVerifier,
        FinalityVerifier,
        FinalitySignal,
        EthereumFinalityType,
        EthereumMetadata,
    },
    state::{BlockRef, ChainId},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create Ethereum chain configuration
    let config = FinalityConfig {
        min_confirmations: 12,  // Standard for high-value transactions
        finality_timeout: Duration::from_secs(30),
        basic_params: Default::default(),
    };

    // Create Ethereum finality verifier
    let verifier = EthereumVerifier::new(config);

    // Create a block reference for verification
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(
        chain_id,
        15_000_000,  // Block number
        [0u8; 32],   // Block hash (simplified for example)
    );

    // Example 1: PoW Finality
    println!("\nVerifying PoW finality:");
    let pow_signal = create_pow_signal(15_000_000, 15);
    match verifier.verify_finality(&block_ref, &pow_signal).await {
        Ok(true) => println!("✓ Block is final (PoW)"),
        Ok(false) => println!("✗ Block is not final (PoW)"),
        Err(e) => println!("! Error verifying PoW finality: {}", e),
    }

    // Example 2: Beacon Chain Finality
    println!("\nVerifying Beacon Chain finality:");
    let beacon_signal = create_beacon_signal(15_000_000);
    match verifier.verify_finality(&block_ref, &beacon_signal).await {
        Ok(true) => println!("✓ Block is final (Beacon)"),
        Ok(false) => println!("✗ Block is not final (Beacon)"),
        Err(e) => println!("! Error verifying Beacon finality: {}", e),
    }

    // Example 3: Check Metrics
    let metrics = verifier.get_metrics().await;
    println!("\nFinality Verification Metrics:");
    println!("Total blocks verified: {}", metrics.total_blocks_verified);
    println!("Failed verifications: {}", metrics.failed_verifications);

    Ok(())
}

/// Create a PoW finality signal
fn create_pow_signal(block_number: u64, confirmations: u32) -> FinalitySignal {
    FinalitySignal::Ethereum {
        block_number,
        block_hash: [0u8; 32],  // Simplified for example
        confirmations,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    }
}

/// Create a Beacon Chain finality signal
fn create_beacon_signal(block_number: u64) -> FinalitySignal {
    FinalitySignal::Ethereum {
        block_number,
        block_hash: [0u8; 32],  // Simplified for example
        confirmations: 1,        // Not used for Beacon finality
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(EthereumMetadata {
            current_slot: Some(32_000),
            head_slot: Some(32_000),
            active_validators: Some(300_000),
            total_validators: Some(400_000),
        }),
    }
} 