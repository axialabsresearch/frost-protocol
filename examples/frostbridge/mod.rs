pub mod types;
pub mod config;
pub mod bridges;
pub mod transfer;

pub use crate::frostbridge::{
    types::*,
    config::*,
    bridges::*,
    transfer::*,
};

use frost_protocol::{
    finality::{FinalityVerifier, EthereumVerifier, SubstrateVerifier, FinalityConfig},
    network::{NetworkConfig, BasicNetwork, SharedNetwork},
    routing::{RoutingConfig, BasicRouter},
    Result, Error,
};

// Re-export the public interface
pub use crate::frostbridge::initialize_components;
pub use crate::frostbridge::get_network_endpoints;
pub use crate::frostbridge::is_dry_run;
pub use crate::frostbridge::print_dry_run_info; 