use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::state::BlockRef;
use crate::message::{FrostMessage, MessageError};

/// Message validation metrics
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Total messages validated
    pub total_validated: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Average validation time
    pub avg_validation_time: f64,
    /// Chain-specific metrics
    pub chain_metrics: HashMap<String, serde_json::Value>,
}

/// Message validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Chain-specific parameters
    pub chain_params: serde_json::Value,
}

/// Message validator trait
#[async_trait]
pub trait MessageValidator: Send + Sync {
    /// Validate a message
    async fn validate_message(&self, message: &FrostMessage) -> Result<(), MessageError>;
    
    /// Get validation metrics
    async fn get_metrics(&self) -> ValidationMetrics;
    
    /// Update validator configuration
    async fn update_config(&mut self, config: ValidatorConfig) -> Result<(), MessageError>;
}

/// Ethereum message validator
pub struct EthereumValidator {
    config: ValidatorConfig,
    metrics: ValidationMetrics,
}

impl EthereumValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
            metrics: ValidationMetrics::default(),
        }
    }
    
    fn validate_calldata(&self, calldata: &[u8]) -> Result<(), MessageError> {
        if calldata.len() > self.config.max_message_size {
            return Err(MessageError::InvalidFormat(
                format!("Calldata size {} exceeds maximum {}", 
                    calldata.len(), self.config.max_message_size)
            ));
        }
        
        // TODO: Add more Ethereum-specific calldata validation
        Ok(())
    }
}

#[async_trait]
impl MessageValidator for EthereumValidator {
    async fn validate_message(&self, message: &FrostMessage) -> Result<(), MessageError> {
        let start = std::time::Instant::now();
        
        let result = match message.payload {
            MessagePayload::Ethereum { ref calldata, .. } => {
                self.validate_calldata(calldata)
            }
            _ => Err(MessageError::InvalidFormat("Not an Ethereum message".into())),
        };
        
        // Update metrics
        let mut metrics = self.metrics.clone();
        let duration = start.elapsed().as_secs_f64();
        
        metrics.total_validated += 1;
        metrics.avg_validation_time = (metrics.avg_validation_time * (metrics.total_validated - 1) as f64
            + duration) / metrics.total_validated as f64;
            
        if result.is_err() {
            metrics.failed_validations += 1;
        }
        
        result
    }
    
    async fn get_metrics(&self) -> ValidationMetrics {
        self.metrics.clone()
    }
    
    async fn update_config(&mut self, config: ValidatorConfig) -> Result<(), MessageError> {
        self.config = config;
        Ok(())
    }
}

/// Solana message validator
pub struct SolanaValidator {
    config: ValidatorConfig,
    metrics: ValidationMetrics,
}

impl SolanaValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
            metrics: ValidationMetrics::default(),
        }
    }
    
    fn validate_instruction(&self, instruction: &[u8]) -> Result<(), MessageError> {
        if instruction.len() > self.config.max_message_size {
            return Err(MessageError::InvalidFormat(
                format!("Instruction size {} exceeds maximum {}", 
                    instruction.len(), self.config.max_message_size)
            ));
        }
        
        // TODO: Add more Solana-specific instruction validation
        Ok(())
    }
}

#[async_trait]
impl MessageValidator for SolanaValidator {
    async fn validate_message(&self, message: &FrostMessage) -> Result<(), MessageError> {
        let start = std::time::Instant::now();
        
        let result = match message.payload {
            MessagePayload::Solana { ref instruction, .. } => {
                self.validate_instruction(instruction)
            }
            _ => Err(MessageError::InvalidFormat("Not a Solana message".into())),
        };
        
        // Update metrics
        let mut metrics = self.metrics.clone();
        let duration = start.elapsed().as_secs_f64();
        
        metrics.total_validated += 1;
        metrics.avg_validation_time = (metrics.avg_validation_time * (metrics.total_validated - 1) as f64
            + duration) / metrics.total_validated as f64;
            
        if result.is_err() {
            metrics.failed_validations += 1;
        }
        
        result
    }
    
    async fn get_metrics(&self) -> ValidationMetrics {
        self.metrics.clone()
    }
    
    async fn update_config(&mut self, config: ValidatorConfig) -> Result<(), MessageError> {
        self.config = config;
        Ok(())
    }
}

/// Cosmos message validator
pub struct CosmosValidator {
    config: ValidatorConfig,
    metrics: ValidationMetrics,
}

impl CosmosValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
            metrics: ValidationMetrics::default(),
        }
    }
    
    fn validate_msg(&self, msg: &[u8]) -> Result<(), MessageError> {
        if msg.len() > self.config.max_message_size {
            return Err(MessageError::InvalidFormat(
                format!("Message size {} exceeds maximum {}", 
                    msg.len(), self.config.max_message_size)
            ));
        }
        
        // TODO: Add more Cosmos-specific message validation
        Ok(())
    }
}

#[async_trait]
impl MessageValidator for CosmosValidator {
    async fn validate_message(&self, message: &FrostMessage) -> Result<(), MessageError> {
        let start = std::time::Instant::now();
        
        let result = match message.payload {
            MessagePayload::Cosmos { ref msg, .. } => {
                self.validate_msg(msg)
            }
            _ => Err(MessageError::InvalidFormat("Not a Cosmos message".into())),
        };
        
        // Update metrics
        let mut metrics = self.metrics.clone();
        let duration = start.elapsed().as_secs_f64();
        
        metrics.total_validated += 1;
        metrics.avg_validation_time = (metrics.avg_validation_time * (metrics.total_validated - 1) as f64
            + duration) / metrics.total_validated as f64;
            
        if result.is_err() {
            metrics.failed_validations += 1;
        }
        
        result
    }
    
    async fn get_metrics(&self) -> ValidationMetrics {
        self.metrics.clone()
    }
    
    async fn update_config(&mut self, config: ValidatorConfig) -> Result<(), MessageError> {
        self.config = config;
        Ok(())
    }
} 