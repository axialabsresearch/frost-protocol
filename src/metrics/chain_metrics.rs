use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// Chain performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    /// Chain identifier
    pub chain_id: String,
    /// Average block time
    pub avg_block_time: f64,
    /// Average finality time
    pub avg_finality_time: f64,
    /// Total blocks processed
    pub total_blocks: u64,
    /// Total messages processed
    pub total_messages: u64,
    /// Failed message count
    pub failed_messages: u64,
    /// Average message size
    pub avg_message_size: f64,
    /// Chain-specific metrics
    pub chain_data: serde_json::Value,
}

impl Default for ChainMetrics {
    fn default() -> Self {
        Self {
            chain_id: String::new(),
            avg_block_time: 0.0,
            avg_finality_time: 0.0,
            total_blocks: 0,
            total_messages: 0,
            failed_messages: 0,
            avg_message_size: 0.0,
            chain_data: serde_json::json!({}),
        }
    }
}

/// Chain metrics collector trait
#[async_trait::async_trait]
pub trait ChainMetricsCollector: Send + Sync {
    /// Record block metrics
    async fn record_block(&mut self, block_time: Duration, finality_time: Duration);
    
    /// Record message metrics
    async fn record_message(&mut self, size: usize, success: bool);
    
    /// Get current metrics
    async fn get_metrics(&self) -> ChainMetrics;
    
    /// Update chain-specific metrics
    async fn update_chain_data(&mut self, data: serde_json::Value);
}

/// Ethereum metrics collector
pub struct EthereumMetrics {
    metrics: Arc<RwLock<ChainMetrics>>,
    consensus_metrics: Arc<RwLock<EthereumConsensusMetrics>>,
}

/// Ethereum consensus-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EthereumConsensusMetrics {
    // Beacon chain metrics
    pub avg_participation_rate: f64,
    pub missed_attestations: u64,
    pub missed_blocks: u64,
    pub total_attestations: u64,
    pub total_blocks_proposed: u64,
    
    // Validator metrics
    pub active_validator_count: u64,
    pub total_validator_count: u64,
    pub avg_validator_balance: f64,
    pub total_validator_rewards: u64,
    
    // Fork choice metrics
    pub fork_choice_updates: u64,
    pub reorgs_count: u64,
    pub max_reorg_depth: u64,
    pub avg_reorg_depth: f64,
    
    // Sync metrics
    pub sync_participation_rate: f64,
    pub sync_committee_size: u64,
    pub sync_aggregate_count: u64,
}

impl EthereumMetrics {
    pub fn new() -> Self {
        let metrics = ChainMetrics {
            chain_id: "ethereum".into(),
            chain_data: serde_json::json!({
                "gas_used": 0u64,
                "avg_gas_price": 0u64,
                "total_value_transferred": "0",
            }),
            ..Default::default()
        };
        
        Self {
            metrics: Arc::new(RwLock::new(metrics)),
            consensus_metrics: Arc::new(RwLock::new(EthereumConsensusMetrics::default())),
        }
    }

    /// Record consensus-specific metrics
    pub async fn record_consensus_metrics(
        &mut self,
        participation_rate: f64,
        validator_count: u64,
        validator_balance: u64,
        reorg_depth: Option<u64>,
    ) {
        let mut metrics = self.consensus_metrics.write().await;
        
        // Update participation metrics
        metrics.total_attestations += 1;
        metrics.avg_participation_rate = (metrics.avg_participation_rate * (metrics.total_attestations - 1) as f64
            + participation_rate) / metrics.total_attestations as f64;
            
        // Update validator metrics
        metrics.total_validator_count = validator_count;
        metrics.avg_validator_balance = (metrics.avg_validator_balance * (metrics.total_validator_count - 1) as f64
            + validator_balance as f64) / metrics.total_validator_count as f64;
            
        // Update reorg metrics if provided
        if let Some(depth) = reorg_depth {
            metrics.reorgs_count += 1;
            metrics.max_reorg_depth = metrics.max_reorg_depth.max(depth);
            metrics.avg_reorg_depth = (metrics.avg_reorg_depth * (metrics.reorgs_count - 1) as f64
                + depth as f64) / metrics.reorgs_count as f64;
        }
    }
}

