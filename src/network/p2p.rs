#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// temporary
#![allow(deprecated)]

use async_trait::async_trait;
use ::futures::stream::StreamExt;
use libp2p::{
    core::{
        transport::PortUse,
        muxing::StreamMuxerBox,
        upgrade,
        transport::Boxed,
        upgrade::{SelectUpgrade, Version},
        Multiaddr,
        Endpoint,
    },
    identity::{self, Keypair},
    swarm::{
        self,
        NetworkBehaviour,
        SwarmEvent,
        ConnectionHandler,
        ConnectionDenied,
        ToSwarm,
        FromSwarm,
        THandlerOutEvent,
        THandlerInEvent,
        derive_prelude::*,
        ConnectionId,
        OneShotHandler,
        dummy,
        ConnectionHandlerEvent,
    },
    StreamProtocol,
    SwarmBuilder,
    PeerId,
    noise,
    yamux,
    tcp,
    ping,
    identify,
    kad::{self, store::MemoryStore},
    gossipsub::{self, MessageAuthenticity, ValidationMode},
    Transport,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{mpsc, broadcast};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::{Result, Error};
use crate::network::peer::{NodeType, PeerState, Peer};
use crate::network::transport::TransportMetrics;
use crate::network::ProtocolConfig;
use thiserror::Error;
use void::Void;
use std::convert::Infallible;

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
#[derive(Debug, Clone)]
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
    event_tx: broadcast::Sender<P2PEvent>,
    /// Event receiver
    event_rx: broadcast::Receiver<P2PEvent>,
    /// Transport
    transport: Boxed<(PeerId, StreamMuxerBox)>,
    /// Swarm instance
    swarm: Arc<RwLock<swarm::Swarm<P2PBehaviour>>>,
}

// Implement Send + Sync for P2PNode
unsafe impl Send for P2PNode {}
unsafe impl Sync for P2PNode {}

impl P2PNode {
    /// Create a new P2P node
    pub async fn new(config: P2PConfig) -> Result<Self> {
        let identity = NodeIdentity::new();
        let (event_tx, event_rx) = broadcast::channel(1000);
        
        // Create transport with noise encryption
        let noise_keys = identity.keypair.clone();
        let auth_config = noise::Config::new(&noise_keys)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = tcp::tokio::Transport::new(tcp::Config::default().port_reuse(true))
            .upgrade(Version::V1)
            .authenticate(auth_config)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create swarm
        let behaviour = P2PBehaviour::new(identity.clone(), event_tx.clone()).await?;
        let swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true),
                noise::Config::new,
                yamux::Config::default
            )
            .map_err(|e| Error::Network(e.to_string()))?
            .with_behaviour(move |_| behaviour)
            .map_err(|e| Error::Network(e.to_string()))?
            .build();

        Ok(Self {
            config,
            identity,
            event_tx,
            event_rx,
            transport,
            swarm: Arc::new(RwLock::new(swarm)),
        })
    }

    /// Handle swarm events
    async fn handle_swarm_event(&self, event: SwarmEvent<P2PBehaviourEvent>) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                self.event_tx.send(P2PEvent::PeerConnected(peer_id))
                    .map_err(|e| Error::Network(format!("Failed to send event: {}", e)))?;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                self.event_tx.send(P2PEvent::PeerDisconnected(peer_id))
                    .map_err(|e| Error::Network(format!("Failed to send event: {}", e)))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Start the P2P node
    pub async fn start(&mut self) -> Result<()> {
        let mut swarm = self.swarm.write();
        
        // Listen on addresses
        for addr_str in &self.config.listen_addresses {
            let addr: Multiaddr = addr_str.parse()
                .map_err(|e| Error::Network(format!("Invalid listen address {}: {}", addr_str, e)))?;
            swarm.listen_on(addr)
                .map_err(|e| Error::Network(format!("Failed to listen: {}", e)))?;
        }

        // Connect to bootstrap peers
        for peer_addr in &self.config.bootstrap_peers {
            let addr: Multiaddr = peer_addr.parse()
                .map_err(|e| Error::Network(format!("Invalid peer address {}: {}", peer_addr, e)))?;
            swarm.dial(addr)
                .map_err(|e| Error::Network(format!("Failed to dial: {}", e)))?;
        }

        // Handle swarm events
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
            }
        }
    }

    /// Send data to a peer
    pub async fn send_data(&self, peer_id: PeerId, data: Vec<u8>) -> Result<()> {
        // Implement sending data through swarm
        Ok(())
    }

    /// Receive events
    pub async fn receive_event(&mut self) -> Option<P2PEvent> {
        self.event_rx.recv().await.ok()
    }

    async fn receive_data(&self, _peer: &Peer) -> Result<Vec<u8>> {
        let mut event_rx = self.event_tx.subscribe();
        match event_rx.recv().await {
            Ok(P2PEvent::DataReceived { data, .. }) => Ok(data),
            _ => Ok(vec![]),
        }
    }
}

