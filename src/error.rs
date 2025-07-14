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

    /// Connection denied error
    #[error("Connection denied error: {0}")]
    ConnectionDenied(String),

    /// Message processing error
    #[error("Message processing error: {0}")]
    MessageProcessing(String),

    /// Generic error with message
    #[error("Generic error: {0}")]
    Generic(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
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
            Error::ConnectionDenied(_) => false,
            Error::MessageProcessing(_) => false,
            Error::Generic(_) => false,
            Error::Custom(_) => false
        }
    }
}

impl From<libp2p::swarm::ConnectionDenied> for Error {
    fn from(err: libp2p::swarm::ConnectionDenied) -> Self {
        Error::ConnectionDenied(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}

impl From<crate::message::MessageError> for Error {
    fn from(err: crate::message::MessageError) -> Self {
        Error::Message(err.to_string())
    }
} 