use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use rand::Rng;
use crate::network::NetworkError;
use crate::Result;

/// Retry policy for network operations
#[async_trait]
pub trait RetryPolicy: Send + Sync {
    /// Check if operation should be retried
    async fn should_retry(&self, error: &NetworkError, attempt: u32) -> bool;
    
    /// Get delay before next retry
    async fn get_delay(&self, attempt: u32) -> Duration;
    
    /// Get retry metrics
    fn metrics(&self) -> RetryMetrics;
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub jitter_factor: f64,
    pub retry_budget: RetryBudget,
}

/// Retry budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryBudget {
    pub ttl: Duration,
    pub min_retries: u32,
    pub retry_ratio: f64,
}

/// Retry metrics
#[derive(Debug, Clone, Default)]
pub struct RetryMetrics {
    pub total_retries: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub average_attempts: f64,
    pub budget_exhaustions: u64,
}

/// Default retry policy implementation
pub struct DefaultRetryPolicy {
    config: RetryConfig,
    metrics: parking_lot::RwLock<RetryMetrics>,
    budget_window: parking_lot::RwLock<Vec<SystemTime>>,
}

impl DefaultRetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            metrics: parking_lot::RwLock::new(RetryMetrics::default()),
            budget_window: parking_lot::RwLock::new(Vec::new()),
        }
    }

    fn calculate_exponential_backoff(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_delay.as_secs_f64();
        let max_delay = self.config.max_delay.as_secs_f64();
        let backoff = base_delay * self.config.backoff_factor.powi(attempt as i32);
        
        // Add jitter
        let jitter_range = backoff * self.config.jitter_factor;
        let jitter = rand::thread_rng().gen_range(-jitter_range..jitter_range);
        let delay = (backoff + jitter).min(max_delay);
        
        Duration::from_secs_f64(delay)
    }

    async fn check_retry_budget(&self) -> bool {
        let now = std::time::SystemTime::now();
        let mut window = self.budget_window.write();
        
        // Remove expired entries
        window.retain(|time| {
            if let Ok(age) = now.duration_since(*time) {
                age < self.config.retry_budget.ttl
            } else {
                false
            }
        });

        // Check if within budget
        let current_retries = window.len() as f64;
        let max_retries = (self.config.retry_budget.min_retries as f64 +
            (current_retries * self.config.retry_budget.retry_ratio)) as usize;
            
        if window.len() < max_retries {
            window.push(now);
            true
        } else {
            let mut metrics = self.metrics.write();
            metrics.budget_exhaustions += 1;
            false
        }
    }

    fn is_retryable_error(&self, error: &NetworkError) -> bool {
        error.is_retryable()
    }
}

#[async_trait]
impl RetryPolicy for DefaultRetryPolicy {
    async fn should_retry(&self, error: &NetworkError, attempt: u32) -> bool {
        if attempt >= self.config.max_attempts {
            return false;
        }

        if !self.is_retryable_error(error) {
            return false;
        }

        self.check_retry_budget().await
    }

    async fn get_delay(&self, attempt: u32) -> Duration {
        self.calculate_exponential_backoff(attempt)
    }

    fn metrics(&self) -> RetryMetrics {
        self.metrics.read().clone()
    }
}

/// Retry operation with policy
pub async fn with_retry<T, F>(
    operation: F,
    policy: &impl RetryPolicy,
) -> Result<T>
where
    F: Fn() -> Result<T> + Send + Sync,
{
    let mut attempt = 0;
    let mut last_error = None;

    loop {
        attempt += 1;
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    let mut metrics = policy.metrics();
                    metrics.successful_retries += 1;
                }
                return Ok(result);
            }
            Err(error) => {
                if let Some(network_error) = error.downcast_ref::<NetworkError>() {
                    if policy.should_retry(network_error, attempt).await {
                        let delay = policy.get_delay(attempt).await;
                        tokio::time::sleep(delay).await;
                        last_error = Some(error);
                        continue;
                    }
                }
                return Err(last_error.unwrap_or(error));
            }
        }
    }
} 