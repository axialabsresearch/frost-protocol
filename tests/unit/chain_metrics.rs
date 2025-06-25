use std::time::Duration;
use frost_protocol::metrics::chain_metrics::{
    ChainMetricsCollector,
    ChainMetrics,
    EthereumMetrics,
    CosmosMetrics,
};

#[tokio::test]
async fn test_ethereum_metrics() {
    let mut collector = EthereumMetrics::new();
    
    // Record block metrics
    collector.record_block(
        Duration::from_secs(15),
        Duration::from_secs(180),
    ).await;
    
    collector.record_block(
        Duration::from_secs(13),
        Duration::from_secs(150),
    ).await;
    
    // Record message metrics
    collector.record_message(1024, true).await;
    collector.record_message(2048, false).await;
    
    // Update chain data
    collector.update_chain_data(serde_json::json!({
        "gas_used": 500000u64,
        "avg_gas_price": 50000000000u64,
        "total_value_transferred": "1000000000000000000",
    })).await;
    
    // Check metrics
    let metrics = collector.get_metrics().await;
    assert_eq!(metrics.chain_id, "ethereum");
    assert_eq!(metrics.total_blocks, 2);
    assert_eq!(metrics.total_messages, 2);
    assert_eq!(metrics.failed_messages, 1);
    assert!((metrics.avg_block_time - 14.0).abs() < f64::EPSILON);
    assert!((metrics.avg_finality_time - 165.0).abs() < f64::EPSILON);
    assert!((metrics.avg_message_size - 1536.0).abs() < f64::EPSILON);
    
    let chain_data = metrics.chain_data.as_object().unwrap();
    assert_eq!(chain_data["gas_used"].as_u64().unwrap(), 500000u64);
    assert_eq!(chain_data["avg_gas_price"].as_u64().unwrap(), 50000000000u64);
    assert_eq!(chain_data["total_value_transferred"].as_str().unwrap(), "1000000000000000000");
}

#[tokio::test]
async fn test_cosmos_metrics() {
    let mut collector = CosmosMetrics::new();
    
    // Record block metrics
    collector.record_block(
        Duration::from_secs(6),
        Duration::from_secs(12),
    ).await;
    
    collector.record_block(
        Duration::from_secs(7),
        Duration::from_secs(14),
    ).await;
    
    // Record message metrics
    collector.record_message(256, true).await;
    collector.record_message(512, false).await;
    
    // Update chain data
    collector.update_chain_data(serde_json::json!({
        "gas_used": 100000u64,
        "avg_block_size": 20000u64,
        "total_fees_collected": "5000000",
    })).await;
    
    // Check metrics
    let metrics = collector.get_metrics().await;
    assert_eq!(metrics.chain_id, "cosmos");
    assert_eq!(metrics.total_blocks, 2);
    assert_eq!(metrics.total_messages, 2);
    assert_eq!(metrics.failed_messages, 1);
    assert!((metrics.avg_block_time - 6.5).abs() < f64::EPSILON);
    assert!((metrics.avg_finality_time - 13.0).abs() < f64::EPSILON);
    assert!((metrics.avg_message_size - 384.0).abs() < f64::EPSILON);
    
    let chain_data = metrics.chain_data.as_object().unwrap();
    assert_eq!(chain_data["gas_used"].as_u64().unwrap(), 100000u64);
    assert_eq!(chain_data["avg_block_size"].as_u64().unwrap(), 20000u64);
    assert_eq!(chain_data["total_fees_collected"].as_str().unwrap(), "5000000");
} 