use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::state::ChainId;
use std::collections::HashSet;
use std::collections::VecDeque;

/// Network topology representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkTopology {
    nodes: HashMap<ChainId, TopologyNode>,
    version: u64,
    last_updated: u64,
}

/// Node in the network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    pub chain_id: ChainId,
    pub connections: Vec<ChainId>,
    pub metadata: NodeMetadata,
    pub status: NodeStatus,
}

/// Metadata for topology nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub name: String,
    pub chain_type: ChainType,
    pub protocol_version: String,
    pub supported_features: Vec<String>,
    pub performance_metrics: PerformanceMetrics,
}

/// Chain types in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainType {
    Layer1,
    Layer2,
    Sidechain,
    Bridge,
    Custom(String),
}

/// Performance metrics for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: f64,
    pub throughput: f64,
    pub reliability: f64,
    pub last_active: u64,
}

/// Status of nodes in the network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Degraded,
    Inactive,
    Maintenance,
}

impl NetworkTopology {
    /// Create new network topology
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            version: 0,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Add node to topology
    pub fn add_node(&mut self, node: TopologyNode) {
        self.nodes.insert(node.chain_id.clone(), node);
        self.version += 1;
        self.update_timestamp();
    }
    
    /// Remove node from topology
    pub fn remove_node(&mut self, chain_id: &ChainId) -> Option<TopologyNode> {
        let node = self.nodes.remove(chain_id);
        if node.is_some() {
            self.version += 1;
            self.update_timestamp();
        }
        node
    }
    
    /// Get node from topology
    pub fn get_node(&self, chain_id: &ChainId) -> Option<&TopologyNode> {
        self.nodes.get(chain_id)
    }
    
    /// Get mutable node from topology
    pub fn get_node_mut(&mut self, chain_id: &ChainId) -> Option<&mut TopologyNode> {
        self.nodes.get_mut(chain_id)
    }
    
    /// Add connection between nodes
    pub fn add_connection(&mut self, from: &ChainId, to: &ChainId) -> bool {
        let mut modified = false;
        
        if let Some(node) = self.nodes.get_mut(from) {
            if !node.connections.contains(to) {
                node.connections.push(to.clone());
                modified = true;
            }
        }
        
        if let Some(node) = self.nodes.get_mut(to) {
            if !node.connections.contains(from) {
                node.connections.push(from.clone());
                modified = true;
            }
        }
        
        if modified {
            self.version += 1;
            self.update_timestamp();
        }
        
        modified
    }
    
    /// Remove connection between nodes
    pub fn remove_connection(&mut self, from: &ChainId, to: &ChainId) -> bool {
        let mut modified = false;
        
        if let Some(node) = self.nodes.get_mut(from) {
            if let Some(pos) = node.connections.iter().position(|x| x == to) {
                node.connections.remove(pos);
                modified = true;
            }
        }
        
        if let Some(node) = self.nodes.get_mut(to) {
            if let Some(pos) = node.connections.iter().position(|x| x == from) {
                node.connections.remove(pos);
                modified = true;
            }
        }
        
        if modified {
            self.version += 1;
            self.update_timestamp();
        }
        
        modified
    }
    
    /// Get all nodes in the topology
    pub fn nodes(&self) -> &HashMap<ChainId, TopologyNode> {
        &self.nodes
    }
    