/// P2P behavior implementation
pub struct P2PBehaviour {
    /// Ping protocol for keepalive
    ping: ping::Behaviour,
    /// Identify protocol
    identify: identify::Behaviour,
    /// Kademlia DHT
    kad: kad::Behaviour<MemoryStore>,
    /// Gossipsub
    gossipsub: gossipsub::Behaviour,
}

impl NetworkBehaviour for P2PBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;
    type ToSwarm = P2PBehaviourEvent;

    fn on_swarm_event(&mut self, event: FromSwarm) {
        // Forward events to sub-behaviours
        self.ping.on_swarm_event(event.clone());
        self.identify.on_swarm_event(event.clone());
        self.kad.on_swarm_event(event.clone());
        self.gossipsub.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        // No custom handling needed for dummy handler events
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<ToSwarm<Self::ToSwarm, Infallible>> {
        // Poll sub-behaviours
        if let std::task::Poll::Ready(ToSwarm::GenerateEvent(event)) = self.ping.poll(cx) {
            return std::task::Poll::Ready(ToSwarm::GenerateEvent(P2PBehaviourEvent::Ping(event)));
        }
        
        if let std::task::Poll::Ready(ToSwarm::GenerateEvent(event)) = self.identify.poll(cx) {
            return std::task::Poll::Ready(ToSwarm::GenerateEvent(P2PBehaviourEvent::Identify(event)));
        }
        
        if let std::task::Poll::Ready(ToSwarm::GenerateEvent(event)) = self.kad.poll(cx) {
            return std::task::Poll::Ready(ToSwarm::GenerateEvent(P2PBehaviourEvent::Kad(event)));
        }
        
        if let std::task::Poll::Ready(ToSwarm::GenerateEvent(event)) = self.gossipsub.poll(cx) {
            return std::task::Poll::Ready(ToSwarm::GenerateEvent(P2PBehaviourEvent::Gossipsub(event)));
        }

        std::task::Poll::Pending
    }

    fn handle_established_inbound_connection(
        &mut self,
        connection: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> std::result::Result<dummy::ConnectionHandler, ConnectionDenied> {
        // Return a new dummy handler for inbound connections
        Ok(dummy::ConnectionHandler)
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
        port_use: PortUse,
    ) -> std::result::Result<dummy::ConnectionHandler, ConnectionDenied> {
        // Return a new dummy handler for outbound connections
        Ok(dummy::ConnectionHandler)
    }
}

/// Events emitted by the P2P behaviour
#[derive(Debug)]
pub enum P2PBehaviourEvent {
    Ping(ping::Event),
    Identify(identify::Event),
    Kad(kad::Event),
    Gossipsub(gossipsub::Event),
}

impl From<ping::Event> for P2PBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        P2PBehaviourEvent::Ping(event)
    }
}

impl From<identify::Event> for P2PBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        P2PBehaviourEvent::Identify(event)
    }
}

impl From<kad::Event> for P2PBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        P2PBehaviourEvent::Kad(event)
    }
}

impl From<gossipsub::Event> for P2PBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        P2PBehaviourEvent::Gossipsub(event)
    }
}

