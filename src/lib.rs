#![cfg_attr(not(feature = "std"), no_std)]

pub mod finality;
pub mod message;
pub mod state;
pub mod network;
pub mod routing;
pub mod metrics;
pub mod extensions;
pub mod substrate;

// Re-exports
pub use finality::{FinalitySignal, FinalityMonitor};
pub use message::{FrostMessage, MessageType};
pub use state::{StateTransition, StateProof};
pub use network::{NetworkProtocol, NetworkConfig, BasicNetwork};
pub use routing::MessageRouter;

// Core types
pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

pub mod error;

#[cfg(feature = "std")]
pub use substrate::*;
