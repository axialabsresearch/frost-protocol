use std::time::Duration;
use frost_protocol::{
    state::BlockRef,
    finality::{
        FinalitySignal,
        predicate::{
            PredicateValidator,
            PredicateConfig,
            PredicateResult,
            PredicateError,
            FinalityVerificationClient,
            EthereumPredicateValidator,
            SolanaPredicateValidator,
            CosmosPredicateValidator,
        },
        EthereumFinalityType,
        SolanaMetadata,
        CosmosMetadata,
    },
};

use crate::common::{test_block_ref, test_chain_id};
use mockall::predicate::*;
use mockall::mock;

// Mock finality verification client for testing
mock! {
    FinalityVerificationClient {
        fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError>;
        fn verify_block_hash(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
        fn get_beacon_block(&self, block_ref: &BlockRef) -> Result<BeaconBlock, FinalityVerificationError>;
        fn is_block_finalized(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
        fn is_block_justified(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
        fn verify_vote_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError>;
        fn verify_validator_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError>;
    }
}

#[tokio::test]
async fn test_ethereum_predicate_validation() {
    let mut mock_client = MockFinalityVerificationClient::new();
    let block_ref = test_block_ref("ethereum", 1000);
    
    // Setup mock expectations
    mock_client.expect_get_block()
        .returning(|_| Ok(Block {
            hash: [0u8; 32],
            number: 1000,
        }));
        
    mock_client.expect_verify_block_hash()
        .returning(|_| Ok(true));
        
    mock_client.expect_get_beacon_block()
        .returning(|_| Ok(BeaconBlock {
            slot: 32000,
            epoch: 1000,
        }));
        
    mock_client.expect_is_block_finalized()
        .returning(|_| Ok(true));
        
    mock_client.expect_is_block_justified()
        .returning(|_| Ok(true));
    
    let validator = EthereumPredicateValidator::new(Box::new(mock_client));
    let config = PredicateConfig {
        min_confirmations: 12,
        evaluation_timeout: Duration::from_secs(5),
        confidence_threshold: 0.9,
        chain_params: serde_json::json!({
            "network": "mainnet",
            "use_beacon": true,
        }),
    };
    
    // Test PoW confirmations
    let pow_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 15,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    let result = validator.validate_predicate(&block_ref, &pow_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
    // Test beacon finality
    let beacon_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: None,
    };
    
    let result = validator.validate_predicate(&block_ref, &beacon_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
    // Test invalid signal
    let invalid_signal = FinalitySignal::Solana {
        slot: 1000,
        epoch: 10,
        bank_hash: [0u8; 32],
        vote_account_signatures: vec![],
        metadata: None,
    };
    
    assert!(validator.validate_predicate(&block_ref, &invalid_signal, &config).await.is_err());
}

#[tokio::test]
async fn test_solana_predicate_validation() {
    let mut mock_client = MockFinalityVerificationClient::new();
    let block_ref = test_block_ref("solana", 1000);
    
    // Setup mock expectations
    mock_client.expect_get_block()
        .returning(|_| Ok(Block {
            hash: [0u8; 32],
            number: 1000,
        }));
        
    mock_client.expect_verify_vote_signatures()
        .returning(|_, _| Ok(true));
    
    let validator = SolanaPredicateValidator::new(Box::new(mock_client));
    let config = PredicateConfig {
        min_confirmations: 1,
        evaluation_timeout: Duration::from_secs(5),
        confidence_threshold: 0.9,
        chain_params: serde_json::json!({
            "network": "mainnet-beta",
        }),
    };
    
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
    
    let result = validator.validate_predicate(&block_ref, &valid_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
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
    
    let result = validator.validate_predicate(&block_ref, &insufficient_signal, &config).await.unwrap();
    assert!(!result.is_satisfied);
    assert!(result.confidence < 1.0);
    
    // Test missing metadata
    let invalid_signal = FinalitySignal::Solana {
        slot: 1000,
        epoch: 10,
        bank_hash: [0u8; 32],
        vote_account_signatures: vec![[1u8; 64].to_vec()],
        metadata: None,
    };
    
    assert!(validator.validate_predicate(&block_ref, &invalid_signal, &config).await.is_err());
}

#[tokio::test]
async fn test_cosmos_predicate_validation() {
    let mut mock_client = MockFinalityVerificationClient::new();
    let block_ref = test_block_ref("cosmos", 1000);
    
    // Setup mock expectations
    mock_client.expect_get_block()
        .returning(|_| Ok(Block {
            hash: [0u8; 32],
            number: 1000,
        }));
        
    mock_client.expect_verify_validator_signatures()
        .returning(|_, _| Ok(true));
    
    let validator = CosmosPredicateValidator::new(Box::new(mock_client));
    let config = PredicateConfig {
        min_confirmations: 1,
        evaluation_timeout: Duration::from_secs(5),
        confidence_threshold: 0.9,
        chain_params: serde_json::json!({
            "network": "cosmoshub-4",
        }),
    };
    
    // Test valid signal
    let valid_signal = FinalitySignal::Cosmos {
        height: 1000,
        round: 0,
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            total_voting_power: 1000,
            signed_voting_power: 700,
        }),
    };
    
    let result = validator.validate_predicate(&block_ref, &valid_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
    // Test insufficient voting power
    let insufficient_signal = FinalitySignal::Cosmos {
        height: 1000,
        round: 0,
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            total_voting_power: 1000,
            signed_voting_power: 500,
        }),
    };
    
    let result = validator.validate_predicate(&block_ref, &insufficient_signal, &config).await.unwrap();
    assert!(!result.is_satisfied);
    assert!(result.confidence < 1.0);
    
    // Test missing metadata
    let invalid_signal = FinalitySignal::Cosmos {
        height: 1000,
        round: 0,
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: None,
    };
    
    assert!(validator.validate_predicate(&block_ref, &invalid_signal, &config).await.is_err());
}

#[tokio::test]
async fn test_predicate_timeout() {
    let mut mock_client = MockFinalityVerificationClient::new();
    let block_ref = test_block_ref("ethereum", 1000);
    
    // Setup mock expectations to simulate slow response
    mock_client.expect_get_block()
        .returning(|_| {
            tokio::time::sleep(Duration::from_secs(6)).await;
            Ok(Block {
                hash: [0u8; 32],
                number: 1000,
            })
        });
        
    let validator = EthereumPredicateValidator::new(Box::new(mock_client));
    let config = PredicateConfig {
        min_confirmations: 12,
        evaluation_timeout: Duration::from_secs(5),
        confidence_threshold: 0.9,
        chain_params: serde_json::json!({}),
    };
    
    let signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash: [0u8; 32],
        confirmations: 15,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    match validator.validate_predicate(&block_ref, &signal, &config).await {
        Err(PredicateError::Timeout) => (),
        _ => panic!("Expected timeout error"),
    }
} 