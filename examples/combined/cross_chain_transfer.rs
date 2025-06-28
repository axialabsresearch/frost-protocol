//! Cross-Chain Transfer Example
//! 
//! This example demonstrates a complete cross-chain transfer flow using
//! all major components of the FROST Protocol.
//! 
//! Components demonstrated:
//! - Finality verification (source and target chains)
//! - State management and verification
//! - Message handling and routing
//! - Network communication
//! - Error handling and recovery

#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::sync::Arc;
use frost_protocol::{
    finality::{
        FinalityVerifier,
        FinalityConfig,
        EthereumVerifier,
        SubstrateVerifier,
        BasicMetrics,
        FinalitySignal,
        EthereumFinalityType,
    },
    state::{
        BlockRef,
        ChainId,
    },
    message::{
        FrostMessage,
        MessageType,
        MessageMetadata,
    },
    network::{
        NetworkProtocol,
        NetworkConfig,
        BasicNetwork,
        PeerInfo,
        NetworkMetrics,
    },
    routing::{
        MessageRouter,
        RoutingConfig,
        BasicRouter,
        RoutingStrategy,
    },
    metrics::{
        ChainMetrics,
        ChainMetricsCollector,
    },
    Result,
};
use tokio::time;
use uuid::Uuid;

const TRANSFER_AMOUNT: u128 = 1_000_000_000_000_000_000; // 1 ETH
const MAX_TRANSFER_TIME: Duration = Duration::from_secs(300);

struct SharedNetwork(Arc<BasicNetwork>);

impl Clone for SharedNetwork {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl NetworkProtocol for SharedNetwork {
    async fn start(&mut self) -> Result<()> {
        Arc::get_mut(&mut self.0).unwrap().start().await
    }

    async fn stop(&mut self) -> Result<()> {
        Arc::get_mut(&mut self.0).unwrap().stop().await
    }

    async fn broadcast(&self, message: FrostMessage) -> Result<()> {
        (*self.0).broadcast(message).await
    }

    async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<()> {
        (*self.0).send_to(peer_id, message).await
    }

