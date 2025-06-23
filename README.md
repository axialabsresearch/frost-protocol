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
   - Basic peer discovery
   - Connection management

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
```

## Testing

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