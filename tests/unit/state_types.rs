use frost_protocol::state::{BlockRef, BlockId, ChainId, StateRoot, StateMetadata};

#[test]
fn test_chain_id_creation_and_display() {
    let chain_id = ChainId::new("ethereum");
    assert_eq!(chain_id.to_string(), "ethereum");
    
    let default_chain = ChainId::default();
    assert_eq!(default_chain.to_string(), "default");
}

#[test]
fn test_block_ref_creation_and_accessors() {
    let chain_id = ChainId::new("ethereum");
    let number = 12345u64;
    let hash = [1u8; 32];
    
    let block_ref = BlockRef::new(chain_id.clone(), number, hash);
    
    assert_eq!(block_ref.chain_id().to_string(), "ethereum");
    assert_eq!(block_ref.number(), number);
    assert_eq!(block_ref.hash(), &hash);
}

#[test]
fn test_block_ref_equality_and_hash() {
    use std::collections::HashSet;
    
    let chain_id1 = ChainId::new("ethereum");
    let chain_id2 = ChainId::new("ethereum");
    let hash = [1u8; 32];
    
    let block_ref1 = BlockRef::new(chain_id1, 100, hash);
    let block_ref2 = BlockRef::new(chain_id2, 100, hash);
    let block_ref3 = BlockRef::new(ChainId::new("solana"), 100, hash);
    
    assert_eq!(block_ref1, block_ref2);
    assert_ne!(block_ref1, block_ref3);
    
    let mut set = HashSet::new();
    set.insert(block_ref1.clone());
    assert!(set.contains(&block_ref2));
    assert!(!set.contains(&block_ref3));
}

#[test]
fn test_block_ref_display() {
    let chain_id = ChainId::new("ethereum");
    let number = 12345u64;
    let hash = [0xaa; 32];
    
    let block_ref = BlockRef::new(chain_id, number, hash);
    let display = block_ref.to_string();
    
    assert!(display.contains("ethereum"));
    assert!(display.contains("12345"));
    assert!(display.contains(&hex::encode(hash)));
}

#[test]
fn test_block_id_variants() {
    let hash = [2u8; 32];
    let number = 5000u64;
    
    let hash_id = BlockId::Hash(hash);
    let number_id = BlockId::Number(number);
    let composite_id = BlockId::Composite {
        number,
        hash,
    };
    
    match hash_id {
        BlockId::Hash(h) => assert_eq!(h, hash),
        _ => panic!("Expected Hash variant"),
    }
    
    match number_id {
        BlockId::Number(n) => assert_eq!(n, number),
        _ => panic!("Expected Number variant"),
    }
    
    match composite_id {
        BlockId::Composite { number: n, hash: h } => {
            assert_eq!(n, number);
            assert_eq!(h, hash);
        },
        _ => panic!("Expected Composite variant"),
    }
}

#[test]
fn test_state_root_serialization() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id, 100, [3u8; 32]);
    let root_hash = [4u8; 32];
    let metadata = serde_json::json!({
        "version": "1.0",
        "timestamp": 12345
    });
    
    let state_root = StateRoot {
        block_ref,
        root_hash,
        metadata: Some(metadata.clone()),
    };
    
    let serialized = serde_json::to_string(&state_root).unwrap();
    let deserialized: StateRoot = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.block_ref, state_root.block_ref);
    assert_eq!(deserialized.root_hash, state_root.root_hash);
    assert_eq!(deserialized.metadata, state_root.metadata);
}

#[test]
fn test_state_metadata() {
    let metadata = StateMetadata {
        version: 1,
        chain_specific: Some(serde_json::json!({
            "network": "mainnet",
            "fork": "shanghai"
        })),
    };
    
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: StateMetadata = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.version, metadata.version);
    assert_eq!(deserialized.chain_specific, metadata.chain_specific);
} 