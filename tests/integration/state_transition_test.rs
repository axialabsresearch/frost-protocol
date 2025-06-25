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
    let msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
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

    // Test with finality signals from both chains
    let eth_finality = FinalitySignal::Ethereum {
        block_number: 100,
        block_hash: [0; 32],
        confirmations: 12,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(EthereumMetadata {
            current_slot: Some(1000),
            head_slot: Some(1000),
            active_validators: Some(300000),
            total_validators: Some(300000),
        }),
    };

    // Create message with finality signals
    let msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
    );

    assert!(msg.validate());
    Ok(())
}

#[tokio::test]
async fn test_invalid_state_transitions() -> Result<()> {
    // Test with invalid block height
    let source_block = create_test_block_ref("ethereum", 100);
    let target_block = create_test_block_ref("ethereum", 99); // Invalid: target height < source height
    
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

    // Test with empty proof
    let invalid_proof = StateProof {
        proof_data: vec![],
        proof_type: ProofType::Merkle,
        metadata: ProofMetadata {
            generation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof_size: 0,
            verification_cost: None,
            chain_specific: None,
        },
    };

    assert!(!invalid_proof.verify(&transition));

    // Test with invalid message
    let invalid_msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![],
        "".to_string(),
        None,
    );

    assert!(!invalid_msg.validate());
    Ok(())
} 