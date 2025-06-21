use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};

use crate::message::FrostMessage;
use crate::network::{Peer, NetworkProtocol};
use crate::state::StateTransition;

/// Unique identifier for protocol extensions
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionId(String);

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
    pub priority: u32,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Core trait for implementing protocol extensions
#[async_trait]
pub trait ProtocolExtension: Send + Sync {
    /// Get extension metadata
    fn metadata(&self) -> &ExtensionMetadata;

    /// Initialize the extension
    async fn initialize(&mut self, config: ExtensionConfig) -> Result<()>;

    /// Start the extension
    async fn start(&mut self) -> Result<()>;

    /// Stop the extension
    async fn stop(&mut self) -> Result<()>;

    /// Handle incoming messages
    async fn handle_message(&self, message: &FrostMessage) -> Result<()>;

    /// Pre-process outgoing messages
    async fn pre_process_message(&self, message: &mut FrostMessage) -> Result<()>;

    /// Post-process messages
    async fn post_process_message(&self, message: &FrostMessage) -> Result<()>;

    /// Handle state transitions
    async fn handle_state_transition(&self, transition: &StateTransition) -> Result<()>;

    /// Handle peer events
    async fn handle_peer_event(&self, peer: &Peer, event_type: PeerEventType) -> Result<()>;

    /// Get extension state
    fn state(&self) -> ExtensionState;

    /// Get extension metrics
    fn metrics(&self) -> ExtensionMetrics;
}

/// Types of peer events that extensions can handle
#[derive(Debug, Clone)]
pub enum PeerEventType {
    Connected,
    Disconnected,
    MessageReceived,
    MessageSent,
    Error(String),
}

/// Metrics for protocol extensions
#[derive(Debug, Clone, Default)]
pub struct ExtensionMetrics {
    pub messages_processed: u64,
    pub processing_time_ms: u64,
    pub errors: u64,
    pub custom_metrics: HashMap<String, f64>,
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
    fn get_dependencies(&self, id: &ExtensionId) -> Result<Vec<ExtensionId>>;

    /// Validate extension compatibility
    fn validate_compatibility(&self, extension: &dyn ProtocolExtension) -> Result<()>;
}

mod manager;
mod registry;
mod hooks;
mod compatibility;

pub use manager::DefaultExtensionManager;
pub use registry::ExtensionRegistry;
pub use hooks::ExtensionHooks; 