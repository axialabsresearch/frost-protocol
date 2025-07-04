/*!
# State Module

This module implements the state management system for the FROST protocol,
providing state transition, proof verification, and caching mechanisms.

## Core Components

### State Transition
The state transition system handles:
- Block state transitions
- State validation
- Transition verification
- State synchronization

### State Proof
Proof management includes:
- Proof generation
- Proof verification
- Proof caching
- Revocation handling

### State Types
Core state types include:
- Block identifiers
- Chain identifiers
- State roots
- Block references

### Error Handling
Comprehensive error management:
- State errors
- Validation errors
- Proof errors
- Cache errors

## Architecture

The state system consists of several key components:

1. **State Transition**
   ```rust
   use frost_protocol::state::{BlockId, StateTransition};
   
   pub struct StateTransition {
       source: BlockId,
       target: BlockId,
       state_data: Vec<u8>,
   }

   // Example usage:
   # fn main() {
   let source = BlockId::Number(1000);
   let target = BlockId::Number(1001);
   let data = vec![1, 2, 3];
   let transition = StateTransition::new(
       frost_protocol::state::ChainId::new("ethereum"),
       source,
       target,
       data
   );
   # }
   ```
   - State changes
   - Transition validation
   - State verification
   - Data management

2. **State Proof**
   ```rust
   use frost_protocol::state::{StateTransition, StateProof, proof::ProofData};
   use std::time::SystemTime;

   pub struct StateProof {
       transition: StateTransition,
       proof: ProofData,
   }

   // Example usage:
   # fn main() {
   let transition = StateTransition::new(
       frost_protocol::state::ChainId::new("ethereum"),
       BlockId::Number(1000),
       BlockId::Number(1001),
       vec![1, 2, 3]
   );
   let proof_data = ProofData {
       proof_type: frost_protocol::state::proof::ProofType::Basic,
       data: vec![1, 2, 3],
       metadata: None,
       generated_at: SystemTime::now(),
       expires_at: None,
       version: 1,
   };
   let proof = StateProof::new(transition, proof_data);
   # }
   ```
   - Proof generation
   - Verification
   - Caching
   - Revocation

3. **State Types**
   ```rust
   use frost_protocol::state::BlockId;

   pub enum BlockId {
       Hash([u8; 32]),
       Number(u64),
       Composite { number: u64, hash: [u8; 32] },
   }

   // Example usage:
   # fn main() {
   let block_by_number = BlockId::Number(1000);
   let block_by_hash = BlockId::Hash([0; 32]);
   let block_composite = BlockId::Composite {
       number: 1000,
       hash: [0; 32],
   };
   # }
   ```
   - Block identification
   - Chain references
   - State roots
   - Metadata

## Features

### State Management
- State transitions
- State verification
- State caching
- State synchronization

### Proof System
- Proof generation
- Proof verification
- Proof caching
- Proof revocation

### Type System
- Block identification
- Chain references
- State roots
- Metadata handling

### Error Handling
- Error categorization
- Error severity
- Error recovery
- Error reporting

## Best Practices

### State Handling
1. Transition Management
   - Proper validation
   - State verification
   - Cache utilization
   - Error handling

2. Proof Management
   - Proof verification
   - Cache management
   - Revocation handling
   - Error recovery

3. Type Usage
   - Proper identification
   - Reference handling
   - Root management
   - Metadata usage

4. Error Management
   - Error categorization
   - Severity assessment
   - Recovery procedures
   - Reporting mechanisms

## Integration

### Chain Integration
- State synchronization
- Block management
- Chain references
- State verification

### Cache System
- State caching
- Proof caching
- Cache invalidation
- Cache optimization

### Proof System
- Proof generation
- Verification
- Revocation
- Cache management

### Error System
- Error handling
- Recovery procedures
- Reporting mechanisms
- Severity management

## Performance Considerations

### Resource Management
- State size
- Cache utilization
- Memory usage
- CPU utilization

### Optimization
- Cache strategies
- Verification speed
- State compression
- Resource sharing

### Monitoring
- State metrics
- Cache performance
- Error rates
- Resource usage

### Tuning
- Cache sizes
- Timeout values
- Retry parameters
- Resource limits
*/

#![allow(unused_imports)]

pub mod transition;
pub mod proof;
pub mod types;
pub mod error;
pub mod cache;
pub mod revocation;

pub use transition::StateTransition;
pub use proof::StateProof;
pub use types::{BlockId, BlockRef, StateRoot, ChainId};
pub use error::{StateError, ErrorSeverity};

use crate::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

// Quick test the state transition validation 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transition_validation() {
        let chain_id = ChainId::new("test-chain");
        let source = BlockId::Composite {
            number: 1,
            hash: [0; 32],  // Source hash all zeros
        };
        let target = BlockId::Composite {
            number: 2,
            hash: [1; 32],  // Target hash all ones
        };
        
        let transition = StateTransition::new(
            chain_id.clone(),
            source,
            target,
            vec![1, 2, 3],
        );
        assert!(transition.validate());

        // Create invalid transition with same hash (will fail validation)
        let invalid_source = BlockId::Composite {
            number: 1,
            hash: [0; 32],
        };
        let invalid_target = BlockId::Composite {
            number: 2,  // Valid height difference
            hash: [0; 32],  // Same hash as source - should fail validation
        };
        let invalid_transition = StateTransition::new(
            chain_id,
            invalid_source,
            invalid_target,
            vec![1],  // Valid data
        );
        assert!(!invalid_transition.validate());
    }
}
