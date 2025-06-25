#![allow(unused_imports)]
#![allow(unused_variables)]

use async_trait::async_trait;
use crate::message::{FrostMessage, MessageError};
use crate::Result;

/// Validator for FROST Protocol messages
#[async_trait]
pub trait MessageValidator: Send + Sync {
    /// Validate a message
    async fn validate(&self, message: &FrostMessage) -> Result<ValidationResult>;
    
    /// Add validation rule
    fn add_rule(&mut self, rule: Box<dyn ValidationRule>);
    
    /// Remove validation rule
    fn remove_rule(&mut self, rule_id: &str);
}

/// Result of message validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub rules_passed: Vec<String>,
    pub rules_failed: Vec<ValidationFailure>,
}

/// Validation failure details
#[derive(Debug, Clone)]
pub struct ValidationFailure {
    pub rule_id: String,
    pub reason: String,
    pub severity: ValidationSeverity,
}

/// Severity of validation failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
    Critical,
}

/// Trait for validation rules
#[async_trait]
pub trait ValidationRule: Send + Sync {
    /// Unique identifier for the rule
    fn rule_id(&self) -> &str;
    
    /// Description of what the rule checks
    fn description(&self) -> &str;
    
    /// Validate a message against this rule
    async fn validate(&self, message: &FrostMessage) -> Result<bool>;
    
    /// Severity if validation fails
    fn severity(&self) -> ValidationSeverity;
}
