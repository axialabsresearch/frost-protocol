#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, warn, error};
use std::collections::HashMap;

use crate::message::{FrostMessage, MessageError};
use crate::state::ChainId;
use crate::Result;
use super::{
    RoutingStrategy,
    DefaultStrategy,
    NetworkTopology,
    TopologyNode,
};

/// Configuration for message routing
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Maximum number of hops for message routing
    pub max_hops: u32,
    /// Timeout for message routing in seconds
    pub route_timeout: u64,
    /// Maximum number of parallel routes
    pub max_parallel_routes: usize,
    /// Chain-specific routing parameters
    pub chain_params: HashMap<ChainId, ChainRouteParams>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_hops: 3,
            route_timeout: 60,
            max_parallel_routes: 4,
            chain_params: HashMap::new(),
        }
    }
}

/// Chain-specific routing parameters
#[derive(Debug, Clone)]
pub struct ChainRouteParams {
    pub preferred_routes: Vec<ChainId>,
    pub blacklisted_routes: Vec<ChainId>,
    pub max_message_size: usize,
    pub priority_level: u8,
}

/// Core message router interface
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// Route a message to its destination
    async fn route_message(&self, message: FrostMessage) -> Result<RouteStatus>;
    
    /// Get optimal route between chains
    async fn get_route(&self, from: ChainId, to: ChainId) -> Result<Vec<ChainId>>;
    
    /// Update routing topology
    async fn update_topology(&mut self, topology: NetworkTopology) -> Result<()>;
    
    /// Get current route metrics
    async fn get_metrics(&self) -> Result<RouteMetrics>;
}

/// Status of a routed message
#[derive(Debug, Clone)]
pub struct RouteStatus {
    pub message_id: uuid::Uuid,
    pub route: Vec<ChainId>,
    pub current_hop: usize,
    pub estimated_time: std::time::Duration,
    pub state: RouteState,
}

/// State of a route
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteState {
    Planning,
    InProgress,
    Completed,
    Failed(String),
}

/// Metrics for message routing
#[derive(Debug, Clone, Default)]
pub struct RouteMetrics {
    pub active_routes: usize,
    pub completed_routes: u64,
    pub failed_routes: u64,
    pub average_route_time: f64,
    pub route_success_rate: f64,
}

/// Implementation of message router
pub struct BasicMessageRouter {
    config: RouterConfig,
    topology: RwLock<NetworkTopology>,
    strategy: Box<dyn RoutingStrategy>,
    active_routes: RwLock<HashMap<uuid::Uuid, RouteStatus>>,
    metrics: RwLock<RouteMetrics>,
}

impl BasicMessageRouter {
    /// Create new message router
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            topology: RwLock::new(NetworkTopology::default()),
            strategy: Box::new(DefaultStrategy::new()),
            active_routes: RwLock::new(HashMap::new()),
            metrics: RwLock::new(RouteMetrics::default()),
        }
    }
    
    /// Update route metrics
    async fn update_metrics(&self, success: bool, duration: std::time::Duration) {
        let mut metrics = self.metrics.write().await;
        if success {
            metrics.completed_routes += 1;
        } else {
            metrics.failed_routes += 1;
        }
        
        let total_routes = metrics.completed_routes + metrics.failed_routes;
        metrics.route_success_rate = metrics.completed_routes as f64 / total_routes as f64;
        
        // Update average route time using exponential moving average
        let alpha = 0.1;
        metrics.average_route_time = (1.0 - alpha) * metrics.average_route_time +
            alpha * duration.as_secs_f64();
    }
}

#[async_trait]
impl MessageRouter for BasicMessageRouter {
    async fn route_message(&self, message: FrostMessage) -> Result<RouteStatus> {
        // Extract source and target chains from message
        let source_chain = ChainId::new(&message.source);
        let target_chain = message.target
            .as_ref()
            .map(|t| ChainId::new(t))
            .ok_or_else(|| MessageError::InvalidFormat("Missing target chain".into()))?;

        // Get route between chains
        let route = self.get_route(source_chain, target_chain).await?;
        
        if route.is_empty() {
            return Err(MessageError::Processing("No valid route found".into()).into());
        }
        
        if route.len() > self.config.max_hops as usize {
            return Err(MessageError::Processing("Route exceeds maximum hops".into()).into());
        }
        
        let status = RouteStatus {
            message_id: uuid::Uuid::new_v4(),
            route: route.clone(),
            current_hop: 0,
            estimated_time: std::time::Duration::from_secs(
                (route.len() as u64) * self.config.route_timeout
            ),
            state: RouteState::Planning,
        };
        
        self.active_routes.write().await.insert(status.message_id, status.clone());
        
        Ok(status)
    }
    
    async fn get_route(&self, from: ChainId, to: ChainId) -> Result<Vec<ChainId>> {
        let topology = self.topology.read().await;
        self.strategy.find_route(&topology, &from, &to).await
    }
    
    async fn update_topology(&mut self, topology: NetworkTopology) -> Result<()> {
        *self.topology.write().await = topology;
        Ok(())
    }
    
    async fn get_metrics(&self) -> Result<RouteMetrics> {
        Ok(self.metrics.read().await.clone())
    }
}
