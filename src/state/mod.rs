#![allow(unused_imports)]

pub mod transition;
pub mod proof;
pub mod types;
pub mod error;
pub mod cache;
pub mod revocation;

pub use transition::StateTransition;
pub use proof::StateProof;
pub use types::{BlockId, BlockRef, StateRoot, ChainId};
pub use error::{StateError, ErrorSeverity};

use crate::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

// Quick test the state transition validation 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transition_validation() {
        let chain_id = ChainId::new("test-chain");
        let source = BlockId::Composite {
            number: 1,
            hash: [0; 32],  // Source hash all zeros
        };
        let target = BlockId::Composite {
            number: 2,
            hash: [1; 32],  // Target hash all ones
        };
        
        let transition = StateTransition::new(
            chain_id.clone(),
            source,
            target,
            vec![1, 2, 3],
        );
        assert!(transition.validate());

        // Create invalid transition with same hash (will fail validation)
        let invalid_source = BlockId::Composite {
            number: 1,
            hash: [0; 32],
        };
        let invalid_target = BlockId::Composite {
            number: 2,  // Valid height difference
            hash: [0; 32],  // Same hash as source - should fail validation
        };
        let invalid_transition = StateTransition::new(
            chain_id,
            invalid_source,
            invalid_target,
            vec![1],  // Valid data
        );
        assert!(!invalid_transition.validate());
    }
}
