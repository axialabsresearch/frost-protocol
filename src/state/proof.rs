use serde::{Serialize, Deserialize};
use crate::state::{StateTransition, ProofType};

/// Proof for state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    pub proof_data: Vec<u8>,
    pub proof_type: ProofType,
    pub metadata: ProofMetadata,
}

/// Metadata for state proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub generation_time: u64,
    pub proof_size: usize,
    pub verification_cost: Option<u64>,
    pub chain_specific: Option<serde_json::Value>,
}

impl StateProof {
    /// Create a new state proof
    pub fn new(
        proof_data: Vec<u8>,
        proof_type: ProofType,
        metadata: ProofMetadata,
    ) -> Self {
        Self {
            proof_data,
            proof_type,
            metadata,
        }
    }

    /// Verify the proof against a state transition
    pub fn verify(&self, transition: &StateTransition) -> bool {
        // Basic verification logic
        // Actual implementation will depend on proof type
        !self.proof_data.is_empty()
    }
} 