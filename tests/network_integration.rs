use frost_protocol::network::{
    NetworkProtocol, Transport, Peer, PeerInfo, ConnectionPool,
    RetryPolicy, TelemetryManager, NetworkEvent, BackpressureController,
    CircuitBreaker
};
use frost_protocol::Result;
use std::time::Duration;
use tokio;

mod common;
use common::MockTransport;

#[tokio::test]
async fn test_network_component_integration() -> Result<()> {
    let transport = MockTransport::new();
    let pool = common::create_connection_pool(transport.clone());
    let retry = common::create_retry_policy();
    let telemetry = common::create_telemetry_manager();
    let backpressure = common::create_backpressure_controller();
    let circuit_breaker = common::create_circuit_breaker();
    
    // Test full message flow with all components
    let result = common::simulate_message_flow(
        &pool,
        &retry,
        &telemetry,
        &backpressure,
        &circuit_breaker
    ).await;
    
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_connection_pool_under_load() -> Result<()> {
    let transport = MockTransport::new();
    let pool = common::create_connection_pool(transport.clone());
    
    // Simulate multiple concurrent connection requests
    let mut handles = Vec::new();
    for _ in 0..100 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let peer = common::create_test_peer();
            pool.acquire(&peer).await
        });
        handles.push(handle);
    }
    
    // Verify all connections were handled correctly
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_retry_with_backpressure() -> Result<()> {
    let retry = common::create_retry_policy();
    let backpressure = common::create_backpressure_controller();
    
    // Test retry behavior under different pressure levels
    for _ in 0..3 {
        let permit = backpressure.acquire().await?;
        let result = common::simulate_failing_operation(&retry).await;
        assert!(result.is_err()); // Should fail but be retried
        drop(permit);
    }
    
    assert_eq!(backpressure.pressure_level().to_string(), "High");
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_integration() -> Result<()> {
    let circuit_breaker = common::create_circuit_breaker();
    
    // Test circuit breaker state transitions
    for _ in 0..5 {
        let result = common::simulate_failing_operation_with_circuit_breaker(
            &circuit_breaker
        ).await;
        assert!(result.is_err());
    }
    
    assert_eq!(circuit_breaker.current_state().to_string(), "Open");
    
    // Wait for reset timeout
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Should be in half-open state
    let result = common::simulate_successful_operation_with_circuit_breaker(
        &circuit_breaker
    ).await;
    assert!(result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_telemetry_integration() -> Result<()> {
    let telemetry = common::create_telemetry_manager();
    
    // Record various network events
    telemetry.record_event(NetworkEvent::Connection {
        peer: common::create_test_peer(),
        status: common::ConnectionStatus::Established,
        timestamp: std::time::SystemTime::now(),
    }).await?;
    
    // Verify metrics
    let data = telemetry.get_telemetry_data();
    assert_eq!(data.metrics.connections.total_connections, 1);
    assert_eq!(data.events.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_integration() -> Result<()> {
    let transport = MockTransport::new();
    let pool = common::create_connection_pool(transport.clone());
    let retry = common::create_retry_policy();
    let telemetry = common::create_telemetry_manager();
    let circuit_breaker = common::create_circuit_breaker();
    
    // Test error propagation through components
    let result = common::simulate_error_scenario(
        &pool,
        &retry,
        &telemetry,
        &circuit_breaker
    ).await;
    
    assert!(result.is_err());
    let data = telemetry.get_telemetry_data();
    assert!(data.metrics.errors.total_errors > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    let transport = MockTransport::new();
    let pool = common::create_connection_pool(transport.clone());
    let backpressure = common::create_backpressure_controller();
    let telemetry = common::create_telemetry_manager();
    
    // Simulate high load scenario
    let start = std::time::Instant::now();
    let mut handles = Vec::new();
    
    for _ in 0..1000 {
        let pool = pool.clone();
        let backpressure = backpressure.clone();
        let telemetry = telemetry.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = backpressure.acquire().await?;
            let peer = common::create_test_peer();
            let conn = pool.acquire(&peer).await?;
            telemetry.record_event(NetworkEvent::Message {
                message_id: uuid::Uuid::new_v4(),
                peer,
                size: 1024,
                direction: common::MessageDirection::Outbound,
                timestamp: std::time::SystemTime::now(),
            }).await?;
            pool.release(conn).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await??;
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(10)); // Should complete within 10 seconds
    
    Ok(())
} 