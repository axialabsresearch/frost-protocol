pub mod finality;
pub mod message;
pub mod state;
pub mod network;
pub mod routing;

// Re-exports
pub use finality::{FinalitySignal, FinalityMonitor};
pub use message::{FrostMessage, MessageType};
pub use state::{StateTransition, StateProof};
pub use network::NetworkProtocol;
pub use routing::MessageRouter;

// Core types
pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

