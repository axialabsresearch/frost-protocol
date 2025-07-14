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
