use thiserror::Error;
use crate::state::BlockRef;

/// Error types for finality operations
#[derive(Debug, Error)]
pub enum FinalityError {
    #[error("Finality timeout for block {block_ref:?} after {timeout_secs} seconds")]
    Timeout {
        block_ref: BlockRef,
        timeout_secs: u64,
    },

    #[error("Invalid finality signal: {0}")]
    InvalidSignal(String),

    #[error("Chain specific error: {0}")]
    ChainSpecific(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl FinalityError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FinalityError::Timeout { .. } |
            FinalityError::Network(_) |
            FinalityError::ChainSpecific(_)
        )
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            FinalityError::Timeout { .. } => ErrorSeverity::Warning,
            FinalityError::InvalidSignal(_) => ErrorSeverity::Error,
            FinalityError::ChainSpecific(_) => ErrorSeverity::Warning,
            FinalityError::Network(_) => ErrorSeverity::Warning,
            FinalityError::Consensus(_) => ErrorSeverity::Critical,
            FinalityError::VerificationFailed(_) => ErrorSeverity::Error,
            FinalityError::Internal(_) => ErrorSeverity::Critical,
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