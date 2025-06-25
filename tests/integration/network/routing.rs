use frost_protocol::{
    network::{
        NetworkRouter,
        NetworkConfig,
        PeerInfo,
        MessageRoute,
        RoutingMetrics,
        RoutingError,
    },
    message::{FrostMessage, MessageType},
    state::ChainId,
};

use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_network_routing() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig {
        max_peers: 50,
        connection_timeout: Duration::from_secs(5),
        message_timeout: Duration::from_secs(10),
        max_message_size: 1024 * 1024, // 1MB
        peer_ping_interval: Duration::from_secs(30),
    };

    let router = NetworkRouter::new(config);

    // Add some test peers
    let peer1 = PeerInfo {
        id: "peer1".to_string(),
        address: "127.0.0.1:8001".parse()?,
        supported_chains: vec!["ethereum".to_string(), "cosmos".to_string()],
        is_validator: true,
    };

    let peer2 = PeerInfo {
        id: "peer2".to_string(),
        address: "127.0.0.1:8002".parse()?,
        supported_chains: vec!["cosmos".to_string(), "substrate".to_string()],
        is_validator: false,
    };

    router.add_peer(peer1.clone()).await?;
    router.add_peer(peer2.clone()).await?;

    // Test message routing
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        Some("cosmos".to_string()),
        ChainId::new("ethereum"),
        ChainId::new("cosmos"),
        None,
        None,
        None,
    );

    let routes = router.get_message_routes(&msg).await?;
    assert_eq!(routes.len(), 2); // Both peers support Cosmos
    assert!(routes.iter().any(|r| r.peer_id == "peer1"));
    assert!(routes.iter().any(|r| r.peer_id == "peer2"));

    // Test chain-specific routing
    let eth_msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        Some("ethereum".to_string()),
        ChainId::new("cosmos"),
        ChainId::new("ethereum"),
        None,
        None,
        None,
    );

    let routes = router.get_message_routes(&eth_msg).await?;
    assert_eq!(routes.len(), 1); // Only peer1 supports Ethereum
    assert_eq!(routes[0].peer_id, "peer1");

    // Test validator-only routing
    let validator_msg = FrostMessage::new(
        MessageType::Custom("ValidatorUpdate".into()),
        vec![1, 2, 3],
        "node1".to_string(),
        None,
    );

    let routes = router.get_message_routes(&validator_msg).await?;
    assert_eq!(routes.len(), 1); // Only peer1 is a validator
    assert_eq!(routes[0].peer_id, "peer1");

    // Test peer disconnection
    router.remove_peer("peer1").await?;
    let routes = router.get_message_routes(&eth_msg).await?;
    assert!(routes.is_empty()); // No peers support Ethereum now

    // Test metrics
    let metrics = router.get_routing_metrics().await?;
    assert!(metrics.total_messages > 0);
    assert!(metrics.active_peers == 1);
    assert!(metrics.validator_peers == 0);

    Ok(())
}

#[tokio::test]
async fn test_network_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig {
        max_peers: 2,
        connection_timeout: Duration::from_secs(1),
        message_timeout: Duration::from_secs(2),
        max_message_size: 1024,
        peer_ping_interval: Duration::from_secs(5),
    };

    let router = NetworkRouter::new(config);

    // Test max peers limit
    let peer1 = PeerInfo {
        id: "peer1".to_string(),
        address: "127.0.0.1:8001".parse()?,
        supported_chains: vec!["ethereum".to_string()],
        is_validator: false,
    };

    let peer2 = PeerInfo {
        id: "peer2".to_string(),
        address: "127.0.0.1:8002".parse()?,
        supported_chains: vec!["ethereum".to_string()],
        is_validator: false,
    };

    let peer3 = PeerInfo {
        id: "peer3".to_string(),
        address: "127.0.0.1:8003".parse()?,
        supported_chains: vec!["ethereum".to_string()],
        is_validator: false,
    };

    router.add_peer(peer1).await?;
    router.add_peer(peer2).await?;
    let result = router.add_peer(peer3).await;
    assert!(matches!(result, Err(RoutingError::MaxPeersReached)));

    // Test message size limit
    let large_msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![0; 2048], // Exceeds max_message_size
        "node1".to_string(),
        None,
    );

    let result = router.get_message_routes(&large_msg).await;
    assert!(matches!(result, Err(RoutingError::MessageTooLarge)));

    // Test timeout handling
    router.set_peer_timeout("peer1", Duration::from_secs(0)).await?;
    let msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
    );

    let routes = router.get_message_routes(&msg).await?;
    assert_eq!(routes.len(), 1); // Only peer2 is responsive
    assert_eq!(routes[0].peer_id, "peer2");

    // Test metrics after errors
    let metrics = router.get_routing_metrics().await?;
    assert!(metrics.routing_errors > 0);
    assert!(metrics.timed_out_peers == 1);

    Ok(())
}

#[tokio::test]
async fn test_network_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig {
        max_peers: 100,
        connection_timeout: Duration::from_secs(5),
        message_timeout: Duration::from_secs(10),
        max_message_size: 1024 * 1024,
        peer_ping_interval: Duration::from_secs(30),
    };

    let router = NetworkRouter::new(config);

    // Add many peers
    for i in 0..50 {
        let peer = PeerInfo {
            id: format!("peer{}", i),
            address: format!("127.0.0.1:{}", 8000 + i).parse()?,
            supported_chains: vec!["ethereum".to_string(), "cosmos".to_string()],
            is_validator: i % 5 == 0, // Every 5th peer is a validator
        };
        router.add_peer(peer).await?;
    }

    // Test routing performance with many messages
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let msg = FrostMessage::new_chain_message(
            MessageType::StateTransition,
            vec![1, 2, 3],
            format!("node{}", i),
            Some("ethereum".to_string()),
            ChainId::new("cosmos"),
            ChainId::new("ethereum"),
            None,
            None,
            None,
        );
        let _routes = router.get_message_routes(&msg).await?;
    }
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(1)); // Should be fast

    // Test metrics under load
    let metrics = router.get_routing_metrics().await?;
    assert!(metrics.total_messages >= 1000);
    assert!(metrics.active_peers == 50);
    assert!(metrics.validator_peers == 10);
    assert!(metrics.avg_routing_time < 0.001); // Average routing time in seconds

    // Test concurrent message routing
    let mut handles = vec![];
    for i in 0..10 {
        let router = router.clone();
        let handle = tokio::spawn(async move {
            let msg = FrostMessage::new_chain_message(
                MessageType::StateTransition,
                vec![1, 2, 3],
                format!("node{}", i),
                Some("ethereum".to_string()),
                ChainId::new("cosmos"),
                ChainId::new("ethereum"),
                None,
                None,
                None,
            );
            router.get_message_routes(&msg).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let routes = handle.await??;
        assert!(!routes.is_empty());
    }

    Ok(())
} 