mod types;
mod handler;
mod validation;
mod error;

pub use types::{FrostMessage, MessageType, MessageMetadata};
pub use handler::MessageHandler;
pub use validation::MessageValidator;
pub use error::MessageError;

use crate::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

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
}

/// Core protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrostMessage {
    /// Message type
    pub msg_type: MessageType,
    /// Message payload
    pub payload: Vec<u8>,
    /// Source node ID
    pub source: String,
    /// Target node ID (if any)
    pub target: Option<String>,
    /// Message timestamp
    pub timestamp: u64,
    /// Message version
    pub version: u16,
}

impl FrostMessage {
    /// Create a new message
    pub fn new(
        msg_type: MessageType,
        payload: Vec<u8>,
        source: String,
        target: Option<String>,
    ) -> Self {
        Self {
            msg_type,
            payload,
            source,
            target,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            version: 0,
        }
    }

    /// Validate message
    pub fn validate(&self) -> bool {
        // Basic validation for v0
        !self.payload.is_empty() && !self.source.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_validation() {
        let msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![1, 2, 3],
            "node1".to_string(),
            None,
        );
        assert!(msg.validate());

        let invalid_msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![],
            "".to_string(),
            None,
        );
        assert!(!invalid_msg.validate());
    }
}
