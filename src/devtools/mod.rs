/*!
# Developer Tools Module

The devtools module provides a comprehensive suite of development and debugging tools for the FROST protocol.
It includes network debugging, extension debugging, performance profiling, and documentation generation capabilities.

## Core Components

* `NetworkDebugger` - Network traffic inspection and simulation
* `ExtensionDebugger` - Extension state inspection and hot reloading
* `Profiler` - Performance profiling and metrics collection
* `DocGenerator` - Documentation generation for protocol, extensions, and APIs

## Usage Example

```rust
use frost_protocol::devtools::{DevTools, DevToolsConfig};

async fn setup_devtools(config: DevToolsConfig) -> Result<()> {
    let mut dev_tools = MyDevTools::new();
    dev_tools.initialize(config).await?;
    dev_tools.start().await?;
    
    // Use network debugger
    let network = dev_tools.network_debugger();
    network.start_capture().await?;
    
    // Profile performance
    let profiler = dev_tools.profiler();
    profiler.start_profiling(config).await?;
    
    Ok(())
}
```

## Features

### Network Debugging
- Packet capture and inspection
- Network condition simulation (latency, packet loss)
- Connection quality monitoring
- Protocol-level metrics

### Extension Debugging  
- Hot reload capability
- State inspection
- Performance profiling
- Error tracking

### Performance Profiling
- CPU and memory profiling
- Stack trace collection
- Allocation tracking
- Custom metric collection

### Documentation Generation
- Protocol documentation
- Extension documentation
- API documentation
- Developer guides

## Configuration

The devtools can be configured via the `DevToolsConfig` struct which allows enabling/disabling
specific features and setting parameters for network debugging, profiling, etc.

## CLI Interface

A command-line interface is provided via the `cli` module for interactive debugging and development.
See the `cli.rs` module for available commands.

## Metrics and Monitoring

The devtools integrate with the monitoring system to provide real-time metrics and alerts for
development and debugging purposes.
*/

use std::sync::Arc;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::network::{NetworkProtocol, Peer};
use crate::extensions::{ExtensionManager, ExtensionId};
use crate::monitoring::MonitoringSystem;

/// Core interface for developer tools
#[async_trait::async_trait]
pub trait DevTools: Send + Sync {
    /// Initialize developer tools
    async fn initialize(&mut self, config: DevToolsConfig) -> Result<()>;

    /// Start developer tools services
    async fn start(&mut self) -> Result<()>;

    /// Stop developer tools services
    async fn stop(&mut self) -> Result<()>;

    /// Get network debugger
    fn network_debugger(&self) -> Arc<NetworkDebugger>;

    /// Get extension debugger
    fn extension_debugger(&self) -> Arc<ExtensionDebugger>;

    /// Get performance profiler
    fn profiler(&self) -> Arc<Profiler>;

    /// Get documentation generator
    fn doc_generator(&self) -> Arc<DocGenerator>;
}

/// Configuration for developer tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Enable debug logging
    pub debug_logging: bool,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Enable documentation generation
    pub enable_docs: bool,
    /// Network debugging configuration
    pub network_debug: NetworkDebugConfig,
    /// Extension debugging configuration
    pub extension_debug: ExtensionDebugConfig,
}

/// Network debugging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDebugConfig {
    /// Enable packet capture
    pub packet_capture: bool,
    /// Maximum packet capture size
    pub max_capture_size: usize,
    /// Enable latency simulation
    pub simulate_latency: bool,
    /// Enable packet loss simulation
    pub simulate_packet_loss: bool,
}

/// Extension debugging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDebugConfig {
    /// Enable extension hot reloading
    pub hot_reload: bool,
    /// Enable extension state inspection
    pub state_inspection: bool,
    /// Enable extension profiling
    pub enable_profiling: bool,
}

/// Network debugging interface
#[async_trait::async_trait]
pub trait NetworkDebugger: Send + Sync {
    /// Start packet capture
    async fn start_capture(&mut self) -> Result<()>;

    /// Stop packet capture
    async fn stop_capture(&mut self) -> Result<()>;

    /// Get captured packets
    async fn get_captured_packets(&self) -> Result<Vec<CapturedPacket>>;

