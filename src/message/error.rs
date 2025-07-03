/*!
# Message Error Module

This module provides comprehensive error handling for the FROST protocol's messaging
system. It defines error types, severity levels, retry strategies, and error
recovery mechanisms.

## Core Components

### Error Types
- Validation errors
- Format errors
- Processing errors
- Chain errors

### Error Severity
- Warning level
- Error level
- Critical level
- Stage tracking

### Retry Handling
- Retry guidance
- Backoff strategy
- Alternative actions
- Recovery paths

## Architecture

The error system consists of several key components:

1. **Message Errors**
   ```rust
   pub enum MessageError {
       ValidationFailed(String),
       InvalidFormat(String),
       TransformationFailed(String),
       HandlingFailed(String),
       // ...
   }
   ```
   - Error categories
   - Error details
   - Recovery info
   - Chain context

2. **Error Severity**
   ```rust
   pub enum ErrorSeverity {
       Warning,
       Error,
       Critical,
   }
   ```
   - Impact levels
   - Recovery paths
   - Handling rules
   - Resource impact

3. **Retry Guidance**
   ```rust
   pub struct RetryGuidance {
       retryable: bool,
       retry_after: Option<Duration>,
       max_retries: Option<u32>,
       alternatives: Vec<String>,
   }
   ```
   - Retry rules
   - Timing control
   - Retry limits
   - Alternatives

## Features

### Error Classification
- Type-based handling
- Severity levels
- Stage tracking
- Chain context

### Retry Management
- Retry decisions
- Backoff timing
- Attempt limits
- Alternative paths

### Recovery Strategy
- Error recovery
- Alternative actions
- Resource cleanup
- State restoration

### Chain Handling
- Chain-specific errors
- Cross-chain issues
- State recovery
- Resource cleanup

## Best Practices

1. **Error Handling**
   - Proper classification
   - Severity assessment
   - Recovery planning
   - Resource cleanup

2. **Retry Strategy**
   - Appropriate timing
   - Attempt limits
   - Resource impact
   - State handling

3. **Recovery Process**
   - State restoration
   - Resource cleanup
   - Chain recovery
   - User feedback

4. **Chain Management**
   - Chain isolation
   - State handling
   - Resource cleanup
   - Cross-chain recovery

## Integration

The error system integrates with:
1. Message handling
2. Chain management
3. State transitions
4. Protocol operations
*/

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