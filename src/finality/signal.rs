use serde::{Serialize, Deserialize};
use crate::state::BlockId;

/// Finality signal from different chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinalitySignal {
    Ethereum {
        block_number: u64,
        confirmations: u32,
    },
    Solana {
        slot: u64,
        epoch: u64,
    },
    Substrate {
        block_hash: [u8; 32],
        justification: Vec<u8>,
    },
    // Extensible for other chains
}

/// Block references for cross-chain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRefs {
    pub source_block: BlockId,
    pub target_block: BlockId,
    pub finality_block: BlockId,
}
