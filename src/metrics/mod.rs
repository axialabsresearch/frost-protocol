#![allow(unused_imports)]
#![allow(unused_variables)]

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub mod chain_metrics;

// Re-export chain metrics types
pub use chain_metrics::{
    ChainMetrics,
    ChainMetricsCollector,
    EthereumMetrics,
    CosmosMetrics,
};

/// Basic metrics trait for collecting component-specific metrics
#[async_trait::async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Get current metrics as JSON value
    async fn get_metrics(&self) -> Value;
    
    /// Reset metrics to default values
    async fn reset(&mut self);
}

/// Common metrics collection utilities
pub struct MetricsUtils;

impl MetricsUtils {
    /// Calculate exponential moving average
    pub fn update_average(current_avg: f64, new_value: f64, total: u64) -> f64 {
        (current_avg * (total - 1) as f64 + new_value) / total as f64
    }
    
    /// Convert duration to milliseconds
    pub fn duration_to_ms(duration: Duration) -> f64 {
        duration.as_secs_f64() * 1000.0
    }
} 