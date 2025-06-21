use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::network::{Peer, NetworkError, Transport};
use crate::Result;

/// Connection pool manager
#[async_trait]
pub trait ConnectionPool: Send + Sync {
    /// Acquire a connection from the pool
    async fn acquire(&self, peer: &Peer) -> Result<PooledConnection>;
    
    /// Release a connection back to the pool
    async fn release(&self, connection: PooledConnection) -> Result<()>;
    
    /// Get pool metrics
    fn metrics(&self) -> PoolMetrics;
    
    /// Clean up idle connections
    async fn cleanup(&self) -> Result<()>;
}

/// Dynamic pool configuration that adapts to network conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPoolConfig {
    /// Base configuration that can be adjusted
    pub base: PoolConfig,
    /// Dynamic adjustment parameters
    pub dynamic: DynamicAdjustment,
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of idle connections per peer
    pub min_idle_per_peer: usize,
    /// Maximum number of connections per peer
    pub max_per_peer: usize,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Idle timeout for connections
    pub idle_timeout: Duration,
    /// Connection establishment timeout
    pub connection_timeout: Duration,
    /// Interval for connection validation
    pub validation_interval: Duration,
}

/// Dynamic adjustment parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAdjustment {
    /// How quickly to adjust to network conditions (0.0-1.0)
    pub adaptation_rate: f64,
    /// Maximum growth rate per adjustment
    pub max_growth_rate: f64,
    /// Minimum connections to maintain
    pub min_total_connections: usize,
    /// Maximum total connections across all peers
    pub max_total_connections: usize,
    /// Load threshold for scaling up
    pub scale_up_threshold: f64,
    /// Load threshold for scaling down
    pub scale_down_threshold: f64,
}

/// Peer-specific connection metrics
#[derive(Debug, Clone, Default)]
pub struct PeerMetrics {
    /// Number of successful operations
    pub successful_ops: u64,
    /// Number of failed operations
    pub failed_ops: u64,
    /// Average operation latency
    pub avg_latency: Duration,
    /// Connection failure rate
    pub failure_rate: f64,
    /// Current load factor (0.0-1.0)
    pub load_factor: f64,
    /// Reputation score (0.0-1.0)
    pub reputation: f64,
}

/// Connection status with peer-specific state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Idle,
    Active { peer_load: u32 },
    Validating,
    Failed { reason: FailureReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureReason {
    Timeout,
    NetworkError,
    PeerDisconnected,
    ValidationFailed,
}

/// Enhanced pooled connection with peer metrics
#[derive(Debug)]
pub struct PooledConnection {
    pub id: uuid::Uuid,
    pub peer: Peer,
    pub created_at: SystemTime,
    pub last_used: SystemTime,
    pub status: ConnectionStatus,
    pub metrics: ConnectionMetrics,
    /// Peer-specific performance metrics
    pub peer_metrics: PeerMetrics,
}

/// Connection metrics with enhanced tracking
#[derive(Debug, Clone, Default)]
pub struct ConnectionMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_processed: u64,
    pub errors: u64,
    pub total_active_time: Duration,
    /// Recent operation latencies
    pub recent_latencies: Vec<Duration>,
    /// Connection quality score (0.0-1.0)
    pub quality_score: f64,
}

/// Pool metrics with peer-specific information
#[derive(Debug, Clone, Default)]
pub struct PoolMetrics {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_connections: usize,
    pub connection_requests: u64,
    pub connection_timeouts: u64,
    pub connection_errors: u64,
    pub average_wait_time: Duration,
    pub peak_connections: usize,
    /// Metrics per peer
    pub peer_metrics: HashMap<uuid::Uuid, PeerMetrics>,
    /// Global load factor
    pub global_load_factor: f64,
}

/// Default connection pool implementation with dynamic sizing
pub struct DefaultConnectionPool<T: Transport> {
    config: DynamicPoolConfig,
    transport: T,
    connections: Mutex<HashMap<uuid::Uuid, PooledConnection>>,
    metrics: parking_lot::RwLock<PoolMetrics>,
    /// Per-peer connection limits
    peer_limits: parking_lot::RwLock<HashMap<uuid::Uuid, usize>>,
}

