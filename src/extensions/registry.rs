// temporary for v0.1.0 
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, warn};
use anyhow::{Result, anyhow};

use super::{
    ExtensionId,
    ExtensionMetadata,
    ExtensionState,
    ExtensionConfig,
    ProtocolExtension,
    dependency::DependencyResolver,
    errors::{ExtensionError, ExtensionResult},
};

const DEFAULT_MAX_EXTENSIONS: usize = 100;
const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Registry entry for a protocol extension
struct RegistryEntry {
    extension: Arc<RwLock<Box<dyn ProtocolExtension>>>,
    config: ExtensionConfig,
    state: ExtensionState,
}

#[derive(Clone)]
pub struct ExtensionSnapshot {
    extensions: Vec<(ExtensionId, ExtensionMetadata)>,
    version: u64,
}

/// Registry for managing protocol extensions
pub struct ExtensionRegistry {
    extensions: RwLock<HashMap<ExtensionId, RegistryEntry>>,
    version_counter: AtomicU64,
    dependency_resolver: RwLock<DependencyResolver>,
    max_extensions: usize,
    operation_timeout: Duration,
}

impl ExtensionRegistry {
    /// Create a new extension registry
    pub fn new() -> Self {
        Self {
            extensions: RwLock::new(HashMap::new()),
            version_counter: AtomicU64::new(0),
            dependency_resolver: RwLock::new(DependencyResolver::new()),
            max_extensions: DEFAULT_MAX_EXTENSIONS,
            operation_timeout: DEFAULT_OPERATION_TIMEOUT,
        }
    }

    /// Register a new extension
    pub async fn register(
        &self,
        extension: Box<dyn ProtocolExtension>,
        config: ExtensionConfig,
    ) -> ExtensionResult<ExtensionId> {
        let id = ExtensionId::new(
            &extension.metadata().name,
            &extension.metadata().version,
        );

        // Check resource limits
        {
            let extensions = self.extensions.read().await;
            if extensions.len() >= self.max_extensions {
                return Err(ExtensionError::ResourceLimitExceeded);
            }
        }

        // Perform registration with timeout
        tokio::time::timeout(self.operation_timeout, async {
        let mut extensions = self.extensions.write().await;
        
        if extensions.contains_key(&id) {
                return Err(ExtensionError::AlreadyExists(id.0.clone()));
        }

        let entry = RegistryEntry {
            extension: Arc::new(RwLock::new(extension)),
            config,
            state: ExtensionState::Registered,
        };

            // Update dependency graph
            let mut resolver = self.dependency_resolver.write().await;
            for dep_id in &entry.extension.read().await.metadata().dependencies {
                resolver.add_dependency(&id, dep_id)?;
            }

        extensions.insert(id.clone(), entry);
            self.version_counter.fetch_add(1, Ordering::SeqCst);
            
        Ok(id)
        })
        .await
        .map_err(|_| ExtensionError::OperationTimeout)?
    }

