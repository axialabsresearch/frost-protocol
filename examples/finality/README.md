# FROST Protocol Finality Examples

This directory contains examples demonstrating how to use FROST Protocol's finality verification capabilities across different blockchain networks. Each example showcases specific finality mechanisms and verification approaches.

## Available Examples

### 1. Ethereum Finality (`ethereum_finality.rs`)
Demonstrates finality verification for both Ethereum PoW and Beacon Chain:
- PoW finality with configurable confirmation depth
- Beacon Chain finality with validator signatures
- Handling of chain reorganizations
- Metrics collection and monitoring

### 2. Cosmos Finality (`cosmos_finality.rs`)
Shows Tendermint consensus finality verification:
- Basic Tendermint consensus verification
- Validator signature verification
- Voting power thresholds
- Custom finality rules
- Metrics tracking

### 3. Substrate Finality (`substrate_finality.rs`)
Illustrates GRANDPA consensus and parachain validation:
- GRANDPA finality verification
- Parachain block verification
- Authority set management
- Voting thresholds
- Performance metrics

## Common Features
All examples demonstrate:
- Proper error handling
- Metrics collection
- Configuration management
- Logging and monitoring
- Best practices for production use

## Running the Examples

Each example can be run using Cargo:

```bash
# Run Ethereum finality example
cargo run --example ethereum_finality

# Run Cosmos finality example
cargo run --example cosmos_finality

# Run Substrate finality example
cargo run --example substrate_finality
```

## Example Structure

Each example follows a consistent structure:
1. Configuration setup
2. Verifier initialization
3. Multiple verification scenarios
4. Error handling demonstration
5. Metrics collection and reporting

## Best Practices

The examples demonstrate these best practices:
- Proper error handling and propagation
- Comprehensive logging
- Metrics collection
- Configuration validation
- Resource cleanup
- Production-ready code structure

## Additional Resources

- [FROST Protocol Documentation](https://docs.frostprotocol.com)
- [Finality Verification Guide](https://docs.frostprotocol.com/guides/finality)
- [API Reference](https://docs.frostprotocol.com/api/finality) 