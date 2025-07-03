/*!
# Network Protocol Module

This module defines the core networking protocol for the FROST system, providing
interfaces and types for peer-to-peer communication, message handling, and network
management.

## Core Components

### Network Protocol
- Protocol management
- Message handling
- Peer communication
- Subscription system

### Message Handling
- Message broadcasting
- Direct messaging
- Message filtering
- Delivery confirmation

### Network Management
- Peer connections
- Network status
- Bandwidth tracking
- Protocol versioning

## Architecture

The protocol system consists of several key components:

1. **Network Protocol**
   ```rust
   pub trait NetworkProtocol: Send + Sync {
       async fn start(&mut self) -> Result<()>;
       async fn stop(&mut self) -> Result<()>;
       async fn broadcast_message(&self, message: FrostMessage) -> Result<BroadcastResult>;
       async fn send_message(&self, peer: &Peer, message: FrostMessage) -> Result<SendResult>;
       async fn subscribe(&mut self, filter: MessageFilter) -> Result<SubscriptionId>;
       async fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> Result<()>;
       async fn network_status(&self) -> Result<NetworkStatus>;
       async fn connected_peers(&self) -> Result<Vec<PeerInfo>>;
   }
   ```
   - Protocol operations
   - Message handling
   - Subscription management
   - Network monitoring

2. **Message Results**
   ```rust
   pub struct BroadcastResult {
       message_id: uuid::Uuid,
       reached_peers: Vec<PeerInfo>,
       failed_peers: Vec<(PeerInfo, NetworkError)>,
       broadcast_time: Duration,
   }
   ```
   - Message tracking
   - Delivery status
   - Error handling
   - Performance metrics

3. **Network Status**
   ```rust
   pub struct NetworkStatus {
       connected_peers: usize,
       active_subscriptions: usize,
       messages_in_flight: usize,
       bandwidth_usage: BandwidthUsage,
       protocol_version: String,
   }
   ```
   - Connection tracking
   - Resource usage
   - Performance metrics
   - Protocol info

## Features

### Protocol Features
- Message broadcasting
- Direct messaging
- Subscription system
- Network monitoring

### Message Features
- Message filtering
- Delivery tracking
- Error handling
- Performance metrics

### Network Features
- Peer management
- Status monitoring
- Bandwidth tracking
- Resource management

### Configuration
- Connection limits
- Timeout settings
- Protocol settings
- Resource limits

## Best Practices

1. **Protocol Usage**
   - Proper initialization
   - Resource management
   - Error handling
   - Cleanup routines

2. **Message Handling**
   - Message validation
   - Delivery tracking
   - Error recovery
   - Resource limits

3. **Network Management**
   - Connection handling
   - Status monitoring
   - Resource tracking
   - Performance tuning

4. **Configuration**
   - Proper timeouts
   - Resource limits
   - Protocol settings
   - Performance tuning

## Integration

The protocol system integrates with:
1. Message handling
2. Peer management
3. Network transport
4. Security systems
*/

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use crate::message::FrostMessage;
use crate::network::{Peer, PeerInfo, NetworkError};
use crate::Result;

/// Core network protocol for FROST
#[async_trait]
pub trait NetworkProtocol: Send + Sync {
    /// Start the network protocol
    async fn start(&mut self) -> Result<()>;

    /// Stop the network protocol
    async fn stop(&mut self) -> Result<()>;

    /// Broadcast a message to the network
    async fn broadcast_message(&self, message: FrostMessage) -> Result<BroadcastResult>;

    /// Send a message to a specific peer
    async fn send_message(&self, peer: &Peer, message: FrostMessage) -> Result<SendResult>;

    /// Subscribe to message types
    async fn subscribe(&mut self, filter: MessageFilter) -> Result<SubscriptionId>;

    /// Unsubscribe from messages
    async fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> Result<()>;

    /// Get network status
    async fn network_status(&self) -> Result<NetworkStatus>;

    /// Get connected peers
    async fn connected_peers(&self) -> Result<Vec<PeerInfo>>;
}

/// Network protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub broadcast_timeout: Duration,
    pub peer_ping_interval: Duration,
    pub max_message_size: usize,
    pub supported_protocols: Vec<String>,
}

/// Result of message broadcast
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    pub message_id: uuid::Uuid,
    pub reached_peers: Vec<PeerInfo>,
    pub failed_peers: Vec<(PeerInfo, NetworkError)>,
    pub broadcast_time: Duration,
}

/// Result of direct message send
#[derive(Debug, Clone)]
pub struct SendResult {
    pub message_id: uuid::Uuid,
    pub delivered: bool,
    pub send_time: Duration,
    pub confirmation: Option<MessageConfirmation>,
}

/// Message confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfirmation {
    pub message_id: uuid::Uuid,
    pub received_at: std::time::SystemTime,
    pub peer_info: PeerInfo,
    pub signature: Option<Vec<u8>>,
}

/// Message filter for subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    pub message_types: Vec<String>,
    pub chains: Vec<u64>,
    pub priority: Option<u8>,
}

/// Subscription identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(uuid::Uuid);

/// Network status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected_peers: usize,
    pub active_subscriptions: usize,
    pub messages_in_flight: usize,
    pub bandwidth_usage: BandwidthUsage,
    pub protocol_version: String,
}

/// Bandwidth usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub current_bandwidth: f64, // bytes per second
}
