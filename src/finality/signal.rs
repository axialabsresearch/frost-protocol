use serde::{Serialize, Deserialize};
use crate::state::BlockId;

/// Finality signal from different chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinalitySignal {
    Ethereum {
        block_number: u64,
        block_hash: [u8; 32],
        confirmations: u32,
        finality_type: EthereumFinalityType,
        metadata: Option<EthereumMetadata>,
    },
    
    Solana {
        slot: u64,
        epoch: u64,
        bank_hash: [u8; 32],
        vote_account_signatures: Vec<[u8; 64]>,
        metadata: Option<SolanaMetadata>,
    },
    
    Substrate {
        block_hash: [u8; 32],
        block_number: u64,
        justification: Vec<u8>,
        finality_type: SubstrateFinalityType,
        metadata: Option<SubstrateMetadata>,
    },
    
    Cosmos {
        height: u64,
        app_hash: [u8; 32],
        validator_signatures: Vec<[u8; 64]>,
        metadata: Option<CosmosMetadata>,
    },
    
    Near {
        block_height: u64,
        block_hash: [u8; 32],
        epoch_id: [u8; 32],
        next_epoch_id: [u8; 32],
        metadata: Option<NearMetadata>,
    },
    
    Custom {
        chain_id: String,
        block_id: String,
        proof_data: Vec<u8>,
        metadata: serde_json::Value,
    },
}

/// Ethereum finality types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EthereumFinalityType {
    Confirmations,
    BeaconFinalized,
    BeaconJustified,
}

/// Substrate finality types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubstrateFinalityType {
    Grandpa,
    Babe,
    Custom(String),
}

/// Chain-specific metadata for Ethereum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumMetadata {
    pub gas_used: u64,
    pub base_fee: u64,
    pub difficulty: u64,
    pub total_difficulty: u64,
}

/// Chain-specific metadata for Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMetadata {
    pub super_majority_root: u64,
    pub vote_account_stake: u64,
    pub total_active_stake: u64,
}

/// Chain-specific metadata for Substrate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateMetadata {
    pub authority_set_id: u64,
    pub validator_set_id: u64,
    pub consensus_version: u32,
}

/// Chain-specific metadata for Cosmos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosMetadata {
    pub validator_power: u64,
    pub total_voting_power: u64,
    pub app_version: u32,
}

/// Chain-specific metadata for NEAR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMetadata {
    pub validator_stake: u64,
    pub total_stake: u64,
    pub epoch_height: u64,
}

/// Block references for cross-chain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRefs {
    pub source_block: BlockId,
    pub target_block: BlockId,
    pub finality_block: BlockId,
    pub timestamp: u64,
    pub metadata: Option<serde_json::Value>,
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
            metadata: None,
        }
    }
}
