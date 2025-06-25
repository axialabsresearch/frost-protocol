use std::time::Duration;
use frost_protocol::{
    state::BlockRef,
    finality::{
        FinalitySignal,
        FinalityError,
        verifier::{FinalityVerifier, FinalityConfig, FinalityMetrics},
        EthereumFinalityType,
        SolanaMetadata,
        CosmosMetadata,
        EthereumMetadata,
    },
};

use crate::common::{test_block_ref, test_chain_id};

#[tokio::test]
async fn test_ethereum_finality_verification() {
    let config = FinalityConfig {
        min_confirmations: 12,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "network": "mainnet",
            "use_beacon": true,
        }),
    };
    
    let mut verifier = EthereumVerifier::new(config);
    let block_ref = test_block_ref("ethereum", 1000);
    
    // Test PoW confirmations (legacy)
    let pow_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 15,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    assert!(verifier.verify_finality(&block_ref, &pow_signal).await.unwrap());
    
    // Test insufficient confirmations
    let insufficient_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 6,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    assert!(!verifier.verify_finality(&block_ref, &insufficient_signal).await.unwrap());
    
    // Test beacon chain finalization
    let beacon_metadata = EthereumMetadata {
        gas_used: 1500000,
        base_fee: 15000000000,
        difficulty: 0,
        total_difficulty: 0,
        current_slot: Some(32000),
        head_slot: Some(31990),
        justified_epoch: Some(1000),
        finalized_epoch: Some(999),
        participation_rate: Some(0.95),
        active_validators: Some(400000),
        total_validators: Some(420000),
        validator_balance: Some(32000000000),
        latest_fork_version: Some([1, 0, 0, 0]),
        fork_choice_head: Some([1u8; 32]),
        justified_checkpoint_root: Some([2u8; 32]),
        finalized_checkpoint_root: Some([3u8; 32]),
        is_syncing: Some(false),
        sync_distance: Some(10),
        chain_id: Some(1),
        network_version: Some("mainnet".into()),
        extra_data: None,
    };
    
    let beacon_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(beacon_metadata.clone()),
    };
    
    assert!(verifier.verify_finality(&block_ref, &beacon_signal).await.unwrap());
    
    // Test beacon chain justification
    let justified_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconJustified,
        metadata: Some(beacon_metadata.clone()),
    };
    
    assert!(verifier.verify_finality(&block_ref, &justified_signal).await.unwrap());
    
    // Test out of sync beacon chain
    let out_of_sync_metadata = EthereumMetadata {
        current_slot: Some(32000),
        head_slot: Some(31900), // Too far behind
        ..beacon_metadata.clone()
    };
    
    let out_of_sync_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(out_of_sync_metadata),
    };
    
    assert!(matches!(
        verifier.verify_finality(&block_ref, &out_of_sync_signal).await,
        Err(FinalityError::NotSynced(_))
    ));
    
    // Test invalid metadata
    let invalid_metadata = EthereumMetadata {
        current_slot: None,
        head_slot: None,
        ..beacon_metadata
    };
    
    let invalid_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(invalid_metadata),
    };
    
    assert!(matches!(
        verifier.verify_finality(&block_ref, &invalid_signal).await,
        Err(FinalityError::InvalidSignal(_))
    ));
    
    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert_eq!(metrics.total_blocks_verified, 4);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
}

#[tokio::test]
async fn test_solana_finality_verification() {
    let config = FinalityConfig {
        min_confirmations: 1,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "network": "mainnet-beta",
        }),
    };
    
    let mut verifier = SolanaVerifier::new(config);
    let block_ref = test_block_ref("solana", 1000);
    
    // Test sufficient stake
    let valid_signal = FinalitySignal::Solana {
        slot: 1000,
        epoch: 10,
        bank_hash: [0u8; 32],
        vote_account_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(SolanaMetadata {
            super_majority_root: 990,
            vote_account_stake: 700,
            total_active_stake: 1000,
        }),
    };
    
    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());
    
    // Test insufficient stake
    let insufficient_signal = FinalitySignal::Solana {
        slot: 1000,
        epoch: 10,
        bank_hash: [0u8; 32],
        vote_account_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(SolanaMetadata {
            super_majority_root: 990,
            vote_account_stake: 500,
            total_active_stake: 1000,
        }),
    };
    
    assert!(!verifier.verify_finality(&block_ref, &insufficient_signal).await.unwrap());
    
    // Test missing metadata
    let invalid_signal = FinalitySignal::Solana {
        slot: 1000,
        epoch: 10,
        bank_hash: [0u8; 32],
        vote_account_signatures: vec![[1u8; 64].to_vec()],
        metadata: None,
    };
    
    assert!(verifier.verify_finality(&block_ref, &invalid_signal).await.is_err());
    
    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert_eq!(metrics.total_blocks_verified, 3);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
}