impl P2PBehaviour {
    /// Create new P2P behavior
    pub async fn new(
        identity: NodeIdentity,
        event_tx: broadcast::Sender<P2PEvent>,
    ) -> std::result::Result<Self, Error> {
        let ping = ping::Behaviour::new(ping::Config::new());

        let identify = identify::Behaviour::new(
            identify::Config::new("frost-protocol/1.0.0".into(), identity.keypair.public())
        );

        let store = MemoryStore::new(identity.peer_id);
        /* let kad_config = kad::Config::default()
            .with_protocol_name("/frost/kad/1.0.0")
            .set_record_ttl(Some(Duration::from_secs(24 * 60 * 60))) // 24 hours
            .set_publication_interval(Some(Duration::from_secs(12 * 60 * 60))) // 12 hours
            .set_provider_record_ttl(Some(Duration::from_secs(24 * 60 * 60))) // 24 hours 
            .set_provider_publication_interval(Some(Duration::from_secs(12 * 60 * 60))); // 12 hours
        */

        let mut kad_config = kad::Config::new(StreamProtocol::new("/frost/kad/1.0.0"));
        kad_config.set_record_ttl(Some(Duration::from_secs(24 * 60 * 60))); // 24 hours
        kad_config.set_publication_interval(Some(Duration::from_secs(12 * 60 * 60))); // 12 hours
        kad_config.set_provider_record_ttl(Some(Duration::from_secs(24 * 60 * 60))); // 24 hours
        kad_config.set_provider_publication_interval(Some(Duration::from_secs(12 * 60 * 60))); // 12 hours
            
        let kad = kad::Behaviour::with_config(identity.peer_id, store, kad_config);

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .build()
            .expect("Valid config");

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(identity.keypair),
            gossipsub_config
        ).map_err(|e| Error::Network(e.to_string()))?;

        Ok(Self {
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
        let mut swarm = self.swarm.write();
        
        // Listen on addresses
        for addr_str in &self.config.listen_addresses {
            let addr: Multiaddr = addr_str.parse()
                .map_err(|e| Error::Network(format!("Invalid listen address {}: {}", addr_str, e)))?;
            swarm.listen_on(addr)
                .map_err(|e| Error::Network(format!("Failed to listen: {}", e)))?;
        }

        // Connect to bootstrap peers
        for peer_addr in &self.config.bootstrap_peers {
            let addr: Multiaddr = peer_addr.parse()
                .map_err(|e| Error::Network(format!("Invalid peer address {}: {}", peer_addr, e)))?;
            swarm.dial(addr)
                .map_err(|e| Error::Network(format!("Failed to dial: {}", e)))?;
        }

        Ok(())
    }

    async fn connect(&mut self, address: &str) -> Result<Peer> {
        let addr: Multiaddr = address.parse()
            .map_err(|e| Error::Network(format!("Invalid address: {}", e)))?;
        
        let mut swarm = self.swarm.write();
        swarm.dial(addr)
            .map_err(|e| Error::Network(format!("Failed to dial: {}", e)))?;
        
        Ok(Peer {
            id: uuid::Uuid::new_v4(),
            info: crate::network::PeerInfo {
                address: address.to_string(),
                protocol_version: "1.0.0".to_string(),
                supported_features: vec!["p2p".to_string()],
                chain_ids: vec![],
                node_type: NodeType::Validator,
            },
            state: PeerState::Connected,
        })
    }

    async fn disconnect(&mut self, _peer: &Peer) -> Result<()> {
        // Implement disconnect logic
        Ok(())
    }

    async fn send_data(&self, peer: &Peer, data: &[u8]) -> Result<usize> {
        let peer_id = peer_id_from_uuid(peer.id);
        let data_len = data.len();
        let data_vec = data.to_vec();
        
        {
            let _swarm = self.swarm.read();
            // Additional swarm operations if needed
        }
        
        self.send_data(peer_id, data_vec).await?;
        Ok(data_len)
    }

    async fn receive_data(&self, _peer: &Peer) -> Result<Vec<u8>> {
        let mut event_rx = self.event_tx.subscribe();
        match event_rx.recv().await {
            Ok(P2PEvent::DataReceived { data, .. }) => Ok(data),
            _ => Ok(vec![]),
        }
    }

    async fn is_connected(&self, _peer: &Peer) -> bool {
        let _swarm = self.swarm.read();
        // Implement connection check using swarm
        true
    }

    fn metrics(&self) -> TransportMetrics {
        TransportMetrics::default()
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

// Add NetworkError type
#[derive(Debug, thiserror::Error, Clone)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Transport error: {0}")]
    TransportError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
} 