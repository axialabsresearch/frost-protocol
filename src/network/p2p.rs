use async_trait::async_trait;
use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::Boxed,
        upgrade::{SelectUpgrade, Version},
    },
    identity,
    noise::{self, NoiseConfig, X25519Spec},
    yamux::YamuxConfig,
    PeerId, Transport,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use crate::Result;

/// P2P configuration for the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Listen addresses for the node
    pub listen_addresses: Vec<String>,
    /// Bootstrap peers to connect to
    pub bootstrap_peers: Vec<String>,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Enable NAT traversal
    pub enable_nat: bool,
    /// Enable mdns discovery
    pub enable_mdns: bool,
}

/// P2P node identity
#[derive(Debug, Clone)]
pub struct NodeIdentity {
    /// Node's peer ID
    pub peer_id: PeerId,
    /// Node's keypair
    pub keypair: identity::Keypair,
}

impl NodeIdentity {
    /// Create a new node identity
    pub fn new() -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        Self { peer_id, keypair }
    }

    /// Create from existing keypair
    pub fn from_keypair(keypair: identity::Keypair) -> Self {
        let peer_id = PeerId::from(keypair.public());
        Self { peer_id, keypair }
    }
}

/// P2P event types
#[derive(Debug)]
pub enum P2PEvent {
    /// New peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// Received data from peer
    DataReceived {
        peer: PeerId,
        data: Vec<u8>,
    },
    /// Error occurred
    Error(String),
}

/// P2P node implementation
pub struct P2PNode {
    /// Node configuration
    config: P2PConfig,
    /// Node identity
    identity: NodeIdentity,
    /// Event sender
    event_tx: mpsc::Sender<P2PEvent>,
    /// Event receiver
    event_rx: mpsc::Receiver<P2PEvent>,
    /// Transport
    transport: Boxed<(PeerId, StreamMuxerBox)>,
}

