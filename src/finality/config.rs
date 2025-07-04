/*!
# Finality Configuration Module

This module provides comprehensive configuration management for the FROST protocol's
finality system. It defines settings for verification, circuit breakers, rate limiting,
caching, and chain-specific parameters.

## Core Components

### Finality Config
- Base settings
- Circuit breaker config
- Rate limiter config
- Cache settings
- Chain-specific rules

### Base Configuration
- Timeout settings
- Block tracking
- Confirmation rules
- Confidence thresholds

### Protection Mechanisms
- Circuit breaker settings
- Rate limiting rules
- Cache management
- Chain isolation

## Architecture

The configuration system consists of several key components:

1. **Base Configuration**
   ```rust
   pub struct BaseConfig {
       default_timeout: Duration,
       max_tracked_blocks: usize,
       min_confirmations: u32,
       confidence_threshold: f64,
   }
   ```
   - Core settings
   - Default values
   - Global limits
   - Basic thresholds

2. **Circuit Breaker Config**
   ```rust
   pub struct CircuitBreakerConfig {
       failure_threshold: u32,
       reset_timeout: Duration,
       backoff_multiplier: f64,
       max_backoff: Duration,
   }
   ```
   - Failure handling
   - Reset timing
   - Backoff strategy
   - Chain isolation

3. **Rate Limiter Config**
   ```rust
   pub struct RateLimiterConfig {
       max_requests: u32,
       window: Duration,
       allow_burst: bool,
       burst_size: u32,
   }
   ```
   - Request limits
   - Time windows
   - Burst handling
   - Chain limits

## Features

### Configuration Management
- Default values
- Validation rules
- Chain overrides
- Dynamic updates

### Protection Settings
- Circuit breaking
- Rate limiting
- Caching rules
- Chain isolation

### Chain Configuration
- Custom parameters
- Specific timeouts
- Custom thresholds
- Isolation rules

### Validation
- Config validation
- Value checking
- Chain validation
- Update verification

## Best Practices

1. **Base Settings**
   - Appropriate timeouts
   - Block limits
   - Confirmation rules
   - Confidence levels

2. **Circuit Breakers**
   - Failure thresholds
   - Reset timing
   - Backoff strategy
   - Chain isolation

3. **Rate Limiting**
   - Request limits
   - Window sizing
   - Burst handling
   - Chain limits

4. **Caching**
   - Size limits
   - TTL settings
   - Warming rules
   - Chain isolation

## Integration

The configuration system integrates with:
1. Finality verification
2. Chain management
3. Protection systems
4. Cache management
*/

use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::collections::HashMap;

/// Core configuration for finality verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityConfig {
    /// Base configuration
    pub base: BaseConfig,
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
    /// Rate limiting settings
    pub rate_limiter: RateLimiterConfig,
    /// Caching settings
    pub cache: CacheConfig,
    /// Chain-specific settings
    pub chain_config: ChainConfig,
}

/// Base configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    /// Default timeout for finality verification
    pub default_timeout: Duration,
    /// Maximum number of blocks to track
    pub max_tracked_blocks: usize,
    /// Minimum required confirmations
    pub min_confirmations: u32,
    /// Required confidence threshold (0.0 - 1.0)
    pub confidence_threshold: f64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before tripping
    pub failure_threshold: u32,
    /// Reset timeout after tripping
    pub reset_timeout: Duration,
    /// Progressive backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Per-chain circuit breakers
    pub per_chain_breakers: bool,
}

/// Rate limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub window: Duration,
    /// Whether to enable burst allowance
    pub allow_burst: bool,
    /// Maximum burst size
    pub burst_size: u32,
    /// Per-chain rate limits
    pub per_chain_limits: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_size: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Whether to enable cache warming
    pub enable_warming: bool,
    /// Maximum number of entries to pre-warm
    pub warm_size: usize,
    /// Per-chain cache settings
    pub per_chain_cache: bool,
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain-specific parameters
    pub params: HashMap<String, serde_json::Value>,
    /// Chain-specific timeouts
    pub timeouts: HashMap<String, Duration>,
    /// Chain-specific thresholds
    pub thresholds: HashMap<String, f64>,
}

impl Default for FinalityConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig {
                default_timeout: Duration::from_secs(300),
                max_tracked_blocks: 1000,
                min_confirmations: 6,
                confidence_threshold: 0.99,
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                reset_timeout: Duration::from_secs(60),
                backoff_multiplier: 2.0,
                max_backoff: Duration::from_secs(3600),
                per_chain_breakers: true,
            },
            rate_limiter: RateLimiterConfig {
                max_requests: 100,
                window: Duration::from_secs(60),
                allow_burst: true,
                burst_size: 20,
                per_chain_limits: true,
            },
            cache: CacheConfig {
                max_size: 1000,
                ttl: Duration::from_secs(300),
                enable_warming: true,
                warm_size: 100,
                per_chain_cache: true,
            },
            chain_config: ChainConfig {
                params: HashMap::new(),
                timeouts: HashMap::new(),
                thresholds: HashMap::new(),
            },
        }
    }
}

impl FinalityConfig {
    /// Get chain-specific timeout
    pub fn get_chain_timeout(&self, chain_id: &str) -> Duration {
        self.chain_config
            .timeouts
            .get(chain_id)
            .copied()
            .unwrap_or(self.base.default_timeout)
    }

    /// Get chain-specific threshold
    pub fn get_chain_threshold(&self, chain_id: &str) -> f64 {
        self.chain_config
            .thresholds
            .get(chain_id)
            .copied()
            .unwrap_or(self.base.confidence_threshold)
    }

    /// Get chain-specific parameters
    pub fn get_chain_params(&self, chain_id: &str) -> Option<&serde_json::Value> {
        self.chain_config.params.get(chain_id)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate base config
        if self.base.confidence_threshold <= 0.0 || self.base.confidence_threshold > 1.0 {
            return Err("Confidence threshold must be between 0 and 1".into());
        }

        // Validate circuit breaker
        if self.circuit_breaker.failure_threshold == 0 {
            return Err("Failure threshold cannot be 0".into());
        }
        if self.circuit_breaker.backoff_multiplier <= 1.0 {
            return Err("Backoff multiplier must be greater than 1".into());
        }

        // Validate rate limiter
        if self.rate_limiter.max_requests == 0 {
            return Err("Max requests cannot be 0".into());
        }
        if self.rate_limiter.window.as_secs() == 0 {
            return Err("Rate limit window cannot be 0".into());
        }

        // Validate cache
        if self.cache.max_size == 0 {
            return Err("Cache size cannot be 0".into());
        }
        if self.cache.ttl.as_secs() == 0 {
            return Err("Cache TTL cannot be 0".into());
        }

        Ok(())
    }
} 