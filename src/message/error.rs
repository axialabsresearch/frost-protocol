use thiserror::Error;
use uuid::Uuid;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Message error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - operation can continue
    Warning,
    /// Error - operation failed but can be retried
    Error,
    /// Critical - operation failed and should not be retried
    Critical,
}

/// Message validation stages where errors can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorStage {
    /// Pre-validation stage
    PreValidation,
    /// Proof validation stage
    ProofValidation,
    /// State validation stage
    StateValidation,
    /// Post-validation stage
    PostValidation,
    /// Transformation stage
    Transformation,
    /// Message handling stage
    Handling,
}

/// Retry guidance for message errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryGuidance {
    /// Whether the operation can be retried
    pub retryable: bool,
    /// Suggested delay before retry
    pub retry_after: Option<Duration>,
    /// Maximum number of retries
    pub max_retries: Option<u32>,
    /// Alternative actions to consider
    pub alternatives: Vec<String>,
}

/// Message processing errors
#[derive(Error, Debug, Clone)]
pub enum MessageError {
    #[error("Message validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Message transformation failed: {0}")]
    TransformationFailed(String),

    #[error("Message handling failed: {0}")]
    HandlingFailed(String),

    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("State validation failed: {0}")]
    StateValidationFailed(String),

    #[error("Batch validation failed: success ratio {success_ratio} below required {required_ratio}")]
    BatchValidationFailed {
        batch_id: Uuid,
        success_ratio: f32,
        required_ratio: f32,
    },

    #[error("Message timeout after {timeout_secs:?}s")]
    Timeout {
        timeout_secs: Duration,
        retry_count: u32,
    },

    #[error("Chain-specific error: {chain_id} - {details}")]
    ChainSpecific {
        chain_id: String,
        details: String,
        metadata: Option<serde_json::Value>,
    },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl MessageError {
    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ValidationFailed(_) => ErrorSeverity::Error,
            Self::InvalidFormat(_) => ErrorSeverity::Error,
            Self::TransformationFailed(_) => ErrorSeverity::Error,
            Self::HandlingFailed(_) => ErrorSeverity::Error,
            Self::ProofVerificationFailed(_) => ErrorSeverity::Critical,
            Self::StateValidationFailed(_) => ErrorSeverity::Critical,
            Self::BatchValidationFailed { .. } => ErrorSeverity::Error,
            Self::Timeout { retry_count, .. } => {
                if *retry_count > 3 {
                    ErrorSeverity::Critical
                } else {
                    ErrorSeverity::Error
                }
            }
            Self::ChainSpecific { .. } => ErrorSeverity::Warning,
            Self::Internal(_) => ErrorSeverity::Critical,
        }
    }

    /// Get error stage
    pub fn stage(&self) -> ErrorStage {
        match self {
            Self::ValidationFailed(_) => ErrorStage::PreValidation,
            Self::InvalidFormat(_) => ErrorStage::PreValidation,
            Self::TransformationFailed(_) => ErrorStage::Transformation,
            Self::HandlingFailed(_) => ErrorStage::Handling,
            Self::ProofVerificationFailed(_) => ErrorStage::ProofValidation,
            Self::StateValidationFailed(_) => ErrorStage::StateValidation,
            Self::BatchValidationFailed { .. } => ErrorStage::PostValidation,
            Self::Timeout { .. } => ErrorStage::Handling,
            Self::ChainSpecific { .. } => ErrorStage::Handling,
            Self::Internal(_) => ErrorStage::Handling,
        }
    }

    /// Get retry guidance
    pub fn retry_guidance(&self) -> RetryGuidance {
        match self {
            Self::ValidationFailed(_) => RetryGuidance {
                retryable: true,
                retry_after: Some(Duration::from_secs(1)),
                max_retries: Some(3),
                alternatives: vec!["Check message format".into()],
            },
            Self::InvalidFormat(_) => RetryGuidance {
                retryable: false,
                retry_after: None,
                max_retries: None,
                alternatives: vec!["Fix message format".into()],
            },
            Self::TransformationFailed(_) => RetryGuidance {
                retryable: true,
                retry_after: Some(Duration::from_secs(1)),
                max_retries: Some(3),
                alternatives: vec!["Try different transformation".into()],
            },
            Self::HandlingFailed(_) => RetryGuidance {
                retryable: true,
                retry_after: Some(Duration::from_secs(5)),
                max_retries: Some(5),
                alternatives: vec!["Check handler availability".into()],
            },
            Self::ProofVerificationFailed(_) => RetryGuidance {
                retryable: false,
                retry_after: None,
                max_retries: None,
                alternatives: vec!["Generate new proof".into()],
            },
            Self::StateValidationFailed(_) => RetryGuidance {
                retryable: false,
                retry_after: None,
                max_retries: None,
                alternatives: vec!["Check state consistency".into()],
            },
            Self::BatchValidationFailed { .. } => RetryGuidance {
                retryable: true,
                retry_after: Some(Duration::from_secs(10)),
                max_retries: Some(2),
                alternatives: vec!["Retry failed messages only".into()],
            },
            Self::Timeout { retry_count, .. } => RetryGuidance {
                retryable: *retry_count < 3,
                retry_after: Some(Duration::from_secs(5 * (retry_count + 1) as u64)),
                max_retries: Some(3),
                alternatives: vec!["Check network connectivity".into()],
            },
            Self::ChainSpecific { .. } => RetryGuidance {
                retryable: true,
                retry_after: Some(Duration::from_secs(30)),
                max_retries: Some(5),
                alternatives: vec!["Check chain status".into()],
            },
            Self::Internal(_) => RetryGuidance {
                retryable: false,
                retry_after: None,
                max_retries: None,
                alternatives: vec!["Report issue".into()],
            },
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        self.retry_guidance().retryable
    }
} 