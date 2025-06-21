mod transition;
mod proof;
mod types;
mod error;

pub use transition::StateTransition;
pub use proof::StateProof;
pub use types::{BlockId, BlockRef, StateRoot, ChainId};
pub use error::StateError;

use crate::Result;
