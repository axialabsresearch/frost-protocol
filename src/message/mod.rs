/*!
# Message Module

This module provides the core messaging system for the FROST protocol, enabling
communication and state synchronization between different chains and components.

## Core Components

### Message Types
- Protocol messages
- State transitions
- Chain-specific messages
- System notifications

### Message Handling
- Message validation
- Priority handling
- Chain routing
- Error management

### Message Validation
- Format validation
- Chain validation
- Content verification
- Security checks

## Architecture

The messaging system consists of several key components:

1. **Message Structure**
   ```rust
   pub struct FrostMessage {
       message_type: MessageType,
       payload: Vec<u8>,
       source: String,
       metadata: Option<MessageMetadata>,
   }
   ```
   - Message types
   - Payload data
   - Source tracking
   - Metadata handling

2. **Message Handler**
   ```rust
   pub trait MessageHandler {
       async fn handle_message(&self, message: &FrostMessage) -> Result<()>;
   }
   ```
   - Message processing
   - Chain routing
   - Error handling
   - State updates

3. **Message Validation**
   ```rust
   pub trait MessageValidator {
       async fn validate(&self, message: &FrostMessage) -> Result<bool>;
   }
   ```
   - Format checks
   - Chain validation
   - Content verification
   - Security validation

## Features

### Message Processing
- Type-based routing
- Priority handling
- Chain validation
- Error recovery

### Chain Communication
- Cross-chain messages
- State transitions
- Chain validation
- Routing rules

### Validation System
- Message format
- Chain validation
- Content verification
- Security checks

### Error Handling
- Error types
- Recovery strategies
- Chain-specific errors
- Validation errors

## Best Practices

1. **Message Creation**
   - Proper typing
   - Valid payloads
   - Required metadata
   - Chain validation

2. **Message Handling**
   - Type checking
   - Chain routing
   - Error handling
   - State validation

3. **Message Validation**
   - Format checks
   - Chain validation
   - Content verification
   - Security rules

4. **Error Management**
   - Error categorization
   - Recovery handling
   - Chain-specific errors
   - Validation errors

## Integration

The messaging system integrates with:
1. Chain management
2. State transitions
3. Protocol handlers
4. Security systems
*/

#![allow(unused_imports)]

pub mod types;
pub mod handler;
pub mod validation;
pub mod error;

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
