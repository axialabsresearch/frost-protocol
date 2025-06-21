use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::message::FrostMessage;
use crate::network::{Peer, NetworkProtocol};
use crate::state::StateTransition;

use super::{
    ExtensionId,
    ExtensionManager,
    PeerEventType,
};

/// Hooks for protocol extensions to interact with core functionality
pub struct ExtensionHooks {
    manager: Arc<RwLock<dyn ExtensionManager>>,
    network: Arc<dyn NetworkProtocol>,
}

impl ExtensionHooks {
    /// Create new extension hooks
    pub fn new(
        manager: Arc<RwLock<dyn ExtensionManager>>,
        network: Arc<dyn NetworkProtocol>,
    ) -> Self {
        Self { manager, network }
    }

    /// Process message through extension pipeline
    pub async fn process_message(&self, message: &mut FrostMessage) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        // Pre-process
        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.pre_process_message(message).await?;
            }
        }

        // Handle message
        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.handle_message(message).await?;
            }
        }

        // Post-process
        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.post_process_message(message).await?;
            }
        }

        Ok(())
    }

    /// Handle state transition through extensions
    pub async fn handle_state_transition(&self, transition: &StateTransition) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in extensions {
            if let Some(extension) = manager.get_extension(&id).await? {
                extension.handle_state_transition(transition).await?;
            }
        }

        Ok(())
    }

    /// Handle peer events through extensions
    pub async fn handle_peer_event(&self, peer: &Peer, event: PeerEventType) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in extensions {
            if let Some(extension) = manager.get_extension(&id).await? {
                extension.handle_peer_event(peer, event.clone()).await?;
            }
        }

        Ok(())
    }

    /// Get network protocol instance
    pub fn network(&self) -> Arc<dyn NetworkProtocol> {
        self.network.clone()
    }

    /// Get extension manager instance
    pub fn manager(&self) -> Arc<RwLock<dyn ExtensionManager>> {
        self.manager.clone()
    }
}

/// Extension context provided to protocol extensions
pub struct ExtensionContext {
    /// Extension's unique ID
    pub id: ExtensionId,
    /// Hooks for interacting with core protocol
    pub hooks: Arc<ExtensionHooks>,
}

impl ExtensionContext {
    /// Create new extension context
    pub fn new(id: ExtensionId, hooks: Arc<ExtensionHooks>) -> Self {
        Self { id, hooks }
    }

    /// Get network protocol instance
    pub fn network(&self) -> Arc<dyn NetworkProtocol> {
        self.hooks.network()
    }

    /// Get extension manager instance
    pub fn manager(&self) -> Arc<RwLock<dyn ExtensionManager>> {
        self.hooks.manager()
    }
} 