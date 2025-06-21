use async_trait::async_trait;
use libp2p::{
    kad::{
        Kademlia, KademliaConfig, KademliaEvent, QueryResult,
        store::MemoryStore, Record, RecordKey,
    },
    PeerId, Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashSet;
use tokio::sync::mpsc;
use crate::network::{Peer, NodeIdentity, P2PEvent};
use crate::Result;

/// Peer discovery mechanism
#[async_trait]
pub trait PeerDiscovery: Send + Sync {
    /// Initialize the discovery service
    async fn init(&mut self, config: DiscoveryConfig) -> Result<()>;

    /// Start peer discovery
    async fn start_discovery(&mut self) -> Result<()>;

    /// Stop peer discovery
    async fn stop_discovery(&mut self) -> Result<()>;

    /// Manually announce presence to the network
    async fn announce(&mut self) -> Result<()>;

    /// Find peers matching criteria
    async fn find_peers(&self, criteria: PeerCriteria) -> Result<Vec<PeerInfo>>;

    /// Get discovery metrics
    fn metrics(&self) -> DiscoveryMetrics;
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Replication interval
    pub replication_interval: Duration,
    /// Record TTL
    pub record_ttl: Duration,
    /// Query timeout
    pub query_timeout: Duration,
    /// Maximum peers to discover
    pub max_peers: usize,
    /// Minimum peers to maintain
    pub min_peers: usize,
    /// Enable provider records
    pub enable_provider_records: bool,
    /// Provider record announcement interval
    pub provider_announce_interval: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![],
            replication_interval: Duration::from_secs(300),
            record_ttl: Duration::from_secs(7200),
            query_timeout: Duration::from_secs(60),
            max_peers: 50,
            min_peers: 10,
            enable_provider_records: true,
            provider_announce_interval: Duration::from_secs(1800),
        }
    }
}

/// Discovery event types
#[derive(Debug)]
pub enum DiscoveryEvent {
    /// New peer discovered
    PeerDiscovered(PeerId, Vec<Multiaddr>),
    /// Peer lost
    PeerLost(PeerId),
    /// Record found
    RecordFound(Record),
    /// Provider found
    ProviderFound(PeerId, Vec<Multiaddr>),
    /// Error occurred
    Error(String),
}

/// Kademlia-based peer discovery
pub struct KademliaPeerDiscovery {
    /// Kademlia DHT
    kad: Kademlia<MemoryStore>,
    /// Node identity
    identity: NodeIdentity,
    /// Configuration
    config: DiscoveryConfig,
    /// Event sender
    event_tx: mpsc::Sender<P2PEvent>,
    /// Known peers
    known_peers: HashSet<PeerId>,
    /// Bootstrap state
    bootstrapped: bool,
}

impl KademliaPeerDiscovery {
    /// Create new Kademlia discovery
    pub fn new(
        identity: NodeIdentity,
        config: DiscoveryConfig,
        event_tx: mpsc::Sender<P2PEvent>,
    ) -> Self {
        let store = MemoryStore::new(identity.peer_id);
        let kad_config = KademliaConfig::default()
            .set_replication_interval(config.replication_interval)
            .set_record_ttl(Some(config.record_ttl))
            .set_query_timeout(config.query_timeout)
            .set_max_packet_size(4096)
            .to_owned();

        let kad = Kademlia::with_config(identity.peer_id, store, kad_config);

        Self {
            kad,
            identity,
            config,
            event_tx,
            known_peers: HashSet::new(),
            bootstrapped: false,
        }
    }

    /// Bootstrap the DHT
    pub async fn bootstrap(&mut self) -> Result<()> {
        // Add bootstrap nodes
        for addr in &self.config.bootstrap_nodes {
            if let Ok(multiaddr) = addr.parse() {
                self.kad.add_address(&PeerId::random(), multiaddr);
            }
        }

        // Start bootstrap process
        if let Err(e) = self.kad.bootstrap() {
            return Err(format!("Failed to bootstrap DHT: {}", e).into());
        }

        self.bootstrapped = true;
        Ok(())
    }