    async fn get_peers(&self) -> Result<Vec<String>> {
        (*self.0).get_peers().await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging and metrics
    tracing_subscriber::fmt::init();
    let metrics = ChainMetrics::default();

    // Step 1: Initialize Components
    println!("\nInitializing protocol components...");
    let (
        eth_verifier,
        sub_verifier,
        network,
        router
    ) = initialize_components().await?;

    // Step 2: Set up transfer parameters
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polkadot");
    let recipient = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    println!("\nInitiating cross-chain transfer:");
    println!("From: {}", source_chain);
    println!("To: {} ({})", target_chain, recipient);
    println!("Amount: {} ETH", TRANSFER_AMOUNT);

    // Step 3: Verify source chain state
    println!("\nVerifying source chain state...");
    let source_block = BlockRef::new(source_chain.clone(), 0, [0u8; 32]);
    let signal = FinalitySignal::Ethereum {
        block_number: 0,
        block_hash: [0u8; 32],
        confirmations: 12,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    let is_final = eth_verifier.verify_finality(&source_block, &signal).await?;
    let source_state = verify_source_state(TRANSFER_AMOUNT)?;
    println!("✓ Source chain state verified");

    // Step 4: Create and send transfer message
    println!("\nCreating transfer message...");
    let message = create_transfer_message(
        &source_chain,
        &target_chain,
        &source_block,
        recipient,
        TRANSFER_AMOUNT,
    );

    // Step 5: Discover and select optimal route
    println!("\nDiscovering routes...");
    let routes = router.get_routes().await.map_err(|e| format!("Route error: {}", e))?;
    println!("Found {} possible routes", routes.len());

    // Step 6: Send message and monitor progress
    println!("\nSending transfer message...");
    let mut transfer_complete = false;
    let start_time = time::Instant::now();

    let message_id = router.route(message.clone()).await.map_err(|e| format!("Routing error: {}", e))?;
    
    while !transfer_complete && start_time.elapsed() < MAX_TRANSFER_TIME {
        // Monitor message progress
        let routes = router.get_routes().await.map_err(|e| format!("Status error: {}", e))?;
        print_transfer_status(&format!("Active routes: {}", routes.len()));

        // Check target chain state
        if !routes.is_empty() {
            let target_block = BlockRef::new(target_chain.clone(), 0, [0u8; 32]);
            let signal = FinalitySignal::Substrate {
                block_number: 0,
                block_hash: [0u8; 32],
                metadata: None,
            };
            let is_final = sub_verifier.verify_finality(&target_block, &signal).await?;
            if verify_target_state(&message)? {
                transfer_complete = true;
                println!("✓ Transfer completed successfully!");
                break;
            }
        }

        time::sleep(Duration::from_secs(5)).await;
    }

    if !transfer_complete {
        println!("! Transfer timed out or failed");
        // Initiate failure recovery...
    }

    // Step 7: Print final metrics
    print_transfer_metrics(&metrics).await?;

    Ok(())
}

/// Initialize all protocol components
async fn initialize_components() -> Result<(
    EthereumVerifier,
    SubstrateVerifier,
    SharedNetwork,
    BasicRouter<SharedNetwork>,
)> {
    // Initialize finality verifiers
    let eth_config = FinalityConfig {
        min_confirmations: 12,  // ~3 minutes for Ethereum
        finality_timeout: Duration::from_secs(600),
        basic_params: HashMap::new(),
    };
    let eth_verifier = EthereumVerifier::new(eth_config);

    let sub_config = FinalityConfig {
        min_confirmations: 1,   // GRANDPA provides instant finality
        finality_timeout: Duration::from_secs(300),
        basic_params: HashMap::new(),
    };
    let sub_verifier = SubstrateVerifier::new(sub_config);

    // Initialize network
    let network_config = NetworkConfig {
        node_id: Uuid::new_v4().to_string(),
        listen_addr: "0.0.0.0:0".to_string(),
        bootstrap_peers: vec![
            "/dns4/bootstrap.frost.network/tcp/443/p2p/QmBootstrap1".to_string(),
        ],
        protocol_version: 1,
    };
    let network = Arc::new(BasicNetwork::new(network_config));
    let shared_network = SharedNetwork(network);

    // Initialize router
    let router_config = RoutingConfig {
        node_id: "node1".to_string(),
        route_timeout: 60,
        max_routes: 1000,
    };
    let router = BasicRouter::new(router_config, shared_network.clone());

    Ok((
        eth_verifier,
        sub_verifier,
        shared_network,
        router,
    ))
}

/// Create a transfer message
fn create_transfer_message(
    source: &ChainId,
    target: &ChainId,
    source_block: &BlockRef,
    recipient: &str,
    amount: u128,
) -> FrostMessage {
    FrostMessage::new(
        MessageType::StateTransition,
        serde_json::json!({
            "action": "transfer",
            "amount": amount.to_string(),
            "recipient": recipient,
        }).to_string().into_bytes(),
        source.to_string(),
        Some(target.to_string()),
    )
}

/// Verify source chain state
fn verify_source_state(amount: u128) -> Result<bool> {
    // Simplified state verification for example
    Ok(true)
}

/// Verify target chain state
fn verify_target_state(message: &FrostMessage) -> Result<bool> {
    // Simplified state verification for example
    Ok(true)
}

/// Print transfer status
fn print_transfer_status(status: &str) {
    use std::fmt::Write;
    let mut status_str = String::new();
    write!(status_str, "\rTransfer status: {}", status).unwrap();
    print!("{}", status_str);
}

/// Print transfer metrics
async fn print_transfer_metrics(metrics: &ChainMetrics) -> Result<()> {
    println!("\n\nTransfer Metrics:");
    println!("Total messages: {}", metrics.total_messages);
    println!("Success rate: {:.1}%", 
        (metrics.total_messages - metrics.failed_messages) as f64 / metrics.total_messages as f64 * 100.0);
    println!("Average block time: {:.2}s", metrics.avg_block_time);
    
    Ok(())
} 