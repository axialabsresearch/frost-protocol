use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::state::{ChainId, StateRoot, StateError};
use crate::Result;

/// State transition representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub chain_id: ChainId,
    pub block_height: u64,
    pub pre_state: StateRoot,
    pub post_state: StateRoot,
    pub transition_proof: Option<Vec<u8>>,
    pub metadata: TransitionMetadata,
}

/// Metadata for state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMetadata {
    pub timestamp: u64,
    pub version: u32,
    pub proof_type: ProofType,
    pub chain_specific: Option<serde_json::Value>,
}

/// Type of proof for state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    ZK,
    Merkle,
    Validity,
    Custom(String),
}

/// State transition verification
#[async_trait]
pub trait StateTransitionVerifier: Send + Sync {
    /// Verify a state transition
    async fn verify_transition(
        &self,
        transition: &StateTransition,
    ) -> Result<bool>;

    /// Generate proof for state transition
    async fn generate_proof(
        &self,
        transition: &StateTransition,
    ) -> Result<Vec<u8>>;
}
