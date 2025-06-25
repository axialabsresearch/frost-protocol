#![allow(unused_imports)]

pub mod transition;
pub mod proof;
pub mod types;
pub mod error;

pub use transition::StateTransition as ImportedStateTransition;
pub use proof::StateProof as ImportedStateProof;
pub use types::{BlockId as ImportedBlockId, BlockRef, StateRoot, ChainId};
pub use error::{StateError, ErrorSeverity};

use crate::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// Block identifier
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockId {
    /// Chain identifier
    pub chain_id: String,
    /// Block number
    pub number: u64,
    /// Block hash
    pub hash: [u8; 32],
}

/// State transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Source block
    pub source: BlockId,
    /// Target block
    pub target: BlockId,
    /// Transition data
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
    /// Version
    pub version: u16,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(source: BlockId, target: BlockId, data: Vec<u8>) -> Self {
        Self {
            source,
            target,
            data,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            version: 0,
        }
    }

    /// Validate state transition
    pub fn validate(&self) -> bool {
        // Basic validation for v0
        !self.data.is_empty() && 
        !self.source.chain_id.is_empty() && 
        !self.target.chain_id.is_empty()
    }
}

/// State proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    /// State transition
    pub transition: StateTransition,
    /// Proof data
    pub proof: Vec<u8>,
    /// Validator signatures
    pub signatures: Vec<Vec<u8>>,
}

impl StateProof {
    /// Create a new state proof
    pub fn new(transition: StateTransition, proof: Vec<u8>, signatures: Vec<Vec<u8>>) -> Self {
        Self {
            transition,
            proof,
            signatures,
        }
    }

    /// Validate state proof
    pub fn validate(&self) -> bool {
        // Basic validation for v0
        self.transition.validate() && 
        !self.proof.is_empty() && 
        !self.signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transition_validation() {
        let source = BlockId {
            chain_id: "chain1".to_string(),
            number: 1,
            hash: [0; 32],
        };
        let target = BlockId {
            chain_id: "chain2".to_string(),
            number: 1,
            hash: [0; 32],
        };
        
        let transition = StateTransition::new(
            source,
            target,
            vec![1, 2, 3],
        );
        assert!(transition.validate());

        let invalid_transition = StateTransition::new(
            BlockId::default(),
            BlockId::default(),
            vec![],
        );
        assert!(!invalid_transition.validate());
    }

    #[test]
    fn test_state_proof_validation() {
        let source = BlockId {
            chain_id: "chain1".to_string(),
            number: 1,
            hash: [0; 32],
        };
        let target = BlockId {
            chain_id: "chain2".to_string(),
            number: 1,
            hash: [0; 32],
        };
        
        let transition = StateTransition::new(
            source,
            target,
            vec![1, 2, 3],
        );
        
        let proof = StateProof::new(
            transition,
            vec![4, 5, 6],
            vec![vec![7, 8, 9]],
        );
        assert!(proof.validate());

        let invalid_proof = StateProof::new(
            StateTransition::new(
                BlockId::default(),
                BlockId::default(),
                vec![],
            ),
            vec![],
            vec![],
        );
        assert!(!invalid_proof.validate());
    }
}
