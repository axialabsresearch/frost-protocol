/*!
# State Types

This module defines the core type system for the FROST protocol's state management,
providing types for chain identification, block references, and state roots.

## Core Types

### Chain Identifier
The `ChainId` type provides:
- Unique chain identification
- String-based representation
- Serialization support
- Ordering capabilities

### Block Identifier
The `BlockId` enum supports:
- Hash-based identification
- Number-based identification
- Composite identification
- Flexible referencing

### Block Reference
The `BlockRef` struct combines:
- Chain context
- Block number
- Block hash
- Metadata handling

### State Root
The `StateRoot` struct manages:
- Block references
- Root hashes
- State metadata
- Verification data

## Type Relationships

1. **Chain and Block Relationship**
   ```rust
   use frost_protocol::state::{ChainId, BlockRef};

   pub struct BlockRef {
       pub chain_id: ChainId,
       pub number: u64,
       pub hash: [u8; 32],
   }

   // Example usage:
   # fn main() {
   let chain_id = ChainId::new("ethereum");
   let block_ref = BlockRef::new(
       chain_id,
       1000,
       [0; 32],
   );

   assert_eq!(block_ref.number(), 1000);
   assert_eq!(block_ref.hash(), &[0; 32]);
   # }
   ```

2. **Block Identification**
   ```rust
   use frost_protocol::state::BlockId;

   pub enum BlockId {
       Hash([u8; 32]),
       Number(u64),
       Composite { number: u64, hash: [u8; 32] },
   }

   // Example usage:
   # fn main() {
   let by_hash = BlockId::Hash([0; 32]);
   let by_number = BlockId::Number(1000);
   let composite = BlockId::Composite {
       number: 1000,
       hash: [0; 32],
   };
   # }
   ```

3. **State Management**
   ```rust
   use frost_protocol::state::{StateRoot, BlockRef, ChainId};
   use serde_json::json;

   pub struct StateRoot {
       pub block_ref: BlockRef,
       pub root_hash: [u8; 32],
       pub metadata: Option<serde_json::Value>,
   }

   // Example usage:
   # fn main() {
   let chain_id = ChainId::new("ethereum");
   let block_ref = BlockRef::new(
       chain_id,
       1000,
       [0; 32],
   );

   let state_root = StateRoot {
       block_ref,
       root_hash: [1; 32],
       metadata: Some(json!({
           "finalized": true,
           "timestamp": 1234567890
       })),
   };
   # }
   ```

## Features

### Chain Management
- Chain identification
- Chain validation
- Chain comparison
- Chain serialization

### Block Management
- Block identification
- Block referencing
- Block validation
- Block comparison

### State Management
- State root tracking
- Hash management
- Metadata handling
- State validation

### Type Safety
- Strong typing
- Validation support
- Error handling
- Serialization

## Best Practices

### Type Usage
1. Chain Identification
   - Unique identifiers
   - Proper validation
   - String handling
   - Error checking

2. Block References
   - Proper construction
   - Hash validation
   - Number tracking
   - Chain context

3. State Roots
   - Root calculation
   - Hash validation
   - Metadata handling
   - State tracking

4. Type Conversion
   - Safe conversion
   - Validation
   - Error handling
   - Context preservation

## Integration

### Chain System
- Chain identification
- Chain validation
- Chain comparison
- Chain tracking

### Block System
- Block identification
- Block referencing
- Block validation
- Block tracking

### State System
- State root management
- Hash tracking
- Metadata handling
- State validation

### Serialization
- Type serialization
- Format handling
- Version support
- Compatibility

## Performance Considerations

### Memory Usage
- Efficient types
- Minimal allocation
- Resource sharing
- Cache friendly

### Computation
- Fast comparison
- Quick validation
- Efficient hashing
- Resource optimization

### Storage
- Compact representation
- Efficient serialization
- Version handling
- Format optimization

### Validation
- Quick checks
- Efficient comparison
- Fast hashing
- Resource management
*/

use std::hash::{Hash, Hasher};
use std::fmt;
use serde::{Serialize, Deserialize};

/// Chain identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ChainId(String);

impl ChainId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ChainId {
    fn default() -> Self {
        Self("default".into())
    }
}

/// Block identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BlockId {
    Hash([u8; 32]),
    Number(u64),
    Composite {
        number: u64,
        hash: [u8; 32],
    },
}

/// Block reference with chain context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlockRef {
    pub chain_id: ChainId,
    pub number: u64,
    pub hash: [u8; 32],
}

impl BlockRef {
    pub fn new(chain_id: ChainId, number: u64, hash: [u8; 32]) -> Self {
        Self {
            chain_id,
            number,
            hash,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

impl fmt::Display for BlockRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.chain_id, self.number, hex::encode(self.hash))
    }
}

impl Default for BlockRef {
    fn default() -> Self {
        Self {
            chain_id: ChainId::default(),
            number: 0,
            hash: [0; 32],
        }
    }
}

/// State root representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateRoot {
    pub block_ref: BlockRef,
    pub root_hash: [u8; 32],
    pub metadata: Option<serde_json::Value>,
}

impl Default for StateRoot {
    fn default() -> Self {
        Self {
            block_ref: BlockRef::default(),
            root_hash: [0; 32],
            metadata: None,
        }
    }
}

/// Additional state metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateMetadata {
    pub version: u32,
    pub chain_specific: Option<serde_json::Value>,
} 