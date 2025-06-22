mod router;
mod strategy;
mod topology;

pub use router::{MessageRouter, RouterConfig};
pub use strategy::{RoutingStrategy, DefaultStrategy};
pub use topology::{NetworkTopology, TopologyNode};

use crate::Result;
