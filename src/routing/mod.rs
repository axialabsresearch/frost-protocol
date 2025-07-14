#![allow(unused_imports)]

pub mod router;
pub mod strategy;
pub mod topology;

pub use router::{MessageRouter as ImportedMessageRouter, RouterConfig};
pub use strategy::{RoutingStrategy, DefaultStrategy};
pub use topology::{NetworkTopology, TopologyNode};

use crate::Result;

use std::collections::HashMap;
use std::error::Error;
use async_trait::async_trait;
use crate::message::{FrostMessage, MessageType};
use crate::network::NetworkProtocol;

/// Message router trait
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// Route a message
    async fn route(&self, message: FrostMessage) -> std::result::Result<(), Box<dyn Error>>;

    /// Update routing table
    async fn update_routes(&mut self, routes: HashMap<String, String>) -> std::result::Result<(), Box<dyn Error>>;

    /// Get current routes
    async fn get_routes(&self) -> std::result::Result<HashMap<String, String>, Box<dyn Error>>;
}

/// Basic routing configuration
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Node ID
    pub node_id: String,
    /// Route timeout
    pub route_timeout: u64,
    /// Max routes
    pub max_routes: usize,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            node_id: "".to_string(),
            route_timeout: 3600,
            max_routes: 1000,
        }
    }
}

/// Basic routing metrics
#[derive(Debug, Clone, Default)]
pub struct RoutingMetrics {
    /// Total messages routed
    pub messages_routed: u64,
    /// Failed routes
    pub failed_routes: u64,
    /// Active routes
    pub active_routes: usize,
}

/// Basic router implementation
pub struct BasicRouter<N: NetworkProtocol> {
    config: RoutingConfig,
    metrics: RoutingMetrics,
    routes: HashMap<String, String>,
    network: N,
}

impl<N: NetworkProtocol> BasicRouter<N> {
    /// Create a new basic router
    pub fn new(config: RoutingConfig, network: N) -> Self {
        Self {
            config,
            metrics: RoutingMetrics::default(),
            routes: HashMap::new(),
            network,
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> RoutingMetrics {
        self.metrics.clone()
    }
}

#[async_trait]
impl<N: NetworkProtocol> MessageRouter for BasicRouter<N> {
    async fn route(&self, message: FrostMessage) -> std::result::Result<(), Box<dyn Error>> {
        // Basic routing for v0
        if let Some(target) = message.target.as_ref() {
            if let Some(next_hop) = self.routes.get(target) {
                self.network.send_to(next_hop, message).await?;
            } else {
                self.network.broadcast(message).await?;
            }
        } else {
            self.network.broadcast(message).await?;
        }
        Ok(())
    }

    async fn update_routes(&mut self, routes: HashMap<String, String>) -> std::result::Result<(), Box<dyn Error>> {
        // Basic route update for v0
        if routes.len() <= self.config.max_routes {
            self.routes = routes;
            Ok(())
        } else {
            Err("Too many routes".into())
        }
    }

    async fn get_routes(&self) -> std::result::Result<HashMap<String, String>, Box<dyn Error>> {
        Ok(self.routes.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock network implementation for testing
    struct MockNetwork {
        sent_messages: Arc<Mutex<Vec<(String, FrostMessage)>>>,
    }

    impl MockNetwork {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl NetworkProtocol for MockNetwork {
        async fn start(&mut self) -> Result<()> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<()> {
            Ok(())
        }

        async fn broadcast(&self, message: FrostMessage) -> Result<()> {
            self.sent_messages.lock().await.push(("broadcast".to_string(), message));
            Ok(())
        }

        async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<()> {
            self.sent_messages.lock().await.push((peer_id.to_string(), message));
            Ok(())
        }

        async fn get_peers(&self) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_basic_routing() {
        let config = RoutingConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        
        let network = MockNetwork::new();
        let sent_messages = network.sent_messages.clone();
        let mut router = BasicRouter::new(config, network);
        
        // Test route update
        let mut routes = HashMap::new();
        routes.insert("node2".to_string(), "peer1".to_string());
        assert!(router.update_routes(routes).await.is_ok());
        
        // Test message routing
        let message = FrostMessage::new(
            MessageType::Discovery,
            vec![1, 2, 3],
            "node1".to_string(),
            Some("node2".to_string()),
        );
        
        assert!(router.route(message).await.is_ok());
        
        let sent = sent_messages.lock().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "peer1");
    }
}
