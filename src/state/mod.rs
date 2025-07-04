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
   pub struct StateTransition {
       source: BlockId,
       target: BlockId,
       state_data: Vec<u8>,
   }
   ```
   - State changes
   - Transition validation
   - State verification
   - Data management

2. **State Proof**
   ```rust
   pub struct StateProof {
       transition: StateTransition,
       proof: ProofData,
   }
   ```
   - Proof generation
   - Verification
   - Caching
   - Revocation

3. **State Types**
   ```rust
   pub enum BlockId {
       Hash([u8; 32]),
       Number(u64),
       Composite { number: u64, hash: [u8; 32] },
   }
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
            hash: [0; 32],
        };
        let target = BlockId::Composite {
            number: 1,
            hash: [0; 32],
        };
        
        let transition = StateTransition::new(
            chain_id.clone(),
            source,
            target,
            vec![1, 2, 3],
        );
        assert!(transition.validate());

        let invalid_transition = StateTransition::new(
            chain_id,
            BlockId::Number(0),
            BlockId::Number(0),
            vec![],
        );
        assert!(!invalid_transition.validate());
    }
}
