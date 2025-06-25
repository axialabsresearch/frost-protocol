//! # Finality Verification
//! 
//! The finality module provides chain-specific finality verification for different blockchain
//! ecosystems. It implements a unified interface for verifying block finality across
//! Ethereum, Cosmos, and Substrate chains.
//! 
//! ## Usage
//! 
//! ### Ethereum Finality
//! 
//! ```rust
//! use frost_protocol::finality::{FinalityConfig, EthereumVerifier, FinalityVerifier};
//! use std::time::Duration;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = FinalityConfig {
//!     min_confirmations: 6,
//!     finality_timeout: Duration::from_secs(30),
//!     basic_params: Default::default(),
//! };
//! 
//! let verifier = EthereumVerifier::new(config);
//! # Ok(())
//! # }
//! ```
//! 
//! ### Cosmos Finality
//! 
//! ```rust
//! use frost_protocol::finality::{FinalityConfig, CosmosVerifier, FinalityVerifier};
//! use std::collections::HashMap;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut params = HashMap::new();
//! params.insert("min_signatures".to_string(), serde_json::json!(8));
//! 
//! let config = FinalityConfig {
//!     min_confirmations: 2,
//!     finality_timeout: std::time::Duration::from_secs(30),
//!     basic_params: params,
//! };
//! 
//! let verifier = CosmosVerifier::new(config);
//! # Ok(())
//! # }
//! ```
//! 
//! ### Substrate Finality
//! 
//! ```rust
//! use frost_protocol::finality::{FinalityConfig, SubstrateVerifier, FinalityVerifier};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = FinalityConfig::default();
//! let verifier = SubstrateVerifier::new(config);
//! # Ok(())
//! # }
//! ```
//! 
//! ## Architecture
//! 
//! The finality verification system consists of several key components:
//! 
//! ### Verifier Trait
//! 
//! The [`FinalityVerifier`] trait defines the core interface for finality verification:
//! - `verify_finality`: Verifies block finality
//! - `get_metrics`: Retrieves verification metrics
//! - `update_config`: Updates verifier configuration
//! 
//! ### Chain-Specific Verifiers
//! 
//! - [`EthereumVerifier`]: Handles both PoW and Beacon Chain finality
//! - [`CosmosVerifier`]: Implements Tendermint consensus verification
//! - [`SubstrateVerifier`]: Manages GRANDPA finality verification
//! 
//! ### Finality Signals
//! 
//! The [`FinalitySignal`] enum represents chain-specific finality information:
//! - Block numbers and hashes
//! - Confirmation counts
//! - Validator signatures
//! - Chain metadata
//! 
//! ### Configuration
//! 
//! The [`FinalityConfig`] struct provides configurable parameters:
//! - Minimum confirmations
//! - Timeout durations
//! - Chain-specific parameters
//! 
//! ### Metrics
//! 
//! Basic metrics collection through [`BasicMetrics`]:
//! - Total blocks verified
//! - Failed verifications
//! 
//! ### Error Handling
//! 
//! Comprehensive error types through [`FinalityError`]:
//! - Invalid signals
//! - Chain sync issues
//! - Consensus errors
//! - Validator errors
//! 
//! ## Example
//! 
//! ```rust
//! use frost_protocol::finality::{
//!     FinalityConfig,
//!     EthereumVerifier,
//!     FinalityVerifier,
//!     FinalitySignal,
//! };
//! use frost_protocol::state::BlockRef;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create verifier
//! let config = FinalityConfig::default();
//! let verifier = EthereumVerifier::new(config);
//! 
//! // Create finality signal
//! let signal = FinalitySignal::Ethereum {
//!     block_number: 100,
//!     block_hash: [0; 32],
//!     confirmations: 10,
//!     finality_type: frost_protocol::finality::EthereumFinalityType::Confirmations,
//!     metadata: None,
//! };
//! 
//! // Verify finality
//! let block_ref = BlockRef::new("eth", 100);
//! let is_final = verifier.verify_finality(&block_ref, &signal).await?;
//! 
//! // Check metrics
//! let metrics = verifier.get_metrics().await;
//! println!("Total blocks verified: {}", metrics.total_blocks_verified);
//! # Ok(())
//! # }
//! ```

mod verifier;
mod signal;
mod error;
mod monitor;

pub use verifier::{FinalityVerifier, FinalityConfig, BasicMetrics};
pub use signal::{
    FinalitySignal,
    EthereumFinalityType,
    EthereumMetadata,
    CosmosMetadata,
    SubstrateMetadata,
};
pub use error::FinalityError;
pub use monitor::FinalityMonitor;

// Chain-specific verifiers
pub use verifier::{EthereumVerifier, CosmosVerifier, SubstrateVerifier};
