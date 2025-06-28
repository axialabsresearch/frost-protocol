use thiserror::Error;
use crate::state::BlockRef;
use std::cmp::Ordering;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

impl PartialOrd for ErrorSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ErrorSeverity {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ErrorSeverity::Critical, ErrorSeverity::Critical) => Ordering::Equal,
            (ErrorSeverity::Critical, _) => Ordering::Greater,
            (ErrorSeverity::Error, ErrorSeverity::Critical) => Ordering::Less,
            (ErrorSeverity::Error, ErrorSeverity::Error) => Ordering::Equal,
            (ErrorSeverity::Error, ErrorSeverity::Warning) => Ordering::Greater,
            (ErrorSeverity::Warning, ErrorSeverity::Warning) => Ordering::Equal,
            (ErrorSeverity::Warning, _) => Ordering::Less,
        }
    }
}

/// State management errors
#[derive(Clone, Error, Debug)]
pub enum StateError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("State proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("Invalid block reference: {0}")]
    InvalidBlockRef(String),

    #[error("State root mismatch for block {block_ref}: expected {expected}, got {actual}")]
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