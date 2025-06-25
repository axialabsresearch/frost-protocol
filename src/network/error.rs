use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::io;

/// Network-related errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Peer error: {0}")]
    PeerError(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl NetworkError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::ConnectionFailed(_) |
            NetworkError::Timeout(_) |
            NetworkError::Io(_)
        )
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            NetworkError::ConnectionFailed(_) => ErrorSeverity::Warning,
            NetworkError::PeerError(_) => ErrorSeverity::Warning,
            NetworkError::TransportError(_) => ErrorSeverity::Error,
            NetworkError::ProtocolError(_) => ErrorSeverity::Error,
            NetworkError::Timeout(_) => ErrorSeverity::Warning,
            NetworkError::Encryption(_) => ErrorSeverity::Critical,
            NetworkError::Compression(_) => ErrorSeverity::Warning,
            NetworkError::Io(_) => ErrorSeverity::Warning,
            NetworkError::Serialization(_) => ErrorSeverity::Error,
            NetworkError::Config(_) => ErrorSeverity::Critical,
            NetworkError::Internal(_) => ErrorSeverity::Critical,
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