#[tokio::test]
async fn test_cosmos_finality_verification() {
    let config = FinalityConfig {
        min_confirmations: 1,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "chain_id": "cosmoshub-4",
        }),
    };
    
    let mut verifier = CosmosVerifier::new(config);
    let block_ref = test_block_ref("cosmos", 1000);
    
    // Test sufficient voting power
    let valid_signal = FinalitySignal::Cosmos {
        height: 1000,
        app_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            validator_power: 700,
            total_voting_power: 1000,
            app_version: 1,
        }),
    };
    
    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());
    
    // Test insufficient voting power
    let insufficient_signal = FinalitySignal::Cosmos {
        height: 1000,
        app_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            validator_power: 500,
            total_voting_power: 1000,
            app_version: 1,
        }),
    };
    
    assert!(!verifier.verify_finality(&block_ref, &insufficient_signal).await.unwrap());
    
    // Test missing metadata
    let invalid_signal = FinalitySignal::Cosmos {
        height: 1000,
        app_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: None,
    };
    
    assert!(verifier.verify_finality(&block_ref, &invalid_signal).await.is_err());
    
    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert_eq!(metrics.total_blocks_verified, 3);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
}

#[tokio::test]
async fn test_ethereum_fork_choice_verification() {
    let config = FinalityConfig {
        min_confirmations: 12,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "network": "mainnet",
            "use_beacon": true,
            "fork_choice_threshold": 0.66,
            "min_validator_participation": 0.75,
            "min_justification_participation": 0.80,
            "min_validator_balance": 32000000000,
        }),
    };
    
    let mut verifier = EthereumVerifier::new(config);
    let block_ref = test_block_ref("ethereum", 1000);

    // Test fork choice verification with valid metadata
    let valid_metadata = EthereumMetadata {
        gas_used: 1500000,
        base_fee: 15000000000,
        difficulty: 0,
        total_difficulty: 0,
        current_slot: Some(32000),
        head_slot: Some(31990),
        justified_epoch: Some(1000),
        finalized_epoch: Some(999),
        participation_rate: Some(0.95),
        active_validators: Some(400000),
        total_validators: Some(420000),
        validator_balance: Some(32000000000),
        latest_fork_version: Some([1, 0, 0, 0]),
        fork_choice_head: Some([1u8; 32]),
        justified_checkpoint_root: Some([2u8; 32]),
        finalized_checkpoint_root: Some([3u8; 32]),
        is_syncing: Some(false),
        sync_distance: Some(10),
        chain_id: Some(1),
        network_version: Some("mainnet".into()),
        extra_data: None,
    };

    let valid_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(valid_metadata.clone()),
    };

    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Test insufficient validator participation
    let low_participation_metadata = EthereumMetadata {
        active_validators: Some(300000), // 71% participation
        ..valid_metadata.clone()
    };

    let low_participation_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(low_participation_metadata),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &low_participation_signal).await,
        Err(FinalityError::InsufficientParticipation(_))
    ));

    // Test low validator balance
    let low_balance_metadata = EthereumMetadata {
        validator_balance: Some(31000000000), // 31 ETH
        ..valid_metadata.clone()
    };

    let low_balance_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(low_balance_metadata),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &low_balance_signal).await,
        Err(FinalityError::InsufficientStake(_))
    ));

    // Test invalid epoch sequence
    let invalid_epoch_metadata = EthereumMetadata {
        justified_epoch: Some(999), // Equal to finalized epoch
        finalized_epoch: Some(999),
        ..valid_metadata.clone()
    };

    let invalid_epoch_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(invalid_epoch_metadata),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &invalid_epoch_signal).await,
        Err(FinalityError::InvalidEpoch(_))
    ));

    // Test low justification participation
    let low_justification_metadata = EthereumMetadata {
        participation_rate: Some(0.75), // Below 80% threshold
        ..valid_metadata.clone()
    };

    let low_justification_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconJustified,
        metadata: Some(low_justification_metadata),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &low_justification_signal).await,
        Err(FinalityError::InsufficientParticipation(_))
    ));

    // Test missing metadata fields
    let incomplete_metadata = EthereumMetadata {
        gas_used: 1500000,
        base_fee: 15000000000,
        difficulty: 0,
        total_difficulty: 0,
        current_slot: None,
        head_slot: None,
        justified_epoch: None,
        finalized_epoch: None,
        participation_rate: None,
        active_validators: None,
        total_validators: None,
        validator_balance: None,
        latest_fork_version: None,
        fork_choice_head: None,
        justified_checkpoint_root: None,
        finalized_checkpoint_root: None,
        is_syncing: None,
        sync_distance: None,
        chain_id: Some(1),
        network_version: Some("mainnet".into()),
        extra_data: None,
    };

    let incomplete_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(incomplete_metadata),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &incomplete_signal).await,
        Err(FinalityError::InvalidSignal(_))
    ));

    // Check metrics after all tests
    let metrics = verifier.get_metrics().await;
    assert!(metrics.total_blocks_verified > 0);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
    
    // Check chain-specific metrics
    let chain_metrics = metrics.chain_metrics.get("ethereum").unwrap();
    assert!(chain_metrics.get("fork_choice_updates").is_some());
    assert!(chain_metrics.get("validator_participation").is_some());
} 