# FROST Protocol

FROST (Finality Reliant Optimized State Transition) Protocol is a cross-chain finality verification system that enables secure state transitions across different blockchain ecosystems.

## Overview

FROST provides a unified interface for verifying finality across three major blockchain ecosystems:
- Ethereum (PoW and Beacon Chain)
- Cosmos (Tendermint)
- Substrate (GRANDPA)

### Key Features

- **Chain-Agnostic Finality**: Standardized finality verification across different consensus mechanisms
- **Simplified State Transitions**: Basic state transition and proof validation
- **Efficient Networking**: Optimized P2P message routing with basic discovery
- **Metrics Collection**: Basic performance and health metrics
- **Error Handling**: Comprehensive error types with retry mechanisms

## Quick Start

### Installation

Add FROST to your Cargo.toml:
```toml
[dependencies]
frost-protocol = "0.1.0"
```

### Basic Usage

```rust
use frost_protocol::{
    finality::{FinalityConfig, EthereumVerifier},
    state::BlockRef,
};

#[tokio::main]
async fn main() {
    // Create finality verifier
    let config = FinalityConfig::default();
    let verifier = EthereumVerifier::new(config);

    // Verify block finality
    let block_ref = BlockRef::new("eth", 100);
    let signal = // ... obtain finality signal
    let is_final = verifier.verify_finality(&block_ref, &signal).await?;
}
```

## Architecture

FROST consists of four main components:

1. **Finality Verification**
   - Chain-specific verifiers
   - Configurable parameters
   - Basic metrics collection

2. **State Management**
   - State transition validation
   - Proof verification
   - Block references

3. **Network Layer**
   - P2P message routing
   - Kademlia DHT-based peer discovery
   - Provider record management
   - Connection management with automatic peer tracking
   - NAT traversal support

4. **Message Routing**
   - Route discovery
   - Message forwarding
   - Basic routing table

## Configuration

### Finality Configuration

```rust
let config = FinalityConfig {
    min_confirmations: 6,
    finality_timeout: Duration::from_secs(30),
    basic_params: HashMap::new(),
};
```

### Network Configuration

```rust
let config = NetworkConfig {
    node_id: "node1".to_string(),
    listen_addr: "127.0.0.1:9000".to_string(),
    bootstrap_peers: vec![],
    protocol_version: 0,
};

// P2P Configuration with Kademlia DHT
let p2p_config = P2PConfig {
    listen_addresses: vec!["0.0.0.0:9000".to_string()],
    bootstrap_peers: vec!["multiaddr_of_bootstrap_node".to_string()],
    connection_timeout: Duration::from_secs(30),
    max_connections: 50,
    enable_nat: true,
    enable_mdns: true,
};

// Initialize P2P node
let mut node = P2PNode::new(p2p_config).await?;
node.start().await?;
```

## Features

### P2P Networking
- Kademlia DHT for decentralized peer discovery
- Provider record management for service advertisement
- Automatic peer tracking and connection management
- Support for NAT traversal and mDNS discovery
- Configurable protocol names and timeouts
- Metrics for network health monitoring

## Testing (Currently inactive due to maintainance issues)

Run the test suite:
```bash
cargo test
```

Integration tests:
```bash
cargo test --test integration
```

## Error Handling

FROST provides comprehensive error types:
- `FinalityError`: Finality verification errors
- `StateError`: State transition errors
- `NetworkError`: Network-related errors
- `RoutingError`: Message routing errors

## Metrics

Basic metrics collection for:
- Block verification counts
- Network message statistics
- Routing performance
- Error rates

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the Apache License, Version 2.0.

## Version History

- v0.1.0 (Initial Release)
  - Basic finality verification
  - Simple state transitions
  - P2P networking
  - Basic metrics 

## Messaging System

### Message Types
- **State Transition**: Core state change messages
- **State Proof**: Verification proofs for state transitions
- **Finality Signal**: Chain finality notifications
- **Discovery**: Network peer discovery messages
- **Custom**: Extensible custom message types

### Message Properties
- Priority levels (Low, Normal, High, Critical)
- Metadata support for protocol versioning
- Chain-specific metadata fields
- Retry mechanisms for failed messages
- Custom metadata extension support

### Message Routing
- Configurable routing strategies
- Multi-hop message delivery
- Route discovery and optimization
- Parallel routing paths
- Chain-specific routing parameters
- Route metrics and performance tracking

## Finality System

### Finality Verification
- Configurable confirmation thresholds
- Chain-specific finality predicates
- Confidence-based verification
- Timeout and evaluation controls
- Caching of verification results

### Predicate System
- Custom predicate support
- Confidence threshold configuration
- Chain-specific parameters
- Performance metrics collection
- Comprehensive error handling

## State Management

### State Transitions
- Atomic state updates
- State proof verification
- Root hash validation
- Chain-specific state rules
- State conflict resolution

### State Synchronization
- Version-based state tracking
- Conflict detection and resolution
- Consensus-based reconciliation
- State timeout handling
- Validator set management

## Error Handling

### Error Categories
- **FinalityError**: Finality verification failures
- **StateError**: State transition issues
- **NetworkError**: Network communication problems
- **MessageError**: Message processing failures
- **RoutingError**: Message routing issues

### Error Features
- Severity levels (Warning, Error, Critical)
- Retryable error identification
- Comprehensive error context
- Chain-specific error handling
- Error metrics collection

## Metrics and Monitoring

### Network Metrics
- Message routing success rates
- Network topology health
- Peer connection statistics
- Protocol performance metrics
- Latency and throughput tracking

### State Metrics
- State transition success rates
- Proof verification performance
- State synchronization status
- Consensus participation rates
- Validator performance tracking

### Finality Metrics
- Verification success rates
- Predicate evaluation times
- Confidence level tracking
- Chain-specific metrics
- Timeout and failure tracking

## Advanced Features

### Network Topology
- Dynamic topology updates
- Node status tracking
- Performance-based routing
- Chain type classification
- Feature compatibility tracking

### Circuit Breaking
- Configurable breaker thresholds
- Automatic failure detection
- Graceful degradation support
- Recovery mechanisms
- Health check integration

### Backpressure Control
- Dynamic load management
- Pressure-based throttling
- Resource utilization tracking
- Adaptive rate limiting
- Queue management 
