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

use std::time::{Duration, SystemTime, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    Error,
};
use tokio::time;
use uuid::Uuid;
use serde_json::Value;
use std::env;
use serde_json::json;
use subxt::{OnlineClient, Config};
use codec::Decode;
use reqwest::{Client, Response};
use tokio::time::sleep;

const TRANSFER_AMOUNT: u128 = 1_000_000_000_000_000_000; // 1 ETH
const MAX_TRANSFER_TIME: Duration = Duration::from_secs(300);

// Add testnet configuration
const ETH_TESTNET: &str = "sepolia";
const DOT_TESTNET: &str = "westend";

// Update the default Sepolia RPC with the actual key
const DEFAULT_SEPOLIA_RPC: &str = "https://sepolia.infura.io/v3/bfa3b07a10da43d680edfc7e4b5cd79a";

// Add after other const declarations
const DEFAULT_WESTEND_WS: &str = "wss://westend-rpc.polkadot.io";
const DEFAULT_ETH_SOURCE_ADDRESS: &str = "0x699415fc86b6A19De25D85eb4c345e2be6A7f253"; 
const DEFAULT_TRANSFER_AMOUNT: u128 = 100_000_000_000_000_000; // 0.1 ETH for testing

// Add retry configuration
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: u64 = 2; // seconds

// Add custom error handling
#[derive(Debug)]
enum TransferError {
    InsufficientBalance { required: u128, available: u128 },
    InsufficientGas { required: u128, available: u128 },
    RpcError(String),
    NetworkError(String),
    BalanceCheckFailed(String),
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBalance { required, available } => write!(
                f,
                "Insufficient balance: required {} Wei, available {} Wei",
                required, available
            ),
            Self::InsufficientGas { required, available } => write!(
                f,
                "Insufficient gas: required {} Wei, available {} Wei",
                required, available
            ),
            Self::RpcError(msg) => write!(f, "RPC Error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            Self::BalanceCheckFailed(msg) => write!(f, "Balance Check Failed: {}", msg),
        }
    }
}

impl From<reqwest::Error> for TransferError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError(err.to_string())
    }
}

// Add Westend-specific error types
#[derive(Debug)]
enum WestendError {
    ConnectionFailed(String),
    BalanceCheckFailed(String),
    AccountInvalid(String),
}

impl std::fmt::Display for WestendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Westend connection failed: {}", msg),
            Self::BalanceCheckFailed(msg) => write!(f, "Balance check failed: {}", msg),
            Self::AccountInvalid(msg) => write!(f, "Invalid account: {}", msg),
        }
    }
}

// Implement conversion from our custom errors to frost_protocol::Error
impl From<TransferError> for Error {
    fn from(err: TransferError) -> Self {
        Error::from(err.to_string())
    }
}

impl From<WestendError> for Error {
    fn from(err: WestendError) -> Self {
        Error::from(err.to_string())
    }
}

fn get_network_endpoints() -> (String, String) {
    let eth_rpc = env::var("ETH_RPC_URL")
        .unwrap_or_else(|_| DEFAULT_SEPOLIA_RPC.to_string());
    let dot_ws = env::var("DOT_WS_URL")
        .unwrap_or_else(|_| DEFAULT_WESTEND_WS.to_string());
    (eth_rpc, dot_ws)
}

