use async_trait::async_trait;
use crate::message::{FrostMessage, MessageType, MessageError};
use crate::Result;

/// Handler for FROST Protocol messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process an incoming message
    async fn handle_message(&self, message: FrostMessage) -> Result<MessageStatus>;
    
    /// Queue a message for processing
    async fn queue_message(&self, message: FrostMessage) -> Result<()>;
    
    /// Get status of a message
    async fn message_status(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
    
    /// Retry a failed message
    async fn retry_message(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
}

/// Status of message processing
#[derive(Debug, Clone)]
pub enum MessageStatus {
    Queued {
        position: u64,
        estimated_time: std::time::Duration,
    },
    Processing {
        started_at: std::time::SystemTime,
        progress: f32,
    },
    Completed {
        completed_at: std::time::SystemTime,
        result: MessageResult,
    },
    Failed {
        error: MessageError,
        can_retry: bool,
    },
}

/// Result of message processing
#[derive(Debug, Clone)]
pub struct MessageResult {
    pub success: bool,
    pub processing_time: std::time::Duration,
    pub metadata: serde_json::Value,
} 