    /// Simulate network conditions
    async fn simulate_network_conditions(&mut self, conditions: NetworkConditions) -> Result<()>;

    /// Inspect peer connection
    async fn inspect_peer(&self, peer: &Peer) -> Result<PeerDebugInfo>;

    /// Get network metrics
    async fn get_network_metrics(&self) -> Result<NetworkDebugMetrics>;
}

/// Extension debugging interface
#[async_trait::async_trait]
pub trait ExtensionDebugger: Send + Sync {
    /// Enable hot reloading for extension
    async fn enable_hot_reload(&mut self, extension_id: &ExtensionId) -> Result<()>;

    /// Inspect extension state
    async fn inspect_extension_state(&self, extension_id: &ExtensionId) -> Result<ExtensionDebugInfo>;

    /// Profile extension performance
    async fn profile_extension(&self, extension_id: &ExtensionId) -> Result<ExtensionProfile>;

    /// Get extension metrics
    async fn get_extension_metrics(&self, extension_id: &ExtensionId) -> Result<ExtensionDebugMetrics>;
}

/// Performance profiling interface
#[async_trait::async_trait]
pub trait Profiler: Send + Sync {
    /// Start profiling session
    async fn start_profiling(&mut self, config: ProfilingConfig) -> Result<()>;

    /// Stop profiling session
    async fn stop_profiling(&mut self) -> Result<ProfilingResults>;

    /// Get current profiling metrics
    async fn get_metrics(&self) -> Result<ProfilingMetrics>;

    /// Generate profiling report
    async fn generate_report(&self, format: ReportFormat) -> Result<Vec<u8>>;
}

/// Documentation generation interface
#[async_trait::async_trait]
pub trait DocGenerator: Send + Sync {
    /// Generate protocol documentation
    async fn generate_protocol_docs(&self, config: DocConfig) -> Result<()>;

    /// Generate extension documentation
    async fn generate_extension_docs(&self, extension_id: &ExtensionId) -> Result<()>;

    /// Generate API documentation
    async fn generate_api_docs(&self, config: DocConfig) -> Result<()>;

    /// Generate developer guide
    async fn generate_developer_guide(&self, config: DocConfig) -> Result<()>;
}

// Types for network debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedPacket {
    pub timestamp: u64,
    pub source: String,
    pub destination: String,
    pub size: usize,
    pub protocol: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditions {
    pub latency_ms: u64,
    pub packet_loss_rate: f64,
    pub bandwidth_limit: Option<u64>,
    pub jitter_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDebugInfo {
    pub connection_info: String,
    pub protocol_version: String,
    pub capabilities: Vec<String>,
    pub connection_quality: f64,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDebugMetrics {
    pub active_connections: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_dropped: u64,
    pub average_latency: f64,
}

// Types for extension debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDebugInfo {
    pub state: String,
    pub config: String,
    pub memory_usage: u64,
    pub active_handlers: Vec<String>,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionProfile {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub handler_latencies: HashMap<String, f64>,
    pub error_rates: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDebugMetrics {
    pub messages_processed: u64,
    pub average_processing_time: f64,
    pub error_count: u64,
    pub memory_usage: u64,
}

// Types for profiling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub sample_rate: u64,
    pub max_samples: usize,
    pub include_stack_traces: bool,
    pub profile_allocations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingResults {
    pub duration: u64,
    pub samples: Vec<ProfilingSample>,
    pub memory_profile: Option<MemoryProfile>,
    pub thread_profile: Option<ThreadProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub gc_stats: Option<GCStats>,
    pub thread_stats: ThreadStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    JSON,
    HTML,
    Flamegraph,
    Chrome,
}

// Types for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocConfig {
    pub output_format: DocFormat,
    pub output_dir: String,
    pub include_examples: bool,
    pub include_diagrams: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocFormat {
    Markdown,
    HTML,
    PDF,
    ManPage,
}

mod cli;
mod debugger;
mod profiler;
mod docs;

pub use cli::DevToolsCLI;
pub use debugger::{NetworkDebuggerImpl, ExtensionDebuggerImpl};
pub use profiler::ProfilerImpl;
pub use docs::DocGeneratorImpl;