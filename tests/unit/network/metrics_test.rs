use frost_protocol::{
    network::{
        metrics::{NetworkMetrics, MetricsCollector},
        P2PMessage,
        MessageType,
        MessagePriority,
    },
    Result,
};

use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_basic_metrics_collection() {
    let metrics = NetworkMetrics::new();
    
    // Record some basic metrics
    metrics.record_message_received().await;
    metrics.record_message_sent().await;
    metrics.record_peer_connected().await;
    metrics.record_peer_disconnected().await;
    
    let stats = metrics.get_stats().await;
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.peers_connected, 1);
    assert_eq!(stats.peers_disconnected, 1);
}

#[tokio::test]
async fn test_message_type_metrics() {
    let metrics = NetworkMetrics::new();
    
    // Record different message types
    metrics.record_message_by_type(MessageType::Discovery).await;
    metrics.record_message_by_type(MessageType::StateTransition).await;
    metrics.record_message_by_type(MessageType::FinalitySignal).await;
    
    let stats = metrics.get_stats().await;
    assert_eq!(stats.discovery_messages, 1);
    assert_eq!(stats.state_transition_messages, 1);
    assert_eq!(stats.finality_signal_messages, 1);
}

#[tokio::test]
async fn test_latency_metrics() {
    let metrics = NetworkMetrics::new();
    
    // Record message latencies
    metrics.record_message_latency(Duration::from_millis(100)).await;
    metrics.record_message_latency(Duration::from_millis(200)).await;
    metrics.record_message_latency(Duration::from_millis(300)).await;
    
    let stats = metrics.get_stats().await;
    assert!(stats.avg_message_latency > Duration::from_millis(0));
    assert!(stats.max_message_latency >= Duration::from_millis(300));
}

#[tokio::test]
async fn test_bandwidth_metrics() {
    let metrics = NetworkMetrics::new();
    
    // Record bandwidth usage
    metrics.record_bytes_sent(1000).await;
    metrics.record_bytes_received(2000).await;
    
    let stats = metrics.get_stats().await;
    assert_eq!(stats.bytes_sent, 1000);
    assert_eq!(stats.bytes_received, 2000);
}

#[tokio::test]
async fn test_error_metrics() {
    let metrics = NetworkMetrics::new();
    
    // Record different types of errors
    metrics.record_message_validation_error().await;
    metrics.record_peer_connection_error().await;
    metrics.record_message_processing_error().await;
    
    let stats = metrics.get_stats().await;
    assert_eq!(stats.validation_errors, 1);
    assert_eq!(stats.connection_errors, 1);
    assert_eq!(stats.processing_errors, 1);
}

#[tokio::test]
async fn test_metrics_reset() {
    let metrics = NetworkMetrics::new();
    
    // Record some metrics
    metrics.record_message_received().await;
    metrics.record_bytes_sent(1000).await;
    metrics.record_message_validation_error().await;
    
    // Reset metrics
    metrics.reset().await;
    
    let stats = metrics.get_stats().await;
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.validation_errors, 0);
}

#[tokio::test]
async fn test_metrics_sampling() {
    let metrics = NetworkMetrics::new();
    let collector = MetricsCollector::new(metrics.clone());
    
    // Generate some traffic
    for _ in 0..10 {
        metrics.record_message_received().await;
        metrics.record_bytes_sent(100).await;
        time::sleep(Duration::from_millis(100)).await;
    }
    
    let samples = collector.get_samples(Duration::from_secs(1)).await;
    assert!(!samples.is_empty(), "Should have collected metrics samples");
    
    // Verify sample data
    for sample in samples {
        assert!(sample.timestamp > 0);
        assert!(sample.messages_received <= 10);
        assert!(sample.bytes_sent <= 1000);
    }
}

#[tokio::test]
async fn test_peer_metrics() {
    let metrics = NetworkMetrics::new();
    
    // Record peer-specific metrics
    metrics.record_peer_message_received("peer1").await;
    metrics.record_peer_message_sent("peer1").await;
    metrics.record_peer_bytes_sent("peer1", 500).await;
    metrics.record_peer_bytes_received("peer1", 1000).await;
    
    let peer_stats = metrics.get_peer_stats("peer1").await;
    assert_eq!(peer_stats.messages_received, 1);
    assert_eq!(peer_stats.messages_sent, 1);
    assert_eq!(peer_stats.bytes_sent, 500);
    assert_eq!(peer_stats.bytes_received, 1000);
} 