#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::{
    state::{
        transition::{StateTransition, StateTransitionVerifier},
        ChainId,
        BlockRef,
        types::BlockId,
        proof::ProofType,
    },
    Result,
};

use std::time::SystemTime;
use async_trait::async_trait;

#[derive(Default)]
struct TestTransitionVerifier;

#[async_trait]
impl StateTransitionVerifier for TestTransitionVerifier {
    async fn verify_transition(&self, transition: &StateTransition) -> Result<bool> {
        // Validate transition proof exists and is not empty
        if transition.transition_proof.is_none() || transition.transition_proof.as_ref().unwrap().is_empty() {
            return Err("Empty transition proof".into());
        }

        // Validate block height is valid
        if transition.block_height == 0 {
            return Err("Invalid block height".into());
        }

        // Validate chain ID is not default
        if transition.chain_id.to_string() == "default" {
            return Err("Invalid chain ID".into());
        }

        // Validate state roots are different
        if transition.pre_state == transition.post_state {
            return Err("State roots must be different".into());
        }

        // Validate chain compatibility
        if transition.chain_id.to_string() == "bitcoin" {
            return Err("Incompatible chain".into());
        }

        Ok(true)
    }

    async fn generate_proof(&self, transition: &StateTransition) -> Result<Vec<u8>> {
        Ok(vec![1, 2, 3])
    }
}

#[tokio::test]
async fn test_transition_creation() {
    let source_chain = ChainId::new("ethereum");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    let transition = StateTransition::new(
        source_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    assert_eq!(&transition.chain_id, &source_chain, "Chain ID should match");
}

#[tokio::test]
async fn test_transition_validation() {
    let source_chain = ChainId::new("ethereum");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    let transition = StateTransition::new(
        source_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let validator = TestTransitionVerifier::default();
    let result = validator.verify_transition(&transition).await;
    assert!(result.is_ok(), "Transition validation failed");
}

#[tokio::test]
async fn test_invalid_transition() {
    let source_chain = ChainId::new("default");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    // Create transition with default chain ID which should fail validation
    let transition = StateTransition::new(
        source_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1], // Valid state data
    );
    
    let validator = TestTransitionVerifier::default();
    let result = validator.verify_transition(&transition).await;
    assert!(result.is_err(), "Invalid transition should fail validation");
}

#[tokio::test]
async fn test_transition_chain_compatibility() {
    let btc_chain = ChainId::new("bitcoin");
    let block_ref = BlockRef::new(btc_chain.clone(), 1000, [0u8; 32]);
    
    // Create transition with bitcoin chain which should be incompatible
    let transition = StateTransition::new(
        btc_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let validator = TestTransitionVerifier::default();
    let result = validator.verify_transition(&transition).await;
    assert!(result.is_err(), "Incompatible chain transition should fail");
}

#[tokio::test]
async fn test_transition_metadata() {
    let source_chain = ChainId::new("ethereum");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    let transition = StateTransition::new(
        source_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let metadata = &transition.metadata;
    assert!(metadata.timestamp > 0, "Should include timestamp");
    assert_eq!(metadata.version, 0, "Should have version 0");
    assert!(matches!(metadata.proof_type, ProofType::Basic), "Should have basic proof type");
} 