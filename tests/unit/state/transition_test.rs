#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::{
    state::{
        transition::{StateTransition, StateTransitionVerifier},
        ChainId,
        BlockRef,
        types::BlockId,
    },
    Result,
};
use frost_protocol::state::transition::ProofType;

use std::time::SystemTime;
use async_trait::async_trait;

#[derive(Default)]
struct TestTransitionVerifier;

#[async_trait]
impl StateTransitionVerifier for TestTransitionVerifier {
    async fn verify_transition(&self, transition: &StateTransition) -> Result<bool> {
        // Validate block height is valid
        if transition.block_height == 0 {
            return Ok(false);
        }

        // Validate chain ID is not default
        if transition.chain_id.to_string() == "default" {
            return Ok(false);
        }

        // Validate state roots are different
        if transition.pre_state == transition.post_state {
            return Ok(false);
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
    let source_chain = ChainId::new("ethereum");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    // Create transition with empty state data
    let transition = StateTransition::new(
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![], // Empty state data
    );
    
    let validator = TestTransitionVerifier::default();
    let result = validator.verify_transition(&transition).await;
    assert!(result.is_err(), "Invalid transition should fail validation");
}

#[tokio::test]
async fn test_transition_chain_compatibility() {
    let eth_chain = ChainId::new("ethereum");
    let btc_chain = ChainId::new("bitcoin");
    let block_ref = BlockRef::new(eth_chain.clone(), 1000, [0u8; 32]);
    
    // Create transition between incompatible chains
    let transition = StateTransition::new(
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
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let metadata = &transition.metadata;
    assert!(metadata.timestamp > 0, "Should include timestamp");
    assert_eq!(metadata.version, 0, "Should have version 0");
    assert!(matches!(metadata.proof_type, ProofType::Basic), "Should have basic proof type");
} 