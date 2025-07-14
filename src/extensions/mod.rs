#![allow(unused_imports)]
#![allow(unused_variables)]

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::any::Any;
use anyhow::{Result, anyhow};

use crate::message::FrostMessage;
use crate::network::{Peer, NetworkProtocol};
use crate::state::{StateTransition, StateProof};
use crate::finality::FinalitySignal;

// Module declarations
pub mod errors;
pub mod dependency;
pub mod compatibility;
pub mod traits;
pub mod registry;
pub mod manager;
pub mod hooks;

// Re-exports
pub use errors::{ExtensionError, ExtensionResult};
pub use dependency::DependencyResolver;
pub use compatibility::CompatibilityChecker;
pub use traits::ProtocolExtension;
pub use registry::ExtensionRegistry;
pub use manager::DefaultExtensionManager;
pub use hooks::ExtensionHooks;

/// Unique identifier for protocol extensions
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionId(pub String);

impl ExtensionId {
    pub fn new(name: &str, version: &str) -> Self {
        Self(format!("{}@{}", name, version))
    }
}

/// Metadata for protocol extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<ExtensionId>,
    pub capabilities: Vec<String>,
}

/// Extension lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionState {
    Registered,
    Initialized,
    Active,
    Suspended,
    Failed,
}

/// Configuration for protocol extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub enabled: bool,
    pub priority: i32,
    pub parameters: std::collections::HashMap<String, String>,
}

/// Types of peer events that extensions can handle
#[derive(Debug, Clone)]
pub enum PeerEventType {
    Connected,
    Disconnected,
    StateChanged(String),
}

/// Metrics for protocol extensions
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtensionMetrics {
    pub messages_processed: u64,
    pub state_transitions: u64,
    pub errors: u64,
}

/// Extension capabilities
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionCapability {
    MessageValidation,
    StateVerification,
    FinalityVerification,
    NetworkRouting,
    Custom(String),
}

/// Manager for protocol extensions
#[async_trait]
pub trait ExtensionManager: Send + Sync {
    /// Register a new extension
    async fn register_extension(
        &mut self,
        extension: Box<dyn ProtocolExtension>,
        config: ExtensionConfig,
    ) -> Result<ExtensionId>;

    /// Unregister an extension
    async fn unregister_extension(&mut self, id: &ExtensionId) -> Result<()>;

    /// Get extension by ID
    async fn get_extension(&self, id: &ExtensionId) -> Result<Option<Arc<dyn ProtocolExtension>>>;

    /// List all registered extensions
    async fn list_extensions(&self) -> Result<Vec<(ExtensionId, ExtensionMetadata)>>;

    /// Enable extension
    async fn enable_extension(&mut self, id: &ExtensionId) -> Result<()>;

    /// Disable extension
    async fn disable_extension(&mut self, id: &ExtensionId) -> Result<()>;

    /// Get extension dependencies
    async fn get_dependencies(&self, id: &ExtensionId) -> Result<Vec<ExtensionId>>;

    /// Validate extension compatibility
    async fn validate_compatibility(&self, extension: &dyn ProtocolExtension) -> Result<()>;

    /// Clean up resources for all extensions
    async fn cleanup_resources(&mut self) -> Result<()>;
} 