//! # FROST Protocol
//! 
//! FROST (Finality Reliant Optimized State Transition) Protocol is a cross-chain finality
//! verification system that enables secure state transitions across different blockchain
//! ecosystems.
//! 
//! ## Architecture
//! 
//! The protocol is built around four main components:
//! 
//! ### Finality Verification
//! The [`finality`] module provides chain-specific finality verification:
//! - Ethereum (PoW and Beacon Chain)
//! - Cosmos (Tendermint)
//! - Substrate (GRANDPA)
//! 
//! ```rust
//! use frost_protocol::finality::{FinalityConfig, EthereumVerifier, FinalityVerifier};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = FinalityConfig::default();
//! let verifier = EthereumVerifier::new(config);
//! # Ok(())
//! # }
//! ```
//! 
//! ### State Management
//! The [`state`] module handles state transitions and proofs:
//! - Block references
//! - State transitions
//! - Proof validation
//! 
//! ```rust
//! use frost_protocol::state::{BlockId, StateTransition};
//! 
//! let source = BlockId::default();
//! let target = BlockId::default();
//! let transition = StateTransition::new(source, target, vec![]);
//! ```
//! 
//! ### Network Layer
//! The [`network`] module provides P2P networking capabilities:
//! - Message broadcasting
//! - Peer discovery
//! - Connection management
//! 
//! ```rust
//! use frost_protocol::network::{NetworkConfig, BasicNetwork, NetworkProtocol};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = NetworkConfig::default();
//! let mut network = BasicNetwork::new(config);
//! network.start().await?;
//! # Ok(())
//! # }
//! ```
//! 
//! ### Message Routing
//! The [`routing`] module handles message routing across the network:
//! - Route discovery
//! - Message forwarding
//! - Routing table management
//! 
//! ```rust
//! use frost_protocol::routing::{RoutingConfig, BasicRouter};
//! use frost_protocol::network::{NetworkConfig, BasicNetwork};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RoutingConfig::default();
//! let network = BasicNetwork::new(NetworkConfig::default());
//! let router = BasicRouter::new(config, network);
//! # Ok(())
//! # }
//! ```
//! 
//! ## Error Handling
//! 
//! The protocol uses a comprehensive error handling system:
//! 
//! ```rust
//! use frost_protocol::{Result, Error};
//! 
//! fn example() -> Result<()> {
//!     // Handle various error types
//!     Ok(())
//! }
//! ```
//! 
//! ## Metrics
//! 
//! Each component provides basic metrics collection:
//! - Finality verification counts
//! - Network message statistics
//! - Routing performance
//! - Error rates
//! 
//! ## Testing
//! 
//! The protocol includes comprehensive test suites:
//! - Unit tests in each module
//! - Integration tests for component interaction
//! - Network simulation tests
//! 
//! ## Version
//! 
//! - Current version: 0.1.0
//! - Status: Initial Release (Basic Functionality)

pub mod finality;
pub mod message;
pub mod state;
pub mod network;
pub mod routing;
pub mod metrics;

// Re-exports
pub use finality::{FinalitySignal, FinalityMonitor};
pub use message::{FrostMessage, MessageType};
pub use state::{StateTransition, StateProof};
pub use network::{NetworkProtocol, NetworkConfig, BasicNetwork};
pub use routing::MessageRouter;

// Core types
pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

mod error;

