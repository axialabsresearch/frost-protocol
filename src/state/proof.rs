/*!
# State Proof System

This module implements the proof system for the FROST protocol's state management,
providing flexible proof generation, verification, and caching mechanisms.

## Core Components

### Proof Types
Supported proof types include:
- Zero-knowledge proofs
- Signature-based proofs
- Light client proofs
- Basic finality proofs
- Custom proof types

### Proof Generation
Generation features:
- Single proof generation
- Batch proof generation
- Context-aware generation
- Extension support

### Proof Verification
Verification capabilities:
- Single proof verification
- Batch verification
- Caching support
- Extension hooks

### Proof Registry
Registry management:
- Generator registration
- Verifier registration
- Cache management
- Batch operations

## Architecture

The proof system implements several key components:

1. **Proof Data**
   ```rust
   use frost_protocol::state::proof::{ProofData, ProofType};
   use std::time::SystemTime;
   use serde_json::json;

   pub struct ProofData {
       pub proof_type: ProofType,
       pub data: Vec<u8>,
       pub metadata: Option<serde_json::Value>,
       pub generated_at: SystemTime,
       pub expires_at: Option<SystemTime>,
       pub version: u32,
   }

   // Example usage:
   # fn main() {
   let proof_data = ProofData {
       proof_type: ProofType::Basic,
       data: vec![1, 2, 3],
       metadata: Some(json!({
           "chain": "ethereum",
           "block": 1000
       })),
       generated_at: SystemTime::now(),
       expires_at: None,
       version: 1,
   };
   # }
   ```

2. **State Proof**
   ```rust
   use frost_protocol::state::{
       proof::{StateProof, ProofData, ProofType, VerificationResult},
       StateTransition,
       ChainId,
       BlockId,
   };
   use std::time::SystemTime;

   pub struct StateProof {
       pub transition: StateTransition,
       pub proof: ProofData,
       pub verification_history: Vec<VerificationResult>,
   }

   // Example usage:
   # fn main() {
   let transition = StateTransition::new(
       ChainId::new("ethereum"),
       BlockId::Number(1000),
       BlockId::Number(1001),
       vec![1, 2, 3],
   );

   let proof_data = ProofData {
       proof_type: ProofType::Basic,
       data: vec![1, 2, 3],
       metadata: None,
       generated_at: SystemTime::now(),
       expires_at: None,
       version: 1,
   };

   let proof = StateProof::new(transition, proof_data);
   # }
   ```

3. **Proof Registry**
   ```rust
   use frost_protocol::state::proof::{ProofRegistry, ProofType, VerificationResult};
   use dashmap::DashMap;
   use std::sync::Arc;

   pub struct ProofRegistry {
       generators: DashMap<ProofType, Arc<dyn ProofGenerator>>,
       verifiers: DashMap<ProofType, Arc<dyn ProofVerifier>>,
       verification_cache: DashMap<String, VerificationResult>,
   }

   // Example usage:
   # fn main() {
   let registry = ProofRegistry::new();
   
   // Register a custom generator
   struct BasicGenerator;
   impl ProofGenerator for BasicGenerator {
       fn proof_type(&self) -> ProofType {
           ProofType::Basic
       }

       async fn generate_proof(
           &self,
           transition: &StateTransition,
           _context: Option<&serde_json::Value>,
       ) -> Result<ProofData, StateError> {
           Ok(ProofData {
               proof_type: ProofType::Basic,
               data: vec![1, 2, 3],
               metadata: None,
               generated_at: SystemTime::now(),
               expires_at: None,
               version: 1,
           })
       }
   }

   registry.register_generator(Arc::new(BasicGenerator));
   # }
   ```

## Features

### Proof Management
- Type-specific proofs
- Proof generation
- Proof verification
- History tracking

### Verification System
- Result caching
- Batch operations
- Extension support
- Error handling

### Registry System
- Generator registration
- Verifier registration
- Cache management
- Concurrent access

### Extension Support
- Custom hooks
- Context handling
- Batch processing
- Error management

## Best Practices

### Proof Handling
1. Generation
   - Type selection
   - Context inclusion
   - Batch processing
   - Error handling

2. Verification
   - Parameter tuning
   - Cache utilization
   - Extension usage
   - Result tracking

3. Registry Usage
   - Registration timing
   - Cache management
   - Concurrent access
   - Resource sharing

4. Extension Management
   - Hook integration
   - Context handling
   - Error processing
   - Resource usage

## Integration

### State System
- Transition binding
- State verification
- History tracking
- Error handling

### Cache System
- Result caching
- Cache invalidation
- Performance tuning
- Resource management

### Extension System
- Hook integration
- Context handling
- Batch processing
- Error management

### Registry System
- Component registration
- Cache management
- Resource sharing
- Concurrent access

## Performance Considerations

### Proof Generation
- Type optimization
- Batch processing
- Resource usage
- Cache utilization

### Verification
- Parameter tuning
- Cache usage
- Batch operations
- Resource management

### Registry Operations
- Concurrent access
- Cache efficiency
- Resource sharing
- Performance scaling

### Extension Processing
- Hook efficiency
- Context handling
- Resource usage
- Error management

## Implementation Notes

### Proof Types
The system supports:
- ZK proofs (zk-SNARKs)
- Signature proofs (BLS, Schnorr)
- Light client proofs
- Basic finality proofs
- Custom proof types

### Verification Process
Verification includes:
- Parameter validation
- Cache checking
- Extension processing
- Result tracking

### Registry Management
Registry handling:
- Component registration
- Cache management
- Resource allocation
- Concurrent access

### Extension Support
Extension features:
- Hook processing
- Context handling
- Error management
- Resource usage
*/

