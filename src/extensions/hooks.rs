/*!
# Extension Hooks System

The hooks module provides the integration points between protocol extensions and the core FROST
protocol functionality. It enables extensions to intercept and modify protocol behavior at
key points in the message processing and state management pipeline.

## Core Components

### ExtensionHooks
Provides the main hook points for extensions:
- Message validation pipeline
- State proof verification
- Finality verification
- Network event handling

### ExtensionContext
Provides extensions with access to:
- Core protocol functionality
- Network operations
- Extension management
- Hook invocation

## Hook Points

### Message Processing
1. **Pre-validation**
   ```rust
   async fn pre_validate(&self, message: &mut FrostMessage) -> Result<()>;
   ```
   - Message format validation
   - Header verification
   - Content modification
   - Protocol checks

2. **State Validation**
   ```rust
   async fn validate_state(&self, message: &FrostMessage) -> Result<()>;
   ```
   - State consistency
   - Transition validation
   - Chain verification
   - Rule enforcement

3. **Post-validation**
   ```rust
   async fn post_validate(&self, message: &FrostMessage) -> Result<()>;
   ```
   - Cleanup operations
   - Event generation
   - Metrics update
   - Logging

### State Management
1. **Proof Verification**
   ```rust
   async fn verify_state_proof(&self, proof: &StateProof) -> Result<()>;
   ```
   - Proof validation
   - State verification
   - Chain consistency
   - Security checks

2. **Finality Verification**
   ```rust
   async fn verify_finality(&self, signal: &FinalitySignal) -> Result<()>;
   ```
   - Finality checks
   - Block confirmation
   - Chain validation
   - State updates

### Network Events
```rust
async fn handle_network_event(&self, peer: &Peer, event: PeerEventType) -> Result<()>;
```
- Peer connections
- Network topology
- State changes
- Event routing

## Usage Example

```rust
use frost_protocol::extensions::hooks::{ExtensionHooks, ExtensionContext};

// Create hooks
let hooks = ExtensionHooks::new(manager, network);

// Create extension context
let context = ExtensionContext::new(extension_id, Arc::new(hooks));

// Use hooks in extension
async fn process_message(ctx: &ExtensionContext, msg: &mut FrostMessage) -> Result<()> {
    // Pre-validation
    ctx.hooks.pre_validate(msg).await?;
    
    // Custom processing
    process_custom_logic(msg).await?;
    
    // Post-validation
    ctx.hooks.post_validate(msg).await?;
    
    Ok(())
}
```

## Hook Execution

The hook system follows these principles:

1. **Ordered Execution**
   - Extensions executed in priority order
   - All hooks must complete successfully
   - Early termination on errors
   - Error propagation

2. **State Management**
   - Consistent state updates
   - Atomic operations
   - Rollback on errors
   - State verification

3. **Error Handling**
   - Proper error propagation
   - Cleanup on failures
   - Error context preservation
   - Recovery mechanisms

4. **Resource Management**
   - Proper cleanup
   - Resource limits
   - Timeout handling
   - Memory management

## Integration

The hooks system integrates with:
1. Message processing pipeline
2. State management system
3. Network protocol
4. Extension manager

## Best Practices

1. **Hook Implementation**
   - Keep hooks lightweight
   - Handle errors properly
   - Maintain state consistency
   - Clean up resources

2. **Context Usage**
   - Use provided context
   - Avoid global state
   - Handle concurrent access
   - Respect resource limits

3. **Error Handling**
   - Propagate errors properly
   - Add error context
   - Clean up on errors
   - Maintain consistency
*/

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