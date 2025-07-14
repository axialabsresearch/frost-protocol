use async_trait::async_trait;
use anyhow::Result;
use std::any::Any;
use std::sync::Arc;

use super::{
    ExtensionMetadata,
    ExtensionConfig,
    ExtensionState,
    ExtensionMetrics,
    ExtensionCapability,
    PeerEventType,
    errors::ExtensionResult,
    ExtensionId,
};
use crate::message::FrostMessage;
use crate::network::Peer;
use crate::state::{StateTransition, StateProof};
use crate::finality::FinalitySignal;

/// Result type for extension operations
//pub type ExtensionResult<T> = Result<T>;

/// Base trait for protocol extensions
#[async_trait]
pub trait Extension: Send + Sync {
    /// Get extension identifier
    fn id(&self) -> ExtensionId;

    /// Get extension name
    fn name(&self) -> String;

    /// Get extension version
    fn version(&self) -> String;

    /// Initialize extension
    async fn initialize(&mut self) -> ExtensionResult<()>;

    /// Start extension
    async fn start(&mut self) -> ExtensionResult<()>;

    /// Stop extension
    async fn stop(&mut self) -> ExtensionResult<()>;

    /// Get current extension state
    async fn get_state(&self) -> ExtensionResult<ExtensionState>;

    /// Get extension metrics
    async fn get_metrics(&self) -> ExtensionResult<ExtensionMetrics>;

    /// Handle incoming message
    async fn handle_message(&self, message: &FrostMessage) -> ExtensionResult<()>;

    /// Pre-process message
    async fn pre_process_message(&self, message: &mut FrostMessage) -> ExtensionResult<()>;

    /// Post-process message
    async fn post_process_message(&self, message: &FrostMessage) -> ExtensionResult<()>;

    /// Handle state transition
    async fn handle_state_transition(&self, transition: &StateTransition) -> ExtensionResult<()>;

    /// Optional: Handle peer events
    async fn handle_peer_event(&self, _event_type: PeerEventType, _peer_id: &str) -> ExtensionResult<()> {
        Ok(()) // Default no-op implementation
    }

    /// Optional: Verify finality signals
    async fn verify_finality(&self, _signal: &FinalitySignal) -> ExtensionResult<()> {
        Ok(()) // Default no-op implementation
    }

    /// Optional: Verify state proofs
    async fn verify_state_proof(&self, _proof: &StateProof) -> ExtensionResult<()> {
        Ok(()) // Default no-op implementation
    }

    /// Optional: Get supported capabilities
    async fn capabilities(&self) -> Vec<ExtensionCapability> {
        Vec::new() // Default no capabilities
    }
}

#[async_trait]
pub trait ProtocolExtension: Send + Sync {
    /// Get extension metadata
    fn metadata(&self) -> &ExtensionMetadata;

    /// Initialize the extension
    async fn initialize(&mut self, config: ExtensionConfig) -> ExtensionResult<()>;

    /// Start the extension
    async fn start(&mut self) -> ExtensionResult<()>;

    /// Stop the extension
    async fn stop(&mut self) -> ExtensionResult<()>;

    /// Clean up extension resources
    async fn cleanup(&mut self) -> ExtensionResult<()> {
        // Default implementation - no cleanup needed
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&self, message: &FrostMessage) -> ExtensionResult<()>;

    /// Pre-process outgoing message
    async fn pre_process_message(&self, message: &mut FrostMessage) -> ExtensionResult<()>;

    /// Post-process message
    async fn post_process_message(&self, message: &FrostMessage) -> ExtensionResult<()>;

    /// Handle state transition
    async fn handle_state_transition(&self, transition: &StateTransition) -> ExtensionResult<()>;

    /// Handle peer event
    async fn handle_peer_event(&self, peer: &Peer, event_type: PeerEventType) -> ExtensionResult<()>;

    /// Get current extension state
    async fn get_state(&self) -> ExtensionResult<ExtensionState>;

    /// Get extension metrics
    async fn get_metrics(&self) -> ExtensionResult<ExtensionMetrics> {
        Ok(ExtensionMetrics::default())
    }

    /// Get extension as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Optional: Verify finality signals
    async fn verify_finality(&self, _signal: &FinalitySignal) -> ExtensionResult<()> {
        Ok(()) // Default no-op implementation
    }

    /// Optional: Verify state proofs
    async fn verify_state_proof(&self, _proof: &StateProof) -> ExtensionResult<()> {
        Ok(()) // Default no-op implementation
    }

    /// Optional: Get supported capabilities
    async fn capabilities(&self) -> Vec<ExtensionCapability> {
        Vec::new() // Default no capabilities
    }

    /// Synchronous state access (deprecated - use get_state instead)
    #[deprecated(note = "Use get_state() instead")]
    fn state(&self) -> ExtensionState {
        tracing::warn!("Synchronous state access in async context - consider using get_state() instead");
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.get_state().await.unwrap_or(ExtensionState::Failed)
            })
        })
    }

    /// Synchronous metrics access (deprecated - use get_metrics instead)
    #[deprecated(note = "Use get_metrics() instead")]
    fn metrics(&self) -> ExtensionMetrics {
        tracing::warn!("Synchronous metrics access in async context - consider using get_metrics() instead");
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.get_metrics().await.unwrap_or_default()
            })
        })
    }
} 