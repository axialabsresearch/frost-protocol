use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::io;

/// Network-related errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Failed to bootstrap network: {0}")]
    BootstrapFailed(String),

    #[error("Failed to send event: {0}")]
    EventSendFailed(String),

    #[error("Failed to connect to peer: {0}")]
    ConnectionFailed(String),

    #[error("Failed to disconnect from peer: {0}")]
    DisconnectionFailed(String),

    #[error("Failed to send message: {0}")]
    MessageSendFailed(String),

    #[error("Failed to receive message: {0}")]
    MessageReceiveFailed(String),

    #[error("Invalid peer address: {0}")]
    InvalidPeerAddress(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl NetworkError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::ConnectionFailed(_) |
            NetworkError::MessageSendFailed(_) |
            NetworkError::MessageReceiveFailed(_) |
            NetworkError::DiscoveryError(_) |
            NetworkError::Timeout(_)
        )
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            NetworkError::BootstrapFailed(_) => ErrorSeverity::Critical,
            NetworkError::EventSendFailed(_) => ErrorSeverity::Error,
            NetworkError::ConnectionFailed(_) => ErrorSeverity::Warning,
            NetworkError::DisconnectionFailed(_) => ErrorSeverity::Warning,
            NetworkError::MessageSendFailed(_) => ErrorSeverity::Warning,
            NetworkError::MessageReceiveFailed(_) => ErrorSeverity::Warning,
            NetworkError::InvalidPeerAddress(_) => ErrorSeverity::Error,
            NetworkError::ProtocolError(_) => ErrorSeverity::Error,
            NetworkError::TransportError(_) => ErrorSeverity::Error,
            NetworkError::DiscoveryError(_) => ErrorSeverity::Warning,
            NetworkError::SecurityError(_) => ErrorSeverity::Critical,
            NetworkError::Timeout(_) => ErrorSeverity::Warning,
            NetworkError::Internal(_) => ErrorSeverity::Critical,
        }
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