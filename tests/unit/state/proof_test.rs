#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::state::{
    ChainId,
    BlockRef,
    proof::{StateProof, ProofVerifier, ProofData, ProofType, VerificationParams},
    transition::StateTransition,
    types::BlockId,
    error::StateError,
};

use std::time::SystemTime;
use serde_json::json;
use async_trait::async_trait;


#[tokio::test]
async fn test_proof_creation() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id.clone(), 1000, [0u8; 32]);
    let state_data = vec![1, 2, 3, 4];
    
    let transition = StateTransition::new(
        chain_id.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        state_data.clone(),
    );
    
    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: state_data,
        metadata: Some(json!({
            "timestamp": SystemTime::now(),
            "chain_id": chain_id.to_string(),
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    
    let proof = StateProof::new(transition, proof_data);
    assert!(proof.proof.data.len() > 0, "Proof data should not be empty");
}

#[tokio::test]
async fn test_proof_verification() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id.clone(), 1000, [0u8; 32]);
    let state_data = vec![1, 2, 3, 4];
    
    let transition = StateTransition::new(
        chain_id.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        state_data.clone(),
    );
    
    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: state_data,
        metadata: Some(json!({
            "timestamp": SystemTime::now(),
            "chain_id": chain_id.to_string(),
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    
    let proof = StateProof::new(transition, proof_data);
    let verifier = TestProofVerifier::default();
    let params = VerificationParams::default();
    
    let result = verifier.verify_proof(&proof, &params, None).await;
    assert!(result.is_ok(), "Proof verification failed");
}

#[tokio::test]
async fn test_invalid_proof_rejection() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id.clone(), 1000, [0u8; 32]);
    
    let transition = StateTransition::new(
        chain_id.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![0], // Invalid data
    );
    
    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: vec![0],
        metadata: Some(json!({
            "timestamp": SystemTime::now(),
            "chain_id": chain_id.to_string(),
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    
    let proof = StateProof::new(transition, proof_data);
    let verifier = TestProofVerifier::default();
    let params = VerificationParams::default();
    
    let result = verifier.verify_proof(&proof, &params, None).await;
    assert!(result.is_err(), "Invalid proof should be rejected");
}

#[tokio::test]
async fn test_proof_chain_validation() {
    let eth_chain = ChainId::new("ethereum");
    let polygon_chain = ChainId::new("polygon");
    
    // Create proof for ethereum
    let eth_block = BlockRef::new(eth_chain.clone(), 1000, [0u8; 32]);
    let transition = StateTransition::new(
        eth_chain.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: vec![1, 2, 3, 4],
        metadata: Some(json!({
            "timestamp": SystemTime::now(),
            "chain_id": eth_chain.to_string(),
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    
    let proof = StateProof::new(transition, proof_data);
    
    // Try to verify with wrong chain
    let wrong_block = BlockRef::new(polygon_chain.clone(), 1000, [0u8; 32]);
    let verifier = TestProofVerifier::default();
    let params = VerificationParams::default();
    
    let context = json!({
        "chain_id": polygon_chain.to_string()
    });
    
    let result = verifier.verify_proof(&proof, &params, Some(&context)).await;
    assert!(result.is_err(), "Proof should not verify for wrong chain");
}

#[tokio::test]
async fn test_proof_metadata() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id.clone(), 1000, [0u8; 32]);
    
    let transition = StateTransition::new(
        chain_id.clone(),
        BlockId::Number(1000),
        BlockId::Number(1001),
        vec![1, 2, 3, 4],
    );
    
    let proof_data = ProofData {
        proof_type: ProofType::Basic,
        data: vec![1, 2, 3, 4],
        metadata: Some(json!({
            "timestamp": SystemTime::now(),
            "chain_id": chain_id.to_string(),
            "block_number": 1000,
        })),
        generated_at: SystemTime::now(),
        expires_at: None,
        version: 1,
    };
    
    let proof = StateProof::new(transition, proof_data);
    
    // Check metadata fields
    let metadata = proof.proof.metadata.as_ref().expect("Metadata should exist");
    assert!(metadata.get("timestamp").is_some(), "Proof should include timestamp");
    assert!(metadata.get("chain_id").is_some(), "Proof should include chain_id");
    assert!(metadata.get("block_number").is_some(), "Proof should include block_number");
}

// Test implementation of ProofVerifier
#[derive(Default)]
struct TestProofVerifier;

#[async_trait]
impl ProofVerifier for TestProofVerifier {
    fn supported_types(&self) -> Vec<ProofType> {
        vec![ProofType::Basic]
    }

    async fn verify_proof(
        &self,
        proof: &StateProof,
        params: &VerificationParams,
        context: Option<&serde_json::Value>,
    ) -> Result<bool, StateError> {
        // Validate chain ID exists and matches
        let chain_id = proof.proof.metadata.as_ref()
            .and_then(|m| m.get("chain_id"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| StateError::InvalidProof("Chain ID missing".to_string()))?;

        // Validate data is not empty or invalid
        if proof.proof.data.len() <= 1 || proof.proof.data.iter().all(|&x| x == 0) {
            return Err(StateError::InvalidProof("Invalid proof data".to_string()));
        }

        // For chain validation test, check if context chain matches proof chain
        if let Some(ctx) = context {
            if let Some(ctx_chain) = ctx.get("chain_id").and_then(|c| c.as_str()) {
                if ctx_chain != chain_id {
                    return Err(StateError::InvalidProof("Chain ID mismatch".to_string()));
                }
            }
        }

        Ok(true)
    }
} 