use frost_protocol::message::{
    validator::{MessageValidator, ValidatorConfig, ValidationMetrics},
    FrostMessage, MessageError, MessagePayload,
};

#[tokio::test]
async fn test_ethereum_message_validation() {
    let config = ValidatorConfig {
        max_message_size: 1024,
        chain_params: serde_json::json!({
            "network": "mainnet",
            "contract_address": "0x1234567890123456789012345678901234567890",
        }),
    };
    
    let mut validator = EthereumValidator::new(config);
    
    // Test valid message
    let valid_msg = FrostMessage {
        source_chain: "ethereum".into(),
        target_chain: "solana".into(),
        nonce: 1,
        payload: MessagePayload::Ethereum {
            calldata: vec![0u8; 512],
            gas_limit: 100000,
            value: 0,
        },
    };
    
    assert!(validator.validate_message(&valid_msg).await.is_ok());
    
    // Test message too large
    let large_msg = FrostMessage {
        source_chain: "ethereum".into(),
        target_chain: "solana".into(),
        nonce: 2,
        payload: MessagePayload::Ethereum {
            calldata: vec![0u8; 2048],
            gas_limit: 100000,
            value: 0,
        },
    };
    
    assert!(validator.validate_message(&large_msg).await.is_err());
    
    // Test invalid message type
    let invalid_msg = FrostMessage {
        source_chain: "ethereum".into(),
        target_chain: "solana".into(),
        nonce: 3,
        payload: MessagePayload::Solana {
            instruction: vec![0u8; 512],
            accounts: vec![],
        },
    };
    
    assert!(validator.validate_message(&invalid_msg).await.is_err());
    
    // Check metrics
    let metrics = validator.get_metrics().await;
    assert_eq!(metrics.total_validated, 3);
    assert!(metrics.failed_validations > 0);
    assert!(metrics.avg_validation_time > 0.0);
}

#[tokio::test]
async fn test_solana_message_validation() {
    let config = ValidatorConfig {
        max_message_size: 1024,
        chain_params: serde_json::json!({
            "network": "mainnet-beta",
            "program_id": "FrostProtocol1111111111111111111111111111111",
        }),
    };
    
    let mut validator = SolanaValidator::new(config);
    
    // Test valid message
    let valid_msg = FrostMessage {
        source_chain: "solana".into(),
        target_chain: "ethereum".into(),
        nonce: 1,
        payload: MessagePayload::Solana {
            instruction: vec![0u8; 512],
            accounts: vec![],
        },
    };
    
    assert!(validator.validate_message(&valid_msg).await.is_ok());
    
    // Test message too large
    let large_msg = FrostMessage {
        source_chain: "solana".into(),
        target_chain: "ethereum".into(),
        nonce: 2,
        payload: MessagePayload::Solana {
            instruction: vec![0u8; 2048],
            accounts: vec![],
        },
    };
    
    assert!(validator.validate_message(&large_msg).await.is_err());
    
    // Test invalid message type
    let invalid_msg = FrostMessage {
        source_chain: "solana".into(),
        target_chain: "ethereum".into(),
        nonce: 3,
        payload: MessagePayload::Ethereum {
            calldata: vec![0u8; 512],
            gas_limit: 100000,
            value: 0,
        },
    };
    
    assert!(validator.validate_message(&invalid_msg).await.is_err());
    
    // Check metrics
    let metrics = validator.get_metrics().await;
    assert_eq!(metrics.total_validated, 3);
    assert!(metrics.failed_validations > 0);
    assert!(metrics.avg_validation_time > 0.0);
}

#[tokio::test]
async fn test_cosmos_message_validation() {
    let config = ValidatorConfig {
        max_message_size: 1024,
        chain_params: serde_json::json!({
            "chain_id": "cosmoshub-4",
            "module_name": "frost",
        }),
    };
    
    let mut validator = CosmosValidator::new(config);
    
    // Test valid message
    let valid_msg = FrostMessage {
        source_chain: "cosmos".into(),
        target_chain: "ethereum".into(),
        nonce: 1,
        payload: MessagePayload::Cosmos {
            msg: vec![0u8; 512],
            funds: vec![],
        },
    };
    
    assert!(validator.validate_message(&valid_msg).await.is_ok());
    
    // Test message too large
    let large_msg = FrostMessage {
        source_chain: "cosmos".into(),
        target_chain: "ethereum".into(),
        nonce: 2,
        payload: MessagePayload::Cosmos {
            msg: vec![0u8; 2048],
            funds: vec![],
        },
    };
    
    assert!(validator.validate_message(&large_msg).await.is_err());
    
    // Test invalid message type
    let invalid_msg = FrostMessage {
        source_chain: "cosmos".into(),
        target_chain: "ethereum".into(),
        nonce: 3,
        payload: MessagePayload::Ethereum {
            calldata: vec![0u8; 512],
            gas_limit: 100000,
            value: 0,
        },
    };
    
    assert!(validator.validate_message(&invalid_msg).await.is_err());
    
    // Check metrics
    let metrics = validator.get_metrics().await;
    assert_eq!(metrics.total_validated, 3);
    assert!(metrics.failed_validations > 0);
    assert!(metrics.avg_validation_time > 0.0);
} 