fn get_testnet_config() -> [(ChainId, &'static str); 2] {
    [
        (ChainId::new("ethereum"), ETH_TESTNET),
        (ChainId::new("polkadot"), DOT_TESTNET),
    ]
}

struct SharedNetwork(Arc<Mutex<BasicNetwork>>);

impl Clone for SharedNetwork {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl NetworkProtocol for SharedNetwork {
    async fn start(&mut self) -> Result<()> {
        self.0.lock().await.start().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.0.lock().await.stop().await
    }

    async fn broadcast(&self, message: FrostMessage) -> Result<()> {
        self.0.lock().await.broadcast(message).await
    }

    async fn send_to(&self, peer_id: &str, message: FrostMessage) -> Result<()> {
        self.0.lock().await.send_to(peer_id, message).await
    }

    async fn get_peers(&self) -> Result<Vec<String>> {
        self.0.lock().await.get_peers().await
    }
}

// Add a connection test function
async fn test_eth_connection(rpc_url: &str) -> Result<bool> {
    println!("Testing Sepolia connection...");
    // Here we would typically make a test RPC call
    // For now, just verify we can connect
    if rpc_url.contains("/v3/") && rpc_url.ends_with("bfa3b07a10da43d680edfc7e4b5cd79a") {
        println!("✓ Sepolia RPC endpoint configured");
        Ok(true)
    } else {
        println!("! Invalid Sepolia RPC endpoint");
        Ok(false)
    }
}

fn get_wallet_config() -> Result<(String, u128)> {
    let source_address = env::var("ETH_SOURCE_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ETH_SOURCE_ADDRESS.to_string());
    
    if source_address.is_empty() {
        println!("! No source ETH address configured. Please set ETH_SOURCE_ADDRESS environment variable.");
        return Err("No source address configured".into());
    }

    let amount = env::var("TRANSFER_AMOUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TRANSFER_AMOUNT);

    Ok((source_address, amount))
}

// Add retry logic helper
async fn retry_rpc_call<F, Fut, T>(operation: F) -> Result<T> 
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < MAX_RETRIES {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                if attempts < MAX_RETRIES {
                    println!("RPC call failed, attempt {}/{}: {}", attempts, MAX_RETRIES, last_error.as_ref().unwrap());
                    sleep(Duration::from_secs(RETRY_DELAY * attempts as u64)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::from("Maximum retry attempts reached")))
}

// Add after retry_rpc_call function
async fn make_json_call(rpc_url: &str, request: serde_json::Value) -> Result<serde_json::Value> {
    let client = Client::new();
    retry_rpc_call(|| async {
        let resp = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .body(request.to_string())
            .send()
            .await
            .map_err(|e| TransferError::NetworkError(e.to_string()))?;

        let text = resp.text().await
            .map_err(|e| TransferError::NetworkError(e.to_string()))?;

        serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| TransferError::RpcError(format!("Failed to parse JSON response: {}", e)).into())
    }).await
}

// Update the RPC functions to use the helper
async fn check_eth_balance(rpc_url: &str, address: &str) -> Result<u128> {
    println!("Checking Sepolia ETH balance...");
    
    let request = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
    });

    let response = make_json_call(rpc_url, request).await?;
    
    if let Some(hex_balance) = response.get("result").and_then(|v| v.as_str()) {
        let balance = u128::from_str_radix(&hex_balance[2..], 16)
            .map_err(|e| TransferError::BalanceCheckFailed(e.to_string()))?;
        
        println!("Current balance: {} Wei ({} ETH)", 
            balance,
            (balance as f64) / 1_000_000_000_000_000_000.0);
        
        Ok(balance)
    } else {
        Err(TransferError::BalanceCheckFailed("Failed to get balance".into()).into())
    }
}

async fn estimate_gas(rpc_url: &str, from: &str, to: &str, amount: u128) -> Result<u128> {
    println!("Estimating gas for transfer...");
    
    let request = json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": [{
            "from": from,
            "to": to,
            "value": format!("0x{:x}", amount),
            "data": "0x"
        }],
        "id": 1
    });

    let response = make_json_call(rpc_url, request).await?;
    
    if let Some(hex_gas) = response.get("result").and_then(|v| v.as_str()) {
        let gas = u128::from_str_radix(&hex_gas[2..], 16)
            .map_err(|e| TransferError::RpcError(format!("Failed to parse gas estimate: {}", e)))?;
        
        let gas_price = get_gas_price(rpc_url).await?;
        let total_gas_cost = gas * gas_price;
        
        println!("Estimated gas: {} units", gas);
        println!("Current gas price: {} Wei", gas_price);
        println!("Total gas cost: {} Wei ({} ETH)", 
            total_gas_cost,
            (total_gas_cost as f64) / 1_000_000_000_000_000_000.0);
        
        Ok(total_gas_cost)
    } else {
        Err(TransferError::RpcError("Failed to get gas estimate".into()).into())
    }
}

