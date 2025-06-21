use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::network::{NetworkProtocol, Peer};
use crate::monitoring::MonitoringSystem;
use crate::devtools::{
    NetworkDebugger,
    CapturedPacket,
    NetworkConditions,
    PeerDebugInfo,
    NetworkDebugMetrics,
    NetworkDebugConfig,
};

/// Implementation of network debugger
pub struct NetworkDebuggerImpl {
    network: Arc<dyn NetworkProtocol>,
    monitoring: Arc<RwLock<MonitoringSystem>>,
    config: NetworkDebugConfig,
    packet_capture: RwLock<VecDeque<CapturedPacket>>,
    network_conditions: RwLock<NetworkConditions>,
}

impl NetworkDebuggerImpl {
    /// Create new network debugger
    pub fn new(
        network: Arc<dyn NetworkProtocol>,
        monitoring: Arc<RwLock<MonitoringSystem>>,
        config: NetworkDebugConfig,
    ) -> Self {
        Self {
            network,
            monitoring,
            config,
            packet_capture: RwLock::new(VecDeque::new()),
            network_conditions: RwLock::new(NetworkConditions {
                latency_ms: 0,
                packet_loss_rate: 0.0,
                bandwidth_limit: None,
                jitter_ms: 0,
            }),
        }
    }

    /// Record captured packet
    async fn record_packet(&self, packet: CapturedPacket) -> Result<()> {
        let mut capture = self.packet_capture.write().await;
        
        // Maintain max capture size
        while capture.len() >= self.config.max_capture_size {
            capture.pop_front();
        }
        
        capture.push_back(packet);
        Ok(())
    }

    /// Apply network conditions to connection
    async fn apply_network_conditions(&self, conditions: &NetworkConditions) -> Result<()> {
        // Simulate latency
        if self.config.simulate_latency {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                conditions.latency_ms + fastrand::u64(0..conditions.jitter_ms)
            )).await;
        }

        // Simulate packet loss
        if self.config.simulate_packet_loss {
            if fastrand::f64() < conditions.packet_loss_rate {
                return Ok(());
            }
        }

        Ok(())
    }
}

#[async_trait]
impl NetworkDebugger for NetworkDebuggerImpl {
    async fn start_capture(&mut self) -> Result<()> {
        if !self.config.packet_capture {
            return Err(anyhow::anyhow!("Packet capture not enabled in config"));
        }

        info!("Starting network packet capture");
        
        // Clear existing capture
        self.packet_capture.write().await.clear();
        
        Ok(())
    }

    async fn stop_capture(&mut self) -> Result<()> {
        info!("Stopping network packet capture");
        Ok(())
    }

    async fn get_captured_packets(&self) -> Result<Vec<CapturedPacket>> {
        let capture = self.packet_capture.read().await;
        Ok(capture.iter().cloned().collect())
    }

    async fn simulate_network_conditions(&mut self, conditions: NetworkConditions) -> Result<()> {
        info!("Setting network conditions: {:?}", conditions);
        *self.network_conditions.write().await = conditions;
        Ok(())
    }

    async fn inspect_peer(&self, peer: &Peer) -> Result<PeerDebugInfo> {
        let monitoring = self.monitoring.read().await;
        
        // Get peer metrics from monitoring system
        let metrics = monitoring.get_peer_metrics(peer).await?;
        
        Ok(PeerDebugInfo {
            connection_info: format!("{}:{}", peer.info.address, peer.info.port),
            protocol_version: peer.info.protocol_version.clone(),
            capabilities: peer.info.supported_features.clone(),
            connection_quality: metrics.connection_quality,
            last_seen: metrics.last_seen,
        })
    }

    async fn get_network_metrics(&self) -> Result<NetworkDebugMetrics> {
        let monitoring = self.monitoring.read().await;
        
        // Get network metrics from monitoring system
        let metrics = monitoring.get_network_metrics().await?;
        
        Ok(NetworkDebugMetrics {
            active_connections: metrics.active_connections,
            bytes_sent: metrics.bytes_sent,
            bytes_received: metrics.bytes_received,
            packets_dropped: metrics.packets_dropped,
            average_latency: metrics.average_latency,
        })
    }
} 