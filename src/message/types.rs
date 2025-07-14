use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::SystemTime;
use crate::state::{ChainId, StateTransition, BlockRef};
use crate::finality::FinalitySignal;

/// Protocol message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// State transition message
    StateTransition,
    /// State proof message
    StateProof,
    /// Finality signal message
    FinalitySignal,
    /// Network discovery message
    Discovery,
    /// Batch message
    Batch,
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

/// Proof type and verification parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofMetadata {
    /// Type of proof being used
    pub proof_type: String,
    /// Version of the proof system
    pub proof_version: u32,
    /// Chain-specific verification parameters
    pub verification_params: Option<serde_json::Value>,
    /// Required security level (0-100)
    pub security_level: u8,
    /// Proof expiration time
    pub expires_at: Option<SystemTime>,
}

/// Additional message metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
    /// Processing metrics
    pub metrics: Option<MessageMetrics>,
}

/// Message processing metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetrics {
    /// When message processing started
    pub processing_start: SystemTime,
    /// Processing duration in milliseconds
    pub processing_duration_ms: Option<u64>,
    /// Number of validation attempts
    pub validation_attempts: u32,
    /// Size of message in bytes
    pub message_size_bytes: usize,
}

/// Core protocol message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Optional proof metadata
    pub proof_metadata: Option<ProofMetadata>,
}

/// Batch of messages for efficient processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMessage {
    /// Unique batch identifier
    pub batch_id: Uuid,
    /// Messages in the batch
    pub messages: Vec<FrostMessage>,
    /// Batch metadata
    pub metadata: MessageMetadata,
    /// Whether messages must be processed in order
    pub ordered: bool,
    /// Minimum success ratio (0.0-1.0) for batch to succeed
    pub min_success_ratio: f32,
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
            metadata: MessageMetadata {
                metrics: Some(MessageMetrics {
                    processing_start: SystemTime::now(),
                    processing_duration_ms: None,
                    validation_attempts: 0,
                    message_size_bytes: payload.len(),
                }),
                ..Default::default()
            },
            source,
            target,
            source_chain: None,
            target_chain: None,
            payload,
            state_transition: None,
            finality_signal: None,
            block_ref: None,
            proof_metadata: None,
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
        proof_metadata: Option<ProofMetadata>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: MessageMetadata {
                metrics: Some(MessageMetrics {
                    processing_start: SystemTime::now(),
                    processing_duration_ms: None,
                    validation_attempts: 0,
                    message_size_bytes: payload.len(),
                }),
                ..Default::default()
            },
            source,
            target,
            source_chain: Some(source_chain),
            target_chain: Some(target_chain),
            payload,
            state_transition,
            finality_signal,
            block_ref,
            proof_metadata,
        }
    }

    /// Create a new batch message
    pub fn new_batch(messages: Vec<FrostMessage>, ordered: bool, min_success_ratio: f32) -> BatchMessage {
        BatchMessage {
            batch_id: Uuid::new_v4(),
            metadata: MessageMetadata {
                metrics: Some(MessageMetrics {
                    processing_start: SystemTime::now(),
                    processing_duration_ms: None,
                    validation_attempts: 0,
                    message_size_bytes: messages.iter().map(|m| m.payload.len()).sum(),
                }),
                ..Default::default()
            },
            messages,
            ordered,
            min_success_ratio,
        }
    }

    /// Update processing metrics
    pub fn update_metrics(&mut self) {
        if let Some(metrics) = &mut self.metadata.metrics {
            metrics.validation_attempts += 1;
            metrics.processing_duration_ms = Some(
                SystemTime::now()
                    .duration_since(metrics.processing_start)
                    .unwrap_or_default()
                    .as_millis() as u64
            );
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
            MessageType::Batch => false, // Batch messages should use BatchMessage type
            _ => true,
        }
    }
}
