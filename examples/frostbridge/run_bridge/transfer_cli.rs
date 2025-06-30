use frost_protocol::{
    Result,
    finality::{FinalityVerifier, EthereumVerifier, SubstrateVerifier},
    network::SharedNetwork,
    routing::BasicRouter,
};

use crate::examples::frostbridge::{
    types::*,
    config::*,
    bridges::{ethereum::EthereumBridge, polkadot::PolkadotBridge},
    transfer::{eth_to_dot::EthToDotTransfer, dot_to_eth::DotToEthTransfer},
    initialize_components,
    get_network_endpoints,
    is_dry_run,
    print_dry_run_info,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load and validate configuration
    println!("Loading configuration...");
    let config = CrossChainConfig::from_env()?;
    config.validate()?;
    
    // Get network endpoints
    let (eth_rpc, dot_ws) = get_network_endpoints()?;

    // Get transfer direction from user
    println!("\nSelect transfer direction:");
    println!("1. Ethereum to Polkadot");
    println!("2. Polkadot to Ethereum");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let direction = match input.trim() {
        "1" => TransferDirection::EthereumToPolkadot,
        "2" => TransferDirection::PolkadotToEthereum,
        _ => return Err("Invalid direction selection".into()),
    };

    let mut monitor = TransferMonitor::new(direction.clone(), 3);

    // Check for dry run mode
    if is_dry_run() {
        print_dry_run_info(&config, &eth_rpc, &dot_ws);
        return Ok(());
    }

    // Initialize components
    let (eth_verifier, sub_verifier, network, router) = initialize_components(&config).await?;

    // Initialize bridges
    let eth_bridge = EthereumBridge::new(
        "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        config.chain.eth_min_confirmations as u64,
        &eth_rpc,
    );

    let mut dot_bridge = PolkadotBridge::new(
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        config.chain.dot_min_confirmations as u64,
        &dot_ws,
    );
    dot_bridge.connect().await?;

    // Execute transfer based on direction
    let result = match direction {
        TransferDirection::EthereumToPolkadot => {
            let mut transfer = EthToDotTransfer::new(eth_bridge, dot_bridge, config);
            transfer.execute(
                &mut monitor,
                &eth_verifier,
                &sub_verifier,
                &router,
            ).await
        },
        TransferDirection::PolkadotToEthereum => {
            let mut transfer = DotToEthTransfer::new(eth_bridge, dot_bridge, config);
            transfer.execute(
                &mut monitor,
                &eth_verifier,
                &sub_verifier,
                &router,
            ).await
        }
    };

    match result {
        Ok(tx_hash) => {
            println!("\n✓ Transfer completed successfully!");
            println!("Transaction hash: {}", tx_hash);
        },
        Err(e) => {
            println!("\n✗ Transfer failed: {}", e);
        }
    }

    Ok(())
} 