impl P2PNode {
    /// Create a new P2P node
    pub async fn new(config: P2PConfig) -> Result<Self> {
        let identity = NodeIdentity::new();
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        // Create noise keys
        let noise_keys = noise::Keypair::<X25519Spec>::new()
            .into_authentic(&identity.keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");

        // Create transport with noise encryption
        let transport = libp2p::tcp::TokioTcpConfig::new()
            .nodelay(true)
            .upgrade(Version::V1)
            .authenticate(NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(YamuxConfig::default())
            .boxed();

        Ok(Self {
            config,
            identity,
            event_tx,
            event_rx,
            transport,
        })
    }

    /// Start the P2P node
    pub async fn start(&mut self) -> Result<()> {
        // Create swarm
        let mut swarm = self.create_swarm().await?;

        // Listen on addresses
        for addr in &self.config.listen_addresses {
            swarm.listen_on(addr.parse()?)?;
        }

        // Connect to bootstrap peers
        for peer in &self.config.bootstrap_peers {
            swarm.dial(peer.parse()?)?;
        }

        // Handle swarm events
        loop {
            tokio::select! {
                event = swarm.next() => {
                    match event {
                        Some(event) => self.handle_swarm_event(event).await?,
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Create libp2p swarm
    async fn create_swarm(&self) -> Result<libp2p::Swarm<P2PBehaviour>> {
        let behaviour = P2PBehaviour::new(
            self.identity.clone(),
            self.event_tx.clone(),
        ).await?;

        let swarm = libp2p::Swarm::new(
            self.transport.clone(),
            behaviour,
            self.identity.peer_id,
        );

        Ok(swarm)
    }

    /// Handle swarm events
    async fn handle_swarm_event(&self, event: SwarmEvent) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                self.event_tx.send(P2PEvent::PeerConnected(peer_id)).await?;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                self.event_tx.send(P2PEvent::PeerDisconnected(peer_id)).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Send data to a peer
    pub async fn send_data(&self, peer_id: PeerId, data: Vec<u8>) -> Result<()> {
        // Implement sending data through swarm
        Ok(())
    }

    /// Receive events
    pub async fn receive_event(&mut self) -> Option<P2PEvent> {
        self.event_rx.recv().await
    }
}

/// P2P behavior implementation
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    /// Noise protocol
    noise: NoiseConfig<X25519Spec>,
    /// Ping protocol for keepalive
    ping: libp2p::ping::Ping,
    /// Identify protocol
    identify: libp2p::identify::Identify,
    /// Kademlia DHT
    kad: libp2p::kad::Kademlia<libp2p::kad::store::MemoryStore>,
    /// Gossipsub
    gossipsub: libp2p::gossipsub::Gossipsub,
}

impl P2PBehaviour {
    /// Create new P2P behavior
    pub async fn new(
        identity: NodeIdentity,
        event_tx: mpsc::Sender<P2PEvent>,
    ) -> Result<Self> {
        let noise = NoiseConfig::xx(
            noise::Keypair::<X25519Spec>::new()
                .into_authentic(&identity.keypair)
                .expect("Signing libp2p-noise static DH keypair failed.")
        ).into_authenticated();

        let ping = libp2p::ping::Ping::default();

        let identify = libp2p::identify::Identify::new(
            libp2p::identify::Config::new("frost-protocol/1.0.0".into(), identity.keypair.public())
        );

        let store = libp2p::kad::store::MemoryStore::new(identity.peer_id);
        let kad = libp2p::kad::Kademlia::new(identity.peer_id, store);

        let gossipsub_config = libp2p::gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(libp2p::gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");

        let gossipsub = libp2p::gossipsub::Gossipsub::new(
            libp2p::gossipsub::MessageAuthenticity::Signed(identity.keypair),
            gossipsub_config
        )?;

        Ok(Self {
            noise,
            ping,
            identify,
            kad,
            gossipsub,
        })
    }
}

#[async_trait]
impl crate::network::Transport for P2PNode {
    async fn init(&mut self, _config: crate::network::TransportConfig) -> Result<()> {
        self.start().await
    }

    async fn connect(&mut self, address: &str) -> Result<crate::network::Peer> {
        // Parse address to multiaddr and connect
        let addr = address.parse()?;
        let peer_id = self.swarm.dial(addr)?;
        
        Ok(crate::network::Peer {
            id: uuid::Uuid::new_v4(), // Map peer_id to UUID
            info: crate::network::PeerInfo {
                address: address.to_string(),
                protocol_version: "1.0.0".to_string(),
                supported_features: vec!["p2p".to_string()],
                chain_ids: vec![],
                node_type: crate::network::NodeType::Validator,
            },
            state: crate::network::PeerState::Connected,
        })
    }

    async fn disconnect(&mut self, peer: &crate::network::Peer) -> Result<()> {
        // Implement disconnect logic
        Ok(())
    }

    async fn send_data(&self, peer: &crate::network::Peer, data: &[u8]) -> Result<usize> {
        // Map UUID back to PeerId and send data
        self.send_data(peer_id_from_uuid(peer.id), data.to_vec()).await?;
        Ok(data.len())
    }

    async fn receive_data(&self, _peer: &crate::network::Peer) -> Result<Vec<u8>> {
        // Implement receive logic using event_rx
        match self.receive_event().await {
            Some(P2PEvent::DataReceived { data, .. }) => Ok(data),
            _ => Ok(vec![]),
        }
    }

    async fn is_connected(&self, _peer: &crate::network::Peer) -> bool {
        // Implement connection check
        true
    }

    fn metrics(&self) -> crate::network::TransportMetrics {
        // Implement metrics collection
        crate::network::TransportMetrics::default()
    }
}

// Helper functions for PeerId <-> UUID mapping
fn peer_id_from_uuid(uuid: uuid::Uuid) -> PeerId {
    // Implement conversion
    PeerId::random()
}

fn uuid_from_peer_id(peer_id: PeerId) -> uuid::Uuid {
    // Implement conversion
    uuid::Uuid::new_v4()
} 