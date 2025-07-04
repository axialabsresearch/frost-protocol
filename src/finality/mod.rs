/*!
# Finality Module

The finality module provides a robust and flexible finality verification system for the FROST protocol.
It ensures that state transitions across different chains are properly finalized and verified.

## Core Components

### Finality Verification
- Verifier implementations for different chain types
- Customizable verification rules and predicates
- Support for different finality mechanisms
- Proof validation and verification

### Signal Processing
- Finality signal generation and validation
- Cross-chain signal propagation
- Signal aggregation and consensus
- Timeout and failure handling

### Monitoring & Recovery
- Real-time finality monitoring
- Performance metrics collection
- Automatic recovery strategies
- Circuit breaker protection

### Configuration
- Chain-specific settings
- Rate limiting parameters
- Cache configuration
- Circuit breaker thresholds

## Architecture

The finality system consists of several key components:

1. **Verifier** (`FinalityVerifier`)
   - Validates finality proofs
   - Checks chain-specific rules
   - Handles state transitions
   - Manages verification lifecycle

2. **Monitor** (`FinalityMonitor`)
   - Tracks finality progress
   - Collects performance metrics
   - Detects verification issues
   - Triggers recovery actions

3. **Recovery** (`RecoveryManager`)
   - Implements recovery strategies
   - Manages circuit breakers
   - Handles error conditions
   - Restores normal operation

4. **Predicates** (`FinalityPredicate`)
   - Custom verification rules
   - Chain-specific logic
   - State validation rules
   - Security constraints

## Features

### Verification
- Multi-chain support
- Custom finality rules
- Proof validation
- State verification

### Monitoring
- Real-time metrics
- Performance tracking
- Resource utilization
- Error detection

### Recovery
- Automatic recovery
- Circuit breakers
- Rate limiting
- Error handling

### Configuration
- Chain settings
- Performance tuning
- Security parameters
- Resource limits

## Integration

The finality system integrates with:
1. State management
2. Network protocol
3. Extension system
4. Metrics collection
*/

pub mod verifier;
pub mod signal;
pub mod error;
pub mod monitor;
pub mod predicate;
pub mod config;
pub mod metrics;
pub mod recovery;

pub use verifier::FinalityVerifier;
pub use signal::FinalitySignal;
pub use error::{FinalityError, ErrorSeverity};
pub use monitor::FinalityMonitor;
pub use config::{
    FinalityConfig, BaseConfig, CircuitBreakerConfig,
    RateLimiterConfig, CacheConfig, ChainConfig,
};
pub use metrics::{
    FinalityMetrics, VerificationMetrics, PerformanceMetrics,
    LatencyMetrics, StateSyncMetrics, ResourceMetrics, ChainMetrics,
};
pub use recovery::{
    RecoveryManager, RecoveryStrategy, ChainRecoveryState,
    CircuitBreakerState, RateLimiterState, ErrorRecord,
};
