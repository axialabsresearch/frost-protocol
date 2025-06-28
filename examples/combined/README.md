# FROST Protocol Combined Examples

This directory contains examples demonstrating how to use multiple FROST Protocol components together to build complete cross-chain applications.

## Cross-Chain Transfer Example

The `cross_chain_transfer.rs` example demonstrates a complete cross-chain transfer flow using the FROST Protocol, specifically showing how to transfer assets between Ethereum and Polkadot networks.

### Overview

This example implements a simplified but practical demonstration of FROST's core capabilities:
1. Cross-chain asset transfer between heterogeneous chains (Ethereum ↔ Polkadot)
2. Chain-specific finality verification
3. Message routing through FROST's network module
4. Transfer monitoring and validation
5. Error handling and automatic retries

> **Note**: This example is not intended for production use. It serves as a demonstration of FROST's core capabilities and cross-chain coordination logic.

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Access to Ethereum testnet (Sepolia) and Polkadot testnet (Westend)
- Infura API key for Ethereum RPC access
- Test tokens for both networks:
  - Sepolia ETH from [Sepolia Faucet](https://sepoliafaucet.com)
  - Westend DOT from [Westend Faucet](https://matrix.to/#/#westend_faucet:matrix.org)

### Configuration

#### Environment Variables

Create a `.env` file in the project root with the following configuration:

```bash
# Network Endpoints
ETH_RPC_URL="https://sepolia.infura.io/v3/YOUR_INFURA_KEY"
DOT_WS_URL="wss://westend-rpc.polkadot.io"

# Network Selection
ETH_NETWORK="sepolia"    # Options: mainnet, sepolia, goerli
DOT_NETWORK="westend"    # Options: mainnet, westend, rococo

# Chain-Specific Settings
ETH_MIN_CONFIRMATIONS=12
ETH_MAX_GAS_PRICE=100    # in Gwei
ETH_GAS_LIMIT=21000      # Standard ETH transfer
DOT_MIN_CONFIRMATIONS=1
DOT_EXISTENTIAL_DEPOSIT=1000000000000  # 1 DOT in planck

# Transfer Settings
TRANSFER_TIMEOUT=300     # seconds
MIN_TRANSFER_AMOUNT=0.01 # in native token (ETH/DOT)
MAX_TRANSFER_AMOUNT=100  # in native token (ETH/DOT)
MAX_ROUTES=10

# Security Settings
ETH_PRIVATE_KEY_PATH=".keys/eth_private_key"
DOT_SEED_PATH=".keys/dot_seed"

# Source Address (Optional)
ETH_SOURCE_ADDRESS="0x..."  # Your Ethereum address
```

#### Security Setup

```bash
# Create and secure key directories
mkdir -p .keys
chmod 700 .keys
```

### Usage

#### Dry Run Mode

Test your configuration without performing actual transfers:

```bash
cargo run --example cross_chain_transfer -- --dry-run
```

This will display:
- Current configuration
- Network endpoints
- Chain settings
- Transfer limits
- Security paths

#### Regular Execution

```bash
# Load environment variables
source .env

# Run the transfer
cargo run --example cross_chain_transfer
```

### Implementation Details

#### Key Components

1. **Chain Configuration**
   - Ethereum and Polkadot specific settings
   - Confirmation requirements
   - Gas limits and pricing
   - Network endpoints

2. **Transfer Settings**
   - Amount limits
   - Timeout configurations
   - Route management
   - Retry mechanisms

3. **Security**
   - Private key management
   - Secure storage paths
   - Network validation

4. **FROST Protocol Integration**
   - Message routing
   - Finality verification
   - State synchronization
   - Network communication

#### Error Handling

The implementation includes comprehensive error handling for:
- Network connectivity issues
- Insufficient funds
- Gas price fluctuations
- Invalid configurations
- Transfer timeouts

#### Automatic Retries

Built-in retry mechanism with:
- Exponential backoff
- Configurable retry limits
- Detailed error reporting
- State preservation between attempts

### Monitoring and Validation

The transfer process includes real-time monitoring of:
- Transaction status
- Route discovery
- Chain finality
- Balance verification
- Gas estimation

### Development Notes

#### Testing

For local testing:
```bash
# Use local endpoints
ETH_RPC_URL="http://localhost:8545"
DOT_WS_URL="ws://localhost:9944"

# Reduce confirmations for faster testing
ETH_MIN_CONFIRMATIONS=1
TRANSFER_TIMEOUT=60
```

#### Debugging

Enable detailed logging:
```bash
RUST_LOG=debug cargo run --example cross_chain_transfer
```

### Limitations

- Testnet only implementation
- Simplified security model
- Basic retry mechanism
- Limited error recovery options
- No production safeguards

### Future Improvements

Potential areas for enhancement:
- Advanced security features
- Multiple route support
- Dynamic gas pricing
- Enhanced error recovery
- Production readiness features

## Additional Resources

- [FROST Protocol Documentation](https://axialabsresearch.github.io/article/frost-v0)
- [Integration Guide](https://github.com/axialabsresearch/frost-protocol/README.md)
- [API Reference]()

## Contributing

Feel free to submit issues and enhancement requests.

## License

This example is part of the FROST Protocol and follows its licensing terms. 