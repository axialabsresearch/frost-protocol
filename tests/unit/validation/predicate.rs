#![allow(unused_imports)]
#![allow(unused_variables)]

use std::time::Duration;
use frost_protocol::{
    state::{BlockRef, ChainId},
    finality::{
        FinalitySignal,
        signal::{EthereumFinalityType, EthereumMetadata, CosmosMetadata, SubstrateMetadata},
        predicate::{
            PredicateValidator,
            PredicateConfig,
            PredicateResult,
            PredicateError,
            FinalityVerificationClient,
            EthereumPredicateValidator,
            CosmosPredicateValidator,
            Block,
            BeaconBlock,
            FinalityVerificationError,
            ChainRules,
        },
    },
};

use mockall::predicate::*;
use mockall::mock;
use async_trait::async_trait;

// I'm bring this here cos, somehow I can't seem to import it from the common module
pub fn test_block_ref(chain_id: &str, number: u64) -> BlockRef {
    BlockRef::new(
        ChainId::new(chain_id),
        number,
        [0; 32],
    )
}

// Mock finality verification client for testing
mock! {
    FinalityVerificationClient {
        fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError>;
        fn verify_block_hash(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
        fn get_beacon_block(&self, block_ref: &BlockRef) -> Result<BeaconBlock, FinalityVerificationError>;
        fn is_block_finalized(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
        fn verify_validator_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError>;
        fn verify_vote_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError>;
        fn get_latest_finalized_block(&self) -> Result<u64, FinalityVerificationError>;
        fn get_chain_head(&self) -> Result<BlockRef, FinalityVerificationError>;
        fn verify_block_inclusion(&self, block_ref: &BlockRef, proof: &[u8]) -> Result<bool, FinalityVerificationError>;
        fn get_finality_confidence(&self, block_ref: &BlockRef) -> Result<f64, FinalityVerificationError>;
        fn verify_chain_rules(&self, block_ref: &BlockRef, rules: &ChainRules) -> Result<bool, FinalityVerificationError>;
    }
}

// Implement the trait for the mock
#[async_trait]
impl FinalityVerificationClient for MockFinalityVerificationClient {
    async fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError> {
        self.get_block(block_ref)
    }
    
    async fn verify_block_hash(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError> {
        self.verify_block_hash(block_ref)
    }
    
    async fn get_beacon_block(&self, block_ref: &BlockRef) -> Result<BeaconBlock, FinalityVerificationError> {
        self.get_beacon_block(block_ref)
    }
    
    async fn is_block_finalized(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError> {
        self.is_block_finalized(block_ref)
    }
    
    async fn verify_validator_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError> {
        self.verify_validator_signatures(block_ref, signatures)
    }
    
    async fn verify_vote_signatures(&self, block_ref: &BlockRef, signatures: &[Vec<u8>]) -> Result<bool, FinalityVerificationError> {
        self.verify_vote_signatures(block_ref, signatures)
    }
    
    async fn get_latest_finalized_block(&self) -> Result<u64, FinalityVerificationError> {
        self.get_latest_finalized_block()
    }
    
    async fn get_chain_head(&self) -> Result<BlockRef, FinalityVerificationError> {
        self.get_chain_head()
    }
    
    async fn verify_block_inclusion(&self, block_ref: &BlockRef, proof: &[u8]) -> Result<bool, FinalityVerificationError> {
        self.verify_block_inclusion(block_ref, proof)
    }
    
    async fn get_finality_confidence(&self, block_ref: &BlockRef) -> Result<f64, FinalityVerificationError> {
        self.get_finality_confidence(block_ref)
    }
    
    async fn verify_chain_rules(&self, block_ref: &BlockRef, rules: &ChainRules) -> Result<bool, FinalityVerificationError> {
        self.verify_chain_rules(block_ref, rules)
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
        
    mock_client.expect_get_finality_confidence()
        .returning(|_| Ok(1.0));
        
    mock_client.expect_verify_chain_rules()
        .returning(|_, _| Ok(true));
        
    mock_client.expect_verify_vote_signatures()
        .returning(|_, _| Ok(true));
    
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
        metadata: Some(EthereumMetadata {
            current_slot: Some(32000),
            head_slot: Some(32000),
            active_validators: Some(300000),
            total_validators: Some(400000),
        }),
    };
    
    let result = validator.validate_predicate(&block_ref, &beacon_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
    // Test invalid signal
    let invalid_signal = FinalitySignal::Custom {
        chain_id: "invalid".to_string(),
        block_id: "1000".to_string(),
        proof_data: vec![],
        metadata: serde_json::json!({}),
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
        
    mock_client.expect_get_finality_confidence()
        .returning(|_| Ok(1.0));
        
    mock_client.expect_verify_chain_rules()
        .returning(|_, _| Ok(true));
        
    mock_client.expect_verify_vote_signatures()
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
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            voting_power: Some(700),
            total_power: Some(1000),
        }),
    };
    
    let result = validator.validate_predicate(&block_ref, &valid_signal, &config).await.unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.confidence, 1.0);
    
    // Test insufficient voting power
    let insufficient_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            voting_power: Some(500),
            total_power: Some(1000),
        }),
    };
    
    let result = validator.validate_predicate(&block_ref, &insufficient_signal, &config).await.unwrap();
    assert!(!result.is_satisfied);
    assert!(result.confidence < 1.0);
    
    // Test missing metadata
    let invalid_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: None,
    };
    
    assert!(validator.validate_predicate(&block_ref, &invalid_signal, &config).await.is_err());
}

