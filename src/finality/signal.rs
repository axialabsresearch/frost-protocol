use serde::{Serialize, Deserialize};
use crate::state::BlockId;

/// Finality signal from different chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinalitySignal {
    /// Ethereum finality signal
    Ethereum {
        /// Block number
        block_number: u64,
        /// Block hash
        block_hash: [u8; 32],
        /// Number of confirmations
        confirmations: u32,
        /// Finality type
        finality_type: EthereumFinalityType,
        /// Chain metadata
        metadata: Option<EthereumMetadata>,
    },

    /// Cosmos finality signal
    Cosmos {
        /// Block height
        height: u64,
        /// Block hash
        block_hash: [u8; 32],
        /// Validator signatures
        validator_signatures: Vec<Vec<u8>>,
        /// Chain metadata
        metadata: Option<CosmosMetadata>,
    },

    /// Substrate finality signal
    Substrate {
        /// Block number
        block_number: u64,
        /// Block hash
        block_hash: [u8; 32],
        /// Chain metadata
        metadata: Option<SubstrateMetadata>,
    },
}

/// Ethereum finality types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EthereumFinalityType {
    /// PoW confirmations
    Confirmations,
    /// Beacon chain finalized
    BeaconFinalized,
}

/// Basic Ethereum metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumMetadata {
    /// Current slot
    pub current_slot: Option<u64>,
    /// Head slot
    pub head_slot: Option<u64>,
    /// Active validator count
    pub active_validators: Option<u64>,
    /// Total validator count
    pub total_validators: Option<u64>,
}

/// Basic Cosmos metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosMetadata {
    /// Voting power
    pub voting_power: Option<u64>,
    /// Total power
    pub total_power: Option<u64>,
}

/// Basic Substrate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateMetadata {
    /// Voting power
    pub voting_power: Option<u64>,
    /// Total power
    pub total_power: Option<u64>,
    /// Active validator count
    pub active_validators: Option<u64>,
    /// Total validator count
    pub total_validators: Option<u64>,
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
            source_block: BlockId::default(),
            target_block: BlockId::default(),
            finality_block: BlockId::default(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
