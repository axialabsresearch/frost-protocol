use std::time::Duration;
use frost_protocol::{
    state::BlockRef,
    finality::{
        predicate::{
            FinalityVerificationClient,
            CachingFinalityClient,
            ChainRules,
            VerificationMetrics,
            ConsensusVerifier,
            ConsensusConfig,
        },
        FinalitySignal,
        FinalityError,
        verifier::{FinalityVerifier, FinalityConfig},
        EthereumFinalityType,
        EthereumMetadata,
        CosmosMetadata,
        SubstrateMetadata,
    },
};

use crate::common::{
    test_block_ref,
    test_ethereum_signal,
    test_cosmos_signal,
    test_substrate_signal,
};

#[tokio::test]
async fn test_ethereum_finality_integration() -> Result<(), Box<dyn std::error::Error>> {
    let client = CachingFinalityClient::new(
        "ethereum",
        Duration::from_secs(60),
        Some(100),
    );

    let config = FinalityConfig {
        chain_id: "ethereum".to_string(),
        min_confirmations: 12,
        confidence_threshold: 0.95,
        verification_timeout: Duration::from_secs(30),
        chain_rules: ChainRules {
            max_reorg_depth: 6,
            min_block_time: Duration::from_secs(12),
            max_future_time: Duration::from_secs(120),
        },
    };

    let verifier = FinalityVerifier::new(Box::new(client), config);
    let block_ref = test_block_ref("ethereum", 1000);

    // Test PoW finality
    let pow_signal = test_ethereum_signal(1000, 15, false);
    let result = verifier.verify_finality(&block_ref, &pow_signal).await?;
    assert!(result.is_final);
    assert!(result.confidence > 0.95);

    // Test beacon chain finality
    let beacon_signal = test_ethereum_signal(1000, 1, true);
    let result = verifier.verify_finality(&block_ref, &beacon_signal).await?;
    assert!(result.is_final);
    assert_eq!(result.confidence, 1.0);

    // Test invalid block hash
    let mut invalid_signal = test_ethereum_signal(1000, 15, false);
    if let FinalitySignal::Ethereum { block_hash, .. } = &mut invalid_signal {
        *block_hash = [1; 32];
    }
    let result = verifier.verify_finality(&block_ref, &invalid_signal).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_cosmos_finality_integration() -> Result<(), Box<dyn std::error::Error>> {
    let client = CachingFinalityClient::new(
        "cosmos",
        Duration::from_secs(60),
        Some(100),
    );

    let config = FinalityConfig {
        chain_id: "cosmos".to_string(),
        min_confirmations: 1,
        confidence_threshold: 0.95,
        verification_timeout: Duration::from_secs(30),
        chain_rules: ChainRules {
            max_reorg_depth: 1,
            min_block_time: Duration::from_secs(6),
            max_future_time: Duration::from_secs(60),
        },
    };

    let verifier = FinalityVerifier::new(Box::new(client), config);
    let block_ref = test_block_ref("cosmos", 1000);

    // Test valid finality
    let valid_signal = test_cosmos_signal(1000, 0, 1000, 700);
    let result = verifier.verify_finality(&block_ref, &valid_signal).await?;
    assert!(result.is_final);
    assert!(result.confidence > 0.95);

    // Test insufficient voting power
    let insufficient_signal = test_cosmos_signal(1000, 0, 1000, 500);
    let result = verifier.verify_finality(&block_ref, &insufficient_signal).await?;
    assert!(!result.is_final);
    assert!(result.confidence < 0.95);

    // Test invalid block hash
    let mut invalid_signal = test_cosmos_signal(1000, 0, 1000, 700);
    if let FinalitySignal::Cosmos { block_hash, .. } = &mut invalid_signal {
        *block_hash = [1; 32];
    }
    let result = verifier.verify_finality(&block_ref, &invalid_signal).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_substrate_finality_integration() -> Result<(), Box<dyn std::error::Error>> {
    let client = CachingFinalityClient::new(
        "substrate",
        Duration::from_secs(60),
        Some(100),
    );

    let config = FinalityConfig {
        chain_id: "substrate".to_string(),
        min_confirmations: 1,
        confidence_threshold: 0.95,
        verification_timeout: Duration::from_secs(30),
        chain_rules: ChainRules {
            max_reorg_depth: 2,
            min_block_time: Duration::from_secs(6),
            max_future_time: Duration::from_secs(60),
        },
    };

    let verifier = FinalityVerifier::new(Box::new(client), config);
    let block_ref = test_block_ref("substrate", 1000);

    // Test GRANDPA finality
    let valid_signal = test_substrate_signal(1000, 5, 100, 67, false);
    let result = verifier.verify_finality(&block_ref, &valid_signal).await?;
    assert!(result.is_final);
    assert_eq!(result.confidence, 1.0);

    // Test parachain finality
    let parachain_signal = test_substrate_signal(1000, 5, 100, 67, true);
    let result = verifier.verify_finality(&block_ref, &parachain_signal).await?;
    assert!(result.is_final);
    assert_eq!(result.confidence, 1.0);

    // Test invalid authority set
    let invalid_signal = test_substrate_signal(1000, 4, 100, 67, false);
    let result = verifier.verify_finality(&block_ref, &invalid_signal).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_consensus_verification() -> Result<(), Box<dyn std::error::Error>> {
    let client = CachingFinalityClient::new(
        "cosmos",
        Duration::from_secs(60),
        Some(100),
    );

    let config = ConsensusConfig {
        chain_id: "cosmos".to_string(),
        min_voting_power: 667, // 2/3 of total
        max_byzantine_faults: 33, // Up to 1/3 can be faulty
        verification_timeout: Duration::from_secs(30),
        chain_rules: ChainRules {
            max_reorg_depth: 1,
            min_block_time: Duration::from_secs(6),
            max_future_time: Duration::from_secs(60),
        },
    };

    let verifier = ConsensusVerifier::new(Box::new(client), config);
    let block_ref = test_block_ref("cosmos", 1000);

    // Test valid consensus
    let valid_signal = test_cosmos_signal(1000, 0, 1000, 700);
    let result = verifier.verify_consensus(&block_ref, &valid_signal).await?;
    assert!(result.is_valid);
    assert!(result.byzantine_faults == 0);
    assert!(result.voting_power_ratio > 0.66);

    // Test consensus with some faults
    let faulty_signal = test_cosmos_signal(1000, 0, 1000, 680);
    let result = verifier.verify_consensus(&block_ref, &faulty_signal).await?;
    assert!(result.is_valid); // Still valid as we have >2/3
    assert!(result.byzantine_faults == 1);
    assert!(result.voting_power_ratio > 0.66);

    // Test invalid consensus (too many faults)
    let invalid_signal = test_cosmos_signal(1000, 0, 1000, 400);
    let result = verifier.verify_consensus(&block_ref, &invalid_signal).await?;
    assert!(!result.is_valid);
    assert!(result.byzantine_faults > config.max_byzantine_faults);
    assert!(result.voting_power_ratio < 0.66);

    // Test consensus with equivocation
    let mut equivocating_signal = test_cosmos_signal(1000, 0, 1000, 700);
    if let FinalitySignal::Cosmos { validator_signatures, .. } = &mut equivocating_signal {
        validator_signatures.push(validator_signatures[0].clone());
    }
    let result = verifier.verify_consensus(&block_ref, &equivocating_signal).await;
    assert!(result.is_err());

    Ok(())
} 