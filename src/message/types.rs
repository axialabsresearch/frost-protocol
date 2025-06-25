use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::SystemTime;
use crate::state::{ChainId, StateTransition, BlockRef};
use crate::finality::FinalitySignal;

/// Protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// State transition message
    StateTransition,
    /// State proof message
    StateProof,
    /// Finality signal message
    FinalitySignal,
    /// Network discovery message
    Discovery,
    /// Custom message type
    Custom(String),
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

/// Additional message metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Protocol version
    pub version: u16,
    /// Message priority
    pub priority: MessagePriority,
    /// Retry count for failed messages
    pub retry_count: u32,
    /// Chain-specific metadata
    pub chain_metadata: Option<serde_json::Value>,
    /// Custom metadata fields
    pub custom_metadata: Option<serde_json::Value>,
}

/// Core protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrostMessage {
    // Core Identity
    /// Unique message identifier
    pub id: Uuid,
    /// Message type
    pub msg_type: MessageType,
    /// Message timestamp
    pub timestamp: u64,
    /// Message metadata
    pub metadata: MessageMetadata,

    // Network Routing
    /// Source node ID
    pub source: String,
    /// Target node ID (if any)
    pub target: Option<String>,

    // Chain Information
    /// Source chain ID
    pub source_chain: Option<ChainId>,
    /// Target chain ID
    pub target_chain: Option<ChainId>,
    
    // Message Content
    /// Raw message payload
    pub payload: Vec<u8>,
    /// Optional state transition
    pub state_transition: Option<StateTransition>,
    /// Optional finality signal
    pub finality_signal: Option<FinalitySignal>,
    /// Optional block reference
    pub block_ref: Option<BlockRef>,
}

impl FrostMessage {
    /// Create a new basic network message
    pub fn new(
        msg_type: MessageType,
        payload: Vec<u8>,
        source: String,
        target: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: MessageMetadata::default(),
            source,
            target,
            source_chain: None,
            target_chain: None,
            payload,
            state_transition: None,
            finality_signal: None,
            block_ref: None,
        }
    }

    /// Create a new chain-specific message
    pub fn new_chain_message(
        msg_type: MessageType,
        payload: Vec<u8>,
        source: String,
        target: Option<String>,
        source_chain: ChainId,
        target_chain: ChainId,
        state_transition: Option<StateTransition>,
        finality_signal: Option<FinalitySignal>,
        block_ref: Option<BlockRef>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: MessageMetadata::default(),
            source,
            target,
            source_chain: Some(source_chain),
            target_chain: Some(target_chain),
            payload,
            state_transition,
            finality_signal,
            block_ref,
        }
    }

    /// Validate basic message properties
    pub fn validate(&self) -> bool {
        // Basic validation
        if self.payload.is_empty() || self.source.is_empty() {
            return false;
        }

        // Chain-specific validation
        match self.msg_type {
            MessageType::StateTransition | MessageType::StateProof => {
                self.source_chain.is_some() && self.target_chain.is_some()
            }
            MessageType::FinalitySignal => {
                self.source_chain.is_some() && self.finality_signal.is_some()
            }
            _ => true,
        }
    }
}
