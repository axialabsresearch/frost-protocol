/*!
# Network Transport Module

This module provides the transport layer functionality for the FROST protocol,
implementing low-level network communication, encryption, compression, and metrics
collection.

## Core Components

### Transport Layer
- Connection handling
- Data transmission
- Protocol support
- Metrics tracking

### Protocol Support
- TCP transport
- WebSocket transport
- QUIC transport
- Protocol configuration

### Security Features
- Encryption support
- Key management
- TLS integration
- Certificate handling

### Performance Features
- Compression support
- Buffer management
- Latency tracking
- Metrics collection

## Architecture

The transport system consists of several key components:

1. **Transport Interface**
   ```rust
   pub trait Transport: Send + Sync {
       async fn init(&mut self, config: TransportConfig) -> Result<()>;
       async fn connect(&mut self, address: &str) -> Result<Peer>;
       async fn disconnect(&mut self, peer: &Peer) -> Result<()>;
       async fn send_data(&self, peer: &Peer, data: &[u8]) -> Result<usize>;
       async fn receive_data(&self, peer: &Peer) -> Result<Vec<u8>>;
       async fn is_connected(&self, peer: &Peer) -> bool;
       fn metrics(&self) -> TransportMetrics;
   }
   ```
   - Connection management
   - Data transmission
   - Connection status
   - Metrics tracking

2. **Transport Protocols**
   ```rust
   pub enum TransportProtocol {
       TCP { port: u16, keep_alive: bool },
       WebSocket { url: String, use_tls: bool },
       QUIC { port: u16, cert_path: String },
   }
   ```
   - Protocol types
   - Configuration
   - Security settings
   - Performance options

3. **Transport Metrics**
   ```rust
   pub struct TransportMetrics {
       active_connections: usize,
       bytes_sent: u64,
       bytes_received: u64,
       connection_errors: u64,
       average_latency: Duration,
   }
   ```
   - Connection tracking
   - Data metrics
   - Error tracking
   - Performance metrics

## Features

### Connection Features
- Connection management
- Peer tracking
- Error handling
- Status monitoring

### Protocol Features
- Multiple protocols
- Protocol configuration
- Security settings
- Performance options

### Security Features
- Encryption support
- TLS integration
- Certificate handling
- Key management

### Performance Features
- Compression support
- Buffer management
- Latency tracking
- Resource management

## Best Practices

1. **Connection Management**
   - Proper initialization
   - Error handling
   - Resource cleanup
   - Status tracking

2. **Protocol Usage**
   - Protocol selection
   - Configuration setup
   - Security settings
   - Performance tuning

3. **Security Implementation**
   - Encryption setup
   - Certificate handling
   - Key management
   - Security updates

4. **Performance Tuning**
   - Buffer sizing
   - Compression settings
   - Latency monitoring
   - Resource limits

## Integration

The transport system integrates with:
1. Network protocol
2. Security system
3. Peer management
4. Metrics collection
*/

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