use frost_protocol::{
    message::{
        FrostMessage,
        MessageType,
        MessageValidator,
        validation::{ValidationRule, ValidationSeverity, ValidationResult, ValidationStage, ValidationFailure},
    },
    state::{ChainId, StateTransition, BlockId},
    Result,
};

use async_trait::async_trait;

// Custom validation rule implementation
struct PayloadSizeRule {
    max_size: usize,
}

#[async_trait]
impl ValidationRule for PayloadSizeRule {
    fn rule_id(&self) -> &str {
        "payload_size"
    }

    fn description(&self) -> &str {
        "Validates message payload size"
    }

    async fn validate(&self, message: &FrostMessage) -> Result<bool> {
        Ok(message.payload.len() <= self.max_size)
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Error
    }
}

// Custom message validator implementation
#[derive(Default)]
struct TestMessageValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

#[async_trait]
impl MessageValidator for TestMessageValidator {
    async fn validate(&self, message: &FrostMessage) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            rules_passed: vec![],
            rules_failed: vec![],
            stage: ValidationStage::PreValidation,
            duration_ms: 0,
            metadata: None,
        };

        for rule in &self.rules {
            if rule.validate(message).await? {
                result.rules_passed.push(rule.rule_id().to_string());
            } else {
                result.is_valid = false;
                result.rules_failed.push(ValidationFailure {
                    rule_id: rule.rule_id().to_string(),
                    reason: "Rule validation failed".to_string(),
                    severity: rule.severity(),
                });
            }
        }

        Ok(result)
    }

    fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    fn remove_rule(&mut self, rule_id: &str) {
        self.rules.retain(|r| r.rule_id() != rule_id);
    }
}

#[tokio::test]
async fn test_basic_message_validation() {
    // Test basic message creation and validation
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "test_node".to_string(),
        None,
        source_chain,
        target_chain,
        None,
        None,
        None,
        None,
    );
    
    assert!(msg.validate(), "Basic message validation failed");
}

#[tokio::test]
async fn test_invalid_message_validation() {
    // Test invalid message cases
    let msg = FrostMessage::new(
        MessageType::StateTransition,
        vec![],  // Empty payload
        "".to_string(), // Empty source
        None,
    );
    
    assert!(!msg.validate(), "Invalid message validation should fail");
}

#[tokio::test]
async fn test_validation_rules() {
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "test_node".to_string(),
        None,
        source_chain.clone(),
        target_chain.clone(),
        None,
        None,
        None,
        None,
    );

    let mut validator = TestMessageValidator::default();
    
    // Add payload size rule
    validator.add_rule(Box::new(PayloadSizeRule { max_size: 10 }));
    
    // Test validation with rules
    let result = validator.validate(&msg).await.unwrap();
    assert!(result.is_valid, "Message should pass validation rules");
    assert!(!result.rules_passed.is_empty(), "Should have passed rules");
    
    // Test with larger payload
    let large_msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![0; 20],  // Payload larger than max_size
        "test_node".to_string(),
        None,
        source_chain,
        target_chain,
        None,
        None,
        None,
        None,
    );
    
    let result = validator.validate(&large_msg).await.unwrap();
    assert!(!result.is_valid, "Large message should fail validation");
    assert!(!result.rules_failed.is_empty(), "Should have failed rules");
}

#[tokio::test]
async fn test_chain_specific_validation() {
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    
    let source = BlockId::Composite {
        number: 1000,
        hash: [0u8; 32],
    };
    
    let target = BlockId::Composite {
        number: 1000,
        hash: [0u8; 32],
    };
    
    let state_transition = StateTransition::new(
        source,
        target,
        vec![4, 5, 6],
    );
    
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        vec![1, 2, 3],
        "test_node".to_string(),
        None,
        source_chain.clone(),
        target_chain.clone(),
        Some(state_transition),
        None,
        None,
        None,
    );
    
    let mut validator = TestMessageValidator::default();
    validator.add_rule(Box::new(PayloadSizeRule { max_size: 10 }));
    
    let result = validator.validate(&msg).await.unwrap();
    assert!(result.is_valid, "Chain-specific validation failed");
    assert!(!result.rules_passed.is_empty(), "Should have passed rules");
}

#[tokio::test]
async fn test_message_size_limits() {
    let source_chain = ChainId::new("ethereum");
    let target_chain = ChainId::new("polygon");
    
    let large_payload = vec![0; 1024 * 1024]; // 1MB payload
    let msg = FrostMessage::new_chain_message(
        MessageType::StateTransition,
        large_payload,
        "test_node".to_string(),
        None,
        source_chain,
        target_chain,
        None,
        None,
        None,
        None,
    );
    
    let mut validator = TestMessageValidator::default();
    validator.add_rule(Box::new(PayloadSizeRule { max_size: 1024 })); // 1KB limit
    
    let result = validator.validate(&msg).await.unwrap();
    assert!(!result.is_valid, "Large message should fail validation");
    assert!(!result.rules_failed.is_empty(), "Should have failed size rule");
} 