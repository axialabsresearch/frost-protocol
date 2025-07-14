use frost_protocol::{
    network::{
        discovery::{PeerDiscovery, DiscoveryConfig, KademliaPeerDiscovery, PeerCriteria},
        p2p::NodeIdentity,
    },
};

use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_discovery_service_startup() {
    // Create a config with a local bootstrap node
    let config = DiscoveryConfig {
        bootstrap_nodes: vec![
            "/ip4/127.0.0.1/tcp/4001".to_string(),
        ],
        replication_interval: Duration::from_secs(300),
        record_ttl: Duration::from_secs(7200),
        query_timeout: Duration::from_secs(60),
        max_peers: 50,
        min_peers: 10,
        enable_provider_records: true,
        provider_announce_interval: Duration::from_secs(1800),
    };
    
    let (event_tx, _) = mpsc::channel(100);
    let identity = NodeIdentity::new();
    let mut discovery = KademliaPeerDiscovery::new(identity, config.clone(), event_tx);
    
    // Initialize discovery service
    let init_result = discovery.init(config).await;
    assert!(init_result.is_ok(), "Failed to initialize discovery service: {:?}", init_result.err());
    
    // Start discovery service
    let start_result = discovery.start_discovery().await;
    assert!(start_result.is_ok(), "Failed to start discovery service: {:?}", start_result.err());
    
    // Verify metrics
    let metrics = discovery.metrics();
    assert_eq!(metrics.discovered_peers, 0);
    assert_eq!(metrics.active_discoveries, 0);
    
    // Stop discovery service
    let stop_result = discovery.stop_discovery().await;
    assert!(stop_result.is_ok(), "Failed to stop discovery service: {:?}", stop_result.err());
}

#[tokio::test]
async fn test_peer_discovery() {
    let config = DiscoveryConfig {
        bootstrap_nodes: vec![],
        replication_interval: Duration::from_secs(300),
        record_ttl: Duration::from_secs(7200),
        query_timeout: Duration::from_secs(60),
        max_peers: 50,
        min_peers: 10,
        enable_provider_records: true,
        provider_announce_interval: Duration::from_secs(1800),
    };
    
    let (event_tx, _) = mpsc::channel(100);
    let identity = NodeIdentity::new();
    let discovery = KademliaPeerDiscovery::new(identity, config.clone(), event_tx);
    
    let criteria = PeerCriteria {
        node_types: vec!["validator".to_string()],
        min_version: None,
        required_features: vec![],
        chain_ids: vec![],
        max_latency: None,
        geographic_region: None,
    };
    
    let peers = discovery.find_peers(criteria).await.unwrap();
    assert!(peers.is_empty(), "Should start with no peers");
}

#[tokio::test]
async fn test_discovery_metrics() {
    let config = DiscoveryConfig {
        bootstrap_nodes: vec![],
        replication_interval: Duration::from_secs(300),
        record_ttl: Duration::from_secs(7200),
        query_timeout: Duration::from_secs(60),
        max_peers: 50,
        min_peers: 10,
        enable_provider_records: true,
        provider_announce_interval: Duration::from_secs(1800),
    };
    
    let (event_tx, _) = mpsc::channel(100);
    let identity = NodeIdentity::new();
    let discovery = KademliaPeerDiscovery::new(identity, config.clone(), event_tx);
    
    let metrics = discovery.metrics();
    assert_eq!(metrics.discovered_peers, 0);
    assert_eq!(metrics.active_discoveries, 0);
} 