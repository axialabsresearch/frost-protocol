#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

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
        let mut prev: HashMap<ChainId, ChainId> = HashMap::new();
        
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

    /// Find shortest path using Dijkstra's algorithm
    fn find_shortest_path(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Option<Vec<ChainId>> {
        use std::collections::BinaryHeap;
        use std::cmp::Ordering;

        // Custom wrapper for f64 to implement Ord
        #[derive(Copy, Clone, PartialEq)]
        struct Distance(f64);

        impl Eq for Distance {}

        impl PartialOrd for Distance {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                other.0.partial_cmp(&self.0)
            }
        }

        impl Ord for Distance {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap_or(Ordering::Equal)
            }
        }

        let mut distances: HashMap<ChainId, f64> = HashMap::new();
        let mut prev: HashMap<ChainId, ChainId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        // Initialize distances
        distances.insert(from.clone(), 0.0);
        heap.push((Distance(0.0), from.clone()));

        while let Some((Distance(dist), current)) = heap.pop() {
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

            // Skip if we've found a better path
            if let Some(&best) = distances.get(&current) {
                if dist > best {
                    continue;
                }
            }

            // Check neighbors
            if let Some(node) = topology.get_node(&current) {
                for neighbor in &node.connections {
                    let edge_weight = self.calculate_edge_weight(&current, neighbor);
                    let new_dist = dist + edge_weight;

                    if !distances.contains_key(neighbor) || new_dist < *distances.get(neighbor).unwrap() {
                        distances.insert(neighbor.clone(), new_dist);
                        prev.insert(neighbor.clone(), current.clone());
                        heap.push((Distance(new_dist), neighbor.clone()));
                    }
                }
            }
        }

        None
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