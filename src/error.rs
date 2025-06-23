use thiserror::Error;

/// Core protocol error type
#[derive(Error, Debug)]
pub enum Error {
    /// Message error
    #[error("Message error: {0}")]
    Message(String),

    /// State error
    #[error("State error: {0}")]
    State(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Routing error
    #[error("Routing error: {0}")]
    Routing(String),

    /// Finality error
    #[error("Finality error: {0}")]
    Finality(#[from] crate::finality::FinalityError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl Error {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Message(_) => true,
            Error::State(_) => false,
            Error::Network(_) => true,
            Error::Routing(_) => true,
            Error::Finality(e) => e.is_retryable(),
            Error::Io(_) => true,
            Error::Other(_) => false,
        }
    }
} 