#![allow(unused_imports)]
#![allow(unused_variables)]

use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use std::fmt;
use std::hash::Hash;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use dashmap::DashMap;

use crate::state::{
    types::{BlockRef, StateRoot},
    error::StateError,
    transition::StateTransition,
};
use crate::extensions::ExtensionHooks;

/// Proof type identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProofType {
    /// Zero-knowledge proof (e.g. zk-SNARKs)
    ZeroKnowledge,
    /// Validator signature based (e.g. BLS, Schnorr)
    Signature,
    /// Light client proof (e.g. Tendermint, GRANDPA)
    LightClient,
    /// Basic finality check
    Basic,
    /// Custom proof type
    Custom(String),
}

/// Parameters for proof verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationParams {
    /// Security level (0-100, higher is more secure but slower)
    pub security_level: u8,
    /// Verification timeout
    pub timeout: Duration,
    /// Whether to use cached results if available
    pub use_cache: bool,
    /// Additional parameters specific to proof type
    pub extra_params: Option<serde_json::Value>,
}

impl Default for VerificationParams {
    fn default() -> Self {
        Self {
            security_level: 80,
            timeout: Duration::from_secs(30),
            use_cache: true,
            extra_params: None,
        }
    }
}

/// Proof data wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofData {
    /// Type of proof
    pub proof_type: ProofType,
    /// Raw proof data
    pub data: Vec<u8>,
    /// Additional proof metadata
    pub metadata: Option<serde_json::Value>,
    /// When the proof was generated
    pub generated_at: SystemTime,
    /// Optional expiration time
    pub expires_at: Option<SystemTime>,
    /// Proof version for compatibility
    pub version: u32,
}

/// State proof with flexible proof system support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateProof {
    /// State transition being proven
    pub transition: StateTransition,
    /// Proof data
    pub proof: ProofData,
    /// Verification history
    #[serde(skip)]
    pub verification_history: Vec<VerificationResult>,
}

/// Result of a proof verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub success: bool,
    /// When verification was performed
    pub verified_at: SystemTime,
    /// Parameters used for verification
    pub params: VerificationParams,
    /// Any errors encountered
    pub error: Option<String>,
}

