use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use frost_protocol::{
    extensions::{
        ExtensionId,
        ExtensionHooks,
        ExtensionContext,
        DefaultExtensionManager,
        ExtensionState,
        ExtensionConfig,
        ProtocolExtension,
        ExtensionMetrics,
        ExtensionCapability,
        errors::{ExtensionResult, ExtensionError},
    },
    network::{NetworkProtocol, Peer},
    message::FrostMessage,
    state::{StateTransition, StateProof},
    finality::FinalitySignal,
};
use async_trait::async_trait;
use std::collections::HashMap;

// Mock NetworkProtocol for testing
struct MockNetwork;

#[async_trait]
impl NetworkProtocol for MockNetwork {
    async fn send_message(&self, _message: FrostMessage) -> Result<()> {
        Ok(())
    }

    async fn broadcast_message(&self, _message: FrostMessage) -> Result<()> {
        Ok(())
    }

    async fn connect_peer(&self, _peer: Peer) -> Result<()> {
        Ok(())
    }

    async fn disconnect_peer(&self, _peer: &Peer) -> Result<()> {
        Ok(())
    }
}

// Mock extension that implements async state access
struct MockExtension {
    state: RwLock<ExtensionState>,
}

impl MockExtension {
    fn new() -> Self {
        Self {
            state: RwLock::new(ExtensionState::Registered),
        }
    }
}

#[async_trait]
impl ProtocolExtension for MockExtension {
    fn metadata(&self) -> &frost_protocol::extensions::ExtensionMetadata {
        static METADATA: once_cell::sync::Lazy<frost_protocol::extensions::ExtensionMetadata> = 
            once_cell::sync::Lazy::new(|| frost_protocol::extensions::ExtensionMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: "Mock extension for testing".to_string(),
                dependencies: vec![],
                capabilities: vec![],
            });
        &METADATA
    }

    async fn initialize(&mut self, _config: ExtensionConfig) -> ExtensionResult<()> {
        *self.state.write().await = ExtensionState::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> ExtensionResult<()> {
        *self.state.write().await = ExtensionState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> ExtensionResult<()> {
        *self.state.write().await = ExtensionState::Suspended;
        Ok(())
    }

    async fn cleanup(&mut self) -> ExtensionResult<()> {
        Ok(())
    }

    async fn handle_message(&self, _message: &FrostMessage) -> ExtensionResult<()> {
        Ok(())
    }

    async fn pre_process_message(&self, _message: &mut FrostMessage) -> ExtensionResult<()> {
        Ok(())
    }

    async fn post_process_message(&self, _message: &FrostMessage) -> ExtensionResult<()> {
        Ok(())
    }

    async fn handle_state_transition(&self, _transition: &StateTransition) -> ExtensionResult<()> {
        Ok(())
    }

    async fn handle_peer_event(&self, _peer: &Peer, _event_type: frost_protocol::extensions::PeerEventType) -> ExtensionResult<()> {
        Ok(())
    }

    async fn verify_finality(&self, _signal: &FinalitySignal) -> ExtensionResult<()> {
        Ok(())
    }

    async fn verify_state_proof(&self, _proof: &StateProof) -> ExtensionResult<()> {
        Ok(())
    }

    async fn get_state(&self) -> ExtensionResult<ExtensionState> {
        Ok(*self.state.read().await)
    }

    async fn get_metrics(&self) -> ExtensionResult<ExtensionMetrics> {
        Ok(ExtensionMetrics::default())
    }

    async fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn test_extension_context() {
    let manager = Arc::new(RwLock::new(DefaultExtensionManager::new()));
    let network = Arc::new(MockNetwork);
    let hooks = Arc::new(ExtensionHooks::new(manager.clone(), network.clone()));
    
    let id = ExtensionId::new("test", "1.0.0");
    let context = ExtensionContext::new(id.clone(), hooks.clone());
    
    assert_eq!(context.id, id);
    assert!(Arc::ptr_eq(&context.network(), &network));
    assert!(Arc::ptr_eq(&context.manager(), &manager));
}

#[tokio::test]
async fn test_extension_hooks() {
    let manager = Arc::new(RwLock::new(DefaultExtensionManager::new()));
    let network = Arc::new(MockNetwork);
    let hooks = ExtensionHooks::new(manager.clone(), network.clone());
    
    // Test network access
    assert!(Arc::ptr_eq(&hooks.network(), &network));
    
    // Test manager access
    assert!(Arc::ptr_eq(&hooks.manager(), &manager));
    
    // Test message validation hooks
    let mut message = FrostMessage::new(
        frost_protocol::message::MessageType::StateTransition,
        vec![1, 2, 3],
        "test".to_string(),
        None,
    );
    
    assert!(hooks.pre_validate(&mut message).await.is_ok());
    assert!(hooks.validate_state(&message).await.is_ok());
    assert!(hooks.post_validate(&message).await.is_ok());
}

#[tokio::test]
async fn test_extension_state_transition() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new());
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    // Register and enable extension
    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();

    // Test state transition handling
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    let source = frost_protocol::state::BlockId::Composite {
        number: 1000,
        hash: [0u8; 32],
    };
    let target = frost_protocol::state::BlockId::Composite {
        number: 1001,
        hash: [0u8; 32],
    };

    let transition = StateTransition::new(
        source,
        target,
        vec![1, 2, 3],
    );

    assert!(ext.handle_state_transition(&transition).await.is_ok());
}

#[tokio::test]
async fn test_extension_lifecycle() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new());
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    // Register extension
    let id = manager.register_extension(extension, config).await.unwrap();
    
    // Get extension and verify initial state
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    assert_eq!(ext.get_state().await.unwrap(), ExtensionState::Registered);
    
    // Enable extension and verify state transition
    manager.enable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    assert_eq!(ext.get_state().await.unwrap(), ExtensionState::Active);
    
    // Disable extension and verify state transition
    manager.disable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    assert_eq!(ext.get_state().await.unwrap(), ExtensionState::Suspended);
} 