use async_trait::async_trait;
use frost_protocol::network::*;
use frost_protocol::Result;
use std::time::Duration;
use std::sync::Arc;

// Mock Transport Implementation
#[derive(Clone)]
pub struct MockTransport {
    state: Arc<parking_lot::RwLock<TransportState>>,
}

#[derive(Default)]
struct TransportState {
    connected: bool,
    error_count: u32,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            state: Arc::new(parking_lot::RwLock::new(TransportState::default())),
        }
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn init(&mut self, _config: TransportConfig) -> Result<()> {
        Ok(())
    }

    async fn connect(&mut self, _address: &str) -> Result<Peer> {
        let mut state = self.state.write();
        state.connected = true;
        Ok(create_test_peer())
    }

    async fn disconnect(&mut self, _peer: &Peer) -> Result<()> {
        let mut state = self.state.write();
        state.connected = false;
        Ok(())
    }

    async fn send_data(&self, _peer: &Peer, data: &[u8]) -> Result<usize> {
        Ok(data.len())
    }

    async fn receive_data(&self, _peer: &Peer) -> Result<Vec<u8>> {
        Ok(vec![1, 2, 3, 4])
    }

    async fn is_connected(&self, _peer: &Peer) -> bool {
        self.state.read().connected
    }

    fn metrics(&self) -> TransportMetrics {
        TransportMetrics::default()
    }
}

// Test Utilities
pub fn create_test_peer() -> Peer {
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

pub fn create_connection_pool(transport: MockTransport) -> impl ConnectionPool {
    DefaultConnectionPool::new(
        PoolConfig {
            min_idle: 5,
            max_size: 20,
            max_lifetime: Duration::from_secs(3600),
            idle_timeout: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(30),
            validation_interval: Duration::from_secs(60),
        },
        transport,
    )
}

pub fn create_retry_policy() -> impl RetryPolicy {
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

pub fn create_circuit_breaker() -> impl CircuitBreaker {
    DefaultCircuitBreaker::new(CircuitConfig {
        failure_threshold: 5,
        success_threshold: 2,
        reset_timeout: Duration::from_secs(5),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    })
}

pub fn create_backpressure_controller() -> impl BackpressureController {
    DefaultBackpressureController::new(BackpressureConfig {
        max_concurrent_requests: 100,
        max_queue_size: 1000,
        pressure_threshold: 0.8,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    })
}

pub fn create_telemetry_manager() -> impl TelemetryManager {
    let tracer = opentelemetry::trace::Tracer::default();
    DefaultTelemetryManager::new(tracer)
}

// Test Scenarios
pub async fn simulate_message_flow(
    pool: &impl ConnectionPool,
    retry: &impl RetryPolicy,
    telemetry: &impl TelemetryManager,
    backpressure: &impl BackpressureController,
    circuit_breaker: &impl CircuitBreaker,
) -> Result<()> {
    let peer = create_test_peer();
    
    // Get backpressure permit
    let _permit = backpressure.acquire().await?;
    
    // Check circuit breaker
    if !circuit_breaker.pre_execute().await? {
        return Err(NetworkError::ConnectionFailed("Circuit breaker open".into()).into());
    }
    
    // Start telemetry span
    let _span = telemetry.start_span("message_flow").await?;
    
    // Acquire connection with retry
    let conn = with_retry(|| pool.acquire(&peer), retry).await?;
    
    // Record success
    telemetry.record_event(NetworkEvent::Message {
        message_id: uuid::Uuid::new_v4(),
        peer: peer.clone(),
        size: 1024,
        direction: MessageDirection::Outbound,
        timestamp: std::time::SystemTime::now(),
    }).await?;
    
    // Release connection
    pool.release(conn).await?;
    
    Ok(())
}

pub async fn simulate_failing_operation(retry: &impl RetryPolicy) -> Result<()> {
    with_retry(|| {
        Err(NetworkError::ConnectionFailed("Simulated failure".into()).into())
    }, retry).await
}

pub async fn simulate_failing_operation_with_circuit_breaker(
    circuit_breaker: &impl CircuitBreaker,
) -> Result<()> {
    if !circuit_breaker.pre_execute().await? {
        return Err(NetworkError::ConnectionFailed("Circuit breaker open".into()).into());
    }
    
    let result = Err(NetworkError::ConnectionFailed("Simulated failure".into()).into());
    circuit_breaker.post_execute(false).await?;
    result
}

pub async fn simulate_successful_operation_with_circuit_breaker(
    circuit_breaker: &impl CircuitBreaker,
) -> Result<()> {
    if !circuit_breaker.pre_execute().await? {
        return Err(NetworkError::ConnectionFailed("Circuit breaker open".into()).into());
    }
    
    circuit_breaker.post_execute(true).await?;
    Ok(())
}

pub async fn simulate_error_scenario(
    pool: &impl ConnectionPool,
    retry: &impl RetryPolicy,
    telemetry: &impl TelemetryManager,
    circuit_breaker: &impl CircuitBreaker,
) -> Result<()> {
    let peer = create_test_peer();
    
    // Start telemetry span
    let _span = telemetry.start_span("error_scenario").await?;
    
    // Simulate failure with retry and circuit breaker
    let result = with_retry(|| {
        if !circuit_breaker.pre_execute().block_on()? {
            return Err(NetworkError::ConnectionFailed("Circuit breaker open".into()).into());
        }
        
        let result = pool.acquire(&peer).block_on();
        circuit_breaker.post_execute(result.is_ok()).block_on()?;
        result
    }, retry).await;
    
    // Record error
    if let Err(e) = &result {
        telemetry.record_event(NetworkEvent::Error {
            error: NetworkError::ConnectionFailed(e.to_string()),
            context: "error_scenario".into(),
            timestamp: std::time::SystemTime::now(),
        }).await?;
    }
    
    result
} 