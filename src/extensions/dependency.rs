use std::collections::HashMap;
use petgraph::{Graph, graph::NodeIndex};
use petgraph::algo::toposort;

use super::{ExtensionId, errors::{ExtensionError, ExtensionResult}};

pub struct DependencyResolver {
    dependency_graph: Graph<ExtensionId, ()>,
    node_indices: HashMap<ExtensionId, NodeIndex>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            dependency_graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }
    
    fn get_or_create_node(&mut self, extension: &ExtensionId) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(extension) {
            idx
        } else {
            let idx = self.dependency_graph.add_node(extension.clone());
            self.node_indices.insert(extension.clone(), idx);
            idx
        }
    }
    
    pub fn add_dependency(&mut self, extension: &ExtensionId, dependency: &ExtensionId) -> ExtensionResult<()> {
        let ext_idx = self.get_or_create_node(extension);
        let dep_idx = self.get_or_create_node(dependency);
        
        // Check for cycles before adding edge
        self.dependency_graph.add_edge(ext_idx, dep_idx, ());
        if toposort(&self.dependency_graph, None).is_err() {
            // Remove the edge using the indices directly
            self.dependency_graph.remove_edge(self.dependency_graph.find_edge(ext_idx, dep_idx).unwrap());
            return Err(ExtensionError::CircularDependency(
                extension.0.clone(),
                dependency.0.clone(),
            ));
        }
        
        Ok(())
    }
    
    pub fn resolve_order(&self) -> ExtensionResult<Vec<ExtensionId>> {
        toposort(&self.dependency_graph, None)
            .map_err(|_| ExtensionError::DependencyResolutionFailed)
            .map(|indices| {
                indices.into_iter()
                    .map(|idx| self.dependency_graph[idx].clone())
                    .collect::<Vec<_>>()
            })
            .map(|mut ids| {
                ids.reverse(); // Reverse to get dependencies first
                ids
            })
    }
    
    pub fn clear(&mut self) {
        self.dependency_graph.clear();
        self.node_indices.clear();
    }
    
    pub fn remove_extension(&mut self, extension: &ExtensionId) {
        if let Some(idx) = self.node_indices.remove(extension) {
            self.dependency_graph.remove_node(idx);
        }
    }
} 