/// Proof generation interface
#[async_trait]
pub trait ProofGenerator: Send + Sync {
    /// Get supported proof type
    fn proof_type(&self) -> ProofType;

    /// Generate proof for state transition
    async fn generate_proof(
        &self,
        transition: &StateTransition,
        context: Option<&serde_json::Value>,
    ) -> Result<ProofData, StateError>;

    /// Generate proofs in batch for multiple transitions
    async fn generate_batch(
        &self,
        transitions: &[StateTransition],
        context: Option<&serde_json::Value>,
    ) -> Result<Vec<ProofData>, StateError> {
        let mut proofs = Vec::with_capacity(transitions.len());
        for transition in transitions {
            proofs.push(self.generate_proof(transition, context).await?);
        }
        Ok(proofs)
    }
}

/// Proof verification interface
#[async_trait]
pub trait ProofVerifier: Send + Sync {
    /// Get supported proof types
    fn supported_types(&self) -> Vec<ProofType>;

    /// Verify state proof
    async fn verify_proof(
        &self,
        proof: &StateProof,
        params: &VerificationParams,
        context: Option<&serde_json::Value>,
    ) -> Result<bool, StateError>;

    /// Verify multiple proofs in batch
    async fn verify_batch(
        &self,
        proofs: &[StateProof],
        params: &VerificationParams,
        context: Option<&serde_json::Value>,
    ) -> Result<Vec<bool>, StateError> {
        let mut results = Vec::with_capacity(proofs.len());
        for proof in proofs {
            results.push(self.verify_proof(proof, params, context).await?);
        }
        Ok(results)
    }
}

impl StateProof {
    /// Create new state proof
    pub fn new(transition: StateTransition, proof: ProofData) -> Self {
        Self {
            transition,
            proof,
            verification_history: Vec::new(),
        }
    }

    /// Get proof type
    pub fn proof_type(&self) -> &ProofType {
        &self.proof.proof_type
    }

    /// Get proof metadata
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.proof.metadata.as_ref()
    }

    /// Check if proof has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.proof.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }

    /// Add verification result to history
    fn record_verification(&mut self, result: VerificationResult) {
        self.verification_history.push(result);
    }

    /// Get last verification result
    pub fn last_verification(&self) -> Option<&VerificationResult> {
        self.verification_history.last()
    }

    /// Verify proof with extensions
    pub async fn verify_with_extensions(
        &mut self,
        verifier: &dyn ProofVerifier,
        params: &VerificationParams,
        hooks: &ExtensionHooks,
        context: Option<&serde_json::Value>,
    ) -> Result<bool, StateError> {
        // Run extension verification first
        hooks.verify_state_proof(self).await.map_err(|e| {
            StateError::ProofVerificationFailed(format!("Extension verification failed: {}", e))
        })?;

        // Run core verification
        let result = verifier.verify_proof(self, params, context).await?;

        // Record verification result
        self.record_verification(VerificationResult {
            success: result,
            verified_at: SystemTime::now(),
            params: params.clone(),
            error: None,
        });

        Ok(result)
    }

    /// Verify batch of proofs with extensions
    pub async fn verify_batch_with_extensions(
        proofs: &mut [StateProof],
        verifier: &dyn ProofVerifier,
        params: &VerificationParams,
        hooks: &ExtensionHooks,
        context: Option<&serde_json::Value>,
    ) -> Result<Vec<bool>, StateError> {
        let mut results = Vec::with_capacity(proofs.len());

        // Run extension verification
        for proof in proofs.iter_mut() {
            // Run extension verification
            hooks.verify_state_proof(proof).await.map_err(|e| {
                StateError::ProofVerificationFailed(format!("Extension verification failed: {}", e))
            })?;
        }

        // Run core batch verification
        let core_results = verifier.verify_batch(proofs, params, context).await?;

        // Record results
        for (proof, &result) in proofs.iter_mut().zip(core_results.iter()) {
            proof.record_verification(VerificationResult {
                success: result,
                verified_at: SystemTime::now(),
                params: params.clone(),
                error: None,
            });
            results.push(result);
        }

        Ok(results)
    }
}

