use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::extensions::{
    ExtensionManager,
    ExtensionId,
    ExtensionState,
    ExtensionMetrics,
};
use crate::monitoring::MonitoringSystem;
use crate::devtools::{
    ExtensionDebugger,
    ExtensionDebugInfo,
    ExtensionProfile,
    ExtensionDebugMetrics,
    ExtensionDebugConfig,
};

/// Implementation of extension debugger
pub struct ExtensionDebuggerImpl {
    manager: Arc<RwLock<dyn ExtensionManager>>,
    monitoring: Arc<RwLock<MonitoringSystem>>,
    config: ExtensionDebugConfig,
    hot_reload_handlers: RwLock<HashMap<ExtensionId, tokio::task::JoinHandle<()>>>,
    profiling_data: RwLock<HashMap<ExtensionId, Vec<ExtensionProfile>>>,
}

impl ExtensionDebuggerImpl {
    /// Create new extension debugger
    pub fn new(
        manager: Arc<RwLock<dyn ExtensionManager>>,
        monitoring: Arc<RwLock<MonitoringSystem>>,
        config: ExtensionDebugConfig,
    ) -> Self {
        Self {
            manager,
            monitoring,
            config,
            hot_reload_handlers: RwLock::new(HashMap::new()),
            profiling_data: RwLock::new(HashMap::new()),
        }
    }

    /// Start hot reload watcher for extension
    async fn start_hot_reload_watcher(&self, extension_id: ExtensionId) -> Result<()> {
        let manager = self.manager.clone();
        let monitoring = self.monitoring.clone();

        let handle = tokio::spawn(async move {
            loop {
                // Check for changes every second
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let manager = manager.read().await;
                if let Some(extension) = manager.get_extension(&extension_id).await? {
                    // Check if extension source has changed
                    if self.has_source_changed(&extension_id).await? {
                        info!("Detected changes in extension {}, reloading", extension_id.0);
                        
                        // Reload extension
                        self.reload_extension(&extension_id).await?;
                    }
                } else {
                    break;
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        self.hot_reload_handlers.write().await.insert(extension_id, handle);
        Ok(())
    }

    /// Check if extension source has changed
    async fn has_source_changed(&self, extension_id: &ExtensionId) -> Result<bool> {
        // In a real implementation, this would check file timestamps, hashes, etc.
        Ok(false)
    }

    /// Reload extension
    async fn reload_extension(&self, extension_id: &ExtensionId) -> Result<()> {
        let mut manager = self.manager.write().await;
        
        // Disable extension
        manager.disable_extension(extension_id).await?;
        
        // Reload extension code (in real implementation)
        
        // Enable extension
        manager.enable_extension(extension_id).await?;
        
        Ok(())
    }

    /// Collect profiling data for extension
    async fn collect_profile_data(&self, extension_id: &ExtensionId) -> Result<ExtensionProfile> {
        let manager = self.manager.read().await;
        let monitoring = self.monitoring.read().await;
        
        if let Some(extension) = manager.get_extension(extension_id).await? {
            let metrics = extension.metrics();
            let resource_usage = monitoring.get_extension_resource_usage(extension_id).await?;
            
            Ok(ExtensionProfile {
                cpu_usage: resource_usage.cpu_usage,
                memory_usage: resource_usage.memory_usage,
                handler_latencies: metrics.handler_latencies,
                error_rates: metrics.error_rates,
            })
        } else {
            Err(anyhow::anyhow!("Extension not found"))
        }
    }
}

#[async_trait]
impl ExtensionDebugger for ExtensionDebuggerImpl {
    async fn enable_hot_reload(&mut self, extension_id: &ExtensionId) -> Result<()> {
        if !self.config.hot_reload {
            return Err(anyhow::anyhow!("Hot reload not enabled in config"));
        }

        info!("Enabling hot reload for extension {}", extension_id.0);
        self.start_hot_reload_watcher(extension_id.clone()).await?;
        Ok(())
    }

    async fn inspect_extension_state(&self, extension_id: &ExtensionId) -> Result<ExtensionDebugInfo> {
        if !self.config.state_inspection {
            return Err(anyhow::anyhow!("State inspection not enabled in config"));
        }

        let manager = self.manager.read().await;
        
        if let Some(extension) = manager.get_extension(extension_id).await? {
            let monitoring = self.monitoring.read().await;
            let resource_usage = monitoring.get_extension_resource_usage(extension_id).await?;
            
            Ok(ExtensionDebugInfo {
                state: format!("{:?}", extension.state()),
                config: serde_json::to_string_pretty(&extension.metadata())?,
                memory_usage: resource_usage.memory_usage,
                active_handlers: extension.metrics().active_handlers,
                error_count: extension.metrics().errors,
            })
        } else {
            Err(anyhow::anyhow!("Extension not found"))
        }
    }

    async fn profile_extension(&self, extension_id: &ExtensionId) -> Result<ExtensionProfile> {
        if !self.config.enable_profiling {
            return Err(anyhow::anyhow!("Profiling not enabled in config"));
        }

        info!("Profiling extension {}", extension_id.0);
        self.collect_profile_data(extension_id).await
    }

    async fn get_extension_metrics(&self, extension_id: &ExtensionId) -> Result<ExtensionDebugMetrics> {
        let manager = self.manager.read().await;
        
        if let Some(extension) = manager.get_extension(extension_id).await? {
            let metrics = extension.metrics();
            let monitoring = self.monitoring.read().await;
            let resource_usage = monitoring.get_extension_resource_usage(extension_id).await?;
            
            Ok(ExtensionDebugMetrics {
                messages_processed: metrics.messages_processed,
                average_processing_time: metrics.processing_time_ms as f64 / metrics.messages_processed as f64,
                error_count: metrics.errors,
                memory_usage: resource_usage.memory_usage,
            })
        } else {
            Err(anyhow::anyhow!("Extension not found"))
        }
    }
} 