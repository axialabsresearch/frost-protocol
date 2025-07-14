use std::hash::{Hash, Hasher};
use std::fmt;
use serde::{Serialize, Deserialize};

/// Chain identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ChainId(String);

impl ChainId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ChainId {
    fn default() -> Self {
        Self("default".into())
    }
}

/// Block identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BlockId {
    Hash([u8; 32]),
    Number(u64),
    Composite {
        number: u64,
        hash: [u8; 32],
    },
}

/// Block reference with chain context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlockRef {
    pub chain_id: ChainId,
    pub number: u64,
    pub hash: [u8; 32],
}

impl BlockRef {
    pub fn new(chain_id: ChainId, number: u64, hash: [u8; 32]) -> Self {
        Self {
            chain_id,
            number,
            hash,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

impl fmt::Display for BlockRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.chain_id, self.number, hex::encode(self.hash))
    }
}

impl Default for BlockRef {
    fn default() -> Self {
        Self {
            chain_id: ChainId::default(),
            number: 0,
            hash: [0; 32],
        }
    }
}

/// State root representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateRoot {
    pub block_ref: BlockRef,
    pub root_hash: [u8; 32],
    pub metadata: Option<serde_json::Value>,
}

impl Default for StateRoot {
    fn default() -> Self {
        Self {
            block_ref: BlockRef::default(),
            root_hash: [0; 32],
            metadata: None,
        }
    }
}

/// Additional state metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateMetadata {
    pub version: u32,
    pub chain_specific: Option<serde_json::Value>,
} 