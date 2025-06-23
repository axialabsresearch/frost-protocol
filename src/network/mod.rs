mod protocol;
mod transport;
mod peer;
mod error;
mod discovery;
mod security;
mod circuit_breaker;
mod backpressure;
mod pool;
mod retry;
mod telemetry;
mod p2p;

pub use protocol::{NetworkProtocol, ProtocolConfig};
pub use transport::{Transport, TransportConfig};
pub use peer::{Peer, PeerInfo, PeerManager};
pub use error::NetworkError;
pub use discovery::{PeerDiscovery, DiscoveryConfig, PeerHealthCheck};
pub use security::{SecurityManager, SecurityConfig, AuthenticationResult};
pub use circuit_breaker::{CircuitBreaker, CircuitConfig, CircuitState};
pub use backpressure::{BackpressureController, BackpressureConfig, PressureLevel};
pub use pool::{ConnectionPool, PoolConfig, PooledConnection};
pub use retry::{RetryPolicy, RetryConfig, with_retry};
pub use telemetry::{TelemetryManager, NetworkMetrics, NetworkEvent};
pub use p2p::{P2PNode, P2PConfig, NodeIdentity};

use crate::Result;
use async_trait::async_trait;
use std::error::Error;
use crate::message::FrostMessage;

/// Network protocol trait
#[async_trait]
pub trait NetworkProtocol: Send + Sync {
    /// Start the network protocol
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;

    /// Stop the network protocol
    async fn stop(&mut self) -> Result<(), Box<dyn Error>>;

    /// Broadcast a message to the network
    async fn broadcast(&self, message: FrostMessage) -> Result<(), Box<dyn Error>>;

    /// Send a message to a specific peer
    async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<(), Box<dyn Error>>;

    /// Get connected peers
    async fn get_peers(&self) -> Result<Vec<String>, Box<dyn Error>>;
}

/// Basic network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Node ID
    pub node_id: String,
    /// Listen address
    pub listen_addr: String,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    /// Protocol version
    pub protocol_version: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            node_id: "".to_string(),
            listen_addr: "127.0.0.1:0".to_string(),
            bootstrap_peers: vec![],
            protocol_version: 0,
        }
    }
}

/// Basic network metrics
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Connected peers count
    pub connected_peers: u64,
}

/// Basic network implementation
pub struct BasicNetwork {
    config: NetworkConfig,
    metrics: NetworkMetrics,
}

impl BasicNetwork {
    /// Create a new basic network
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            metrics: NetworkMetrics::default(),
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.clone()
    }
}

#[async_trait]
impl NetworkProtocol for BasicNetwork {
    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        // Basic startup for v0
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        // Basic shutdown for v0
        Ok(())
    }

    async fn broadcast(&self, message: FrostMessage) -> Result<(), Box<dyn Error>> {
        // Basic broadcast for v0
        Ok(())
    }

    async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<(), Box<dyn Error>> {
        // Basic send for v0
        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // Basic peer list for v0
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_network() {
        let config = NetworkConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        
        let mut network = BasicNetwork::new(config);
        
        assert!(network.start().await.is_ok());
        assert!(network.stop().await.is_ok());
        
        let message = FrostMessage::new(
            crate::message::MessageType::Discovery,
            vec![1, 2, 3],
            "node1".to_string(),
            None,
        );
        
        assert!(network.broadcast(message.clone()).await.is_ok());
        assert!(network.send_to("node2", message).await.is_ok());
        
        let peers = network.get_peers().await.unwrap();
        assert!(peers.is_empty());
    }
}