/// Registry for proof generators and verifiers with caching
pub struct ProofRegistry {
    generators: DashMap<ProofType, Arc<dyn ProofGenerator>>,
    verifiers: DashMap<ProofType, Arc<dyn ProofVerifier>>,
    verification_cache: DashMap<String, VerificationResult>,
}

impl ProofRegistry {
    /// Create new proof registry
    pub fn new() -> Self {
        Self {
            generators: DashMap::new(),
            verifiers: DashMap::new(),
            verification_cache: DashMap::new(),
        }
    }

    /// Register proof generator
    pub fn register_generator(&self, generator: Arc<dyn ProofGenerator>) {
        self.generators.insert(generator.proof_type(), generator);
    }

    /// Register proof verifier
    pub fn register_verifier(&self, verifier: Arc<dyn ProofVerifier>) {
        for proof_type in verifier.supported_types() {
            self.verifiers.insert(proof_type, verifier.clone());
        }
    }

    /// Generate proof using registered generator
    pub async fn generate_proof(
        &self,
        proof_type: &ProofType,
        transition: &StateTransition,
        context: Option<&serde_json::Value>,
    ) -> Result<ProofData, StateError> {
        let generator = self.generators.get(proof_type).ok_or_else(|| {
            StateError::Internal(format!("No generator found for proof type: {:?}", proof_type))
        })?;
        generator.generate_proof(transition, context).await
    }

    /// Generate proofs in batch
    pub async fn generate_batch(
        &self,
        proof_type: &ProofType,
        transitions: &[StateTransition],
        context: Option<&serde_json::Value>,
    ) -> Result<Vec<ProofData>, StateError> {
        let generator = self.generators.get(proof_type).ok_or_else(|| {
            StateError::Internal(format!("No generator found for proof type: {:?}", proof_type))
        })?;
        generator.generate_batch(transitions, context).await
    }

    /// Verify proof using registered verifier
    pub async fn verify_proof(
        &self,
        proof: &mut StateProof,
        params: &VerificationParams,
        context: Option<&serde_json::Value>,
    ) -> Result<bool, StateError> {
        // Check expiration
        if proof.is_expired() {
            return Err(StateError::Internal("Proof has expired".into()));
        }

        // Try cache first
        if params.use_cache {
            let cache_key = format!("{:?}:{:?}", proof.transition, proof.proof);
            if let Some(cached) = self.verification_cache.get(&cache_key) {
                if SystemTime::now().duration_since(cached.verified_at).unwrap() < Duration::from_secs(300) {
                    return Ok(cached.success);
                }
            }
        }

        // Verify using appropriate verifier
        let verifier = self.verifiers.get(proof.proof_type()).ok_or_else(|| {
            StateError::Internal(format!("No verifier found for proof type: {:?}", proof.proof_type()))
        })?;

        let result = verifier.verify_proof(proof, params, context).await?;

        // Record verification
        let verification = VerificationResult {
            success: result,
            verified_at: SystemTime::now(),
            params: params.clone(),
            error: None,
        };
        proof.record_verification(verification.clone());

        // Update cache
        if params.use_cache {
            let cache_key = format!("{:?}:{:?}", proof.transition, proof.proof);
            self.verification_cache.insert(cache_key, verification);
        }

        Ok(result)
    }

    /// Verify proofs in batch
    pub async fn verify_batch(
        &self,
        proofs: &mut [StateProof],
        params: &VerificationParams,
        context: Option<&serde_json::Value>,
    ) -> Result<Vec<bool>, StateError> {
        let mut results = Vec::with_capacity(proofs.len());
        for proof in proofs {
            results.push(self.verify_proof(proof, params, context).await?);
        }
        Ok(results)
    }

    /// Clear verification cache
    pub fn clear_cache(&self) {
        self.verification_cache.clear();
    }
} 