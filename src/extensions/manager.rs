use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use tokio::sync::RwLock;

use super::{
    ExtensionId,
    ExtensionMetadata,
    ExtensionConfig,
    ProtocolExtension,
    ExtensionManager,
    ExtensionRegistry,
};

/// Default implementation of the extension manager
pub struct DefaultExtensionManager {
    registry: Arc<ExtensionRegistry>,
    dependency_graph: RwLock<HashMap<ExtensionId, HashSet<ExtensionId>>>,
}

impl DefaultExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ExtensionRegistry::new()),
            dependency_graph: RwLock::new(HashMap::new()),
        }
    }

    /// Build dependency graph for extensions
    async fn build_dependency_graph(&self) -> Result<()> {
        let mut graph = HashMap::new();
        let extensions = self.registry.list_extensions().await;

        for (id, metadata) in extensions {
            let mut deps = HashSet::new();
            for dep_id in metadata.dependencies {
                if !self.registry.get_extension(&dep_id).await?.is_some() {
                    return Err(anyhow!(
                        "Missing dependency {} for extension {}",
                        dep_id.0,
                        id.0
                    ));
                }
                deps.insert(dep_id);
            }
            graph.insert(id, deps);
        }

        *self.dependency_graph.write().await = graph;
        Ok(())
    }

    /// Check for dependency cycles
    async fn check_dependency_cycles(&self) -> Result<()> {
        let graph = self.dependency_graph.read().await;
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for id in graph.keys() {
            if !visited.contains(id) {
                if self.has_cycle(id, &graph, &mut visited, &mut stack).await? {
                    return Err(anyhow!("Dependency cycle detected"));
                }
            }
        }

        Ok(())
    }

    async fn has_cycle(
        &self,
        id: &ExtensionId,
        graph: &HashMap<ExtensionId, HashSet<ExtensionId>>,
        visited: &mut HashSet<ExtensionId>,
        stack: &mut HashSet<ExtensionId>,
    ) -> Result<bool> {
        visited.insert(id.clone());
        stack.insert(id.clone());

        if let Some(deps) = graph.get(id) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, graph, visited, stack).await? {
                        return Ok(true);
                    }
                } else if stack.contains(dep) {
                    return Ok(true);
                }
            }
        }

        stack.remove(id);
        Ok(false)
    }

    /// Get extension dependencies in order
    async fn get_ordered_dependencies(&self, id: &ExtensionId) -> Result<Vec<ExtensionId>> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();

        self.visit_dependencies(id, &mut ordered, &mut visited).await?;
        
        // Remove the target extension itself from the list
        ordered.pop();
        
        Ok(ordered)
    }

    async fn visit_dependencies(
        &self,
        id: &ExtensionId,
        ordered: &mut Vec<ExtensionId>,
        visited: &mut HashSet<ExtensionId>,
    ) -> Result<()> {
        if visited.contains(id) {
            return Ok(());
        }

        visited.insert(id.clone());

        let graph = self.dependency_graph.read().await;
        if let Some(deps) = graph.get(id) {
            for dep in deps {
                self.visit_dependencies(dep, ordered, visited).await?;
            }
        }

        ordered.push(id.clone());
        Ok(())
    }
}

#[async_trait]
impl ExtensionManager for DefaultExtensionManager {
    async fn register_extension(
        &mut self,
        extension: Box<dyn ProtocolExtension>,
        config: ExtensionConfig,
    ) -> Result<ExtensionId> {
        let id = self.registry.register(extension, config).await?;
        self.build_dependency_graph().await?;
        self.check_dependency_cycles().await?;
        Ok(id)
    }

    async fn unregister_extension(&mut self, id: &ExtensionId) -> Result<()> {
        // Check if any other extensions depend on this one
        let graph = self.dependency_graph.read().await;
        for (ext_id, deps) in graph.iter() {
            if deps.contains(id) {
                return Err(anyhow!(
                    "Cannot unregister extension {} because {} depends on it",
                    id.0,
                    ext_id.0
                ));
            }
        }

        self.registry.unregister(id).await?;
        self.build_dependency_graph().await?;
        Ok(())
    }

    async fn get_extension(&self, id: &ExtensionId) -> Result<Option<Arc<dyn ProtocolExtension>>> {
        self.registry.get_extension(id).await
    }

    async fn list_extensions(&self) -> Result<Vec<(ExtensionId, ExtensionMetadata)>> {
        Ok(self.registry.list_extensions().await)
    }

    async fn enable_extension(&mut self, id: &ExtensionId) -> Result<()> {
        // Enable dependencies first
        let deps = self.get_ordered_dependencies(id).await?;
        for dep_id in deps {
            self.registry.enable(&dep_id).await?;
        }

        self.registry.enable(id).await
    }

    async fn disable_extension(&mut self, id: &ExtensionId) -> Result<()> {
        // Check if any active extensions depend on this one
        let graph = self.dependency_graph.read().await;
        for (ext_id, deps) in graph.iter() {
            if deps.contains(id) {
                let state = self.registry.get_state(ext_id).await?;
                if state == super::ExtensionState::Active {
                    return Err(anyhow!(
                        "Cannot disable extension {} because active extension {} depends on it",
                        id.0,
                        ext_id.0
                    ));
                }
            }
        }

        self.registry.disable(id).await
    }

    fn get_dependencies(&self, id: &ExtensionId) -> Result<Vec<ExtensionId>> {
        let graph = self.dependency_graph.blocking_read();
        Ok(graph
            .get(id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default())
    }

    fn validate_compatibility(&self, extension: &dyn ProtocolExtension) -> Result<()> {
        // For now, just check if all dependencies are available
        // In the future, this could check version compatibility, feature requirements, etc.
        for dep_id in &extension.metadata().dependencies {
            if self.registry.get_extension(dep_id).blocking_ok().flatten().is_none() {
                return Err(anyhow!(
                    "Missing required dependency: {}",
                    dep_id.0
                ));
            }
        }
        Ok(())
    }
} 