    /// Handle Kademlia events
    pub async fn handle_event(&mut self, event: KademliaEvent) -> Result<()> {
        match event {
            KademliaEvent::OutboundQueryCompleted { result, .. } => {
                match result {
                    QueryResult::Bootstrap(Ok(_)) => {
                        // Bootstrap successful
                        self.bootstrapped = true;
                    }
                    QueryResult::GetClosestPeers(Ok(peers)) => {
                        for peer in peers {
                            if !self.known_peers.contains(&peer) {
                                self.known_peers.insert(peer);
                                if let Some(addrs) = self.kad.addresses_of_peer(&peer) {
                                    self.event_tx.send(P2PEvent::PeerConnected(peer)).await?;
                                }
                            }
                        }
                    }
                    QueryResult::GetProviders(Ok(providers)) => {
                        for provider in providers.providers {
                            if !self.known_peers.contains(&provider) {
                                self.known_peers.insert(provider);
                                if let Some(addrs) = self.kad.addresses_of_peer(&provider) {
                                    self.event_tx.send(P2PEvent::PeerConnected(provider)).await?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            KademliaEvent::RoutingUpdated { peer, .. } => {
                // Update routing table
                if !self.known_peers.contains(&peer) {
                    self.known_peers.insert(peer);
                    if let Some(addrs) = self.kad.addresses_of_peer(&peer) {
                        self.event_tx.send(P2PEvent::PeerConnected(peer)).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Start provider announcements
    pub async fn start_provider_announcements(&mut self) -> Result<()> {
        if !self.config.enable_provider_records {
            return Ok(());
        }

        let provider_key = RecordKey::new(&self.identity.peer_id.to_bytes());
        
        // Announce self as provider periodically
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.config.provider_announce_interval).await;
                if let Err(e) = self.kad.start_providing(provider_key.clone()) {
                    println!("Failed to announce provider record: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Find peers
    pub async fn find_peers(&mut self) -> Result<()> {
        if !self.bootstrapped {
            self.bootstrap().await?;
        }

        // Query for random peer IDs to discover new peers
        for _ in 0..5 {
            let random_peer = PeerId::random();
            self.kad.get_closest_peers(random_peer);
        }

        Ok(())
    }

    /// Get known peers
    pub fn get_known_peers(&self) -> HashSet<PeerId> {
        self.known_peers.clone()
    }

    /// Check if we need more peers
    pub fn needs_more_peers(&self) -> bool {
        self.known_peers.len() < self.config.min_peers
    }

    /// Get Kademlia instance
    pub fn kademlia(&mut self) -> &mut Kademlia<MemoryStore> {
        &mut self.kad
    }
}

#[async_trait]
impl crate::network::PeerDiscovery for KademliaPeerDiscovery {
    async fn discover_peers(&mut self) -> Result<Vec<Peer>> {
        if self.needs_more_peers() {
            self.find_peers().await?;
        }

        let mut peers = Vec::new();
        for peer_id in &self.known_peers {
            if let Some(addrs) = self.kad.addresses_of_peer(peer_id) {
                peers.push(Peer {
                    id: uuid::Uuid::new_v4(), // Map PeerId to UUID
                    info: crate::network::PeerInfo {
                        address: addrs[0].to_string(),
                        protocol_version: "1.0.0".to_string(),
                        supported_features: vec!["kad".to_string()],
                        chain_ids: vec![],
                        node_type: crate::network::NodeType::Validator,
                    },
                    state: crate::network::PeerState::Connected,
                });
            }
        }

        Ok(peers)
    }

    async fn announce(&mut self) -> Result<()> {
        if self.config.enable_provider_records {
            let provider_key = RecordKey::new(&self.identity.peer_id.to_bytes());
            self.kad.start_providing(provider_key)?;
        }
        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<Peer>> {
        let mut peers = Vec::new();
        for peer_id in &self.known_peers {
            if let Some(addrs) = self.kad.addresses_of_peer(peer_id) {
                peers.push(Peer {
                    id: uuid::Uuid::new_v4(), // Map PeerId to UUID
                    info: crate::network::PeerInfo {
                        address: addrs[0].to_string(),
                        protocol_version: "1.0.0".to_string(),
                        supported_features: vec!["kad".to_string()],
                        chain_ids: vec![],
                        node_type: crate::network::NodeType::Validator,
                    },
                    state: crate::network::PeerState::Connected,
                });
            }
        }
        Ok(peers)
    }
}

/// Criteria for peer discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCriteria {
    pub node_types: Vec<String>,
    pub min_version: Option<String>,
    pub required_features: Vec<String>,
    pub chain_ids: Vec<u64>,
    pub max_latency: Option<Duration>,
    pub geographic_region: Option<String>,
}

/// Discovery metrics
#[derive(Debug, Clone, Default)]
pub struct DiscoveryMetrics {
    pub discovered_peers: u64,
    pub active_discoveries: usize,
    pub last_discovery: Option<std::time::SystemTime>,
    pub cached_peers: usize,
    pub successful_announcements: u64,
    pub failed_announcements: u64,
    pub average_discovery_time: Duration,
}

/// Health check for discovered peers
#[async_trait]
pub trait PeerHealthCheck: Send + Sync {
    /// Check peer health
    async fn check_health(&self, peer: &Peer) -> Result<HealthStatus>;

    /// Get health metrics for a peer
    async fn get_health_metrics(&self, peer: &Peer) -> Result<HealthMetrics>;
}

/// Peer health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency: Duration,
    pub last_seen: std::time::SystemTime,
    pub response_time: Duration,
    pub error_count: u32,
}

/// Health metrics
#[derive(Debug, Clone, Default)]
pub struct HealthMetrics {
    pub uptime_percentage: f64,
    pub average_latency: Duration,
    pub error_rate: f64,
    pub bandwidth_usage: f64,
    pub response_times: Vec<Duration>,
} 