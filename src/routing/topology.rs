use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::state::ChainId;

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
} 