impl<T: Transport> DefaultConnectionPool<T> {
    pub fn new(config: DynamicPoolConfig, transport: T) -> Self {
        Self {
            config,
            transport,
            connections: Mutex::new(HashMap::new()),
            metrics: parking_lot::RwLock::new(PoolMetrics::default()),
            peer_limits: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Adjust connection limits based on peer performance
    async fn adjust_peer_limit(&self, peer: &Peer) -> Result<()> {
        let mut limits = self.peer_limits.write();
        let metrics = self.metrics.read();
        
        if let Some(peer_metrics) = metrics.peer_metrics.get(&peer.id) {
            let current_limit = *limits.get(&peer.id).unwrap_or(&self.config.base.min_idle_per_peer);
            
            // Calculate new limit based on peer performance
            let load_factor = peer_metrics.load_factor;
            let reputation = peer_metrics.reputation;
            let failure_rate = peer_metrics.failure_rate;
            
            let mut new_limit = current_limit;
            
            // Scale up if peer is performing well
            if load_factor > self.config.dynamic.scale_up_threshold 
               && reputation > 0.7 
               && failure_rate < 0.1 {
                new_limit = ((current_limit as f64) * 
                    (1.0 + self.config.dynamic.max_growth_rate)) as usize;
            }
            
            // Scale down if peer is performing poorly
            if load_factor < self.config.dynamic.scale_down_threshold 
               || reputation < 0.3 
               || failure_rate > 0.3 {
                new_limit = ((current_limit as f64) * 
                    (1.0 - self.config.dynamic.adaptation_rate)) as usize;
            }
            
            // Enforce limits
            new_limit = new_limit
                .max(self.config.base.min_idle_per_peer)
                .min(self.config.base.max_per_peer);
            
            limits.insert(peer.id, new_limit);
        }
        
        Ok(())
    }

    /// Update peer metrics based on connection performance
    async fn update_peer_metrics(&self, conn: &PooledConnection) -> Result<()> {
        let mut metrics = self.metrics.write();
        let peer_metrics = metrics.peer_metrics.entry(conn.peer.id)
            .or_insert_with(PeerMetrics::default);
        
        // Update metrics based on connection performance
        if !conn.metrics.recent_latencies.is_empty() {
            let avg_latency: Duration = conn.metrics.recent_latencies.iter().sum::<Duration>() / 
                conn.metrics.recent_latencies.len() as u32;
            peer_metrics.avg_latency = avg_latency;
        }
        
        peer_metrics.failure_rate = if conn.metrics.requests_processed > 0 {
            conn.metrics.errors as f64 / conn.metrics.requests_processed as f64
        } else {
            0.0
        };
        
        // Update reputation based on performance
        peer_metrics.reputation = calculate_reputation(
            peer_metrics.failure_rate,
            peer_metrics.avg_latency,
            conn.metrics.quality_score
        );
        
        Ok(())
    }

    async fn create_connection(&self, peer: &Peer) -> Result<PooledConnection> {
        let connection = PooledConnection {
            id: uuid::Uuid::new_v4(),
            peer: peer.clone(),
            created_at: SystemTime::now(),
            last_used: SystemTime::now(),
            status: ConnectionStatus::Idle,
            metrics: ConnectionMetrics::default(),
            peer_metrics: PeerMetrics::default(),
        };

        let mut connections = self.connections.lock().await;
        connections.insert(connection.id, connection.clone());

        let mut metrics = self.metrics.write();
        metrics.total_connections += 1;
        metrics.idle_connections += 1;
        metrics.peak_connections = metrics.peak_connections.max(connections.len());

        Ok(connection)
    }

    async fn validate_connection(&self, connection: &mut PooledConnection) -> Result<bool> {
        connection.status = ConnectionStatus::Validating;
        
        // Implement connection validation logic here
        // For example, send a ping message or check the underlying transport
        
        connection.status = ConnectionStatus::Idle;
        Ok(true)
    }
}

/// Calculate peer reputation score
fn calculate_reputation(failure_rate: f64, avg_latency: Duration, quality_score: f64) -> f64 {
    let latency_factor = 1.0 - (avg_latency.as_secs_f64() / 1.0).min(1.0);
    let failure_factor = 1.0 - failure_rate;
    
    // Weighted average of factors
    0.4 * failure_factor + 0.3 * latency_factor + 0.3 * quality_score
}

#[async_trait]
impl<T: Transport> ConnectionPool for DefaultConnectionPool<T> {
    async fn acquire(&self, peer: &Peer) -> Result<PooledConnection> {
        let start_time = SystemTime::now();
        let mut connections = self.connections.lock().await;
        
        // Update peer limits based on performance
        self.adjust_peer_limit(peer).await?;
        
        // Count current connections for this peer
        let peer_conn_count = connections.values()
            .filter(|c| c.peer.id == peer.id)
            .count();
        
        let peer_limit = self.peer_limits.read()
            .get(&peer.id)
            .copied()
            .unwrap_or(self.config.base.min_idle_per_peer);
        
        // Try to find an idle connection
        let mut available_connection = None;
        for conn in connections.values_mut() {
            if conn.peer.id == peer.id && matches!(conn.status, ConnectionStatus::Idle) {
                if let Ok(true) = self.validate_connection(conn).await {
                    available_connection = Some(conn.clone());
                    break;
                }
            }
        }

        // Create new connection if needed and within limits
        let mut connection = match available_connection {
            Some(conn) => conn,
            None => {
                if peer_conn_count >= peer_limit {
                    return Err(NetworkError::ConnectionFailed(
                        format!("Peer connection limit reached: {}", peer_limit)
                    ).into());
                }
                self.create_connection(peer).await?
            }
        };

        // Update connection state
        connection.status = ConnectionStatus::Active { peer_load: peer_conn_count as u32 };
        connection.last_used = SystemTime::now();
        
        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.connection_requests += 1;
        metrics.active_connections += 1;
        metrics.idle_connections -= 1;
        
        if let Ok(wait_time) = SystemTime::now().duration_since(start_time) {
            metrics.average_wait_time = (metrics.average_wait_time + wait_time) / 2;
        }
        
        // Update peer metrics
        self.update_peer_metrics(&connection).await?;

        Ok(connection)
    }

    async fn release(&self, mut connection: PooledConnection) -> Result<()> {
        let mut connections = self.connections.lock().await;
        
        // Update connection state
        connection.status = ConnectionStatus::Idle;
        connection.last_used = SystemTime::now();
        
        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.active_connections -= 1;
        metrics.idle_connections += 1;
        
        // Update peer metrics before release
        self.update_peer_metrics(&connection).await?;

        connections.insert(connection.id, connection);
        Ok(())
    }

    fn metrics(&self) -> PoolMetrics {
        self.metrics.read().clone()
    }

    async fn cleanup(&self) -> Result<()> {
        let mut connections = self.connections.lock().await;
        let now = SystemTime::now();
        
        // Remove expired and idle connections while respecting per-peer minimums
        let mut peer_counts = HashMap::new();
        
        // First pass: count connections per peer
        for conn in connections.values() {
            *peer_counts.entry(conn.peer.id).or_insert(0) += 1;
        }
        
        // Second pass: remove connections if above minimum
        connections.retain(|_, conn| {
            let peer_count = peer_counts.get_mut(&conn.peer.id).unwrap();
            
            let should_remove = if let Ok(idle_duration) = now.duration_since(conn.last_used) {
                if idle_duration > self.config.base.idle_timeout {
                    *peer_count > self.config.base.min_idle_per_peer
                } else {
                    false
                }
            } else {
                false
            };
            
            if should_remove {
                *peer_count -= 1;
                false
            } else {
                true
            }
        });

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.total_connections = connections.len();
        metrics.idle_connections = connections.values()
            .filter(|c| matches!(c.status, ConnectionStatus::Idle))
            .count();
        metrics.active_connections = connections.values()
            .filter(|c| matches!(c.status, ConnectionStatus::Active { .. }))
            .count();
        
        // Calculate global load factor
        metrics.global_load_factor = if metrics.total_connections > 0 {
            metrics.active_connections as f64 / metrics.total_connections as f64
        } else {
            0.0
        };

        Ok(())
    }
} 