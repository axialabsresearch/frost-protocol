/*!
# Routing Strategy Implementation

This module provides routing strategy implementations for the FROST protocol,
offering various algorithms for path selection and optimization.

## Core Components

### Strategy Interface
The routing strategy interface defines:
- Path computation
- Metric updates
- Route optimization
- Strategy selection

### Default Strategy
Basic routing implementation:
- Shortest path routing
- Load awareness
- Failure handling
- Performance tracking

### Advanced Strategies
Specialized routing algorithms:
- Priority-based routing
- Load-balanced routing
- Latency-optimized routing
- Reliability-focused routing

## Architecture

The strategy system implements several key algorithms:

1. **Path Finding**
   ```rust
   fn compute_path(&self, from: &ChainId, to: &ChainId, topology: &NetworkTopology) -> Vec<ChainId> {
       // Dijkstra's algorithm with weighted edges
   }
   ```
   - Shortest path
   - Weight consideration
   - Constraint handling
   - Alternative paths

2. **Load Balancing**
   ```rust
   fn balance_load(&self, paths: Vec<Vec<ChainId>>, metrics: &RouteMetrics) -> Vec<ChainId> {
       // Load distribution across available paths
   }
   ```
   - Load distribution
   - Performance weighting
   - Health consideration
   - Resource balancing

3. **Route Optimization**
   ```rust
   fn optimize_route(&self, path: Vec<ChainId>, constraints: &RouteConstraints) -> Vec<ChainId> {
       // Path optimization with constraints
   }
   ```
   - Path refinement
   - Constraint satisfaction
   - Performance optimization
   - Resource efficiency

## Features

### Path Selection
- Multiple algorithms
- Weight consideration
- Constraint handling
- Alternative paths

### Load Distribution
- Load balancing
- Performance weighting
- Resource allocation
- Health awareness

### Optimization
- Path refinement
- Constraint handling
- Performance tuning
- Resource efficiency

### Adaptability
- Dynamic adjustment
- Metric response
- Health adaptation
- Load response

## Best Practices

### Strategy Selection
1. Algorithm Choice
   - Use case matching
   - Performance needs
   - Resource constraints
   - Reliability requirements

2. Configuration
   - Weight settings
   - Timeout values
   - Retry limits
   - Health thresholds

3. Optimization
   - Regular tuning
   - Metric analysis
   - Performance monitoring
   - Resource tracking

4. Health Management
   - Health checks
   - Failure handling
   - Recovery procedures
   - Performance monitoring

## Integration

### Router Integration
- Strategy selection
- Path computation
- Metric updates
- Health monitoring

### Topology Integration
- Node discovery
- Path computation
- Health tracking
- Resource monitoring

### Metrics Integration
- Performance tracking
- Resource monitoring
- Health tracking
- Usage statistics

### Constraint Handling
- Path constraints
- Resource limits
- Performance requirements
- Health thresholds

## Performance Considerations

### Algorithm Efficiency
- Computation cost
- Memory usage
- Cache utilization
- Resource sharing

### Optimization
- Path computation
- Load distribution
- Resource allocation
- Cache management

### Monitoring
- Performance metrics
- Resource usage
- Health status
- Error rates

### Tuning
- Algorithm parameters
- Weight values
- Cache sizes
- Timeout settings

## Implementation Notes

### Path Finding
The path finding algorithms consider:
- Node distance
- Link quality
- Resource availability
- Health status

### Load Balancing
Load distribution takes into account:
- Current load
- Node capacity
- Link quality
- Health status

### Route Optimization
Path optimization considers:
- Performance requirements
- Resource constraints
- Health status
- Current load

### Health Management
Health tracking includes:
- Node status
- Link quality
- Resource availability
- Error rates
*/

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use async_trait::async_trait;
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::RwLock;
use crate::state::ChainId;
use crate::Result;
use super::topology::{NetworkTopology, TopologyNode};



/// Strategy for finding routes between chains
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Find route between chains
    async fn find_route(
        &mut self,
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
        &mut self,
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
        &mut self,
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

/// Multi-path routing strategy for resilience
pub struct MultiPathStrategy {
    route_cache: RwLock<HashMap<(ChainId, ChainId), Vec<Vec<ChainId>>>>,
    max_paths: usize,
    min_disjoint_paths: usize,
}

impl Clone for MultiPathStrategy {
    fn clone(&self) -> Self {
        Self {
            route_cache: RwLock::new(HashMap::new()),
            max_paths: self.max_paths,
            min_disjoint_paths: self.min_disjoint_paths,
        }
    }
}

impl MultiPathStrategy {
    /// Create new multi-path strategy
    pub fn new(max_paths: usize, min_disjoint_paths: usize) -> Self {
        Self {
            route_cache: RwLock::new(HashMap::new()),
            max_paths,
            min_disjoint_paths,
        }
    }
    
    /// Find multiple disjoint paths using modified BFS
    fn find_disjoint_paths(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Vec<Vec<ChainId>> {
        let mut paths = Vec::new();
        let mut used_nodes = HashSet::new();
        
        // Find paths until we reach max_paths or can't find more
        while paths.len() < self.max_paths {
            if let Some(path) = self.find_path_avoiding_nodes(topology, from, to, &used_nodes) {
                // Add nodes from this path to used set
                for node in &path {
                    used_nodes.insert(node.clone());
                }
                paths.push(path);
            } else {
                break;
            }
        }
        
        paths
    }
    
    /// Find a path avoiding certain nodes
    fn find_path_avoiding_nodes(
        &self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
        avoid_nodes: &HashSet<ChainId>,
    ) -> Option<Vec<ChainId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut prev = HashMap::new();
        
        queue.push_back(from.clone());
        visited.insert(from.clone());
        
        while let Some(current) = queue.pop_front() {
            if &current == to {
                return Some(self.reconstruct_path(from, to, &prev));
            }
            
            if let Some(node) = topology.get_node(&current) {
                for neighbor in &node.connections {
                    if !visited.contains(neighbor) && !avoid_nodes.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                        visited.insert(neighbor.clone());
                        prev.insert(neighbor.clone(), current.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Reconstruct path from prev map
    fn reconstruct_path(
        &self,
        from: &ChainId,
        to: &ChainId,
        prev: &HashMap<ChainId, ChainId>,
    ) -> Vec<ChainId> {
        let mut path = Vec::new();
        let mut current = to.clone();
        
        while current != *from {
            path.push(current.clone());
            current = prev.get(&current).unwrap().clone();
        }
        path.push(from.clone());
        path.reverse();
        
        path
    }
}

#[async_trait]
impl RoutingStrategy for MultiPathStrategy {
    async fn find_route(
        &mut self,
        topology: &NetworkTopology,
        from: &ChainId,
        to: &ChainId,
    ) -> Result<Vec<ChainId>> {
        // Check cache first
        let cache_key = (from.clone(), to.clone());
        let cache = self.route_cache.read().await;
        if let Some(routes) = cache.get(&cache_key) {
            // Return first available path
            return Ok(routes[0].clone());
        }
        drop(cache); // Release read lock
        
        // Find multiple disjoint paths
        let paths = self.find_disjoint_paths(topology, from, to);
        
        if paths.len() >= self.min_disjoint_paths {
            // Cache the paths
            let mut cache = self.route_cache.write().await;
            cache.insert(cache_key, paths.clone());
            Ok(paths[0].clone())
        } else {
            Ok(Vec::new()) // Not enough disjoint paths found
        }
    }
} 