use frost_protocol::network::circuit_breaker::{CircuitBreaker, CircuitConfig, DefaultCircuitBreaker, CircuitState};

use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_initialization() {
    let config = CircuitConfig {
        failure_threshold: 3,
        success_threshold: 1,
        reset_timeout: Duration::from_secs(5),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    };
    
    let breaker = DefaultCircuitBreaker::new(config);
    assert_eq!(breaker.current_state(), CircuitState::Closed, "Circuit breaker should start closed");
}

#[tokio::test]
async fn test_circuit_breaker_trip() {
    let config = CircuitConfig {
        failure_threshold: 3,
        success_threshold: 1,
        reset_timeout: Duration::from_secs(5),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    };
    
    let breaker = DefaultCircuitBreaker::new(config);
    
    // Record failures until threshold
    for _ in 0..3 {
        breaker.post_execute(false).await.unwrap();
        // Give time for state transition
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    assert_eq!(breaker.current_state(), CircuitState::Open, "Circuit breaker should be open after threshold failures");
}

#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = CircuitConfig {
        failure_threshold: 2,
        success_threshold: 1,
        reset_timeout: Duration::from_secs(1),
        half_open_timeout: Duration::from_millis(500),
        window_size: Duration::from_secs(60),
    };
    
    let breaker = DefaultCircuitBreaker::new(config);
    
    // Trip the breaker
    for _ in 0..2 {
    breaker.post_execute(false).await.unwrap();
        // Give time for state transition
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(breaker.current_state(), CircuitState::Open);
    
    // Wait for reset timeout
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Should be in half-open state
    assert!(breaker.pre_execute().await.unwrap(), "Should allow request in half-open state");
    assert_eq!(breaker.current_state(), CircuitState::HalfOpen, "Should be in half-open state");
}

#[tokio::test]
async fn test_circuit_breaker_half_open() {
    let config = CircuitConfig {
        failure_threshold: 2,
        success_threshold: 1,
        reset_timeout: Duration::from_secs(2),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    };
    
    let breaker = DefaultCircuitBreaker::new(config);
    
    // Trip the breaker
    for _ in 0..2 {
    breaker.post_execute(false).await.unwrap();
        // Give time for state transition
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(breaker.current_state(), CircuitState::Open);
    
    // Wait for reset timeout
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Should be in half-open state
    assert!(breaker.pre_execute().await.unwrap(), "Should allow request in half-open state");
    assert_eq!(breaker.current_state(), CircuitState::HalfOpen, "Should be in half-open state");
    
    // Record success in half-open state
    breaker.post_execute(true).await.unwrap();
    // Give time for state transition
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    assert_eq!(breaker.current_state(), CircuitState::Closed, "Circuit should close after success in half-open state");
}

#[tokio::test]
async fn test_circuit_breaker_metrics() {
    let config = CircuitConfig {
        failure_threshold: 3,
        success_threshold: 1,
        reset_timeout: Duration::from_secs(5),
        half_open_timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
    };
    
    let breaker = DefaultCircuitBreaker::new(config);
    
    // Record some activity
    breaker.post_execute(false).await.unwrap();
    breaker.post_execute(true).await.unwrap();
    breaker.post_execute(false).await.unwrap();
    
    let metrics = breaker.metrics();
    assert!(metrics.failed_requests > 0, "Should record failures");
    assert!(metrics.successful_requests > 0, "Should record successes");
    assert_eq!(breaker.current_state(), CircuitState::Closed, "Should be in closed state");
} 