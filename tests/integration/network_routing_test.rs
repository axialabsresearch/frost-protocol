use crate::common::{TestNode, create_test_message, create_test_routes, wait_for_condition};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_network_startup_shutdown() {
    let nodes: Vec<_> = vec!["node1", "node2", "node3"]
        .into_iter()
        .map(|id| TestNode::new(id))
        .collect::<futures::future::join_all>()
        .await;

    // Start all nodes
    for node in &nodes {
        assert!(node.start().await.is_ok());
    }

    // Verify nodes are running
    for node in &nodes {
        let peers = node.network.lock().await.get_peers().await.unwrap();
        assert!(peers.is_empty()); // Initially no peers
    }

    // Stop all nodes
    for node in &nodes {
        assert!(node.stop().await.is_ok());
    }
}

#[tokio::test]
async fn test_message_routing() {
    let node1 = TestNode::new("node1").await;
    let node2 = TestNode::new("node2").await;
    let node3 = TestNode::new("node3").await;

    // Start nodes
    node1.start().await.unwrap();
    node2.start().await.unwrap();
    node3.start().await.unwrap();

    // Setup routing tables
    let routes = create_test_routes(&["node1", "node2", "node3"]);
    node1.router.lock().await.update_routes(routes.clone()).await.unwrap();
    node2.router.lock().await.update_routes(routes.clone()).await.unwrap();
    node3.router.lock().await.update_routes(routes).await.unwrap();

    // Send message from node1 to node3
    let message = create_test_message("node1", Some("node3"));
    node1.router.lock().await.route(message.clone()).await.unwrap();

    // Verify message routing
    let sent_messages = Arc::new(Mutex::new(Vec::new()));
    let sent_clone = sent_messages.clone();

    // Mock message receipt
    tokio::spawn(async move {
        let mut received = false;
        while !received {
            if let Ok(peers) = node3.network.lock().await.get_peers().await {
                if !peers.is_empty() {
                    sent_clone.lock().await.push(message.clone());
                    received = true;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Wait for message to be routed
    let received = wait_for_condition(|| {
        sent_messages.try_lock().map(|m| !m.is_empty()).unwrap_or(false)
    }, 5000).await;

    assert!(received, "Message should be routed within timeout");

    // Stop nodes
    node1.stop().await.unwrap();
    node2.stop().await.unwrap();
    node3.stop().await.unwrap();
}

#[tokio::test]
async fn test_network_metrics() {
    let node = TestNode::new("metrics_test").await;
    node.start().await.unwrap();

    // Send some messages
    let message = create_test_message("metrics_test", None);
    for _ in 0..5 {
        node.network.lock().await.broadcast(message.clone()).await.unwrap();
    }

    // Verify metrics
    let metrics = node.network.lock().await.get_metrics();
    assert!(metrics.messages_sent > 0);

    node.stop().await.unwrap();
}

#[tokio::test]
async fn test_routing_updates() {
    let node = TestNode::new("route_test").await;
    node.start().await.unwrap();

    // Test route updates
    let mut routes = create_test_routes(&["route_test", "node2", "node3"]);
    assert!(node.router.lock().await.update_routes(routes.clone()).await.is_ok());

    // Verify routes
    let stored_routes = node.router.lock().await.get_routes().await.unwrap();
    assert_eq!(stored_routes.len(), routes.len());

    // Test route limits
    routes.extend((0..1000).map(|i| (format!("node{}", i), "target".to_string())));
    assert!(node.router.lock().await.update_routes(routes).await.is_err());

    node.stop().await.unwrap();
}

#[tokio::test]
async fn test_network_broadcast() {
    let nodes: Vec<_> = vec!["broadcast1", "broadcast2", "broadcast3"]
        .into_iter()
        .map(|id| TestNode::new(id))
        .collect::<futures::future::join_all>()
        .await;

    // Start all nodes
    for node in &nodes {
        node.start().await.unwrap();
    }

    // Broadcast message from first node
    let message = create_test_message("broadcast1", None);
    nodes[0].network.lock().await.broadcast(message).await.unwrap();

    // Verify metrics after broadcast
    let metrics = nodes[0].network.lock().await.get_metrics();
    assert!(metrics.messages_sent > 0);

    // Stop all nodes
    for node in &nodes {
        node.stop().await.unwrap();
    }
} 