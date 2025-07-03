/*!
# State Error Handling

This module implements comprehensive error handling for the FROST protocol's state management,
providing structured error types, severity levels, and error recovery mechanisms.

## Core Components

### Error Severity
The severity system provides:
- Warning level (operation can continue)
- Error level (operation failed but retryable)
- Critical level (operation failed and not retryable)
- Severity comparison and ordering

### State Errors
Core state errors include:
- Transition errors
- Proof errors
- Block reference errors
- Root mismatch errors
- Chain-specific errors

### Proof Errors
Specialized proof error handling:
- Error categories
- Error context
- Retry guidance
- Error chaining

### Error Context
Contextual information includes:
- Chain identification
- Block information
- Error metadata
- Recovery options

## Architecture

The error system implements several key components:

1. **Error Severity**
   ```rust
   pub enum ErrorSeverity {
       Warning,
       Error,
       Critical,
   }
   ```
   - Severity levels
   - Comparison logic
   - Recovery guidance
   - Operation handling

2. **State Errors**
   ```rust
   pub enum StateError {
       InvalidTransition(String),
       InvalidProof(String),
       ProofVerificationFailed(String),
       InvalidBlockRef(String),
       RootMismatch { ... },
       ChainSpecific(String),
       Internal(String),
   }
   ```
   - Error types
   - Error messages
   - Error context
   - Recovery options

3. **Proof Errors**
   ```rust
   pub struct ProofError {
       category: ProofErrorCategory,
       severity: ErrorSeverity,
       message: String,
       context: Option<ChainErrorContext>,
       retry: RetryGuidance,
       cause: Option<Box<ProofError>>,
   }
   ```
   - Error categorization
   - Severity tracking
   - Context management
   - Retry handling

## Features

### Error Management
- Error categorization
- Severity tracking
- Context handling
- Recovery guidance

### Retry System
- Retry guidance
- Delay calculation
- Retry limits
- Alternative actions

### Context Handling
- Chain context
- Block context
- Error metadata
- Recovery options

### Error Recovery
- Recovery strategies
- Alternative actions
- Retry handling
- Error resolution

## Best Practices

### Error Handling
1. Error Creation
   - Proper categorization
   - Severity assessment
   - Context inclusion
   - Recovery options

2. Error Processing
   - Error analysis
   - Recovery planning
   - Retry handling
   - Alternative actions

3. Context Management
   - Chain information
   - Block details
   - Error metadata
   - Recovery data

4. Recovery Handling
   - Strategy selection
   - Retry management
   - Alternative paths
   - Resolution tracking

## Integration

### State System
- Error detection
- Error handling
- Recovery management
- State restoration

### Proof System
- Proof validation
- Error tracking
- Recovery options
- Proof management

### Chain System
- Chain context
- Error handling
- Recovery strategies
- Chain management

### Retry System
- Retry strategies
- Delay management
- Alternative paths
- Recovery tracking

## Performance Considerations

### Error Creation
- Efficient construction
- Context management
- Memory usage
- Resource sharing

### Error Processing
- Quick analysis
- Fast categorization
- Efficient handling
- Resource management

### Recovery Management
- Strategy selection
- Resource allocation
- Performance impact
- System stability

### Monitoring
- Error tracking
- Recovery metrics
- System health
- Resource usage

## Implementation Notes

### Error Categories
Error categories include:
- Validation errors
- Verification errors
- Expiration errors
- Revocation errors
- Cache errors
- System errors

### Severity Levels
Severity assessment considers:
- Operation impact
- Recovery options
- System stability
- Resource availability

### Recovery Strategies
Recovery handling includes:
- Retry mechanisms
- Alternative paths
- Resource management
- System stability

### Performance Impact
Performance considerations:
- Error overhead
- Recovery cost
- Resource usage
- System stability
*/

use thiserror::Error;
use crate::state::BlockRef;
use std::cmp::Ordering;
use std::fmt;
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - operation can continue
    Warning,
    /// Error - operation failed but can be retried
    Error,
    /// Critical - operation failed and should not be retried
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

    #[error("Invalid proof metadata: {0}")]
    InvalidProof(String),

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
            StateError::InvalidProof(_) => ErrorSeverity::Error,
            StateError::ProofVerificationFailed(_) => ErrorSeverity::Critical,
            StateError::InvalidBlockRef(_) => ErrorSeverity::Error,
            StateError::RootMismatch { .. } => ErrorSeverity::Critical,
            StateError::ChainSpecific(_) => ErrorSeverity::Warning,
            StateError::Internal(_) => ErrorSeverity::Critical,
        }
    }
}

/// Categories of proof errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofErrorCategory {
    /// Validation errors (malformed proof, invalid format)
    Validation,
    /// Verification errors (proof verification failed)
    Verification,
    /// Expiration errors (proof expired)
    Expiration,
    /// Revocation errors (proof was revoked)
    Revocation,
    /// Cache errors (cache inconsistency, corruption)
    Cache,
    /// System errors (resource exhaustion, internal errors)
    System,
}

/// Context for chain-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainErrorContext {
    /// Chain identifier
    pub chain_id: String,
    /// Block number
    pub block_number: u64,
    /// Error-specific metadata
    pub metadata: Option<serde_json::Value>,
}

/// Retry guidance for errors
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

/// Comprehensive proof error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofError {
    /// Error category
    pub category: ProofErrorCategory,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error message
    pub message: String,
    /// Chain-specific context
    pub context: Option<ChainErrorContext>,
    /// Retry guidance
    pub retry: RetryGuidance,
    /// Error cause chain
    pub cause: Option<Box<ProofError>>,
}

impl ProofError {
    /// Create a new proof error
    pub fn new(
        category: ProofErrorCategory,
        severity: ErrorSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            message: message.into(),
            context: None,
            retry: RetryGuidance {
                retryable: severity != ErrorSeverity::Critical,
                retry_after: None,
                max_retries: None,
                alternatives: Vec::new(),
            },
            cause: None,
        }
    }

    /// Add chain context to error
    pub fn with_context(mut self, context: ChainErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add retry guidance
    pub fn with_retry(mut self, guidance: RetryGuidance) -> Self {
        self.retry = guidance;
        self
    }

    /// Add error cause
    pub fn with_cause(mut self, cause: ProofError) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        self.retry.retryable
    }
}

impl std::error::Error for ProofError {}

impl fmt::Display for ProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error: {}", self.category, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " (chain: {}, block: {})", ctx.chain_id, ctx.block_number)?;
        }
        if let Some(cause) = &self.cause {
            write!(f, "\nCaused by: {}", cause)?;
        }
        Ok(())
    }
}

impl fmt::Display for ProofErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "Validation"),
            Self::Verification => write!(f, "Verification"),
            Self::Expiration => write!(f, "Expiration"),
            Self::Revocation => write!(f, "Revocation"),
            Self::Cache => write!(f, "Cache"),
            Self::System => write!(f, "System"),
        }
    }
} 