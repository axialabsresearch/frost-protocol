use async_trait::async_trait;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::state::ChainId;
use crate::Result;
use super::topology::{NetworkTopology, TopologyNode};

/// Strategy for finding routes between chains
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Find route between chains
    async fn find_route(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Result<Vec<ChainId>>;
}

/// Default routing strategy using shortest path
pub struct DefaultStrategy {
    route_cache: HashMap<(ChainId, ChainId), Vec<ChainId>>,
}

impl DefaultStrategy {
    /// Create new default strategy
    pub fn new() -> Self {
        Self {
            route_cache: HashMap::new(),
        }
    }
    
    /// Find shortest path using BFS
    fn find_shortest_path(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Option<Vec<ChainId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut prev = HashMap::new();
        
        queue.push_back(from.clone());
        visited.insert(from.clone());
        
        while let Some(current) = queue.pop_front() {
            if &current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = to.clone();
                
                while current != *from {
                    path.push(current.clone());
                    current = prev.get(&current).unwrap().clone();
                }
                path.push(from.clone());
                path.reverse();
                
                return Some(path);
            }
            
            // Check neighbors
            if let Some(node) = topology.get_node(&current) {
                for neighbor in &node.connections {
                    if !visited.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                        visited.insert(neighbor.clone());
                        prev.insert(neighbor.clone(), current.clone());
                    }
                }
            }
        }
        
        None
    }
}

#[async_trait]
impl RoutingStrategy for DefaultStrategy {
    async fn find_route(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Result<Vec<ChainId>> {
        // Check cache first
        let cache_key = (from.clone(), to.clone());
        if let Some(route) = self.route_cache.get(&cache_key) {
            return Ok(route.clone());
        }
        
        // Find shortest path
        if let Some(path) = self.find_shortest_path(topology, from, to) {
            Ok(path)
        } else {
            Ok(Vec::new()) // No route found
        }
    }
}

/// Weighted routing strategy considering chain parameters
pub struct WeightedStrategy {
    weights: HashMap<ChainId, f64>,
    route_cache: HashMap<(ChainId, ChainId), Vec<ChainId>>,
}

impl WeightedStrategy {
    /// Create new weighted strategy
    pub fn new(weights: HashMap<ChainId, f64>) -> Self {
        Self {
            weights,
            route_cache: HashMap::new(),
        }
    }
    
    /// Calculate edge weight between nodes
    fn calculate_edge_weight(&self, from: &ChainId, to: &ChainId) -> f64 {
        let from_weight = self.weights.get(from).unwrap_or(&1.0);
        let to_weight = self.weights.get(to).unwrap_or(&1.0);
        from_weight * to_weight
    }
}

#[async_trait]
impl RoutingStrategy for WeightedStrategy {
    async fn find_route(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Result<Vec<ChainId>> {
        // TODO: Implement Dijkstra's algorithm with weights
        Ok(Vec::new())
    }
} 