#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::{
    extensions::{
        ExtensionId,
        ExtensionMetadata,
        ExtensionConfig,
        ExtensionState,
        ProtocolExtension,
        ExtensionManager,
        DefaultExtensionManager,
        PeerEventType,
        ExtensionMetrics,
        ExtensionCapability,
        errors::{ExtensionResult, ExtensionError},
    },
    message::FrostMessage,
    network::{NetworkProtocol, Peer},
    state::{StateTransition, StateProof},
    state::proof::{ProofType, ProofData},
    finality::FinalitySignal,   
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use serde_json;
use std::time::SystemTime;

// Mock extension for testing
struct MockExtension {
    state: tokio::sync::RwLock<ExtensionState>,
    metadata: ExtensionMetadata,
    metrics: tokio::sync::RwLock<ExtensionMetrics>,
}

impl MockExtension {
    fn new(name: &str, version: &str, dependencies: Vec<ExtensionId>) -> Self {
        Self {
            state: tokio::sync::RwLock::new(ExtensionState::Registered),
            metadata: ExtensionMetadata {
                name: name.to_string(),
                version: version.to_string(),
                description: "Mock extension for testing".to_string(),
                dependencies,
                capabilities: vec![],
            },
            metrics: tokio::sync::RwLock::new(ExtensionMetrics::default()),
        }
    }
}

#[async_trait]
impl ProtocolExtension for MockExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ExtensionConfig) -> ExtensionResult<()> {
        let mut state = self.state.write().await;
        *state = ExtensionState::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> ExtensionResult<()> {
        let mut state = self.state.write().await;
        *state = ExtensionState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> ExtensionResult<()> {
        let mut state = self.state.write().await;
        *state = ExtensionState::Suspended;
        Ok(())
    }

    async fn cleanup(&mut self) -> ExtensionResult<()> {
        Ok(())
    }

    async fn handle_message(&self, _message: &FrostMessage) -> ExtensionResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.messages_processed += 1;
        Ok(())
    }

    async fn pre_process_message(&self, _message: &mut FrostMessage) -> ExtensionResult<()> {
        Ok(())
    }

    async fn post_process_message(&self, _message: &FrostMessage) -> ExtensionResult<()> {
        Ok(())
    }

    async fn handle_state_transition(&self, _transition: &StateTransition) -> ExtensionResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.state_transitions += 1;
        Ok(())
    }

    async fn handle_peer_event(&self, _peer: &Peer, _event_type: PeerEventType) -> ExtensionResult<()> {
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
        Ok(self.metrics.read().await.clone())
    }

    async fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn test_extension_lifecycle() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
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
    
    // Enable extension and verify state transitions
    manager.enable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    assert_eq!(ext.get_state().await.unwrap(), ExtensionState::Active);
    
    // Disable extension and verify state transition
    manager.disable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    assert_eq!(ext.get_state().await.unwrap(), ExtensionState::Suspended);
}

#[tokio::test]
async fn test_extension_dependencies() {
    let mut manager = DefaultExtensionManager::new();
    
    // Create and register dependency extension
    let dep_id = ExtensionId::new("dep", "1.0.0");
    let dep_extension = Box::new(MockExtension::new("dep", "1.0.0", vec![]));
    let dep_config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };
    manager.register_extension(dep_extension, dep_config).await.unwrap();

    // Create and register dependent extension
    let ext_id = ExtensionId::new("test", "1.0.0");
    let extension = Box::new(MockExtension::new(
        "test",
        "1.0.0",
        vec![dep_id.clone()],
    ));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };
    manager.register_extension(extension, config).await.unwrap();

    // Verify dependencies
    let deps = manager.get_dependencies(&ext_id).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], dep_id);

    // Test dependency order during enable
    manager.enable_extension(&ext_id).await.unwrap();
    
    // Both extensions should be active
    let dep_ext = manager.get_extension(&dep_id).await.unwrap().unwrap();
    let main_ext = manager.get_extension(&ext_id).await.unwrap().unwrap();
    assert_eq!(dep_ext.get_state().await.unwrap(), ExtensionState::Active);
    assert_eq!(main_ext.get_state().await.unwrap(), ExtensionState::Active);

    // Test dependency constraints during disable
    // Should fail to disable dependency while dependent is active
    assert!(manager.disable_extension(&dep_id).await.is_err());
}

#[tokio::test]
async fn test_extension_compatibility() {
    let manager = DefaultExtensionManager::new();
    
    // Test compatible extension
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    assert!(manager.validate_compatibility(extension.as_ref()).await.is_ok());

    // Test incompatible extension (missing dependency)
    let dep_id = ExtensionId::new("missing_dep", "1.0.0");
    let incompatible = Box::new(MockExtension::new(
        "test2",
        "1.0.0",
        vec![dep_id],
    ));
    assert!(manager.validate_compatibility(incompatible.as_ref()).await.is_err());
}

#[tokio::test]
async fn test_extension_message_handling() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    // Register and enable extension
    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();

    // Test message handling
    let ext = manager.get_extension(&id).await.unwrap().unwrap();
    let message = FrostMessage::new(
        frost_protocol::message::MessageType::StateTransition,
        vec![1, 2, 3],
        "test".to_string(),
        None,
    );

    assert!(ext.handle_message(&message).await.is_ok());
    assert!(ext.pre_process_message(&mut message.clone()).await.is_ok());
    assert!(ext.post_process_message(&message).await.is_ok());
}

#[tokio::test]
async fn test_extension_state_transition() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
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
async fn test_finality_verification() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();

    let signal = FinalitySignal {
        chain_id: "test-chain".to_string(),
        block_number: 1000,
        block_hash: [0u8; 32],
        proof_data: vec![1, 2, 3],
        metadata: serde_json::json!({}),
    };

    assert!(ext.verify_finality(&signal).await.is_ok());
}

#[tokio::test]
async fn test_state_proof_verification() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();

    let transition = StateTransition::new(
        frost_protocol::state::BlockId::Composite {
            number: 1000,
            hash: [0u8; 32],
        },
        frost_protocol::state::BlockId::Composite {
            number: 1001,
            hash: [0u8; 32],
        },
        vec![],
    );

    let proof = StateProof {
        transition,
        proof: ProofData {
            proof_type: ProofType::Basic,
            data: vec![1, 2, 3],
            metadata: None,
            generated_at: std::time::SystemTime::now(),
            expires_at: None,
            version: 1,
        },
        verification_history: vec![],
    };

    assert!(ext.verify_state_proof(&proof).await.is_ok());
}

#[tokio::test]
async fn test_async_metrics_access() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();
    let ext = manager.get_extension(&id).await.unwrap().unwrap();

    // Test async metrics access
    let metrics = ext.get_metrics().await.unwrap();
    
    // Verify metrics fields
    assert_eq!(metrics.messages_processed, 0);
    assert_eq!(metrics.state_transitions, 0);
    assert_eq!(metrics.errors, 0);
}

#[tokio::test]
async fn test_cleanup() {
    let mut manager = DefaultExtensionManager::new();
    let extension = Box::new(MockExtension::new("test", "1.0.0", vec![]));
    let config = ExtensionConfig {
        enabled: true,
        priority: 0,
        parameters: HashMap::new(),
    };

    let id = manager.register_extension(extension, config).await.unwrap();
    manager.enable_extension(&id).await.unwrap();
    
    // Test cleanup through the registry
    manager.cleanup_resources().await.unwrap();
} 