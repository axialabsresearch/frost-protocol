/*!
# Metrics Module

This module provides comprehensive metrics collection and monitoring functionality for
the FROST protocol. It implements chain-specific metrics, generic collectors, and
utility functions for metrics processing.

## Core Components

### Metrics Collection
- Chain metrics
- Performance metrics
- Resource usage
- System health

### Metrics Processing
- Data aggregation
- Moving averages
- Time conversion
- Statistical analysis

### Chain Monitoring
- Chain-specific metrics
- Cross-chain metrics
- Performance tracking
- Health monitoring

## Architecture

The metrics system consists of several key components:

1. **Metrics Collector**
   ```rust
   pub trait MetricsCollector: Send + Sync {
       async fn get_metrics(&self) -> Value;
       async fn reset(&mut self);
   }
   ```
   - Data collection
   - Metric storage
   - Reset capability
   - JSON formatting

2. **Chain Metrics**
   ```rust
   pub struct ChainMetrics {
       block_metrics: BlockMetrics,
       performance_metrics: PerformanceMetrics,
       resource_metrics: ResourceMetrics,
       // ...
   }
   ```
   - Block tracking
   - Performance data
   - Resource usage
   - Health status

3. **Metrics Utils**
   ```rust
   pub struct MetricsUtils {
       pub fn update_average(current_avg: f64, new_value: f64, total: u64) -> f64;
       pub fn duration_to_ms(duration: Duration) -> f64;
   }
   ```
   - Average calculation
   - Time conversion
   - Data processing
   - Utility functions

## Features

### Data Collection
- Real-time metrics
- Chain monitoring
- Resource tracking
- Health checks

### Data Processing
- Moving averages
- Time conversion
- Data aggregation
- Statistical analysis

### Chain Monitoring
- Block metrics
- Performance data
- Resource usage
- Health status

### Utility Functions
- Average calculation
- Time conversion
- Data formatting
- Statistical tools

## Best Practices

1. **Metrics Collection**
   - Regular updates
   - Efficient storage
   - Data validation
   - Resource limits

2. **Data Processing**
   - Accurate calculations
   - Proper averaging
   - Time handling
   - Resource usage

3. **Chain Monitoring**
   - Regular updates
   - Health checks
   - Resource tracking
   - Alert thresholds

4. **Utility Usage**
   - Proper calculation
   - Time conversion
   - Data formatting
   - Resource efficiency

## Integration

The metrics system integrates with:
1. Chain management
2. Performance monitoring
3. Resource tracking
4. Health checking
*/

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
    GenericChainMetrics,
    ChainMetricsAggregator,
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