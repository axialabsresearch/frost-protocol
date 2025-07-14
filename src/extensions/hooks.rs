use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::message::FrostMessage;
use crate::network::{Peer, NetworkProtocol};
use crate::state::{StateProof, BlockRef, ChainId};
use crate::finality::{FinalitySignal, FinalityVerifier};

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

    /// Pre-validate message
    pub async fn pre_validate(&self, message: &mut FrostMessage) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.pre_process_message(message).await?;
            }
        }
        Ok(())
    }

    /// Validate proof
    pub async fn validate_proof(&self, message: &FrostMessage) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                if let Some(state_transition) = &message.state_transition {
                    extension.handle_state_transition(state_transition).await?;
                }
            }
        }
        Ok(())
    }

    /// Validate state
    pub async fn validate_state(&self, message: &FrostMessage) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.handle_message(message).await?;
            }
        }
        Ok(())
    }

    /// Post-validate message
    pub async fn post_validate(&self, message: &FrostMessage) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.post_process_message(message).await?;
            }
        }
        Ok(())
    }

    /// Handle state proof verification
    pub async fn verify_state_proof(&self, proof: &StateProof) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                extension.handle_state_transition(&proof.transition).await?;
            }
        }
        Ok(())
    }

    /// Handle finality signal
    pub async fn verify_finality(&self, signal: &FinalitySignal) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
                // First verify finality through the ProtocolExtension trait
                extension.verify_finality(signal).await?;
                
                // Then check if it implements FinalityVerifier for additional verification
                if let Some(verifier) = extension.as_any().downcast_ref::<Box<dyn FinalityVerifier + Send + Sync>>() {
                    let block_ref = BlockRef::new(
                        ChainId::new(signal.chain_id.clone()),
                        signal.block_number,
                        signal.block_hash,
                    );
                    verifier.verify_finality(&block_ref, signal).await?;
                }
            }
        }
        Ok(())
    }

    /// Handle network events
    pub async fn handle_network_event(&self, peer: &Peer, event: PeerEventType) -> Result<()> {
        let manager = self.manager.read().await;
        let extensions = manager.list_extensions().await?;

        for (id, _) in &extensions {
            if let Some(extension) = manager.get_extension(id).await? {
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