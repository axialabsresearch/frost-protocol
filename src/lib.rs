/*!
# FROST Protocol

FROST (Finality Reliant Optimized State Transition) Protocol is a cross-chain finality
verification system that enables secure state transitions across different blockchain
ecosystems.

## Core Components

### Finality System
The finality system provides:
- Chain-specific verification
- Finality signals
- Monitoring capabilities
- Extension support

### State Management
State handling includes:
- State transitions
- Proof verification
- State caching
- Error handling

### Network Layer
Network features include:
- P2P communication
- Peer discovery
- Connection management
- Protocol handling

### Message Routing
Routing capabilities:
- Route discovery
- Message forwarding
- Path optimization
- Error handling

## Architecture

The protocol implements several key components:

1. **Finality Verification**
   ```rust
   use frost_protocol::finality::{FinalityConfig, EthereumVerifier, FinalityVerifier};
   
   let config = FinalityConfig::default();
   let verifier = EthereumVerifier::new(config);
   ```
   - Chain verification
   - Signal processing
   - Monitor management
   - Extension support

2. **State Management**
   ```rust
   use frost_protocol::state::{BlockId, StateTransition};
   
   let source = BlockId::default();
   let target = BlockId::default();
   let transition = StateTransition::new(source, target, vec![]);
   ```
   - State transitions
   - Proof handling
   - Cache management
   - Error processing

3. **Network Layer**
   ```rust
   use frost_protocol::network::{NetworkConfig, BasicNetwork, NetworkProtocol};
   
   let config = NetworkConfig::default();
   let mut network = BasicNetwork::new(config);
   network.start().await?;
   ```
   - P2P networking
   - Peer management
   - Protocol handling
   - Connection management

4. **Message Routing**
   ```rust
   use frost_protocol::routing::{RoutingConfig, BasicRouter};
   use frost_protocol::network::{NetworkConfig, BasicNetwork};
   
   let config = RoutingConfig::default();
   let network = BasicNetwork::new(NetworkConfig::default());
   let router = BasicRouter::new(config, network);
   ```
   - Route discovery
   - Message handling
   - Path optimization
   - Error management

## Features

### Chain Support
- Ethereum (PoW/PoS)
- Cosmos (Tendermint)
- Substrate (GRANDPA)
- Custom chains

### State Management
- Transition handling
- Proof verification
- Cache management
- Error handling

### Network Features
- P2P communication
- Peer discovery
- Protocol handling
- Connection management

### Routing Capabilities
- Route discovery
- Message forwarding
- Path optimization
- Error handling

## Best Practices

### System Usage
1. Finality Handling
   - Chain selection
   - Signal processing
   - Monitor usage
   - Error handling

2. State Management
   - Transition handling
   - Proof verification
   - Cache usage
   - Error processing

3. Network Usage
   - Connection handling
   - Peer management
   - Protocol usage
   - Error handling

4. Routing Usage
   - Route discovery
   - Message handling
   - Path selection
   - Error management

## Integration

### Chain Integration
- Chain selection
- Verification setup
- Signal handling
- Error management

### State System
- Transition setup
- Proof handling
- Cache management
- Error processing

### Network System
- Connection setup
- Peer handling
- Protocol management
- Error processing

### Routing System
- Route setup
- Message handling
- Path management
- Error processing

## Performance Considerations

### Resource Management
- Memory usage
- CPU utilization
- Network bandwidth
- Storage handling

### Optimization
- Cache strategies
- Protocol efficiency
- Path optimization
- Resource sharing

### Monitoring
- System metrics
- Performance tracking
- Error monitoring
- Resource usage

### Tuning
- Cache settings
- Protocol parameters
- Network options
- Resource limits

## Implementation Notes

### Chain Support
FROST supports different chains regardless of underlying architecture

### State Handling
State management:
- Transition processing
- Proof verification
- Cache management
- Error handling

### Network Layer
Network features:
- P2P communication
- Peer management
- Protocol handling
- Connection management

### Routing System
Routing capabilities:
- Route discovery
- Message handling
- Path optimization
- Error management

## Version Information

- Current version: 0.1.0
- Status: Initial Release (Basic Functionality)
- Features: Core components implemented
- Extensions: Basic support available

## Testing

The protocol includes:
- Unit tests
- Integration tests
- Network simulations
- Performance tests

## Metrics

System metrics include:
- Finality metrics
- State metrics
- Network metrics
- Routing metrics
*/

pub mod finality;
pub mod message;
pub mod state;
pub mod network;
pub mod routing;
pub mod metrics;
pub mod extensions;

// Re-exports
pub use finality::{FinalitySignal, FinalityMonitor};
pub use message::{FrostMessage, MessageType};
pub use state::{StateTransition, StateProof};
pub use network::{NetworkProtocol, NetworkConfig, BasicNetwork};
pub use routing::MessageRouter;

// Core types
pub type Result<T> = std::result::Result<T, Error>;
pub use error::Error;

pub mod error;

