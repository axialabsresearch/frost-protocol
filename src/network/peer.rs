/*!
# Network Peer Module

This module provides peer management functionality for the FROST protocol,
implementing peer discovery, connection management, and peer statistics tracking.

## Core Components

### Peer Management
- Peer discovery
- Connection handling
- State tracking
- Statistics collection

### Peer Types
- Validator nodes
- Observer nodes
- Relay nodes
- Gateway nodes

### Peer States
- Connection states
- Ban management
- Handshaking
- State transitions

### Peer Statistics
- Connection metrics
- Message tracking
- Data transfer
- Uptime monitoring

## Architecture

The peer system consists of several key components:

1. **Peer Structure**
   ```rust
   pub struct Peer {
       id: uuid::Uuid,
       info: PeerInfo,
       state: PeerState,
   }
   ```
   - Peer identity
   - Node information
   - Connection state
   - Feature support

2. **Peer Manager**
   ```rust
   pub trait PeerManager: Send + Sync {
       async fn add_peer(&mut self, info: PeerInfo) -> Result<Peer>;
       async fn remove_peer(&mut self, peer: &Peer) -> Result<()>;
       async fn ban_peer(&mut self, peer: &Peer, reason: String) -> Result<()>;
       async fn unban_peer(&mut self, peer: &Peer) -> Result<()>;
       async fn get_peer(&self, id: uuid::Uuid) -> Result<Option<Peer>>;
       async fn list_peers(&self) -> Result<Vec<Peer>>;
       fn peer_stats(&self) -> PeerStats;
   }
   ```
   - Peer operations
   - State management
   - Ban control
   - Statistics tracking

3. **Peer Statistics**
   ```rust
   pub struct PeerStats {
       total_peers: usize,
       connected_peers: usize,
       banned_peers: usize,
       handshaking_peers: usize,
       peer_uptime: Duration,
       last_message: Option<SystemTime>,
       messages_sent: u64,
       messages_received: u64,
       bytes_transferred: u64,
   }
   ```
   - Connection stats
   - Message metrics
   - Data transfer
   - Uptime tracking

## Features

### Peer Features
- Peer discovery
- Connection management
- State tracking
- Ban control

### Node Types
- Validator support
- Observer nodes
- Relay functionality
- Gateway services

### State Management
- Connection states
- Ban management
- Handshaking
- State transitions

### Statistics
- Connection tracking
- Message counting
- Data monitoring
- Performance metrics

## Best Practices

1. **Peer Management**
   - Regular discovery
   - State monitoring
   - Ban management
   - Resource cleanup

2. **Connection Handling**
   - State tracking
   - Error handling
   - Resource limits
   - Cleanup routines

3. **Ban Management**
   - Ban criteria
   - Unban policies
   - Resource impact
   - State tracking

4. **Statistics Collection**
   - Regular updates
   - Data validation
   - Resource usage
   - Performance impact

## Integration

The peer system integrates with:
1. Network protocol
2. Transport layer
3. Security system
4. Metrics collection
*/

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