use frost_protocol::network::*;
use std::time::Duration;

/// This example demonstrates how to use the network layer components together
#[tokio::main]
async fn main() -> frost_protocol::Result<()> {
    // Initialize components
    let transport = create_transport().await?;
    let pool = create_connection_pool(transport).await?;
    let retry = create_retry_policy();
    let telemetry = create_telemetry_manager().await?;
    let backpressure = create_backpressure_controller();
    let circuit_breaker = create_circuit_breaker();

    // Example: Send message with all safety mechanisms
    send_message_safely(
        &pool,
        &retry,
        &telemetry,
        &backpressure,
        &circuit_breaker,
    ).await?;

    Ok(())
}

/// Create and configure transport
async fn create_transport() -> frost_protocol::Result<impl Transport> {
    let mut transport = TCPTransport::new();
    transport.init(TransportConfig {
        protocol: TransportProtocol::TCP {
            port: 8000,
            keep_alive: true,
        },
        encryption: EncryptionConfig {
            enabled: true,
            algorithm: "AES-256-GCM".into(),
            key_size: 256,
        },
        compression: CompressionConfig {
            enabled: true,
            algorithm: "snappy".into(),
            level: 6,
        },
        timeout: Duration::from_secs(30),
        buffer_size: 65536,
    }).await?;
    
    Ok(transport)
}

/// Create and configure connection pool
async fn create_connection_pool(
    transport: impl Transport,
) -> frost_protocol::Result<impl ConnectionPool> {
    let pool = DefaultConnectionPool::new(
        PoolConfig {
            min_idle_per_peer: 5,
            max_per_peer: 20,
            max_lifetime: Duration::from_secs(3600),
            idle_timeout: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(30),
            validation_interval: Duration::from_secs(60),
        },
        transport,
    );
    
    Ok(pool)
}

/// Create retry policy
fn create_retry_policy() -> impl RetryPolicy {
    DefaultRetryPolicy::new(RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_factor: 2.0,
        jitter_factor: 0.1,
        retry_budget: RetryBudget {
            ttl: Duration::from_secs(60),
            min_retries: 10,
            retry_ratio: 0.1,
        },
    })
}

/// Create telemetry manager
async fn create_telemetry_manager() -> frost_protocol::Result<impl TelemetryManager> {
    // Configure OpenTelemetry
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("frost-protocol")
        .install_simple()?;
    
    Ok(DefaultTelemetryManager::new(tracer))
}

/// Create backpressure controller
fn create_backpressure_controller() -> impl BackpressureController {
    DefaultBackpressureController::new(BackpressureConfig {
        max_concurrent_requests: 100,
        max_queue_size: 1000,
        pressure_threshold: 0.8,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    })
}

/// Create circuit breaker
fn create_circuit_breaker() -> impl CircuitBreaker {
    DefaultCircuitBreaker::new(CircuitConfig {
        failure_threshold: 5,
        success_threshold: 2,
        reset_timeout: Duration::from_secs(5),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    })
}

/// Example: Send message with all safety mechanisms
async fn send_message_safely(
    pool: &impl ConnectionPool,
    retry: &impl RetryPolicy,
    telemetry: &impl TelemetryManager,
    backpressure: &impl BackpressureController,
    circuit_breaker: &impl CircuitBreaker,
) -> frost_protocol::Result<()> {
    // Start telemetry span
    let span = telemetry.start_span("send_message").await?;
    
    // Get backpressure permit
    let permit = backpressure.acquire().await?;
    
    // Check circuit breaker
    if !circuit_breaker.pre_execute().await? {
        telemetry.record_event(NetworkEvent::Error {
            error: NetworkError::ConnectionFailed("Circuit breaker open".into()),
            context: "send_message".into(),
            timestamp: std::time::SystemTime::now(),
        }).await?;
        return Err(NetworkError::ConnectionFailed("Circuit breaker open".into()).into());
    }
    
    // Create test message
    let message = FrostMessage::new(
        ChainId(1),
        ChainId(2),
        StateTransition::default(),
        MessageType::StateTransition,
    );
    
    // Get peer connection with retry
    let result = with_retry(|| async {
        let peer = create_test_peer();
        let conn = pool.acquire(&peer).await?;
        
        // Send message
        // ... implement message sending logic ...
        
        pool.release(conn).await?;
        Ok(())
    }, retry).await;
    
    // Update circuit breaker
    circuit_breaker.post_execute(result.is_ok()).await?;
    
    // Record telemetry
    if let Err(e) = &result {
        telemetry.record_event(NetworkEvent::Error {
            error: NetworkError::ConnectionFailed(e.to_string()),
            context: "send_message".into(),
            timestamp: std::time::SystemTime::now(),
        }).await?;
    } else {
        telemetry.record_event(NetworkEvent::Message {
            message_id: message.id,
            peer: create_test_peer(),
            size: 1024, // example size
            direction: MessageDirection::Outbound,
            timestamp: std::time::SystemTime::now(),
        }).await?;
    }
    
    // Release backpressure permit
    drop(permit);
    
    result
}

/// Helper function to create a test peer
fn create_test_peer() -> Peer {
    Peer {
        id: uuid::Uuid::new_v4(),
        info: PeerInfo {
            address: "127.0.0.1:8000".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_features: vec!["basic".to_string()],
            chain_ids: vec![1],
            node_type: NodeType::Validator,
        },
        state: PeerState::Connected,
    }
} 