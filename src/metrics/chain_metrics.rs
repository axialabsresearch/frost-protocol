#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use serde_json::json;

use super::MetricsUtils;

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
            chain_data: json!({}),
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

/// Generic chain metrics implementation
pub struct GenericChainMetrics {
    metrics: Arc<RwLock<ChainMetrics>>,
}

impl GenericChainMetrics {
    pub fn new(chain_id: impl Into<String>) -> Self {
        let metrics = ChainMetrics {
            chain_id: chain_id.into(),
            ..Default::default()
        };
        
        Self {
            metrics: Arc::new(RwLock::new(metrics)),
        }
    }

    /// Create a new metrics collector with initial chain data
    pub fn with_chain_data(chain_id: impl Into<String>, chain_data: serde_json::Value) -> Self {
        let metrics = ChainMetrics {
            chain_id: chain_id.into(),
            chain_data,
            ..Default::default()
        };
        
        Self {
            metrics: Arc::new(RwLock::new(metrics)),
        }
    }
}

#[async_trait::async_trait]
impl ChainMetricsCollector for GenericChainMetrics {
    async fn record_block(&mut self, block_time: Duration, finality_time: Duration) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_blocks += 1;
        metrics.avg_block_time = MetricsUtils::update_average(
            metrics.avg_block_time,
            block_time.as_secs_f64(),
            metrics.total_blocks
        );
        metrics.avg_finality_time = MetricsUtils::update_average(
            metrics.avg_finality_time,
            finality_time.as_secs_f64(),
            metrics.total_blocks
        );
    }
    
    async fn record_message(&mut self, size: usize, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_messages += 1;
        metrics.avg_message_size = MetricsUtils::update_average(
            metrics.avg_message_size,
            size as f64,
            metrics.total_messages
        );
            
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

/// Metrics aggregator for multiple chains
pub struct ChainMetricsAggregator {
    collectors: Arc<RwLock<HashMap<String, Box<dyn ChainMetricsCollector>>>>,
}

impl ChainMetricsAggregator {
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new chain metrics collector
    pub async fn register_collector(
        &self,
        chain_id: impl Into<String>,
        collector: Box<dyn ChainMetricsCollector>,
    ) {
        let mut collectors = self.collectors.write().await;
        collectors.insert(chain_id.into(), collector);
    }

    /// Get metrics for a specific chain
    pub async fn get_chain_metrics(&self, chain_id: &str) -> Option<ChainMetrics> {
        let collectors = self.collectors.read().await;
        if let Some(collector) = collectors.get(chain_id) {
            Some(collector.get_metrics().await)
        } else {
            None
        }
    }

    /// Get metrics for all chains
    pub async fn get_all_metrics(&self) -> HashMap<String, ChainMetrics> {
        let collectors = self.collectors.read().await;
        let mut all_metrics = HashMap::new();
        
        for (chain_id, collector) in collectors.iter() {
            all_metrics.insert(chain_id.clone(), collector.get_metrics().await);
        }
        
        all_metrics
    }
} 