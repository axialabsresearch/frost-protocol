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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_validation() {
        let msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![1, 2, 3],
            "node1".to_string(),
            None,
        );
        assert!(msg.validate());

        let invalid_msg = FrostMessage::new(
            MessageType::StateTransition,
            vec![],
            "".to_string(),
            None,
        );
        assert!(!invalid_msg.validate());
    }
}
