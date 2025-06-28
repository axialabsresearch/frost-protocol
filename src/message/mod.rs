#![allow(unused_imports)]

mod types;
mod handler;
mod validation;
mod error;

pub use types::{
    FrostMessage,
    MessageType,
    MessageMetadata,
    MessagePriority,
};
pub use handler::MessageHandler;
pub use validation::MessageValidator;
pub use error::MessageError;

use crate::Result;
use crate::state::ChainId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_validation() {
        let source_chain = ChainId::new("ethereum");
        let target_chain = ChainId::new("polygon");
        
        // Valid message with chain info
        let msg = FrostMessage::new_chain_message(
            MessageType::StateTransition,
            vec![1, 2, 3],
            "node1".to_string(),
            None,
            source_chain.clone(),
            target_chain.clone(),
            None,
            None,
            None,
        );
        assert!(msg.validate());

        // Invalid message (empty payload and source)
        let invalid_msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![],
            "".to_string(),
            None,
        );
        assert!(!invalid_msg.validate());

        // Invalid state transition message (missing chain info)
        let invalid_chain_msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![1, 2, 3],
            "node1".to_string(),
            None,
        );
        assert!(!invalid_chain_msg.validate());
    }
}
