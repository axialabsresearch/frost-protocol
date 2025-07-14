#![allow(unused_imports)]

use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use super::proof::ProofType;
use crate::state::{ChainId, StateRoot, StateError, BlockId, BlockRef};
use crate::Result;
use std::time::SystemTime;
use serde_json::json;

/// State transition between blocks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateTransition {
    pub chain_id: ChainId,
    pub block_height: u64,
    pub pre_state: StateRoot,
    pub post_state: StateRoot,
    pub transition_proof: Option<Vec<u8>>,
    pub metadata: TransitionMetadata,
}

impl StateTransition {
    /// Create a new state transition between chains
    /// 
    /// # Arguments
    /// * `chain_id` - The identifier of the chain this transition belongs to
    /// * `source` - The source block identifier
    /// * `target` - The target block identifier
    /// * `data` - The transition proof data
    pub fn new(chain_id: ChainId, source: BlockId, target: BlockId, data: Vec<u8>) -> Self {
        // Validate data is not empty
        if data.is_empty() {
            panic!("State transition data cannot be empty");
        }

        // Extract source block info
        let (source_height, source_hash) = match source {
            BlockId::Number(n) => (n, [0; 32]),
            BlockId::Composite { number, hash } => (number, hash),
            BlockId::Hash(hash) => (0, hash),
        };

        // Extract target block info
        let (target_height, target_hash) = match target {
            BlockId::Number(n) => (n, [1; 32]),
            BlockId::Composite { number, hash } => (number, hash),
            BlockId::Hash(hash) => (source_height + 1, hash),
        };

        // Validate block heights
        if target_height <= source_height {
            panic!("Target block height must be greater than source block height");
        }

        // Create block references
        let source_ref = BlockRef::new(chain_id.clone(), source_height, source_hash);
        let target_ref = BlockRef::new(chain_id.clone(), target_height, target_hash);

        // Create state roots
        let pre_state = StateRoot {
            block_ref: source_ref,
            root_hash: source_hash,
            metadata: None,
        };
        let post_state = StateRoot {
            block_ref: target_ref,
            root_hash: target_hash,
            metadata: None,
        };
        
        Self {
            chain_id,
            block_height: source_height,
            pre_state,
            post_state,
            transition_proof: Some(data),
            metadata: TransitionMetadata {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                version: 0,
                proof_type: ProofType::Basic,
                chain_specific: None,
            },
        }
    }

    /// Validate the transition
    pub fn validate(&self) -> bool {
        // Check that data is not empty
        if self.transition_proof.is_none() || self.transition_proof.as_ref().unwrap().is_empty() {
            return false;
        }

        // Check that pre and post states are different
        if self.pre_state.root_hash == self.post_state.root_hash {
            return false;
        }

        // Check that chain IDs match
        if self.pre_state.block_ref.chain_id != self.post_state.block_ref.chain_id {
            return false;
        }

        // Check that block heights are sequential
        if self.post_state.block_ref.number <= self.pre_state.block_ref.number {
            return false;
        }

        // Check that block height matches source block
        if self.block_height != self.pre_state.block_ref.number {
            return false;
        }

        // Check that chain ID matches block refs
        if self.chain_id != self.pre_state.block_ref.chain_id ||
           self.chain_id != self.post_state.block_ref.chain_id {
            return false;
        }

        true
    }
}

/// Metadata for state transitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionMetadata {
    pub timestamp: u64,
    pub version: u32,
    pub proof_type: ProofType,
    pub chain_specific: Option<serde_json::Value>,
}

impl Default for TransitionMetadata {
    fn default() -> Self {
        Self {
            timestamp: 0,
            version: 1,
            proof_type: ProofType::Basic,
            chain_specific: None,
        }
    }
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
