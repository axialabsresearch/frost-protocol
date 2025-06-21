use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};

use super::{
    ExtensionId,
    ExtensionMetadata,
    ExtensionState,
    ExtensionConfig,
    ProtocolExtension,
};

/// Registry entry for a protocol extension
struct RegistryEntry {
    extension: Box<dyn ProtocolExtension>,
    config: ExtensionConfig,
    state: ExtensionState,
}

/// Registry for managing protocol extensions
pub struct ExtensionRegistry {
    extensions: RwLock<HashMap<ExtensionId, RegistryEntry>>,
}

impl ExtensionRegistry {
    /// Create a new extension registry
    pub fn new() -> Self {
        Self {
            extensions: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new extension
    pub async fn register(
        &self,
        extension: Box<dyn ProtocolExtension>,
        config: ExtensionConfig,
    ) -> Result<ExtensionId> {
        let id = ExtensionId::new(
            &extension.metadata().name,
            &extension.metadata().version,
        );

        let mut extensions = self.extensions.write().await;
        
        if extensions.contains_key(&id) {
            return Err(anyhow!("Extension already registered: {}", id.0));
        }

        let entry = RegistryEntry {
            extension,
            config,
            state: ExtensionState::Registered,
        };

        extensions.insert(id.clone(), entry);
        Ok(id)
    }

    /// Unregister an extension
    pub async fn unregister(&self, id: &ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(mut entry) = extensions.remove(id) {
            if entry.state == ExtensionState::Active {
                entry.extension.stop().await?;
            }
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }

    /// Get extension by ID
    pub async fn get_extension(&self, id: &ExtensionId) -> Result<Option<Arc<dyn ProtocolExtension>>> {
        let extensions = self.extensions.read().await;
        
        Ok(extensions.get(id).map(|entry| {
            Arc::new(entry.extension.as_ref()) as Arc<dyn ProtocolExtension>
        }))
    }

    /// List all registered extensions
    pub async fn list_extensions(&self) -> Vec<(ExtensionId, ExtensionMetadata)> {
        let extensions = self.extensions.read().await;
        
        extensions
            .iter()
            .map(|(id, entry)| {
                (id.clone(), entry.extension.metadata().clone())
            })
            .collect()
    }

    /// Enable an extension
    pub async fn enable(&self, id: &ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(entry) = extensions.get_mut(id) {
            if entry.state != ExtensionState::Active {
                if entry.state == ExtensionState::Registered {
                    entry.extension.initialize(entry.config.clone()).await?;
                }
                entry.extension.start().await?;
                entry.state = ExtensionState::Active;
            }
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }

    /// Disable an extension
    pub async fn disable(&self, id: &ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(entry) = extensions.get_mut(id) {
            if entry.state == ExtensionState::Active {
                entry.extension.stop().await?;
                entry.state = ExtensionState::Suspended;
            }
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }

    /// Get extension state
    pub async fn get_state(&self, id: &ExtensionId) -> Result<ExtensionState> {
        let extensions = self.extensions.read().await;
        
        if let Some(entry) = extensions.get(id) {
            Ok(entry.state)
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }

    /// Update extension configuration
    pub async fn update_config(
        &self,
        id: &ExtensionId,
        config: ExtensionConfig,
    ) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(entry) = extensions.get_mut(id) {
            let was_active = entry.state == ExtensionState::Active;
            
            if was_active {
                entry.extension.stop().await?;
            }
            
            entry.config = config;
            
            if was_active {
                entry.extension.initialize(entry.config.clone()).await?;
                entry.extension.start().await?;
                entry.state = ExtensionState::Active;
            }
            
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }
} 