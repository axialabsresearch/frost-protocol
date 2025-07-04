/*!
# Extension Traits

This module defines the core traits that all FROST protocol extensions must implement.
It provides the interface contracts for extension behavior, lifecycle management, and
protocol integration.

## Core Traits

### Extension
The base trait for all protocol extensions, providing:
- Basic lifecycle management (initialize, start, stop)
- State and metrics access
- Message handling
- Event processing
- Optional capabilities

### ProtocolExtension
An enhanced extension interface with additional capabilities:
- Metadata access
- Configuration management
- Resource cleanup
- Advanced message processing
- Peer event handling
- State verification
- Finality verification

## Extension Lifecycle

Extensions implement a consistent lifecycle through these traits:

1. **Initialization**
   ```rust
   async fn initialize(&mut self) -> ExtensionResult<()>;
   ```
   - Load configuration
   - Set up resources
   - Initialize state
   - Validate dependencies

2. **Operation**
   ```rust
   async fn start(&mut self) -> ExtensionResult<()>;
   async fn handle_message(&self, message: &FrostMessage) -> ExtensionResult<()>;
   ```
   - Process messages
   - Handle state transitions
   - Manage resources
   - Report metrics

3. **Shutdown**
   ```rust
   async fn stop(&mut self) -> ExtensionResult<()>;
   async fn cleanup(&mut self) -> ExtensionResult<()>;
   ```
   - Clean shutdown
   - Resource cleanup
   - State persistence
   - Error reporting

## Message Processing Pipeline

Extensions can hook into the message processing pipeline at multiple points:

1. **Pre-processing**
   ```rust
   async fn pre_process_message(&self, message: &mut FrostMessage) -> ExtensionResult<()>;
   ```
   - Message validation
   - Content modification
   - Header processing
   - Routing decisions

2. **Main Processing**
   ```rust
   async fn handle_message(&self, message: &FrostMessage) -> ExtensionResult<()>;
   ```
   - Core logic
   - State updates
   - Event generation
   - Error handling

3. **Post-processing**
   ```rust
   async fn post_process_message(&self, message: &FrostMessage) -> ExtensionResult<()>;
   ```
   - Cleanup
   - Metrics update
   - Event notification
   - Logging

## State Management

Extensions can participate in state management through:

1. **State Transitions**
   ```rust
   async fn handle_state_transition(&self, transition: &StateTransition) -> ExtensionResult<()>;
   ```
   - Validate transitions
   - Update local state
   - Generate events
   - Verify proofs

2. **Proof Verification**
   ```rust
   async fn verify_state_proof(&self, proof: &StateProof) -> ExtensionResult<()>;
   ```
   - Proof validation
   - State verification
   - Security checks
   - Error reporting

## Event Handling

Extensions can process various events:

1. **Peer Events**
   ```rust
   async fn handle_peer_event(&self, peer: &Peer, event_type: PeerEventType) -> ExtensionResult<()>;
   ```
   - Connection events
   - State changes
   - Network topology
   - Peer metadata

2. **Finality Events**
   ```rust
   async fn verify_finality(&self, signal: &FinalitySignal) -> ExtensionResult<()>;
   ```
   - Finality verification
   - Chain validation
   - Block confirmation
   - State updates

## Best Practices

1. **Error Handling**
   - Use `ExtensionResult` for all operations
   - Provide detailed error context
   - Handle cleanup on errors
   - Maintain consistent state

2. **Async Operations**
   - Use async/await properly
   - Avoid blocking operations
   - Handle cancellation
   - Manage timeouts

3. **Resource Management**
   - Clean up resources in `stop()`
   - Handle partial initialization
   - Manage memory carefully
   - Close handles properly

4. **State Access**
   - Use async state access
   - Avoid deprecated sync methods
   - Handle concurrent access
   - Validate state transitions
*/

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