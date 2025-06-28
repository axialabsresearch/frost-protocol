# FROST Protocol Examples

This directory contains examples demonstrating how to use various components of the FROST Protocol.

## Directory Structure

```
examples/
├── finality/           # Finality verification examples
├── message/            # Message handling examples
├── network/            # Network layer examples
├── routing/            # Message routing examples
├── state/              # State management examples
└── combined/           # Examples combining multiple components
```

## Running Examples

Each example can be run using cargo:

```bash
cargo run --example <example_name>
```

For instance, to run the basic Ethereum finality verification example:

```bash
cargo run --example ethereum_finality
```

## Example Categories

### Finality Verification
- Basic finality verification for different chains
- Custom finality rules
- Finality monitoring

### Message Handling
- Message creation and validation
- Message serialization
- Custom message types

### Network Layer
- Basic network setup
- Peer discovery
- Message broadcasting

### Message Routing
- Route discovery
- Message forwarding
- Custom routing rules

### State Management
- State transitions
- State proofs
- State synchronization

### Combined Examples
- Cross-chain verification
- Multi-chain synchronization
- Complete node setup

## Contributing

To add a new example:
1. Create a new file in the appropriate subdirectory
2. Add comprehensive documentation
3. Update the relevant README.md
4. Add the example to Cargo.toml 