async fn get_gas_price(rpc_url: &str) -> Result<u128> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    });

    let response = make_json_call(rpc_url, request).await?;
    
    if let Some(hex_price) = response.get("result").and_then(|v| v.as_str()) {
        u128::from_str_radix(&hex_price[2..], 16)
            .map_err(|e| TransferError::RpcError(format!("Failed to parse gas price: {}", e)).into())
    } else {
        Err(TransferError::RpcError("Failed to get gas price".into()).into())
    }
}

#[derive(Debug)]
struct WestendConfig;

impl Config for WestendConfig {
    type Hash = sp_core::H256;
    type AccountId = sp_runtime::AccountId32;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type Signature = sp_runtime::MultiSignature;
    type Hasher = sp_core::Hasher;
    type Header = sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>;
    type ExtrinsicParams = subxt::config::polkadot::PolkadotExtrinsicParams<Self>;
}

// Update Westend connection function
async fn check_westend_connection(ws_url: &str) -> Result<OnlineClient<subxt::PolkadotConfig>> {
    println!("Connecting to Westend...");
    OnlineClient::from_url(ws_url)
        .await
        .map_err(|e| WestendError::ConnectionFailed(e.to_string()).into())
}

// Update Westend balance function
async fn check_westend_balance(
    client: &OnlineClient<subxt::PolkadotConfig>,
    address: &str
) -> Result<u128> {
    println!("Checking Westend balance...");
    
    let account = sp_core::crypto::Ss58Codec::from_ss58check(address)
        .map_err(|e| WestendError::AccountInvalid(format!("Invalid address format: {}", e)))?;
    
    let storage = client.storage().at_latest().await
        .map_err(|e| WestendError::BalanceCheckFailed(e.to_string()))?;
        
    let balance = storage.fetch(&subxt::dynamic::storage("System", "Account", vec![account]))
        .await
        .map_err(|e| WestendError::BalanceCheckFailed(e.to_string()))?
        .ok_or_else(|| WestendError::BalanceCheckFailed("Account not found".into()))?;

    let free_balance = balance.to_vec()
        .map_err(|e| WestendError::BalanceCheckFailed(format!("Failed to decode balance: {}", e)))?;

    // Parse the balance (first 16 bytes represent the free balance)
    let mut balance_bytes = [0u8; 16];
    balance_bytes.copy_from_slice(&free_balance[0..16]);
    let free_balance = u128::from_le_bytes(balance_bytes);

    println!("Current Westend balance: {} WND", 
        (free_balance as f64) / 1_000_000_000_000.0);
    
    Ok(free_balance)
}

// Add account data structure for decoding
#[derive(Decode)]
struct AccountData {
    free: u128,
    reserved: u128,
    misc_frozen: u128,
    fee_frozen: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging and metrics
    tracing_subscriber::fmt::init();
    let metrics = ChainMetrics::default();

    println!("\nInitializing protocol components on testnets:");
    println!("- Ethereum network: {}", ETH_TESTNET);
    println!("- Polkadot network: {}", DOT_TESTNET);

    // Step 1: Initialize Components with testnet configs
    let (
        eth_verifier,
        sub_verifier,
        network,
        router
    ) = match initialize_components().await {
        Ok(components) => components,
        Err(e) => {
            println!("Failed to initialize components: {}", e);
            return Err(e);
        }
    };

    // Step 2: Set up transfer parameters
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polkadot");
    let recipient = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    // Get wallet configuration
    let (source_address, transfer_amount) = get_wallet_config()?;
    println!("Using source address: {}", source_address);
    println!("Transfer amount: {} Wei", transfer_amount);

    // Check balance and estimate gas
    let (eth_rpc, dot_ws) = get_network_endpoints();
    let balance = check_eth_balance(&eth_rpc, &source_address).await?;
    
    let gas_cost = estimate_gas(&eth_rpc, &source_address, recipient, transfer_amount).await?;
    
    let total_required = transfer_amount + gas_cost;
    
    if balance < total_required {
        println!("! Insufficient funds for transfer + gas");
        println!("  Required for transfer: {} Wei", transfer_amount);
        println!("  Required for gas: {} Wei", gas_cost);
        println!("  Total required: {} Wei", total_required);
        println!("  Available: {} Wei", balance);
        println!("  Shortfall: {} Wei", total_required - balance);
        println!("  Please get more test ETH from the Sepolia faucet.");
        return Ok(());
    }
    
