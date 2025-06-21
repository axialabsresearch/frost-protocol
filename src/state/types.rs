use serde::{Serialize, Deserialize};

/// Chain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(pub u64);

/// Block identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockId {
    Hash([u8; 32]),
    Number(u64),
    Composite {
        number: u64,
        hash: [u8; 32],
    },
}

/// Block reference with chain context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRef {
    pub chain_id: ChainId,
    pub block_id: BlockId,
    pub parent_id: Option<BlockId>,
}

/// State root representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRoot {
    pub root: [u8; 32],
    pub height: u64,
    pub timestamp: u64,
    pub metadata: StateMetadata,
}

/// Additional state metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateMetadata {
    pub version: u32,
    pub chain_specific: Option<serde_json::Value>,
} 