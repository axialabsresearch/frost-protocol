use thiserror::Error;
use crate::state::StateError;
use crate::finality::FinalityError;

/// Error types for message handling
#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("State error: {0}")]
    State(#[from] StateError),

    #[error("Finality error: {0}")]
    Finality(#[from] FinalityError),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl MessageError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MessageError::Network(_) | 
            MessageError::Queue(_) |
            MessageError::Processing(_)
        )
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MessageError::InvalidFormat(_) => ErrorSeverity::Error,
            MessageError::ValidationFailed(_) => ErrorSeverity::Error,
            MessageError::State(_) => ErrorSeverity::Critical,
            MessageError::Finality(_) => ErrorSeverity::Critical,
            MessageError::Processing(_) => ErrorSeverity::Warning,
            MessageError::Queue(_) => ErrorSeverity::Warning,
            MessageError::Network(_) => ErrorSeverity::Warning,
            MessageError::Internal(_) => ErrorSeverity::Critical,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
} 