#[async_trait::async_trait]
impl ChainMetricsCollector for EthereumMetrics {
    async fn record_block(&mut self, block_time: Duration, finality_time: Duration) {
        let mut metrics = self.metrics.write().await;
        let mut consensus_metrics = self.consensus_metrics.write().await;
        
        metrics.total_blocks += 1;
        consensus_metrics.total_blocks_proposed += 1;
        
        metrics.avg_block_time = (metrics.avg_block_time * (metrics.total_blocks - 1) as f64
            + block_time.as_secs_f64()) / metrics.total_blocks as f64;
        metrics.avg_finality_time = (metrics.avg_finality_time * (metrics.total_blocks - 1) as f64
            + finality_time.as_secs_f64()) / metrics.total_blocks as f64;
    }
    
    async fn record_message(&mut self, size: usize, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_messages += 1;
        metrics.avg_message_size = (metrics.avg_message_size * (metrics.total_messages - 1) as f64
            + size as f64) / metrics.total_messages as f64;
            
        if !success {
            metrics.failed_messages += 1;
        }
    }
    
    async fn get_metrics(&self) -> ChainMetrics {
        let mut metrics = self.metrics.read().await.clone();
        let consensus_metrics = self.consensus_metrics.read().await.clone();
        
        // Include consensus metrics in chain data
        metrics.chain_data = serde_json::json!({
            "execution": {
                "gas_used": metrics.chain_data["gas_used"],
                "avg_gas_price": metrics.chain_data["avg_gas_price"],
                "total_value_transferred": metrics.chain_data["total_value_transferred"],
            },
            "consensus": {
                "participation_rate": consensus_metrics.avg_participation_rate,
                "validator_count": consensus_metrics.total_validator_count,
                "avg_validator_balance": consensus_metrics.avg_validator_balance,
                "reorgs": {
                    "count": consensus_metrics.reorgs_count,
                    "max_depth": consensus_metrics.max_reorg_depth,
                    "avg_depth": consensus_metrics.avg_reorg_depth,
                },
                "sync": {
                    "participation_rate": consensus_metrics.sync_participation_rate,
                    "committee_size": consensus_metrics.sync_committee_size,
                },
            },
        });
        
        metrics
    }
    
    async fn update_chain_data(&mut self, data: serde_json::Value) {
        let mut metrics = self.metrics.write().await;
        metrics.chain_data = data;
    }
}

/// Cosmos metrics collector
pub struct CosmosMetrics {
    metrics: Arc<RwLock<ChainMetrics>>,
}

impl CosmosMetrics {
    pub fn new() -> Self {
        let metrics = ChainMetrics {
            chain_id: "cosmos".into(),
            chain_data: serde_json::json!({
                "gas_used": 0u64,
                "avg_block_size": 0u64,
                "total_fees_collected": "0",
            }),
            ..Default::default()
        };
        
        Self {
            metrics: Arc::new(RwLock::new(metrics)),
        }
    }
}

#[async_trait::async_trait]
impl ChainMetricsCollector for CosmosMetrics {
    async fn record_block(&mut self, block_time: Duration, finality_time: Duration) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_blocks += 1;
        metrics.avg_block_time = (metrics.avg_block_time * (metrics.total_blocks - 1) as f64
            + block_time.as_secs_f64()) / metrics.total_blocks as f64;
        metrics.avg_finality_time = (metrics.avg_finality_time * (metrics.total_blocks - 1) as f64
            + finality_time.as_secs_f64()) / metrics.total_blocks as f64;
    }
    
    async fn record_message(&mut self, size: usize, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_messages += 1;
        metrics.avg_message_size = (metrics.avg_message_size * (metrics.total_messages - 1) as f64
            + size as f64) / metrics.total_messages as f64;
            
        if !success {
            metrics.failed_messages += 1;
        }
    }
    
    async fn get_metrics(&self) -> ChainMetrics {
        self.metrics.read().await.clone()
    }
    
    async fn update_chain_data(&mut self, data: serde_json::Value) {
        let mut metrics = self.metrics.write().await;
        metrics.chain_data = data;
    }
} 