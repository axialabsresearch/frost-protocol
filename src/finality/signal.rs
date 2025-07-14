#![allow(unused_imports)]
#![allow(unused_variables)]

use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::state::BlockId;
use std::time::SystemTime;

/// Generic finality signal for any chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinalitySignal {
    /// Chain identifier
    pub chain_id: String,
    
    /// Block number/height
    pub block_number: u64,
    
    /// Block hash
    pub block_hash: [u8; 32],

    /// Finality proof data
    pub proof_data: Vec<u8>,
    
        /// Chain-specific metadata
    pub metadata: Value,
}

/// Block references for cross-chain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRefs {
    pub source_block: BlockId,
    pub target_block: BlockId,
    pub finality_block: BlockId,
    pub timestamp: u64,
}

impl Default for BlockRefs {
    fn default() -> Self {
        Self {
            source_block: BlockId::Number(0),
            target_block: BlockId::Number(0),
            finality_block: BlockId::Number(0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
