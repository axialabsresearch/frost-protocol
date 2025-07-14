use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use crate::network::{Peer, NetworkError};
use crate::Result;

/// Transport layer for network communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Initialize the transport
    async fn init(&mut self, config: TransportConfig) -> Result<()>;

    /// Connect to a peer
    async fn connect(&mut self, address: &str) -> Result<Peer>;

    /// Disconnect from a peer
    async fn disconnect(&mut self, peer: &Peer) -> Result<()>;

    /// Send raw data to a peer
    async fn send_data(&self, peer: &Peer, data: &[u8]) -> Result<usize>;

    /// Receive raw data from a peer
    async fn receive_data(&self, peer: &Peer) -> Result<Vec<u8>>;

    /// Check if connected to a peer
    async fn is_connected(&self, peer: &Peer) -> bool;

    /// Get transport metrics
    fn metrics(&self) -> TransportMetrics;
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub protocol: TransportProtocol,
    pub encryption: EncryptionConfig,
    pub compression: CompressionConfig,
    pub timeout: Duration,
    pub buffer_size: usize,
}

/// Supported transport protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP {
        port: u16,
        keep_alive: bool,
    },
    WebSocket {
        url: String,
        use_tls: bool,
    },
    QUIC {
        port: u16,
        cert_path: String,
    },
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_size: usize,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub level: u8,
}

/// Transport metrics
#[derive(Debug, Clone, Default)]
pub struct TransportMetrics {
    pub active_connections: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_errors: u64,
    pub average_latency: Duration,
} 