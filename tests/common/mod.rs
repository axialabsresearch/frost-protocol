use frost_protocol::{
    state::{BlockRef, ChainId},
    state::types::BlockId,
    finality::{
        FinalitySignal,
        EthereumFinalityType,
        EthereumMetadata,
        CosmosMetadata,
        SubstrateMetadata,
    },
};

use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a test chain ID
pub fn test_chain_id(name: &str) -> ChainId {
    ChainId::new(name)
}

/// Create a test block reference
pub fn test_block_ref(chain_id: &str, number: u64) -> BlockRef {
    BlockRef::new(
        ChainId::new(chain_id),
        number,
        [0; 32],
    )
}

/// Create a test Ethereum finality signal
pub fn test_ethereum_signal(block_number: u64, confirmations: u64, use_beacon: bool) -> FinalitySignal {
    if use_beacon {
        FinalitySignal::Ethereum {
            block_number,
            block_hash: [0; 32],
            confirmations: confirmations as u32,
            finality_type: EthereumFinalityType::BeaconFinalized,
            metadata: Some(EthereumMetadata {
                current_slot: Some(block_number * 32),
                head_slot: Some(block_number * 32),
                active_validators: Some(300_000),
                total_validators: Some(400_000),
            }),
        }
    } else {
        FinalitySignal::Ethereum {
            block_number,
            block_hash: [0; 32],
            confirmations: confirmations as u32,
            finality_type: EthereumFinalityType::Confirmations,
            metadata: None,
        }
    }
}

/// Create a test Cosmos finality signal
pub fn test_cosmos_signal(
    height: u64,
    _round: u32,
    total_power: u64,
    signed_power: u64,
) -> FinalitySignal {
    FinalitySignal::Cosmos {
        height,
        block_hash: [0; 32],
        validator_signatures: vec![[1u8; 64].to_vec()],
        metadata: Some(CosmosMetadata {
            voting_power: Some(signed_power),
            total_power: Some(total_power),
        }),
    }
}

/// Create a test Substrate finality signal
pub fn test_substrate_signal(
    block_number: u64,
    _authority_set_id: u64,
    _validator_set_len: u32,
    _signed_precommits: u32,
    is_parachain: bool,
) -> FinalitySignal {
    let parachain_status = if is_parachain {
        Some(json!({
            "para_id": 2000,
            "relay_parent_number": block_number - 1,
            "relay_parent_hash": format!("{:?}", [0u8; 32]),
            "backed_in_blocks": [block_number]
        }))
    } else {
        None
    };

    FinalitySignal::Substrate {
        block_number,
        block_hash: [0; 32],
        metadata: Some(SubstrateMetadata {
            voting_power: Some(800),
            total_power: Some(1000),
            active_validators: Some(150),
            total_validators: Some(200),
        }),
    }
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Test networks configuration
pub struct TestNetworks {
    pub ethereum: ChainId,
    pub substrate: ChainId,
    pub cosmos: ChainId,
}

impl Default for TestNetworks {
    fn default() -> Self {
        Self {
            ethereum: test_chain_id("ethereum"),
            substrate: test_chain_id("substrate"),
            cosmos: test_chain_id("cosmos"),
        }
    }
}

/// Create a test block ID
pub fn test_block_id(number: u64) -> BlockId {
    BlockId::Composite {
        number,
        hash: {
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&number.to_be_bytes());
            hash
        },
    }
}

/// Assert that two timestamps are close (within margin)
pub fn assert_timestamps_close(ts1: u64, ts2: u64, margin_secs: u64) {
    let diff = if ts1 > ts2 { ts1 - ts2 } else { ts2 - ts1 };
    assert!(diff <= margin_secs, "Timestamps differ by {} seconds, expected <= {}", diff, margin_secs);
} 