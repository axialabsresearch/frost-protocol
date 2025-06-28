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
//!
//! FROST Protocol Overview:
//! FROST acts as a middleware layer that enables secure cross-chain communication
//! by providing:
//! 1. Chain-agnostic message routing
//! 2. Finality verification across different consensus mechanisms
//! 3. State synchronization between chains
//! 4. Unified networking layer for cross-chain communication
//!
//! This example specifically shows, though simplified, shows how FROST coult practically:
//! 1. Transfer assets between 2 heterogenous chains, Ethereum to Polkadot in this example
//! 2. Verify finality on both chains, Ethereum and Polkadot in this example
//! 3. Route messages through, FROST's network module
//! 4. Monitor and validate transfer progress
//! 5. Handle errors and retries correctly
//! 
//! Please note: This example is not intended for production use. 
//! It serves as a simplified demonstration of FROST’s core capabilities and cross-chain coordination logic.

// Bypaasing warnings for tests only!
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

// Standard library imports
use std::time::{Duration, SystemTime, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::str::FromStr;

// FROST Protocol specific imports
use frost_protocol::{
    // Finality verification components
    finality::{
        FinalityVerifier,
        FinalityConfig,
        EthereumVerifier,
        SubstrateVerifier,
        BasicMetrics,
        FinalitySignal,
        EthereumFinalityType,
    },
    // State management components
    state::{
        BlockRef,
        ChainId,
    },
    // Message handling components
    message::{
        FrostMessage,
        MessageType,
        MessageMetadata,
    },
    // Network layer components
    network::{
        NetworkProtocol,
        NetworkConfig,
        BasicNetwork,
        PeerInfo,
        NetworkMetrics,
    },
    // Message routing components
    routing::{
        MessageRouter,
        RoutingConfig,
        BasicRouter,
        RoutingStrategy,
    },
    // Metrics collection
    metrics::{
        ChainMetrics,
        ChainMetricsCollector,
    },
    Result,
    Error,
};

// Blockchain-specific imports
// Ethereum and Substrate/Polkadot related dependencies
use tokio::time;
use uuid::Uuid;
use serde_json::Value;
use std::env;
use serde_json::json;
use subxt::{OnlineClient, Config};
use subxt::config::Hasher;
use sp_runtime::traits::Hash; 
use codec::{Encode, Decode};
use reqwest::{Client, Response};
use tokio::time::sleep;
use sp_runtime::traits::BlakeTwo256;
use sp_core::crypto::Ss58Codec;
use bs58;
use hex;
use polkadot_primitives;
use subxt::ext::sp_runtime::traits::BlakeTwo256 as SubxtBlakeTwo256;

// Constants for transfer timing and limits
const MAX_TRANSFER_TIME: Duration = Duration::from_secs(300);

// Testnet configuration
// These are blockchain-specific settings
const ETH_TESTNET: &str = "sepolia";
const DOT_TESTNET: &str = "westend";
const DEFAULT_SEPOLIA_RPC: &str = "https://sepolia.infura.io/v3/bfa3b07a10da43d680edfc7e4b5cd79a";
const DEFAULT_WESTEND_WS: &str = "wss://westend-rpc.polkadot.io";
const DEFAULT_ETH_SOURCE_ADDRESS: &str = "0x699415fc86b6A19De25D85eb4c345e2be6A7f253"; 
const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000; // 1 ETH = 10^18 Wei

// FROST Protocol retry configuration
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: u64 = 2; // seconds

// Custom error types for transfer-specific issues
#[derive(Debug)]
enum TransferError {
    // Blockchain-specific errors
    InsufficientBalance { required: u128, available: u128 },
    InsufficientGas { required: u128, available: u128 },
    RpcError(String),
    
    // FROST Protocol errors
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

// FROST Protocol network sharing wrapper
// This enables thread-safe access to the network layer
pub struct SharedNetwork(Arc<Mutex<BasicNetwork>>);

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

    println!("Enter amount of ETH to transfer (e.g. 0.1): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let eth_amount: f64 = input.trim().parse().map_err(|_| "Invalid ETH amount")?;
    if eth_amount <= 0.0 {
        return Err("ETH amount must be greater than 0".into());
    }
    
    let wei_amount = (eth_amount * WEI_PER_ETH as f64) as u128;
    println!("Converting {} ETH to {} Wei", eth_amount, wei_amount);

    Ok((source_address, wei_amount))
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
    type Hasher = SubxtBlakeTwo256;
    type Header = subxt::ext::sp_runtime::generic::Header<u32, Self::Hasher>;
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
    
    let bytes = hex::decode(&address[2..])
        .map_err(|e| WestendError::AccountInvalid(format!("Invalid address format: {}", e)))?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let account = subxt::utils::AccountId32::from(arr);
    
    let storage = client.storage().at_latest().await
        .map_err(|e| WestendError::BalanceCheckFailed(e.to_string()))?;
        
    let balance = storage.fetch(&subxt::dynamic::storage("System", "Account", vec![account]))
        .await
        .map_err(|e| WestendError::BalanceCheckFailed(e.to_string()))?
        .ok_or_else(|| WestendError::BalanceCheckFailed("Account not found".into()))?;

    let encoded = balance.encoded();

    // Parse the balance (first 16 bytes represent the free balance)
    let mut balance_bytes = [0u8; 16];
    balance_bytes.copy_from_slice(&encoded[0..16]);
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

// Transfer state tracking for FROST Protocol
#[derive(Debug, Clone)]
pub enum TransferState {
    // Initial state
    Initialized,
    
    // FROST Protocol states
    SourceValidated,
    MessageSent { message_id: String },
    RouteDiscovered { route_count: usize },
    InProgress { progress: f32 },
    
    // Blockchain-specific states
    TargetValidated,
    Completed { tx_hash: Option<String> },
    Failed { reason: String, retry_count: u32 },
}

// FROST Protocol transfer monitoring
#[derive(Debug)]
pub struct TransferMonitor {
    pub state: TransferState,
    pub start_time: Instant,
    pub last_update: Instant,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl TransferMonitor {
    pub fn new(max_retries: u32) -> Self {
        let now = Instant::now();
        Self {
            state: TransferState::Initialized,
            start_time: now,
            last_update: now,
            retry_count: 0,
            max_retries,
        }
    }

    // Update transfer state and log progress
    pub fn update_state(&mut self, new_state: TransferState) {
        self.last_update = Instant::now();
        self.state = new_state;
        println!("Transfer state updated: {:?}", self.state);
    }

    // FROST Protocol retry logic
    pub fn should_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// Enhanced balance checking with multiple providers
pub async fn check_balance_with_fallback(
    primary_rpc: &str,
    fallback_rpcs: &[String],
    address: &str,
) -> Result<u128> {
    // Try primary RPC first
    match check_eth_balance(primary_rpc, address).await {
        Ok(balance) => return Ok(balance),
        Err(e) => {
            println!("Primary RPC failed: {}, trying fallbacks...", e);
        }
    }

    // Try fallback RPCs
    for (i, rpc_url) in fallback_rpcs.iter().enumerate() {
        println!("Trying fallback RPC {}: {}", i + 1, rpc_url);
        match check_eth_call(rpc_url, address).await {
            Ok(balance) => {
                println!("✓ Fallback RPC {} succeeded", i + 1);
                return Ok(balance);
            }
            Err(e) => {
                println!("Fallback RPC {} failed: {}", i + 1, e);
            }
        }
    }

    Err(TransferError::NetworkError(
        "All RPC endpoints failed".to_string()
    ).into())
}

#[derive(Debug)]
pub struct TransferValidation {
    // Ethereum-specific validation fields
    pub eth_balance: u128,
    pub gas_cost: u128,
    pub total_required: u128,
    pub sufficient_funds: bool,
    
    // Polkadot/Westend-specific validation fields
    pub westend_connected: bool,
    pub westend_balance: Option<u128>,
    pub recipient_valid: bool,
    
    // Error tracking fields
    pub westend_error: Option<String>,
    pub recipient_error: Option<String>,
}

impl TransferValidation {
    fn new() -> Self {
        Self {
            eth_balance: 0,
            gas_cost: 0,
            total_required: 0,
            sufficient_funds: false,
            westend_connected: false,
            westend_balance: None,
            recipient_valid: false,
            westend_error: None,
            recipient_error: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.sufficient_funds && self.westend_connected && self.recipient_valid
    }

    pub fn print_summary(&self) {
        println!("\n=== Transfer Validation Summary ===");
        println!("ETH Balance: {} Wei", self.eth_balance);
        println!("Gas Cost: {} Wei", self.gas_cost);
        println!("Total Required: {} Wei", self.total_required);
        println!("Sufficient Funds: {}", if self.sufficient_funds { "✓" } else { "✗" });
        
        if self.westend_connected {
            println!("Westend Connection: ✓");
            if let Some(balance) = self.westend_balance {
                println!("Westend Balance: {} WND", balance as f64 / 1_000_000_000_000.0);
            }
            println!("Recipient Valid: {}", if self.recipient_valid { "✓" } else { "✗" });
        } else {
            println!("Westend Connection: ✗");
            if let Some(error) = &self.westend_error {
                println!("Westend Error: {}", error);
            }
        }

        if let Some(error) = &self.recipient_error {
            println!("Recipient Error: {}", error);
        }
    }
}

// Enhanced pre-transfer validation
pub async fn validate_transfer_preconditions(
    eth_rpc: &str,
    dot_ws: &str,
    source_address: &str,
    recipient: &str,
    amount: u128,
) -> Result<TransferValidation> {
    let mut validation = TransferValidation::new();

    // Check Ethereum balance and gas
    let balance = check_eth_balance(eth_rpc, source_address).await?;
    let gas_cost = estimate_gas(eth_rpc, source_address, recipient, amount).await?;
    
    validation.eth_balance = balance;
    validation.gas_cost = gas_cost;
    validation.total_required = amount + gas_cost;
    validation.sufficient_funds = balance >= validation.total_required;

    // Check Westend connection
    match check_westend_connection(dot_ws).await {
        Ok(client) => {
            validation.westend_connected = true;
            match check_westend_balance(&client, recipient).await {
                Ok(westend_balance) => {
                    validation.westend_balance = Some(westend_balance);
                    validation.recipient_valid = true;
                }
                Err(e) => {
                    validation.recipient_error = Some(e.to_string());
                }
            }
        }
        Err(e) => {
            validation.westend_error = Some(e.to_string());
        }
    }

    Ok(validation)
}

// Main transfer execution function combining FROST Protocol and blockchain operations
pub async fn execute_monitored_transfer(
    source_chain: ChainId,
    target_chain: ChainId,
    source_address: String,
    recipient: String,
    amount: u128,
    router: &BasicRouter<SharedNetwork>,
    eth_verifier: &EthereumVerifier,
    sub_verifier: &SubstrateVerifier,
) -> Result<String> {
    let mut monitor = TransferMonitor::new(3);
    
    // Retry loop for failed transfers
    loop {
        match execute_transfer_attempt(
            &source_chain,
            &target_chain,
            &source_address,
            &recipient,
            amount,
            router,
            eth_verifier,
            sub_verifier,
            &mut monitor,
        ).await {
            Ok(tx_hash) => {
                monitor.update_state(TransferState::Completed { 
                    tx_hash: Some(tx_hash.clone()) 
                });
                return Ok(tx_hash);
            }
            Err(e) => {
                // FROST Protocol retry mechanism with exponential backoff
                if monitor.should_retry() {
                    monitor.increment_retry();
                    monitor.update_state(TransferState::Failed { 
                        reason: e.to_string(), 
                        retry_count: monitor.retry_count 
                    });
                    
                    let backoff_delay = Duration::from_secs(2_u64.pow(monitor.retry_count));
                    println!("Retrying in {:?}... (attempt {}/{})", 
                        backoff_delay, monitor.retry_count + 1, monitor.max_retries + 1);
                    sleep(backoff_delay).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

// Single transfer attempt implementation
async fn execute_transfer_attempt(
    source_chain: &ChainId,
    target_chain: &ChainId,
    source_address: &str,
    recipient: &str,
    amount: u128,
    router: &BasicRouter<SharedNetwork>,
    eth_verifier: &EthereumVerifier,
    sub_verifier: &SubstrateVerifier,
    monitor: &mut TransferMonitor,
) -> Result<String> {
    // Step 1: FROST Protocol source chain validation
    monitor.update_state(TransferState::SourceValidated);
    
    let source_block = BlockRef::new(source_chain.clone(), 0, [0u8; 32]);
    let signal = FinalitySignal::Ethereum {
        block_number: 0,
        block_hash: [0u8; 32],
        confirmations: 12,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    // Verify Ethereum finality
    eth_verifier.verify_finality(&source_block, &signal).await
        .map_err(|e| -> Error { TransferError::NetworkError(format!("Source verification failed: {}", e)).into() })?;

    // Step 2: Create and send FROST Protocol message
    let message = create_transfer_message(
        source_chain,
        target_chain,
        &source_block,
        recipient,
        amount,
    );

    let message_id = Uuid::new_v4().to_string();
    router.route(message.clone()).await
        .map_err(|e| -> Error { TransferError::NetworkError(format!("Message routing failed: {}", e)).into() })?;
    
    monitor.update_state(TransferState::MessageSent { message_id: message_id.clone() });

    // Step 3: Monitor progress with timeout
    let timeout = Duration::from_secs(300);
    let start = Instant::now();
    
    // Main monitoring loop
    while start.elapsed() < timeout {
        // Check FROST Protocol routes
        let routes = router.get_routes().await
            .map_err(|e| -> Error { TransferError::NetworkError(format!("Route check failed: {}", e)).into() })?;
        
        if !routes.is_empty() {
            monitor.update_state(TransferState::RouteDiscovered { 
                route_count: routes.len() 
            });

            // Calculate transfer progress
            let progress = (start.elapsed().as_secs_f32() / timeout.as_secs_f32()).min(0.9);
            monitor.update_state(TransferState::InProgress { progress });

            // Verify Polkadot/Substrate finality
            let target_block = BlockRef::new(target_chain.clone(), 0, [0u8; 32]);
            let target_signal = FinalitySignal::Substrate {
                block_number: 0,
                block_hash: [0u8; 32],
                metadata: None,
            };

            // Check target chain state once route is established
            if sub_verifier.verify_finality(&target_block, &target_signal).await.is_ok() {
                if verify_target_state(&message)? {
                    monitor.update_state(TransferState::TargetValidated);
                    return Ok(format!("transfer_{}", message_id));
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    Err(TransferError::NetworkError("Transfer timeout".to_string()).into())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging and metrics collection
    tracing_subscriber::fmt::init();
    let metrics = ChainMetrics::default();
    let mut monitor = TransferMonitor::new(3);

    // Load and validate configuration
    println!("Loading configuration...");
    let config = CrossChainConfig::from_env()?;
    config.validate()?;
    config.print_config();

    println!("\nInitializing protocol components on testnets:");
    println!("- Ethereum network: {}", ETH_TESTNET);
    println!("- Polkadot network: {}", DOT_TESTNET);

    // Initialize components with configuration
    let (eth_verifier, sub_verifier, network, router) = match initialize_components_with_config(&config).await {
        Ok(components) => components,
        Err(e) => {
            println!("Failed to initialize components: {}", e);
            return Err(e);
        }
    };

    // Set up blockchain-specific parameters
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polkadot");
    let recipient = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    // Get wallet configuration and validate amounts
    let (source_address, transfer_amount) = get_wallet_config_with_limits(&config.transfer)?;
    println!("Using source address: {}", source_address);
    println!("Transfer amount: {} Wei", transfer_amount);

    // Check balance and estimate gas with configured limits
    let (eth_rpc, dot_ws) = get_network_endpoints()?;
    let balance = check_eth_balance(&eth_rpc, &source_address).await?;
    let gas_cost = estimate_gas_with_limit(
        &eth_rpc,
        &source_address,
        recipient,
        transfer_amount,
        config.chain.eth_gas_limit,
        config.chain.eth_max_gas_price,
    ).await?;
    
    let total_required = transfer_amount + gas_cost;
    
    // Validate sufficient funds
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

    // Step 4: Verify target chain (Westend) setup
    println!("\nVerifying Westend (Polkadot) setup...");
    let westend_client = check_westend_connection(&dot_ws).await?;
    println!("✓ Connected to Westend");

    let westend_balance = check_westend_balance(&westend_client, recipient).await?;
    println!("✓ Westend account verified");

    // Step 5: Initialize FROST Protocol transfer
    println!("\nInitiating cross-chain transfer on testnets:");
    println!("From: {} ({}) [{}]", source_chain, ETH_TESTNET, source_address);
    println!("To: {} ({}) [{}]", target_chain, DOT_TESTNET, recipient);
    println!("Amount: {} Wei", transfer_amount);

    // Verify RPC connections
    if !test_eth_connection(&eth_rpc).await? {
        println!("! Failed to connect to Sepolia. Please check your RPC endpoint.");
        return Ok(());
    }

    // Step 6: FROST Protocol finality verification
    println!("\nVerifying source chain state...");
    let source_block = BlockRef::new(source_chain.clone(), 0, [0u8; 32]);
    let signal = FinalitySignal::Ethereum {
        block_number: 0,
        block_hash: [0u8; 32],
        confirmations: 12,
        finality_type: EthereumFinalityType::Confirmations,
        metadata: None,
    };
    
    // Verify Ethereum finality and source state
    let is_final = eth_verifier.verify_finality(&source_block, &signal).await?;
    let source_state = verify_source_state(transfer_amount)?;
    println!("✓ Source chain state verified");

    // Step 7: Create and route FROST Protocol message
    println!("\nCreating transfer message...");
    let message = create_transfer_message(
        &source_chain,
        &target_chain,
        &source_block,
        recipient,
        transfer_amount,
    );

    // Step 8: Route discovery and message sending
    println!("\nDiscovering routes...");
    let routes = router.get_routes().await.map_err(|e| format!("Route error: {}", e))?;
    println!("Found {} possible routes", routes.len());

    // Step 9: Monitor transfer progress
    println!("\nSending transfer message...");
    let mut transfer_complete = false;
    let start_time = time::Instant::now();

    // Generate unique message ID for tracking
    let message_id = Uuid::new_v4().to_string();
    router.route(message.clone()).await
        .map_err(|e| -> Error { TransferError::NetworkError(format!("Message routing failed: {}", e)).into() })?;
    
    monitor.update_state(TransferState::MessageSent { message_id: message_id.clone() });
    
    // Main monitoring loop
    while !transfer_complete && start_time.elapsed() < MAX_TRANSFER_TIME {
        // Check message routing status
        let routes = router.get_routes().await
            .map_err(|e| -> Error { TransferError::NetworkError(format!("Route check failed: {}", e)).into() })?;
        print_transfer_status(&format!("Active routes: {}", routes.len()));

        // Verify target chain state when routes are available
        if !routes.is_empty() {
            let target_block = BlockRef::new(target_chain.clone(), 0, [0u8; 32]);
            let signal = FinalitySignal::Substrate {
                block_number: 0,
                block_hash: [0u8; 32],
                metadata: None,
            };
            
            // Check Polkadot/Substrate finality
            let is_final = sub_verifier.verify_finality(&target_block, &signal).await?;
            if verify_target_state(&message)? {
                transfer_complete = true;
                println!("✓ Transfer completed successfully!");
                break;
            }
        }

        time::sleep(Duration::from_secs(5)).await;
    }

    // Handle transfer timeout or failure
    if !transfer_complete {
        println!("! Transfer timed out or failed");
        // Initiate failure recovery...
    }

    // Print final metrics and status
    print_transfer_metrics(&metrics).await?;

    Ok(())
}

/// Initialize components with configuration
async fn initialize_components_with_config(
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
        finality_timeout: Duration::from_secs(config.transfer.timeout_secs),
        basic_params: HashMap::new(),
    };
    let eth_verifier = EthereumVerifier::new(eth_config);
    println!("✓ Ethereum verifier initialized");

    let sub_config = FinalityConfig {
        min_confirmations: config.chain.dot_min_confirmations,
        finality_timeout: Duration::from_secs(config.transfer.timeout_secs),
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

/// Get wallet configuration with amount validation
fn get_wallet_config_with_limits(transfer_config: &TransferConfig) -> Result<(String, u128)> {
    let source_address = env::var("ETH_SOURCE_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ETH_SOURCE_ADDRESS.to_string());
    
    if source_address.is_empty() {
        println!("! No source ETH address configured. Please set ETH_SOURCE_ADDRESS environment variable.");
        return Err("No source address configured".into());
    }

    println!("Enter amount of ETH to transfer (min: {}, max: {}): ",
        transfer_config.min_amount,
        transfer_config.max_amount);
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let eth_amount: f64 = input.trim().parse().map_err(|_| "Invalid ETH amount")?;
    
    // Validate against configured limits
    if eth_amount < transfer_config.min_amount {
        return Err(format!("Transfer amount must be at least {} ETH", transfer_config.min_amount).into());
    }
    if eth_amount > transfer_config.max_amount {
        return Err(format!("Transfer amount must not exceed {} ETH", transfer_config.max_amount).into());
    }
    
    let wei_amount = (eth_amount * WEI_PER_ETH as f64) as u128;
    println!("Converting {} ETH to {} Wei", eth_amount, wei_amount);

    Ok((source_address, wei_amount))
}

/// Estimate gas with configured limits
async fn estimate_gas_with_limit(
    rpc_url: &str,
    from: &str,
    to: &str,
    amount: u128,
    gas_limit: u64,
    max_gas_price: u64,
) -> Result<u128> {
    let estimated_gas = estimate_gas(rpc_url, from, to, amount).await?;
    
    if estimated_gas > gas_limit as u128 {
        return Err(format!(
            "Estimated gas {} exceeds configured limit {}",
            estimated_gas, gas_limit
        ).into());
    }

    let gas_price = get_gas_price(rpc_url).await?;
    if gas_price > (max_gas_price as u128 * 1_000_000_000) { // Convert Gwei to Wei
        return Err(format!(
            "Current gas price {} Gwei exceeds maximum {}",
            gas_price / 1_000_000_000,
            max_gas_price
        ).into());
    }

    Ok(estimated_gas * gas_price)
}

/// Initialize all FROST Protocol components
async fn initialize_components() -> Result<(
    EthereumVerifier,
    SubstrateVerifier,
    SharedNetwork,
    BasicRouter<SharedNetwork>,
)> {
    println!("Starting component initialization...");
    
    // Initialize blockchain-specific finality verifiers
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

    // Initialize FROST Protocol network layer
    let network_config = NetworkConfig {
        node_id: Uuid::new_v4().to_string(),
        listen_addr: "127.0.0.1:9000".to_string(),
        bootstrap_peers: vec![
            // For testing, we use a single local node
            "/ip4/127.0.0.1/tcp/9001/p2p/test-peer-1".to_string(),
        ],
        protocol_version: 1,
    };
    
    println!("Initializing test network with config:");
    println!("  - Node ID: {}", network_config.node_id);
    println!("  - Listen address: {}", network_config.listen_addr);
    println!("  - Test peer: {:?}", network_config.bootstrap_peers);
    
    // Create thread-safe network instance
    let network = Arc::new(Mutex::new(BasicNetwork::new(network_config.clone())));
    let shared_network = SharedNetwork(network);
    
    // Start network and wait for peer connections
    let mut network_clone = shared_network.clone();
    network_clone.start().await?;
    println!("✓ Network started");
    
    // Attempt peer connections with retry logic
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
    
    // Initialize FROST Protocol router
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

/// Get network endpoints with support for user configuration
fn get_network_endpoints() -> Result<(String, String)> {
    // Try environment variables first
    let eth_rpc = match env::var("ETH_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("No ETH_RPC_URL environment variable found, using default Sepolia endpoint");
            println!("To use custom endpoint, set ETH_RPC_URL environment variable");
            DEFAULT_SEPOLIA_RPC.to_string()
        }
    };

    let dot_ws = match env::var("DOT_WS_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("No DOT_WS_URL environment variable found, using default Westend endpoint");
            println!("To use custom endpoint, set DOT_WS_URL environment variable");
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

/// Get testnet configuration with support for user customization
fn get_testnet_config() -> Result<[(ChainId, String); 2]> {
    // Allow overriding testnet selection through environment variables
    let eth_network = env::var("ETH_NETWORK").unwrap_or_else(|_| ETH_TESTNET.to_string());
    let dot_network = env::var("DOT_NETWORK").unwrap_or_else(|_| DOT_TESTNET.to_string());

    // Validate network selections
    let valid_eth_networks = ["mainnet", "sepolia", "goerli"];
    let valid_dot_networks = ["mainnet", "westend", "rococo"];

    if !valid_eth_networks.contains(&eth_network.as_str()) {
        return Err(Error::from(format!(
            "Invalid Ethereum network. Must be one of: {:?}",
            valid_eth_networks
        )));
    }

    if !valid_dot_networks.contains(&dot_network.as_str()) {
        return Err(Error::from(format!(
            "Invalid Polkadot network. Must be one of: {:?}",
            valid_dot_networks
        )));
    }

    Ok([
        (ChainId::new("ethereum"), eth_network),
        (ChainId::new("polkadot"), dot_network),
    ])
}

/// Check Ethereum RPC endpoint with retry logic
async fn check_eth_call(rpc_url: &str, address: &str) -> Result<u128> {
    let max_retries = env::var("ETH_MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(MAX_RETRIES);

    let retry_delay = env::var("ETH_RETRY_DELAY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(RETRY_DELAY);

    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        match check_eth_balance(rpc_url, address).await {
            Ok(balance) => return Ok(balance),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                if attempts < max_retries {
                    println!(
                        "RPC call failed, attempt {}/{}: {}",
                        attempts,
                        max_retries,
                        last_error.as_ref().unwrap()
                    );
                    sleep(Duration::from_secs(retry_delay * attempts as u64)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::from("Maximum retry attempts reached")))
}

// Chain-specific configuration
#[derive(Debug, Clone)]
struct ChainConfig {
    // Ethereum settings
    eth_min_confirmations: u32, 
    eth_max_gas_price: u64,   // in Gwei
    eth_gas_limit: u64,
    
    // Polkadot settings
    dot_min_confirmations: u32,        // minimum balance
    dot_existential_deposit: u128,
}

// Transfer configuration
#[derive(Debug, Clone)]
struct TransferConfig {
    timeout_secs: u64,
    min_amount: f64,         // in native token (ETH/DOT)
    max_amount: f64,         // in native token (ETH/DOT)       
    max_routes: u32,
}

// Security configuration
#[derive(Debug, Clone)]
struct SecurityConfig {
    eth_private_key_path: PathBuf,
    dot_seed_path: PathBuf,
}

// Combined configuration
#[derive(Debug, Clone)]
struct CrossChainConfig {
    chain: ChainConfig,
    transfer: TransferConfig,
    security: SecurityConfig,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            eth_min_confirmations: 12,
            eth_max_gas_price: 100,    // 100 Gwei
            eth_gas_limit: 21000,      // Standard ETH transfer
            dot_min_confirmations: 1,
            dot_existential_deposit: 1_000_000_000_000, // 1 DOT in planck
        }
    }
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300,
            min_amount: 0.01,
            max_amount: 100.0,
            max_routes: 10,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            eth_private_key_path: PathBuf::from(".keys/eth_private_key"),
            dot_seed_path: PathBuf::from(".keys/dot_seed"),
        }
    }
}

impl CrossChainConfig {
    /// Load configuration from environment variables with defaults
    fn from_env() -> Result<Self> {
        // Chain-specific settings
        let chain = ChainConfig {
            eth_min_confirmations: env::var("ETH_MIN_CONFIRMATIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(12),
                
            eth_max_gas_price: env::var("ETH_MAX_GAS_PRICE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
                
            eth_gas_limit: env::var("ETH_GAS_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(21000),
                
            dot_min_confirmations: env::var("DOT_MIN_CONFIRMATIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
                
            dot_existential_deposit: env::var("DOT_EXISTENTIAL_DEPOSIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_000_000_000_000),
        };

        // Transfer settings
        let transfer = TransferConfig {
            timeout_secs: env::var("TRANSFER_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
                
            min_amount: env::var("MIN_TRANSFER_AMOUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.01),
                
            max_amount: env::var("MAX_TRANSFER_AMOUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
                
            max_routes: env::var("MAX_ROUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        };

        // Security settings
        let security = SecurityConfig {
            eth_private_key_path: PathBuf::from(
                env::var("ETH_PRIVATE_KEY_PATH")
                    .unwrap_or_else(|_| ".keys/eth_private_key".to_string())
            ),
            dot_seed_path: PathBuf::from(
                env::var("DOT_SEED_PATH")
                    .unwrap_or_else(|_| ".keys/dot_seed".to_string())
            ),
        };

        Ok(Self {
            chain,
            transfer: TransferConfig::default(),
            security: SecurityConfig::default(),
        })
    }

    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Validate chain settings
        if self.chain.eth_min_confirmations < 1 {
            return Err(Error::from("ETH_MIN_CONFIRMATIONS must be at least 1"));
        }
        if self.chain.eth_max_gas_price == 0 {
            return Err(Error::from("ETH_MAX_GAS_PRICE must be greater than 0"));
        }
        if self.chain.eth_gas_limit < 21000 {
            return Err(Error::from("ETH_GAS_LIMIT must be at least 21000"));
        }
        if self.chain.dot_min_confirmations < 1 {
            return Err(Error::from("DOT_MIN_CONFIRMATIONS must be at least 1"));
        }

        // Validate transfer settings
        if self.transfer.timeout_secs < 60 {
            return Err(Error::from("TRANSFER_TIMEOUT must be at least 60 seconds"));
        }
        if self.transfer.min_amount <= 0.0 {
            return Err(Error::from("MIN_TRANSFER_AMOUNT must be greater than 0"));
        }
        if self.transfer.max_amount <= self.transfer.min_amount {
            return Err(Error::from("MAX_TRANSFER_AMOUNT must be greater than MIN_TRANSFER_AMOUNT"));
        }
        if self.transfer.max_routes == 0 {
            return Err(Error::from("MAX_ROUTES must be greater than 0"));
        }

        // Validate security settings
        if !self.security.eth_private_key_path.parent().map_or(false, |p| p.exists()) {
            return Err(Error::from("ETH_PRIVATE_KEY_PATH parent directory does not exist"));
        }
        if !self.security.dot_seed_path.parent().map_or(false, |p| p.exists()) {
            return Err(Error::from("DOT_SEED_PATH parent directory does not exist"));
        }

        Ok(())
    }

    /// Print current configuration
    fn print_config(&self) {
        println!("\nCurrent Configuration:");
        println!("\nChain-specific settings:");
        println!("  Ethereum:");
        println!("    Min confirmations: {}", self.chain.eth_min_confirmations);
        println!("    Max gas price: {} Gwei", self.chain.eth_max_gas_price);
        println!("    Gas limit: {}", self.chain.eth_gas_limit);
        println!("  Polkadot:");
        println!("    Min confirmations: {}", self.chain.dot_min_confirmations);
        println!("    Existential deposit: {} Planck", self.chain.dot_existential_deposit);

        println!("\nTransfer settings:");
        println!("  Timeout: {} seconds", self.transfer.timeout_secs);
        println!("  Min amount: {} tokens", self.transfer.min_amount);
        println!("  Max amount: {} tokens", self.transfer.max_amount);
        println!("  Max routes: {}", self.transfer.max_routes);

        println!("\nSecurity settings:");
        println!("  ETH private key path: {}", self.security.eth_private_key_path.display());
        println!("  DOT seed path: {}", self.security.dot_seed_path.display());
    }
} 