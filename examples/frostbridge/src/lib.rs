pub mod types;
pub mod config;
pub mod bridges;
pub mod transfer;

use frost_protocol::{
    finality::{FinalityVerifier, EthereumVerifier, SubstrateVerifier, FinalityConfig},
    network::{NetworkConfig, BasicNetwork},
    routing::{RoutingConfig, BasicRouter},
    Result, Error,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Define SharedNetwork type that was previously imported
pub struct SharedNetwork(Arc<Mutex<BasicNetwork>>);

impl Clone for SharedNetwork {
    fn clone(&self) -> Self {
        SharedNetwork(Arc::clone(&self.0))
    }
}

// Re-export all public items
pub use crate::{
    types::*,
    config::*,
    bridges::*,
    transfer::*,
};

// Network constants
const DEFAULT_SEPOLIA_RPC: &str = "https://rpc.sepolia.org";
const DEFAULT_WESTEND_WS: &str = "wss://westend-rpc.polkadot.io";
const ALTERNATIVE_SEPOLIA_RPCS: [&str; 3] = [
    "https://eth-sepolia.public.blastapi.io",
    "https://rpc.sepolia.org",
    "https://rpc2.sepolia.org",
];

// Bridge contract constant
const FROST_BRIDGE_CONTRACT: &str = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";

pub async fn initialize_components(
    config: &CrossChainConfig,
) -> Result<(
    EthereumVerifier,
    SubstrateVerifier,
    SharedNetwork,
    BasicRouter<SharedNetwork>,
)> {
    println!("Starting component initialization...");
    
    // Initialize finality verifiers with configured confirmation requirements
    let eth_config = FinalityConfig {
        min_confirmations: config.chain.eth_min_confirmations,
        finality_timeout: std::time::Duration::from_secs(config.transfer.timeout_secs),
        basic_params: HashMap::new(),
    };
    let eth_verifier = EthereumVerifier::new(eth_config);
    println!("✓ Ethereum verifier initialized");

    let sub_config = FinalityConfig {
        min_confirmations: config.chain.dot_min_confirmations,
        finality_timeout: std::time::Duration::from_secs(config.transfer.timeout_secs),
        basic_params: HashMap::new(),
    };
    let sub_verifier = SubstrateVerifier::new(sub_config);
    println!("✓ Substrate verifier initialized");

    // Initialize network with configured settings
    let network_config = NetworkConfig {
        node_id: Uuid::new_v4().to_string(),
        listen_addr: "127.0.0.1:9000".to_string(),
        bootstrap_peers: vec![
            "/ip4/127.0.0.1/tcp/9001/p2p/test-peer-1".to_string(),
        ],
        protocol_version: 1,
    };
    
    let network = Arc::new(Mutex::new(BasicNetwork::new(network_config.clone())));
    let shared_network = SharedNetwork(network);
    
    let mut network_clone = shared_network.clone();
    network_clone.start().await?;
    println!("✓ Network started");
    
    // Initialize router with configured max routes
    let router_config = RoutingConfig {
        node_id: network_config.node_id,
        route_timeout: config.transfer.timeout_secs,
        max_routes: config.transfer.max_routes as usize,
    };
    let router = BasicRouter::new(router_config, shared_network.clone());
    println!("✓ Router initialized");

    Ok((
        eth_verifier,
        sub_verifier,
        shared_network,
        router,
    ))
}

pub fn get_network_endpoints() -> Result<(String, String)> {
    // Try environment variables first
    let eth_rpc = match std::env::var("ETH_RPC_URL") {
        Ok(url) => {
            println!("\n[DEBUG] Using ETH_RPC_URL from environment: {}", url);
            url
        },
        Err(_) => {
            println!("\n[DEBUG] No ETH_RPC_URL environment variable found");
            println!("[DEBUG] Using default Sepolia endpoint: {}", DEFAULT_SEPOLIA_RPC);
            println!("[DEBUG] ⚠️  Note: Using public RPC endpoint. For better reliability, consider:");
            println!("  1. Get your own API key from https://infura.io (recommended)");
            println!("  2. Try alternative public endpoints:");
            for (i, url) in ALTERNATIVE_SEPOLIA_RPCS.iter().enumerate() {
                println!("     {}: {}", i + 1, url);
            }
            DEFAULT_SEPOLIA_RPC.to_string()
        }
    };

    let dot_ws = match std::env::var("DOT_WS_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("[DEBUG] Using default Westend endpoint");
            DEFAULT_WESTEND_WS.to_string()
        }
    };

    // Validate endpoints
    if !eth_rpc.starts_with("http") && !eth_rpc.starts_with("ws") {
        return Err(Error::from("Invalid Ethereum RPC URL format"));
    }

    if !dot_ws.starts_with("ws") {
        return Err(Error::from("Invalid Westend WebSocket URL format"));
    }

    Ok((eth_rpc, dot_ws))
}

pub fn is_dry_run() -> bool {
    std::env::args().any(|arg| arg == "--dry-run")
}

pub fn print_dry_run_info(config: &CrossChainConfig, eth_rpc: &str, dot_ws: &str) {
    println!("\n=== DRY RUN MODE ===");
    println!("\nNetwork Configuration:");
    println!("Ethereum RPC: {}", eth_rpc);
    println!("Polkadot WS: {}", dot_ws);
    
    println!("\nChain Settings:");
    println!("Ethereum:");
    println!("  - Network: Sepolia");
    println!("  - Min Confirmations: {}", config.chain.eth_min_confirmations);
    println!("  - Max Gas Price: {} Gwei", config.chain.eth_max_gas_price);
    println!("  - Gas Limit: {}", config.chain.eth_gas_limit);
    
    println!("\nPolkadot:");
    println!("  - Network: Westend");
    println!("  - Min Confirmations: {}", config.chain.dot_min_confirmations);
    println!("  - Existential Deposit: {} Planck", config.chain.dot_existential_deposit);
    
    println!("\nTransfer Limits:");
    println!("  - Minimum: {} ETH/DOT", config.transfer.min_amount);
    println!("  - Maximum: {} ETH/DOT", config.transfer.max_amount);
    println!("  - Timeout: {}s", config.transfer.timeout_secs);
    println!("  - Max Routes: {}", config.transfer.max_routes);
    
    println!("\nSecurity Paths:");
    println!("  - ETH Key: {}", config.security.eth_private_key_path.display());
    println!("  - DOT Seed: {}", config.security.dot_seed_path.display());
    
    println!("\nNo actual transfers will be performed in dry run mode.");
    println!("===================\n");
}