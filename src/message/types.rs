use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::state::{ChainId, StateTransition, BlockRefs};
use crate::finality::FinalitySignal;

/// Core message type for FROST Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrostMessage {
    // Core Identity
    pub id: Uuid,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    
    // State Transition
    pub state_transition: StateTransition,
    pub finality_signal: FinalitySignal,
    pub block_refs: BlockRefs,
    
    // Message Data
    pub payload: Vec<u8>,
    pub message_type: MessageType,
    pub nonce: u64,
    pub timestamp: u64,
    
    // Metadata
    pub metadata: MessageMetadata,
}

/// Types of messages in the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    StateTransition,
    ProofGeneration,
    ProofVerification,
    FinalityUpdate,
    NetworkSync,
    Custom(String),
}

/// Additional message metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub version: u32,
    pub priority: MessagePriority,
    pub retry_count: u32,
    pub chain_specific: Option<serde_json::Value>,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl FrostMessage {
    /// Create a new message
    pub fn new(
        from_chain: ChainId,
        to_chain: ChainId,
        state_transition: StateTransition,
        message_type: MessageType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_chain,
            to_chain,
            state_transition,
            finality_signal: FinalitySignal::Ethereum { block_number: 0, confirmations: 0 }, // Default
            block_refs: BlockRefs::default(),
            payload: Vec::new(),
            message_type,
            nonce: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: MessageMetadata::default(),
        }
    }
}