    /// Unregister an extension
    pub async fn unregister(&self, id: &ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(entry) = extensions.remove(id) {
            if entry.state == ExtensionState::Active {
                entry.extension.write().await.stop().await?;
            }
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }

    /// Get extension by ID
    pub async fn get_extension(&self, id: &ExtensionId) -> Result<Option<Arc<RwLock<Box<dyn ProtocolExtension>>>>> {
        let extensions = self.extensions.read().await;
        
        Ok(extensions.get(id).map(|entry| Arc::clone(&entry.extension)))
    }

    /// List all registered extensions
    pub async fn list_extensions(&self) -> Vec<(ExtensionId, ExtensionMetadata)> {
        let extensions = self.extensions.read().await;
        let mut result = Vec::new();
        
        for (id, entry) in extensions.iter() {
            let metadata = entry.extension.read().await.metadata().clone();
            result.push((id.clone(), metadata));
        }
        
        result
    }

    /// Enable an extension and its dependencies
    pub async fn enable(&self, id: &ExtensionId) -> ExtensionResult<()> {
        // Get dependency order
        let deps_to_enable = {
            let resolver = self.dependency_resolver.read().await;
            resolver.resolve_order()?
        };

        // Enable dependencies first
        for dep_id in deps_to_enable {
            if !self.is_enabled(&dep_id).await? {
                self.enable_single(&dep_id).await?;
            }
        }

        // Enable the target extension
        self.enable_single(id).await?;

        Ok(())
    }

    /// Enable a single extension
    async fn enable_single(&self, id: &ExtensionId) -> ExtensionResult<()> {
        let mut rollback_state = None;
        
        let result = tokio::time::timeout(self.operation_timeout, async {
            // Get extension first with shorter lock scope
            let extension_arc = {
                let extensions = self.extensions.read().await;
                match extensions.get(id) {
                    Some(entry) => Arc::clone(&entry.extension),
                    None => return Err(ExtensionError::NotFound(id.0.clone()))
                }
            };
            
            // Initialize and start without holding registry lock
            {
                let mut ext = extension_arc.write().await;
                if ext.get_state().await? == ExtensionState::Registered {
                    ext.initialize(ExtensionConfig {
                        enabled: true,
                        priority: 0,
                        parameters: HashMap::new(),
                    }).await?;
                }
                ext.start().await?;
            }
            
            // Update state with minimal lock time
            let mut extensions = self.extensions.write().await;
            if let Some(entry) = extensions.get_mut(id) {
                rollback_state = Some(entry.state);
                entry.state = ExtensionState::Active;
                self.version_counter.fetch_add(1, Ordering::SeqCst);
            }
            
            Ok(())
        })
        .await
        .map_err(|_| ExtensionError::OperationTimeout)?;

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                if let Some(state) = rollback_state {
                    if let Err(re) = self.rollback_extension(id, state).await {
                        error!("Rollback failed for extension {}: {}", id.0, re);
                    }
                }
                Err(e)
            }
        }
    }

    /// Rollback extension state
    async fn rollback_extension(&self, id: &ExtensionId, previous_state: ExtensionState) -> ExtensionResult<()> {
        let mut extensions = self.extensions.write().await;
        if let Some(entry) = extensions.get_mut(id) {
            entry.state = previous_state;
            entry.extension.write().await.stop().await?;
        }
        Ok(())
    }

    /// Get extension state
    pub async fn is_enabled(&self, id: &ExtensionId) -> ExtensionResult<bool> {
        let extensions = self.extensions.read().await;
        Ok(extensions.get(id)
            .map(|entry| entry.state == ExtensionState::Active)
            .unwrap_or(false))
    }

    /// Get a snapshot of current extensions
    pub async fn get_snapshot(&self) -> ExtensionSnapshot {
        let extensions = self.extensions.read().await;
        let version = self.version_counter.load(Ordering::SeqCst);
        
        let mut snapshot_extensions = Vec::new();
        for (id, entry) in extensions.iter() {
            let metadata = entry.extension.read().await.metadata().clone();
            snapshot_extensions.push((id.clone(), metadata));
        }
        
        ExtensionSnapshot {
            extensions: snapshot_extensions,
            version,
        }
    }

    /// Cleanup resources for all extensions
    pub async fn cleanup_resources(&self) -> ExtensionResult<()> {
        let extensions = self.extensions.read().await;
        for (id, entry) in extensions.iter() {
            if let Err(e) = entry.extension.write().await.cleanup().await {
                error!("Failed to cleanup extension {}: {}", id.0, e);
        }
        }
        Ok(())
    }

    /// Disable an extension
    pub async fn disable(&self, id: &ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        
        if let Some(entry) = extensions.get_mut(id) {
            if entry.state == ExtensionState::Active {
                entry.extension.write().await.stop().await?;
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
                entry.extension.write().await.stop().await?;
            }
            
            entry.config = config;
            
            if was_active {
                entry.extension.write().await.initialize(entry.config.clone()).await?;
                entry.extension.write().await.start().await?;
                entry.state = ExtensionState::Active;
            }
            
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id.0))
        }
    }
} 