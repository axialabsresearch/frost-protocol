#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use frost_protocol::{
    network::{
        backpressure::{BackpressureController, BackpressureConfig, DefaultBackpressureController, LoadMetrics},
        error::NetworkError,
    },
    message::FrostMessage,
};

use std::time::Duration;
use std::sync::Arc;
use tokio::sync::OnceCell;

static CONTROLLER: OnceCell<Arc<DefaultBackpressureController>> = OnceCell::const_new();

#[tokio::test]
async fn test_backpressure_initialization() {
    let config = BackpressureConfig {
        max_concurrent_requests: 100,
        max_queue_size: 1000,
        pressure_threshold: 0.8,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    };
    
    let controller = DefaultBackpressureController::new(config);
    assert!(controller.metrics().current_load == 0.0, "Initial load should be zero");
}

#[tokio::test]
async fn test_request_throttling() {
    let config = BackpressureConfig {
        max_concurrent_requests: 2,
        max_queue_size: 1,
        pressure_threshold: 0.7,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    };
    
    let controller = CONTROLLER.get_or_init(|| async {
        Arc::new(DefaultBackpressureController::new(config))
    }).await;
    
    // Acquire all permits
    let permit1 = controller.acquire().await.expect("Should get first permit");
    let permit2 = controller.acquire().await.expect("Should get second permit");
    
    // This should be queued - spawn task that will complete with success/failure
    let permit3_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    // Use the permit briefly, then drop it
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };

    // This should be rejected (queue full)
    let permit4_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    // Use the permit briefly, then drop it
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };

    // Wait a bit to ensure tasks have started
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check metrics
    let metrics = controller.metrics();
    assert_eq!(metrics.queued_requests, 1, "Should have one queued request");
    
    // Drop a permit to allow queued request through
    drop(permit1);
    
    // Verify results
    let permit3_result = permit3_task.await.expect("Task should complete");
    let permit4_result = permit4_task.await.expect("Task should complete");
    
    assert!(permit3_result.is_ok(), "Queued request should succeed");
    assert!(permit4_result.is_err(), "Excess request should be rejected");
}

#[tokio::test]
async fn test_burst_handling() {
    let config = BackpressureConfig {
        max_concurrent_requests: 2,
        max_queue_size: 2,
        pressure_threshold: 0.6,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    };
    
    let controller = Arc::new(DefaultBackpressureController::new(config));
    
    // Acquire all permits
    let permit1 = controller.acquire().await.expect("Should get first permit");
    let permit2 = controller.acquire().await.expect("Should get second permit");
    
    // These should be queued - spawn tasks that use permits and return success/failure
    let permit3_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };
    
    let permit4_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };

    // This should be rejected (queue full)
    let permit5_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };

    // Wait a bit to ensure tasks have started
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check metrics
    let metrics = controller.metrics();
    assert_eq!(metrics.queued_requests, 2, "Should have two queued requests");
    
    // Drop permits to allow queued requests through
    drop(permit1);
    drop(permit2);
    
    // Verify results
    let permit3_result = permit3_task.await.expect("Task should complete");
    let permit4_result = permit4_task.await.expect("Task should complete");
    let permit5_result = permit5_task.await.expect("Task should complete");
    
    assert!(permit3_result.is_ok(), "First queued request should succeed");
    assert!(permit4_result.is_ok(), "Second queued request should succeed");
    assert!(permit5_result.is_err(), "Excess request should be rejected");
}

#[tokio::test]
async fn test_window_reset() {
    let config = BackpressureConfig {
        max_concurrent_requests: 2,
        max_queue_size: 2,
        pressure_threshold: 0.7,
        sampling_window: Duration::from_secs(1),
        decay_factor: 0.95,
    };
    
    let controller = DefaultBackpressureController::new(config);
    
    // Use up quota
    let permit1 = controller.acquire().await.expect("Should get first permit");
    let permit2 = controller.acquire().await.expect("Should get second permit");
    
    // Wait for window reset
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Drop old permits
    drop(permit1);
    drop(permit2);
    
    // Should allow requests again after window reset
    assert!(controller.acquire().await.is_ok(), "Should allow requests after window reset");
}

#[tokio::test]
async fn test_backpressure_metrics() {
    let config = BackpressureConfig {
        max_concurrent_requests: 2,
        max_queue_size: 2,
        pressure_threshold: 0.7,
        sampling_window: Duration::from_millis(100),
        decay_factor: 0.95,
    };
    
    let controller = Arc::new(DefaultBackpressureController::new(config));
    
    // Update initial load metrics
    controller.update_load(LoadMetrics {
        cpu_usage: 0.5,
        memory_usage: 0.4,
        queue_size: 0,
        request_rate: 5.0,
        error_rate: 0.0,
    }).await.expect("Should update load metrics");

    // Acquire permits sequentially to ensure proper order
    let permit1 = controller.acquire().await.expect("Should get first permit");
    let permit2 = controller.acquire().await.expect("Should get second permit");
    
    // These should be queued since we're at max concurrent requests
    let permit3_task = {
        let controller = controller.clone();
        tokio::spawn(async move { 
            match controller.acquire().await {
                Ok(_permit) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };

    let permit4_task = {
        let controller = controller.clone();
        tokio::spawn(async move {
            match controller.acquire().await {
                Ok(_permit) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        })
    };
    
    // Wait for queued requests to be registered
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Check initial metrics
    let metrics = controller.metrics();
    assert_eq!(metrics.queued_requests, 2, "Should have two queued requests");
    assert_eq!(metrics.rejected_requests, 0, "Should not have rejected requests yet");
    assert!(metrics.current_load > 0.0, "Should calculate current load");
    
    // Update load to indicate high pressure
    controller.update_load(LoadMetrics {
        cpu_usage: 0.8,
        memory_usage: 0.7,
        queue_size: 2,
        request_rate: 15.0,
        error_rate: 0.0,
    }).await.expect("Should update load metrics");
    
    // Try one more request that should be rejected (queue is full)
    let reject_result = controller.acquire().await;
    assert!(reject_result.is_err(), "Should reject request when system is under load");
    
    let metrics_after_reject = controller.metrics();
    assert_eq!(metrics_after_reject.rejected_requests, 1, "Should record rejected request");
    assert!(metrics_after_reject.current_load > metrics.current_load, "Load should increase");
    
    // Cleanup
    drop(permit1);
    drop(permit2);
    let _ = permit3_task.await;
    let _ = permit4_task.await;
}