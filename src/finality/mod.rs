mod signal;
mod monitor;
mod error;

pub use signal::{FinalitySignal, BlockRefs};
pub use monitor::FinalityMonitor;
pub use error::FinalityError;

use crate::Result;