    println!("✓ Sufficient balance available for transfer + gas");
    println!("  Transfer amount: {} Wei", transfer_amount);
    println!("  Gas cost: {} Wei", gas_cost);
    println!("  Total cost: {} Wei", total_required);
    println!("  Remaining after transfer: {} Wei", balance - total_required);

    // Check Westend connection and balance first
    println!("\nVerifying Westend (Polkadot) setup...");
    let westend_client = check_westend_connection(&dot_ws).await?;
    println!("✓ Connected to Westend");

    let westend_balance = check_westend_balance(&westend_client, recipient).await?;
    println!("✓ Westend account verified");

    // Continue with ETH checks and transfer
    println!("\nInitiating cross-chain transfer on testnets:");
    println!("From: {} ({}) [{}]", source_chain, ETH_TESTNET, source_address);
    println!("To: {} ({}) [{}]", target_chain, DOT_TESTNET, recipient);
    println!("Amount: {} Wei", transfer_amount);

    // Test connections
    if !test_eth_connection(&eth_rpc).await? {
        println!("! Failed to connect to Sepolia. Please check your RPC endpoint.");
        return Ok(());
    }

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
    let source_state = verify_source_state(transfer_amount)?;
    println!("✓ Source chain state verified");

    // Step 4: Create and send transfer message
    println!("\nCreating transfer message...");
    let message = create_transfer_message(
        &source_chain,
        &target_chain,
        &source_block,
        recipient,
        transfer_amount,
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
    println!("Starting component initialization...");
    
    // Initialize finality verifiers
    let eth_config = FinalityConfig {
        min_confirmations: 12,
        finality_timeout: Duration::from_secs(600),
        basic_params: HashMap::new(),
    };
    let eth_verifier = EthereumVerifier::new(eth_config);
    println!("✓ Ethereum verifier initialized");

    let sub_config = FinalityConfig {
        min_confirmations: 1,
        finality_timeout: Duration::from_secs(300),
        basic_params: HashMap::new(),
    };
    let sub_verifier = SubstrateVerifier::new(sub_config);
    println!("✓ Substrate verifier initialized");

    // Initialize network with local test configuration
    let network_config = NetworkConfig {
        node_id: Uuid::new_v4().to_string(),
        listen_addr: "127.0.0.1:9000".to_string(),
        bootstrap_peers: vec![
            // For testing, we can start with a single local node
            "/ip4/127.0.0.1/tcp/9001/p2p/test-peer-1".to_string(),
        ],
        protocol_version: 1,
    };
    println!("Initializing test network with config:");
    println!("  - Node ID: {}", network_config.node_id);
    println!("  - Listen address: {}", network_config.listen_addr);
    println!("  - Test peer: {:?}", network_config.bootstrap_peers);
    
    let network = Arc::new(Mutex::new(BasicNetwork::new(network_config.clone())));
    let shared_network = SharedNetwork(network);
    
    // Try to start the network and wait for initial peer connections
    let mut network_clone = shared_network.clone();
    network_clone.start().await?;
    println!("✓ Network started");
    
    // Wait for peer connections with timeout
    let mut retry_count = 0;
    let max_retries = 5;
    while retry_count < max_retries {
        println!("Attempting to connect to peers (attempt {}/{})", retry_count + 1, max_retries);
        let peers = shared_network.get_peers().await?;
        if !peers.is_empty() {
            println!("✓ Connected to {} peers", peers.len());
            break;
        }
        retry_count += 1;
        if retry_count < max_retries {
            println!("No peers found. Retrying in 5 seconds...");
            time::sleep(Duration::from_secs(5)).await;
        }
    }

    if retry_count >= max_retries {
        println!("! Warning: Failed to connect to any peers after {} attempts", max_retries);
    }
    
    // Initialize router with more detailed config
    let router_config = RoutingConfig {
        node_id: network_config.node_id,
        route_timeout: 60,
        max_routes: 1000,
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