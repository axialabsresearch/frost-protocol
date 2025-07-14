use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::{Duration, SystemTime};
use crate::Result;

/// Peer representation in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Peer {
    pub id: uuid::Uuid,
    pub info: PeerInfo,
    pub state: PeerState,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    pub address: String,
    pub protocol_version: String,
    pub supported_features: Vec<String>,
    pub chain_ids: Vec<u64>,
    pub node_type: NodeType,
}

/// Peer connection state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PeerState {
    Connected,
    Disconnected,
    Banned,
    Handshaking,
}

/// Node type in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    Validator,
    Observer,
    Relay,
    Gateway,
}

/// Peer manager for handling peer connections
#[async_trait]
pub trait PeerManager: Send + Sync {
    /// Add a new peer
    async fn add_peer(&mut self, info: PeerInfo) -> Result<Peer>;

    /// Remove a peer
    async fn remove_peer(&mut self, peer: &Peer) -> Result<()>;

    /// Ban a peer
    async fn ban_peer(&mut self, peer: &Peer, reason: String) -> Result<()>;

    /// Unban a peer
    async fn unban_peer(&mut self, peer: &Peer) -> Result<()>;

    /// Get peer by ID
    async fn get_peer(&self, id: uuid::Uuid) -> Result<Option<Peer>>;

    /// List all peers
    async fn list_peers(&self) -> Result<Vec<Peer>>;

    /// Get peer statistics
    fn peer_stats(&self) -> PeerStats;
}

/// Peer connection statistics
#[derive(Debug, Clone, Default)]
pub struct PeerStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub banned_peers: usize,
    pub handshaking_peers: usize,
    pub peer_uptime: Duration,
    pub last_message: Option<SystemTime>,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_transferred: u64,
} 