mod protocol;
mod transport;
mod peer;
mod error;
mod discovery;
mod security;
mod circuit_breaker;
mod backpressure;
mod pool;
mod retry;
mod telemetry;
mod p2p;

pub use protocol::{NetworkProtocol, ProtocolConfig};
pub use transport::{Transport, TransportConfig};
pub use peer::{Peer, PeerInfo, PeerManager};
pub use error::NetworkError;
pub use discovery::{PeerDiscovery, DiscoveryConfig, PeerHealthCheck};
pub use security::{SecurityManager, SecurityConfig, AuthenticationResult};
pub use circuit_breaker::{CircuitBreaker, CircuitConfig, CircuitState};
pub use backpressure::{BackpressureController, BackpressureConfig, PressureLevel};
pub use pool::{ConnectionPool, PoolConfig, PooledConnection};
pub use retry::{RetryPolicy, RetryConfig, with_retry};
pub use telemetry::{TelemetryManager, NetworkMetrics, NetworkEvent};
pub use p2p::{P2PNode, P2PConfig, NodeIdentity};

use crate::Result;
