use crate::common::{
    create_test_block_id,
    create_test_transition,
    create_test_proof,
};

use frost_protocol::{
    state::{StateTransition, StateProof},
    finality::{
        FinalityConfig,
        FinalitySignal,
        EthereumVerifier,
        CosmosVerifier,
        SubstrateVerifier,
        FinalityVerifier,
    },
};

use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_state_transition_flow() {
    // Create and validate state transition
    let transition = create_test_transition("eth", "cosmos");
    assert!(transition.validate());

    // Create and validate state proof
    let proof = create_test_proof("eth", "cosmos");
    assert!(proof.validate());

    // Verify proof validates transition
    assert_eq!(proof.transition.source.chain_id, "eth");
    assert_eq!(proof.transition.target.chain_id, "cosmos");
}

#[tokio::test]
async fn test_ethereum_finality() {
    let config = FinalityConfig {
        min_confirmations: 6,
        finality_timeout: Duration::from_secs(30),
        basic_params: HashMap::new(),
    };

    let verifier = EthereumVerifier::new(config);

    // Test PoW finality
    let pow_signal = FinalitySignal::Ethereum {
        block_number: 100,
        block_hash: [0; 32],
        confirmations: 10,
        finality_type: frost_protocol::finality::EthereumFinalityType::Confirmations,
        metadata: None,
    };

    let block_ref = create_test_block_id("eth", 100);
    assert!(verifier.verify_finality(&block_ref, &pow_signal).await.unwrap());

    // Test beacon finality
    let beacon_signal = FinalitySignal::Ethereum {
        block_number: 100,
        block_hash: [0; 32],
        confirmations: 1,
        finality_type: frost_protocol::finality::EthereumFinalityType::BeaconFinalized,
        metadata: Some(frost_protocol::finality::EthereumMetadata {
            current_slot: Some(1000),
            head_slot: Some(990),
            active_validators: Some(300000),
            total_validators: Some(400000),
        }),
    };

    assert!(verifier.verify_finality(&block_ref, &beacon_signal).await.unwrap());
}

#[tokio::test]
async fn test_cosmos_finality() {
    let config = FinalityConfig {
        min_confirmations: 2,
        finality_timeout: Duration::from_secs(30),
        basic_params: {
            let mut params = HashMap::new();
            params.insert("min_signatures".to_string(), serde_json::json!(8));
            params
        },
    };

    let verifier = CosmosVerifier::new(config);

    let signal = FinalitySignal::Cosmos {
        height: 100,
        block_hash: [0; 32],
        validator_signatures: vec![vec![1; 32]; 10],
        metadata: Some(frost_protocol::finality::CosmosMetadata {
            voting_power: Some(800),
            total_power: Some(1000),
        }),
    };

    let block_ref = create_test_block_id("cosmos", 100);
    assert!(verifier.verify_finality(&block_ref, &signal).await.unwrap());
}

#[tokio::test]
async fn test_substrate_finality() {
    let config = FinalityConfig {
        min_confirmations: 2,
        finality_timeout: Duration::from_secs(30),
        basic_params: HashMap::new(),
    };

    let verifier = SubstrateVerifier::new(config);

    let signal = FinalitySignal::Substrate {
        block_number: 100,
        block_hash: [0; 32],
        metadata: Some(frost_protocol::finality::SubstrateMetadata {
            voting_power: Some(800),
            total_power: Some(1000),
            active_validators: Some(150),
            total_validators: Some(200),
        }),
    };

    let block_ref = create_test_block_id("substrate", 100);
    assert!(verifier.verify_finality(&block_ref, &signal).await.unwrap());
}

#[tokio::test]
async fn test_finality_metrics() {
    let config = FinalityConfig {
        min_confirmations: 6,
        finality_timeout: Duration::from_secs(30),
        basic_params: HashMap::new(),
    };

    let verifier = EthereumVerifier::new(config);
    let block_ref = create_test_block_id("eth", 100);

    // Generate some finality signals
    for i in 0..5 {
        let signal = FinalitySignal::Ethereum {
            block_number: 100 + i,
            block_hash: [0; 32],
            confirmations: 10,
            finality_type: frost_protocol::finality::EthereumFinalityType::Confirmations,
            metadata: None,
        };

        let _ = verifier.verify_finality(&block_ref, &signal).await;
    }

    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert!(metrics.total_blocks_verified > 0);
}

#[tokio::test]
async fn test_invalid_finality_signals() {
    let config = FinalityConfig {
        min_confirmations: 6,
        finality_timeout: Duration::from_secs(30),
        basic_params: HashMap::new(),
    };

    let verifier = EthereumVerifier::new(config);
    let block_ref = create_test_block_id("eth", 100);

    // Test insufficient confirmations
    let invalid_pow = FinalitySignal::Ethereum {
        block_number: 100,
        block_hash: [0; 32],
        confirmations: 2, // Less than required
        finality_type: frost_protocol::finality::EthereumFinalityType::Confirmations,
        metadata: None,
    };
    assert!(!verifier.verify_finality(&block_ref, &invalid_pow).await.unwrap());

    // Test invalid beacon sync
    let invalid_beacon = FinalitySignal::Ethereum {
        block_number: 100,
        block_hash: [0; 32],
        confirmations: 1,
        finality_type: frost_protocol::finality::EthereumFinalityType::BeaconFinalized,
        metadata: Some(frost_protocol::finality::EthereumMetadata {
            current_slot: Some(1000),
            head_slot: Some(900), // Too far behind
            active_validators: Some(300000),
            total_validators: Some(400000),
        }),
    };
    assert!(verifier.verify_finality(&block_ref, &invalid_beacon).await.is_err());
} 