    /// Get topology version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get last update timestamp
    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }
    
    /// Update timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Calculate network density
    pub fn network_density(&self) -> f64 {
        let n = self.nodes.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        
        let max_edges = n * (n - 1.0) / 2.0;
        let actual_edges = self.nodes.values()
            .map(|node| node.connections.len() as f64)
            .sum::<f64>() / 2.0;
            
        actual_edges / max_edges
    }
    
    /// Get node degree distribution
    pub fn degree_distribution(&self) -> HashMap<usize, usize> {
        let mut distribution = HashMap::new();
        
        for node in self.nodes.values() {
            let degree = node.connections.len();
            *distribution.entry(degree).or_insert(0) += 1;
        }
        
        distribution
    }
    
    /// Check for network partitions
    pub fn detect_partitions(&self) -> Vec<Vec<ChainId>> {
        let mut partitions = Vec::new();
        let mut visited = HashSet::new();
        
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                let mut partition = Vec::new();
                let mut queue = VecDeque::new();
                
                queue.push_back(node_id.clone());
                visited.insert(node_id.clone());
                
                while let Some(current) = queue.pop_front() {
                    partition.push(current.clone());
                    
                    if let Some(node) = self.nodes.get(&current) {
                        for neighbor in &node.connections {
                            if !visited.contains(neighbor) {
                                queue.push_back(neighbor.clone());
                                visited.insert(neighbor.clone());
                            }
                        }
                    }
                }
                
                partitions.push(partition);
            }
        }
        
        partitions
    }
    
    /// Calculate node health score (0.0-1.0)
    pub fn node_health(&self, chain_id: &ChainId) -> f64 {
        if let Some(node) = self.nodes.get(chain_id) {
            let metrics = &node.metadata.performance_metrics;
            
            // Weight different factors
            let latency_score = 1.0 / (1.0 + metrics.latency_ms / 1000.0);
            let throughput_score = metrics.throughput / 100.0;
            let reliability_score = metrics.reliability;
            
            // Active nodes get full score
            let status_score = match node.status {
                NodeStatus::Active => 1.0,
                NodeStatus::Degraded => 0.5,
                NodeStatus::Inactive => 0.0,
                NodeStatus::Maintenance => 0.0,
            };
            
            // Combine scores with weights
            let score = 0.3 * latency_score +
                       0.2 * throughput_score +
                       0.3 * reliability_score +
                       0.2 * status_score;
                       
            score.min(1.0).max(0.0)
        } else {
            0.0
        }
    }
    
    /// Get critical nodes (high centrality)
    pub fn critical_nodes(&self) -> Vec<ChainId> {
        let mut centrality = HashMap::new();
        
        // Calculate betweenness centrality
        for start in self.nodes.keys() {
            for end in self.nodes.keys() {
                if start == end {
                    continue;
                }
                
                let mut visited = HashSet::new();
                let mut queue = VecDeque::new();
                let mut paths = HashMap::new();
                
                queue.push_back(start.clone());
                visited.insert(start.clone());
                paths.insert(start.clone(), vec![start.clone()]);
                
                while let Some(current) = queue.pop_front() {
                    if &current == end {
                        // Found path, increment centrality for intermediate nodes
                        let path = paths.get(&current).unwrap();
                        for node in path.iter().skip(1).take(path.len() - 2) {
                            *centrality.entry(node.clone()).or_insert(0) += 1;
                        }
                    }
                    
                    if let Some(node) = self.nodes.get(&current) {
                        for neighbor in &node.connections {
                            if !visited.contains(neighbor) {
                                visited.insert(neighbor.clone());
                                queue.push_back(neighbor.clone());
                                
                                // Extend path
                                let mut new_path = paths.get(&current).unwrap().clone();
                                new_path.push(neighbor.clone());
                                paths.insert(neighbor.clone(), new_path);
                            }
                        }
                    }
                }
            }
        }
        
        // Return nodes with high centrality
        let threshold = (self.nodes.len() as f64 * 0.1).ceil() as usize;
        let mut critical: Vec<_> = centrality.into_iter()
            .filter(|(_, count)| *count > threshold)
            .map(|(node, _)| node)
            .collect();
            
        critical.sort_by_key(|node| self.node_health(node) as i32);
        critical
    }
    
    /// Get network resilience score (0.0-1.0)
    pub fn resilience_score(&self) -> f64 {
        let partitions = self.detect_partitions();
        let critical = self.critical_nodes();
        
        // Factors affecting resilience:
        // 1. Number of partitions (fewer is better)
        let partition_score = 1.0 / partitions.len() as f64;
        
        // 2. Critical node ratio (fewer is better)
        let critical_ratio = critical.len() as f64 / self.nodes.len() as f64;
        let critical_score = 1.0 - critical_ratio;
        
        // 3. Average node health
        let health_score = self.nodes.keys()
            .map(|node| self.node_health(node))
            .sum::<f64>() / self.nodes.len() as f64;
            
        // 4. Network density (higher is better)
        let density_score = self.network_density();
        
        // Combine scores
        let score = 0.3 * partition_score +
                   0.3 * critical_score +
                   0.2 * health_score +
                   0.2 * density_score;
                   
        score.min(1.0).max(0.0)
    }
} 