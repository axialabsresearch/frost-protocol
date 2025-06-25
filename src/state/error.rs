use thiserror::Error;
use crate::state::BlockRef;

#[derive(Clone, Debug, Error)]
pub enum StateError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("State proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("Invalid block reference: {0}")]
    InvalidBlockRef(String),

    #[error("State root mismatch for block {block_ref:?}: expected {expected}, got {actual}")]
    RootMismatch {
        block_ref: BlockRef,
        expected: String,
        actual: String,
    },

    #[error("Chain specific error: {0}")]
    ChainSpecific(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl StateError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, StateError::ChainSpecific(_))
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            StateError::InvalidTransition(_) => ErrorSeverity::Error,
            StateError::ProofVerificationFailed(_) => ErrorSeverity::Critical,
            StateError::InvalidBlockRef(_) => ErrorSeverity::Error,
            StateError::RootMismatch { .. } => ErrorSeverity::Critical,
            StateError::ChainSpecific(_) => ErrorSeverity::Warning,
            StateError::Internal(_) => ErrorSeverity::Critical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
} 