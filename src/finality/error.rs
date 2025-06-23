use thiserror::Error;
use crate::state::BlockRef;

/// Finality verification errors
#[derive(Error, Debug)]
pub enum FinalityError {
    /// Invalid finality signal
    #[error("Invalid finality signal: {0}")]
    InvalidSignal(String),

    /// Chain not synced
    #[error("Chain not synced: {0}")]
    NotSynced(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    ConsensusError(String),

    /// Validator error
    #[error("Validator error: {0}")]
    ValidatorError(String),

    /// Chain-specific error
    #[error("Chain error: {0}")]
    ChainError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl FinalityError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FinalityError::NotSynced(_) |
            FinalityError::ConsensusError(_) |
            FinalityError::ValidatorError(_)
        )
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Warning - operation can continue
    Warning,
    /// Error - operation should be retried
    Error,
    /// Critical - operation cannot continue
    Critical,
} 