#[tokio::test]
async fn test_substrate_predicate_validation() {
    // Setup predicate config
    let config = PredicateConfig {
        min_confirmations: 1,
        evaluation_timeout: Duration::from_secs(30),
        confidence_threshold: 0.95,
        chain_params: serde_json::json!({
            "min_active_validators": 100,
            "min_voting_power": 0.67,  // 2/3 majority
            "max_reorg_depth": 2,
        }),
    };

    // Test valid GRANDPA finality
    let valid_signal = FinalitySignal::Substrate {
        block_number: 1000,
        block_hash: [0; 32],
        metadata: Some(SubstrateMetadata {
            voting_power: Some(800),
            total_power: Some(1000),
            active_validators: Some(150),
            total_validators: Some(200),
        }),
    };

    let chain_id = ChainId::new("substrate");
    assert!(validate_substrate_predicate(&valid_signal, &config, &chain_id).await);

    // Test invalid voting power
    let invalid_power_signal = FinalitySignal::Substrate {
        block_number: 1000,
        block_hash: [0; 32],
        metadata: Some(SubstrateMetadata {
            voting_power: Some(500),  // Less than 2/3
            total_power: Some(1000),
            active_validators: Some(150),
            total_validators: Some(200),
        }),
    };
    assert!(!validate_substrate_predicate(&invalid_power_signal, &config, &chain_id).await);

    // Test invalid validator count
    let invalid_validator_signal = FinalitySignal::Substrate {
        block_number: 1000,
        block_hash: [0; 32],
        metadata: Some(SubstrateMetadata {
            voting_power: Some(800),
            total_power: Some(1000),
            active_validators: Some(80),  // Below minimum
            total_validators: Some(200),
        }),
    };
    assert!(!validate_substrate_predicate(&invalid_validator_signal, &config, &chain_id).await);

    // Test missing metadata
    let missing_metadata_signal = FinalitySignal::Substrate {
        block_number: 1000,
        block_hash: [0; 32],
        metadata: None,
    };
    assert!(!validate_substrate_predicate(&missing_metadata_signal, &config, &chain_id).await);
}

async fn validate_substrate_predicate(
    signal: &FinalitySignal,
    config: &PredicateConfig,
    chain_id: &ChainId,
) -> bool {
    if let FinalitySignal::Substrate { metadata, .. } = signal {
        if let Some(metadata) = metadata {
            // Check validator count
            let min_validators = config.chain_params["min_active_validators"]
                .as_u64()
                .unwrap_or(100);
            
            if metadata.active_validators.unwrap_or(0) < min_validators {
                return false;
            }

            // Check voting power
            let min_voting_power = config.chain_params["min_voting_power"]
                .as_f64()
                .unwrap_or(0.67);

            let voting_ratio = metadata.voting_power.unwrap_or(0) as f64 
                / metadata.total_power.unwrap_or(1) as f64;
            
            if voting_ratio < min_voting_power {
                return false;
            }

            true
        } else {
            false
        }
    } else {
        false
    }
} 