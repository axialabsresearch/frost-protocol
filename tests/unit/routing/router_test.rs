#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::{
    routing::{
        MessageRouter,
        RoutingConfig,
        NetworkTopology,
        TopologyNode,
        BasicRouter,
        RoutingMetrics,
        topology::{NodeMetadata, NodeStatus, ChainType, PerformanceMetrics},
    },
    message::{FrostMessage, MessageType},
    state::ChainId,
    network::NetworkProtocol,
    Result,
};

use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;

// Mock network implementation for testing
#[derive(Default)]
struct MockNetwork {
    sent_messages: HashMap<String, Vec<FrostMessage>>,
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
        Ok(())
    }

    async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<()> {
        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<String>> {
        Ok(vec!["peer1".to_string(), "peer2".to_string()])
    }
}

#[tokio::test]
async fn test_basic_routing() {
    let config = RoutingConfig {
        node_id: "test_node".to_string(),
        route_timeout: 3600,
        max_routes: 1000,
    };
    let network = MockNetwork::default();
    let router = BasicRouter::new(config, network);
    
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    
    let msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "test_node".to_string(),
        Some("target_node".to_string()),
    );
    
    let result = router.route(msg).await;
    assert!(result.is_ok(), "Basic routing failed");
}

#[tokio::test]
async fn test_route_discovery() {
    let config = RoutingConfig {
        node_id: "test_node".to_string(),
        route_timeout: 3600,
        max_routes: 1000,
    };
    let network = MockNetwork::default();
    let router = BasicRouter::new(config, network);
    
    let source = ChainId::new("ethereum");
    let target = ChainId::new("polygon");
    
    let routes = router.get_routes().await;
    assert!(routes.is_ok(), "Route discovery failed");
}

#[tokio::test]
async fn test_topology_update() {
    let config = RoutingConfig {
        node_id: "test_node".to_string(),
        route_timeout: 3600,
        max_routes: 1000,
    };
    let network = MockNetwork::default();
    let mut router = BasicRouter::new(config, network);
    
    let mut topology = NetworkTopology::new();
    
    // Create nodes with chain IDs
    let eth_chain = ChainId::new("ethereum");
    let polygon_chain = ChainId::new("polygon");
    
    let eth_node = TopologyNode {
        chain_id: eth_chain.clone(),
        connections: vec![polygon_chain.clone()],
        metadata: NodeMetadata {
            name: "eth_node".to_string(),
            chain_type: ChainType::Layer1,
            protocol_version: "1.0".to_string(),
            supported_features: vec![],
            performance_metrics: PerformanceMetrics {
                latency_ms: 0.0,
                throughput: 0.0,
                reliability: 1.0,
                last_active: 0,
            },
        },
        status: NodeStatus::Active,
    };
    
    let polygon_node = TopologyNode {
        chain_id: polygon_chain.clone(),
        connections: vec![eth_chain.clone()],
        metadata: NodeMetadata {
            name: "polygon_node".to_string(),
            chain_type: ChainType::Layer2,
            protocol_version: "1.0".to_string(),
            supported_features: vec![],
            performance_metrics: PerformanceMetrics {
                latency_ms: 0.0,
                throughput: 0.0,
                reliability: 1.0,
                last_active: 0,
            },
        },
        status: NodeStatus::Active,
    };
    
    topology.add_node(eth_node);
    topology.add_node(polygon_node);
    
    let mut routes = HashMap::new();
    routes.insert("node2".to_string(), "peer1".to_string());
    let result = router.update_routes(routes).await;
    assert!(result.is_ok(), "Route update failed");
}

#[tokio::test]
async fn test_route_metrics() {
    let config = RoutingConfig {
        node_id: "test_node".to_string(),
        route_timeout: 3600,
        max_routes: 1000,
    };
    let network = MockNetwork::default();
    let router = BasicRouter::new(config, network);
    
    let metrics = router.get_metrics();
    assert_eq!(metrics.messages_routed, 0);
    assert_eq!(metrics.failed_routes, 0);
    assert_eq!(metrics.active_routes, 0);
} 