#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, warn, error};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use rand::Rng;
use std::error::Error;

use crate::message::{FrostMessage, MessageError};
use crate::state::ChainId;
use crate::Result;
use super::{
    RoutingStrategy,
    DefaultStrategy,
    NetworkTopology,
    TopologyNode,
    NetworkProtocol,
    RoutingConfig,
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

/// Router health status
#[derive(Debug, Clone, PartialEq)]
pub enum RouterHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Circuit breaker state
#[derive(Debug, Clone)]
struct CircuitBreaker {
    failures: u32,
    last_failure: Instant,
    reset_timeout: Duration,
    failure_threshold: u32,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failures: 0,
            last_failure: Instant::now(),
            reset_timeout,
            failure_threshold,
        }
    }

    fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Instant::now();
    }

    fn record_success(&mut self) {
        if self.last_failure.elapsed() >= self.reset_timeout {
            self.failures = 0;
        }
    }

    fn is_open(&self) -> bool {
        self.failures >= self.failure_threshold && 
        self.last_failure.elapsed() < self.reset_timeout
    }
}

/// Enhanced router with load balancing and circuit breaking
pub struct EnhancedRouter<N: NetworkProtocol> {
    config: RoutingConfig,
    routes: RwLock<HashMap<String, Vec<String>>>, // Multiple next hops per target
    route_weights: RwLock<HashMap<String, f64>>,
    circuit_breakers: RwLock<HashMap<String, CircuitBreaker>>,
    route_usage: RwLock<HashMap<String, VecDeque<Instant>>>,
    topology: RwLock<NetworkTopology>,
    strategy: RwLock<Box<dyn RoutingStrategy>>,
    metrics: RwLock<RouteMetrics>,
    network: N,
}

impl<N: NetworkProtocol> EnhancedRouter<N> {
    /// Create new enhanced router
    pub fn new(config: RoutingConfig, network: N) -> Self {
        Self {
            config,
            routes: RwLock::new(HashMap::new()),
            route_weights: RwLock::new(HashMap::new()),
            circuit_breakers: RwLock::new(HashMap::new()),
            route_usage: RwLock::new(HashMap::new()),
            topology: RwLock::new(NetworkTopology::new()),
            strategy: RwLock::new(Box::new(DefaultStrategy::new())),
            metrics: RwLock::new(RouteMetrics::default()),
            network,
        }
    }
    
    /// Get router health status
    pub async fn health(&self) -> RouterHealth {
        let breakers = self.circuit_breakers.read().await;
        let open_circuits = breakers.values().filter(|b| b.is_open()).count();
        
        if open_circuits == 0 {
            RouterHealth::Healthy
        } else if open_circuits < breakers.len() / 2 {
            RouterHealth::Degraded
        } else {
            RouterHealth::Unhealthy
        }
    }

    /// Select next hop using weighted round robin
    async fn select_next_hop(&self, target: &str) -> Option<String> {
        let routes = self.routes.read().await;
        let weights = self.route_weights.read().await;
        let breakers = self.circuit_breakers.read().await;
        
        if let Some(hops) = routes.get(target) {
            // Filter out nodes with open circuit breakers
            let available_hops: Vec<_> = hops.iter()
                .filter(|hop| {
                    !breakers.get(*hop)
                        .map(|b| b.is_open())
                        .unwrap_or(false)
                })
                .collect();

            if available_hops.is_empty() {
                return None;
            }

            // Calculate total weight
            let total_weight: f64 = available_hops.iter()
                .map(|hop| weights.get(hop.as_str()).unwrap_or(&1.0))
                .sum();

            // Select hop based on weights
            let mut rng = rand::rng();
            let mut choice = rng.random::<f64>() * total_weight;

            for hop in available_hops {
                let weight = weights.get(hop.as_str()).unwrap_or(&1.0);
                if choice <= *weight {
                    return Some(hop.clone());
                }
                choice -= weight;
            }
        }
        
        None
    }

    /// Update route weights based on performance
    async fn update_weights(&self) {
        let mut weights = self.route_weights.write().await;
        let usage = self.route_usage.read().await;
        let window = Duration::from_secs(60); // 1 minute window
        
        for (route, times) in usage.iter() {
            let recent_count = times.iter()
                .filter(|t| t.elapsed() < window)
                .count();
            
            // Update weight based on usage
            let new_weight = 1.0 / (1.0 + recent_count as f64 / 100.0);
            weights.insert(route.clone(), new_weight);
        }
    }
}

#[async_trait]
impl<N: NetworkProtocol> MessageRouter for EnhancedRouter<N> {
    async fn route_message(&self, message: FrostMessage) -> Result<RouteStatus> {
        if let Some(target) = message.target.as_ref() {
            // Try to get next hop with load balancing
            if let Some(next_hop) = self.select_next_hop(target).await {
        let status = RouteStatus {
            message_id: uuid::Uuid::new_v4(),
                    route: vec![ChainId::new(&next_hop)],
            current_hop: 0,
                    estimated_time: std::time::Duration::from_secs(self.config.route_timeout),
            state: RouteState::Planning,
        };
        
                Ok(status)
            } else {
                // Fallback to broadcast if no available route
                let status = RouteStatus {
                    message_id: uuid::Uuid::new_v4(),
                    route: Vec::new(),
                    current_hop: 0,
                    estimated_time: std::time::Duration::from_secs(self.config.route_timeout),
                    state: RouteState::Failed("No available route".into()),
                };
                Ok(status)
            }
        } else {
            // Broadcast messages without target
            let status = RouteStatus {
                message_id: uuid::Uuid::new_v4(),
                route: Vec::new(),
                current_hop: 0,
                estimated_time: std::time::Duration::from_secs(self.config.route_timeout),
                state: RouteState::Failed("Broadcast message".into()),
            };
        Ok(status)
        }
    }
    
    async fn get_route(&self, from: ChainId, to: ChainId) -> Result<Vec<ChainId>> {
        let topology = self.topology.read().await;
        let mut strategy = self.strategy.write().await;
        strategy.find_route(&topology, &from, &to).await
    }
    
    async fn update_topology(&mut self, topology: NetworkTopology) -> Result<()> {
        *self.topology.write().await = topology;
        Ok(())
    }
    
    async fn get_metrics(&self) -> Result<RouteMetrics> {
        Ok(self.metrics.read().await.clone())
    }
}
