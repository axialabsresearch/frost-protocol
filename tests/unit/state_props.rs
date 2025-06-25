use proptest::prelude::*;
use frost_protocol::state::{
    BlockRef, ChainId,
    error::{StateError, ErrorSeverity},
};
use frost_protocol::state::types::{BlockId, StateRoot};
use std::cmp::Ordering;

proptest! {
    #[test]
    fn test_block_ref_properties(
        chain in "[a-z][a-z0-9_]{1,31}",
        number in 0u64..1_000_000u64,
        hash in prop::array::uniform32(0u8..),
    ) {
        let chain_id = ChainId::new(&chain);
        let block_ref = BlockRef::new(chain_id.clone(), number, hash);
        
        // Properties that should always hold
        prop_assert_eq!(block_ref.chain_id().to_string(), chain.clone());
        prop_assert_eq!(block_ref.number(), number);
        prop_assert_eq!(block_ref.hash(), &hash);
        
        // Reflexive equality
        let block_ref_clone = block_ref.clone();
        let display = block_ref_clone.to_string();
        prop_assert_eq!(block_ref, block_ref_clone);
        
        // Display formatting should contain all components
        prop_assert!(display.contains(&chain));
        prop_assert!(display.contains(&number.to_string()));
        prop_assert!(display.contains(&hex::encode(hash)));
    }
    
    #[test]
    fn test_block_id_properties(
        number in 0u64..1_000_000u64,
        hash in prop::array::uniform32(0u8..),
    ) {
        // Test different BlockId variants
        let hash_id = BlockId::Hash(hash);
        let number_id = BlockId::Number(number);
        let composite_id = BlockId::Composite { number, hash };
        
        match hash_id {
            BlockId::Hash(h) => prop_assert_eq!(h, hash),
            _ => prop_assert!(false),
        }
        
        match number_id {
            BlockId::Number(n) => prop_assert_eq!(n, number),
            _ => prop_assert!(false),
        }
        
        match composite_id {
            BlockId::Composite { number: n, hash: h } => {
                prop_assert_eq!(n, number);
                prop_assert_eq!(h, hash);
            },
            _ => prop_assert!(false),
        }
    }
    
    #[test]
    fn test_state_root_properties(
        chain in "[a-z][a-z0-9_]{1,31}",
        number in 0u64..1_000_000u64,
        root_hash in prop::array::uniform32(0u8..),
    ) {
        let chain_id = ChainId::new(&chain);
        let block_ref = BlockRef::new(chain_id, number, [0u8; 32]);
        let metadata = serde_json::json!({
            "version": format!("{}.{}.{}", 
                number % 10,
                (number / 10) % 10,
                (number / 100) % 10
            ),
            "timestamp": number * 1000,
        });
        
        let state_root = StateRoot {
            block_ref: block_ref.clone(),
            root_hash,
            metadata: Some(metadata.clone()),
        };
        
        // Test serialization roundtrip
        let serialized = serde_json::to_string(&state_root).unwrap();
        let deserialized: StateRoot = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(deserialized.block_ref, state_root.block_ref);
        prop_assert_eq!(deserialized.root_hash, state_root.root_hash);
        prop_assert_eq!(deserialized.metadata, state_root.metadata);
    }
    
    #[test]
    fn test_error_properties(
        chain in "[a-z][a-z0-9_]{1,31}",
        number in 0u64..1_000_000u64,
        msg in "[a-zA-Z0-9_. -]{1,100}",
    ) {
        let block_ref = BlockRef::new(ChainId::new(&chain), number, [0u8; 32]);
        
        // Test different error variants
        let errors = vec![
            StateError::InvalidTransition(msg.clone()),
            StateError::ProofVerificationFailed(msg.clone()),
            StateError::InvalidBlockRef(msg.clone()),
            StateError::RootMismatch {
                block_ref: block_ref.clone(),
                expected: format!("0x{}", hex::encode(&[1u8; 32])),
                actual: format!("0x{}", hex::encode(&[2u8; 32])),
            },
            StateError::ChainSpecific(msg.clone()),
            StateError::Internal(msg.clone()),
        ];
        
        for error in errors {
            let error_str = error.to_string();
            
            // Error messages should contain the message or block reference
            match &error {
                StateError::RootMismatch { .. } => {
                    prop_assert!(error_str.contains(&block_ref.to_string()));
                }
                _ => {
                    prop_assert!(error_str.contains(&msg));
                }
            }
            
            // Test severity ordering using partial_cmp
            let severity = error.severity();
            match severity {
                ErrorSeverity::Critical => {
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Error), Some(Ordering::Greater)));
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Warning), Some(Ordering::Greater)));
                }
                ErrorSeverity::Error => {
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Warning), Some(Ordering::Greater)));
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Critical), Some(Ordering::Less)));
                }
                ErrorSeverity::Warning => {
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Error), Some(Ordering::Less)));
                    prop_assert!(matches!(severity.partial_cmp(&ErrorSeverity::Critical), Some(Ordering::Less)));
                }
            }
            
            // Test retryability consistency
            prop_assert_eq!(
                error.is_retryable(),
                matches!(error, StateError::ChainSpecific(_))
            );
        }
    }
} 