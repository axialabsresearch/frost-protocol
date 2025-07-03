use frost_protocol::{
    network::{
        message_handler::{MessageHandler, MessageProcessor},
        P2PMessage,
        MessageType,
        MessagePriority,
    },
    state::{ChainId, BlockRef},
    Result,
};

use std::time::Duration;
use std::collections::HashMap;

// Mock telemetry implementation
struct MockTelemetry;
impl MockTelemetry {
    async fn record_message_processed(&self) -> Result<()> {
        Ok(())
    }
    
    async fn record_discovery_message_processed(&self) -> Result<()> {
        Ok(())
    }
    
    async fn record_state_transition_processed(&self) -> Result<()> {
        Ok(())
    }
    
    async fn record_finality_signal_processed(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_discovery_message_handling() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    let message = P2PMessage::new(
        MessageType::Discovery,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::Normal,
    );
    
    let result = handler.handle_message(message).await;
    assert!(result.is_ok(), "Discovery message handling failed");
}

#[tokio::test]
async fn test_state_transition_handling() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    let block_ref = BlockRef::new(source_chain.clone(), 1000, [0u8; 32]);
    
    let mut metadata = HashMap::new();
    metadata.insert("source_chain".to_string(), source_chain.to_string());
    metadata.insert("target_chain".to_string(), target_chain.to_string());
    
    let message = P2PMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::High,
    ).with_metadata(metadata);
    
    let result = handler.handle_message(message).await;
    assert!(result.is_ok(), "State transition handling failed");
}

#[tokio::test]
async fn test_finality_signal_handling() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id.clone(), 1000, [0u8; 32]);
    
    let mut metadata = HashMap::new();
    metadata.insert("chain_id".to_string(), chain_id.to_string());
    metadata.insert("block_number".to_string(), "1000".to_string());
    
    let message = P2PMessage::new(
        MessageType::FinalitySignal,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::High,
    ).with_metadata(metadata);
    
    let result = handler.handle_message(message).await;
    assert!(result.is_ok(), "Finality signal handling failed");
}

#[tokio::test]
async fn test_invalid_message_handling() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    // Test with empty payload
    let message = P2PMessage::new(
        MessageType::StateTransition,
        vec![],
        "node1".to_string(),
        None,
        MessagePriority::Normal,
    );
    
    let result = handler.handle_message(message).await;
    assert!(result.is_err(), "Invalid message should fail handling");
}

#[tokio::test]
async fn test_message_validation() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    // Test message size validation
    let large_payload = vec![0; 1024 * 1024 * 11]; // 11MB payload
    let message = P2PMessage::new(
        MessageType::StateTransition,
        large_payload,
        "node1".to_string(),
        None,
        MessagePriority::Normal,
    );
    
    let result = handler.handle_message(message).await;
    assert!(result.is_err(), "Oversized message should fail validation");
}

#[tokio::test]
async fn test_message_processing_order() {
    let telemetry = MockTelemetry;
    let handler = MessageHandler::new(telemetry);
    
    // Create messages with different priorities
    let high_priority = P2PMessage::new(
        MessageType::FinalitySignal,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::High,
    );
    
    let normal_priority = P2PMessage::new(
        MessageType::Discovery,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::Normal,
    );
    
    let low_priority = P2PMessage::new(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "node1".to_string(),
        None,
        MessagePriority::Low,
    );
    
    // Process messages
    let results = tokio::join!(
        handler.handle_message(high_priority),
        handler.handle_message(normal_priority),
        handler.handle_message(low_priority)
    );
    
    assert!(results.0.is_ok() && results.1.is_ok() && results.2.is_ok());
} 