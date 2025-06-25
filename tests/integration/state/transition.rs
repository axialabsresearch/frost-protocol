use frost_protocol::state::{
    StateTransition, StateProof, BlockRef, ChainId, StateRoot,
    ProofType, TransitionMetadata, ProofMetadata
};
use frost_protocol::message::{FrostMessage, MessageType};
use frost_protocol::finality::{
    FinalitySignal,
    EthereumFinalityType,
    EthereumMetadata
};
use frost_protocol::Result;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio;

// Helper function to create a test block reference
fn create_test_block_ref(chain_id: &str, number: u64) -> BlockRef {
    BlockRef::new(
        ChainId::new(chain_id),
        number,
        [0; 32]
    )
}

// Helper function to create a test state root
fn create_test_state_root(block_ref: BlockRef) -> StateRoot {
    StateRoot {
        block_ref,
        root_hash: [0; 32],
        metadata: None,
    }
}

#[tokio::test]
async fn test_ethereum_state_transition() -> Result<()> {
    // Create Ethereum state transition
    let source_block = create_test_block_ref("ethereum", 100);
    let target_block = create_test_block_ref("ethereum", 101);
    
    let transition = StateTransition {
        chain_id: ChainId::new("ethereum"),
        block_height: 101,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::Merkle,
            chain_specific: None,
        },
    };

    // Create proof
    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::Merkle,
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(1000),
            chain_specific: None,
        },
    };

    // Verify proof
    assert!(proof.verify(&transition));

    // Create message with finality signal
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        ChainId::new("ethereum"),
        ChainId::new("ethereum"),
        Some(transition),
        Some(FinalitySignal::Ethereum {
            block_number: 101,
            block_hash: vec![0; 32],
            confirmations: 12,
            finality_type: EthereumFinalityType::Confirmations,
            metadata: EthereumMetadata::default(),
        }),
        Some(target_block),
    );

    assert!(msg.validate());
    Ok(())
}

#[tokio::test]
async fn test_cosmos_state_transition() -> Result<()> {
    // Create Cosmos state transition
    let source_block = create_test_block_ref("cosmos", 1000);
    let target_block = create_test_block_ref("cosmos", 1001);
    
    let transition = StateTransition {
        chain_id: ChainId::new("cosmos"),
        block_height: 1001,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::Validity,
            chain_specific: None,
        },
    };

    // Create proof with Tendermint-style validation
    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::Validity,
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(500),
            chain_specific: None,
        },
    };

    assert!(proof.verify(&transition));
    Ok(())
}

#[tokio::test]
async fn test_substrate_state_transition() -> Result<()> {
    // Create Substrate state transition
    let source_block = create_test_block_ref("substrate", 500);
    let target_block = create_test_block_ref("substrate", 501);
    
    let transition = StateTransition {
        chain_id: ChainId::new("substrate"),
        block_height: 501,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::Custom("grandpa".to_string()),
            chain_specific: None,
        },
    };

    // Create proof with GRANDPA-style validation
    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::Custom("grandpa".to_string()),
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(750),
            chain_specific: None,
        },
    };

    assert!(proof.verify(&transition));
    Ok(())
}

#[tokio::test]
async fn test_cross_chain_state_transition() -> Result<()> {
    // Create cross-chain state transition (Ethereum -> Cosmos)
    let source_block = create_test_block_ref("ethereum", 100);
    let target_block = create_test_block_ref("cosmos", 1000);
    
    let transition = StateTransition {
        chain_id: ChainId::new("ethereum"),
        block_height: 100,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::ZK,
            chain_specific: None,
        },
    };

    // Create ZK proof for cross-chain validation
    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::ZK,
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(2000),
            chain_specific: None,
        },
    };

    assert!(proof.verify(&transition));
    Ok(())
}

#[tokio::test]
async fn test_invalid_state_transitions() -> Result<()> {
    // Test invalid block height
    let source_block = create_test_block_ref("ethereum", 100);
    let target_block = create_test_block_ref("ethereum", 99); // Invalid: decreasing height
    
    let transition = StateTransition {
        chain_id: ChainId::new("ethereum"),
        block_height: 99,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::Merkle,
            chain_specific: None,
        },
    };

    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::Merkle,
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(1000),
            chain_specific: None,
        },
    };

    assert!(!proof.verify(&transition));

    // Test mismatched proof type
    let source_block = create_test_block_ref("ethereum", 100);
    let target_block = create_test_block_ref("ethereum", 101);
    
    let transition = StateTransition {
        chain_id: ChainId::new("ethereum"),
        block_height: 101,
        pre_state: create_test_state_root(source_block),
        post_state: create_test_state_root(target_block),
        transition_proof: Some(vec![1, 2, 3]),
        metadata: TransitionMetadata {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 0,
            proof_type: ProofType::Merkle,
            chain_specific: None,
        },
    };

    let proof = StateProof {
        proof_data: vec![4, 5, 6],
        proof_type: ProofType::ZK, // Mismatched proof type
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 3,
            verification_cost: Some(1000),
            chain_specific: None,
        },
    };

    assert!(!proof.verify(&transition));
    Ok(())
} 