#![allow(unused_imports)]

use thiserror::Error;
use crate::state::BlockRef;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Finality verification errors
#[derive(Clone, Error, Debug, Serialize, Deserialize)]
pub enum FinalityError {
    /// Invalid finality signal
    #[error("Invalid finality signal: {0}")]
    InvalidSignal(String),

    /// Chain not synced
    #[error("Chain not synced: {details}")]
    NotSynced {
        details: String,
        last_synced: Option<u64>,
        current_height: Option<u64>,
    },

    /// Consensus error
    #[error("Consensus error: {details}")]
    ConsensusError {
        details: String,
        required_power: u64,
        actual_power: u64,
    },

    /// Validator error
    #[error("Validator error: {details}")]
    ValidatorError {
        details: String,
        validator_count: Option<u32>,
    },

    /// Chain-specific error
    #[error("Chain error: {0}")]
    ChainError(String),

    /// Invalid chain
    #[error("Invalid chain: {chain_id}")]
    InvalidChain {
        chain_id: String,
        supported_chains: Vec<String>,
    },

    /// Network error
    #[error("Network error: {details}")]
    NetworkError {
        details: String,
        retryable: bool,
        retry_after: Option<Duration>,
    },

    /// Timeout error
    #[error("Timeout error for block {block_ref:?} after {timeout_secs:?}")]
    Timeout { 
        block_ref: BlockRef,
        timeout_secs: Duration,
        retry_count: u32,
    },

    /// Rate limit error
    #[error("Rate limit exceeded: {details}")]
    RateLimit {
        details: String,
        retry_after: Duration,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl FinalityError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FinalityError::NotSynced { .. } |
            FinalityError::ConsensusError { .. } |
            FinalityError::ValidatorError { .. } |
            FinalityError::NetworkError { retryable: true, .. } |
            FinalityError::RateLimit { .. }
        )
    }

    /// Get the recommended retry delay
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            FinalityError::NetworkError { retry_after, .. } => *retry_after,
            FinalityError::RateLimit { retry_after, .. } => Some(*retry_after),
            FinalityError::NotSynced { .. } => Some(Duration::from_secs(10)),
            FinalityError::ConsensusError { .. } => Some(Duration::from_secs(5)),
            _ => None,
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            FinalityError::InvalidSignal(_) => ErrorSeverity::Error,
            FinalityError::NotSynced { .. } => ErrorSeverity::Warning,
            FinalityError::ConsensusError { .. } => ErrorSeverity::Error,
            FinalityError::ValidatorError { .. } => ErrorSeverity::Error,
            FinalityError::ChainError(_) => ErrorSeverity::Critical,
            FinalityError::InvalidChain { .. } => ErrorSeverity::Critical,
            FinalityError::NetworkError { retryable, .. } => {
                if *retryable { ErrorSeverity::Warning } else { ErrorSeverity::Critical }
            },
            FinalityError::Timeout { retry_count, .. } => {
                if *retry_count < 3 { ErrorSeverity::Warning } else { ErrorSeverity::Error }
            },
            FinalityError::RateLimit { .. } => ErrorSeverity::Warning,
            FinalityError::Internal(_) => ErrorSeverity::Critical,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - operation can continue
    Warning,
    /// Error - operation should be retried
    Error,
    /// Critical - operation cannot continue
    Critical,
} 