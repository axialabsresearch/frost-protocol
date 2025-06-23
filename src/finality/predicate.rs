use std::time::Duration;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, warn, error};
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::LruCache;
use std::num::NonZeroUsize;

use crate::state::BlockRef;
use crate::finality::{FinalitySignal, FinalityError};

/// Errors that can occur during predicate validation
#[derive(Error, Debug)]
pub enum PredicateError {
    #[error("Invalid predicate format: {0}")]
    InvalidFormat(String),
    
    #[error("Predicate validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Predicate timeout: {0}")]
    Timeout(String),
    
    #[error("Finality verification error: {0}")]
    FinalityVerificationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Finality predicate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateConfig {
    /// Minimum required confirmations
    pub min_confirmations: u32,
    
    /// Maximum time to wait for predicate evaluation
    pub evaluation_timeout: Duration,
    
    /// Required confidence level (0.0 - 1.0)
    pub confidence_threshold: f64,
    
    /// Chain-specific predicate parameters
    pub chain_params: serde_json::Value,
}

/// Finality predicate validation result
#[derive(Debug, Clone)]
pub struct PredicateResult {
    /// Whether the predicate was satisfied
    pub is_satisfied: bool,
    
    /// Confidence level in the result (0.0 - 1.0)
    pub confidence: f64,
    
    /// Time taken to evaluate the predicate
    pub evaluation_time: Duration,
    
    /// Additional chain-specific validation data
    pub validation_data: serde_json::Value,
}

/// Finality predicate validator trait
#[async_trait::async_trait]
pub trait PredicateValidator: Send + Sync {
    /// Validate a finality predicate
    async fn validate_predicate(
        &self,
        block_ref: &BlockRef,
        signal: &FinalitySignal,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError>;
}

/// Ethereum predicate validator
pub struct EthereumPredicateValidator {
    finality_client: Box<dyn FinalityVerificationClient>,
}

impl EthereumPredicateValidator {
    pub fn new(finality_client: Box<dyn FinalityVerificationClient>) -> Self {
        Self { finality_client }
    }
    
    async fn validate_pow_confirmations(
        &self,
        block_ref: &BlockRef,
        confirmations: u32,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        // Verify block exists in finality client
        let block = self.finality_client.get_block(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        // Verify confirmation count
        if confirmations < config.min_confirmations {
            return Ok(PredicateResult {
                is_satisfied: false,
                confidence: confirmations as f64 / config.min_confirmations as f64,
                evaluation_time: Duration::from_secs(0),
                validation_data: serde_json::json!({
                    "required_confirmations": config.min_confirmations,
                    "actual_confirmations": confirmations,
                }),
            });
        }
        
        // Verify block hash matches
        let hash_matches = self.finality_client.verify_block_hash(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        if !hash_matches {
            return Err(PredicateError::ValidationFailed(
                "Block hash mismatch".into()
            ));
        }
        
        Ok(PredicateResult {
            is_satisfied: true,
            confidence: 1.0,
            evaluation_time: Duration::from_secs(0),
            validation_data: serde_json::json!({
                "block_hash": block.hash,
                "confirmations": confirmations,
            }),
        })
    }
    
    async fn validate_beacon_finality(
        &self,
        block_ref: &BlockRef,
        finality_type: EthereumFinalityType,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        // Verify block exists in finality client
        let block = self.finality_client.get_block(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        // Get beacon block
        let beacon_block = self.finality_client.get_beacon_block(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        match finality_type {
            EthereumFinalityType::BeaconFinalized => {
                // Verify block is finalized in beacon chain
                let is_finalized = self.finality_client.is_block_finalized(block_ref).await
                    .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
                    
                if !is_finalized {
                    return Ok(PredicateResult {
                        is_satisfied: false,
                        confidence: 0.0,
                        evaluation_time: Duration::from_secs(0),
                        validation_data: serde_json::json!({
                            "status": "not_finalized",
                            "beacon_block": beacon_block.slot,
                        }),
                    });
                }
                
                Ok(PredicateResult {
                    is_satisfied: true,
                    confidence: 1.0,
                    evaluation_time: Duration::from_secs(0),
                    validation_data: serde_json::json!({
                        "status": "finalized",
                        "beacon_block": beacon_block.slot,
                        "finalization_epoch": beacon_block.epoch,
                    }),
                })
            }
            EthereumFinalityType::BeaconJustified => {
                // Verify block is justified in beacon chain
                let is_justified = self.finality_client.is_block_justified(block_ref).await
                    .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
                    
                if !is_justified {
                    return Ok(PredicateResult {
                        is_satisfied: false,
                        confidence: 0.0,
                        evaluation_time: Duration::from_secs(0),
                        validation_data: serde_json::json!({
                            "status": "not_justified",
                            "beacon_block": beacon_block.slot,
                        }),
                    });
                }
                
                Ok(PredicateResult {
                    is_satisfied: true,
                    confidence: 0.9, // Justified has slightly lower confidence than finalized
                    evaluation_time: Duration::from_secs(0),
                    validation_data: serde_json::json!({
                        "status": "justified",
                        "beacon_block": beacon_block.slot,
                        "justification_epoch": beacon_block.epoch,
                    }),
                })
            }
            _ => Err(PredicateError::InvalidFormat(
                "Invalid beacon finality type".into()
            )),
        }
    }
}

#[async_trait::async_trait]
impl PredicateValidator for EthereumPredicateValidator {
    async fn validate_predicate(
        &self,
        block_ref: &BlockRef,
        signal: &FinalitySignal,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        let start = std::time::Instant::now();
        
        let result = match signal {
            FinalitySignal::Ethereum {
                confirmations,
                finality_type,
                ..
            } => {
                match finality_type {
                    EthereumFinalityType::Confirmations => {
                        self.validate_pow_confirmations(block_ref, *confirmations, config).await
                    }
                    _ => self.validate_beacon_finality(block_ref, *finality_type, config).await,
                }
            }
            _ => Err(PredicateError::InvalidFormat(
                "Not an Ethereum finality signal".into()
            )),
        };
        
        // Check timeout
        let elapsed = start.elapsed();
        if elapsed > config.evaluation_timeout {
            return Err(PredicateError::Timeout(
                format!("Predicate evaluation exceeded timeout: {:?}", elapsed)
            ));
        }
        
        // Update evaluation time in result
        if let Ok(mut predicate_result) = result {
            predicate_result.evaluation_time = elapsed;
            Ok(predicate_result)
        } else {
            result
        }
    }
}

/// Solana predicate validator
pub struct SolanaPredicateValidator {
    finality_client: Box<dyn FinalityVerificationClient>,
}

impl SolanaPredicateValidator {
    pub fn new(finality_client: Box<dyn FinalityVerificationClient>) -> Self {
        Self { finality_client }
    }
    
    async fn validate_vote_accounts(
        &self,
        block_ref: &BlockRef,
        vote_signatures: &[Vec<u8>],
        metadata: &Option<SolanaMetadata>,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        // Verify block exists in finality client
        let block = self.finality_client.get_block(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        if let Some(metadata) = metadata {
            // Verify vote signatures
            let valid_signatures = self.finality_client.verify_vote_signatures(
                block_ref,
                vote_signatures,
            ).await.map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
            if !valid_signatures {
                return Err(PredicateError::ValidationFailed(
                    "Invalid vote signatures".into()
                ));
            }
            
            // Calculate stake-weighted confidence
            let stake_ratio = metadata.vote_account_stake as f64 / metadata.total_active_stake as f64;
            let confidence = if stake_ratio >= 2.0/3.0 { 1.0 } else { stake_ratio * 1.5 };
            
            Ok(PredicateResult {
                is_satisfied: stake_ratio >= 2.0/3.0,
                confidence,
                evaluation_time: Duration::from_secs(0),
                validation_data: serde_json::json!({
                    "vote_account_stake": metadata.vote_account_stake,
                    "total_active_stake": metadata.total_active_stake,
                    "stake_ratio": stake_ratio,
                    "super_majority_root": metadata.super_majority_root,
                }),
            })
        } else {
            Err(PredicateError::ValidationFailed("Missing Solana metadata".into()))
        }
    }
}

#[async_trait::async_trait]
impl PredicateValidator for SolanaPredicateValidator {
    async fn validate_predicate(
        &self,
        block_ref: &BlockRef,
        signal: &FinalitySignal,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        let start = std::time::Instant::now();
        
        let result = match signal {
            FinalitySignal::Solana {
                vote_account_signatures,
                metadata,
                ..
            } => {
                self.validate_vote_accounts(block_ref, vote_account_signatures, metadata, config).await
            }
            _ => Err(PredicateError::InvalidFormat(
                "Not a Solana finality signal".into()
            )),
        };
        
        // Check timeout
        let elapsed = start.elapsed();
        if elapsed > config.evaluation_timeout {
            return Err(PredicateError::Timeout(
                format!("Predicate evaluation exceeded timeout: {:?}", elapsed)
            ));
        }
        
        // Update evaluation time in result
        if let Ok(mut predicate_result) = result {
            predicate_result.evaluation_time = elapsed;
            Ok(predicate_result)
        } else {
            result
        }
    }
}

/// Cosmos predicate validator
pub struct CosmosPredicateValidator {
    finality_client: Box<dyn FinalityVerificationClient>,
}

impl CosmosPredicateValidator {
    pub fn new(finality_client: Box<dyn FinalityVerificationClient>) -> Self {
        Self { finality_client }
    }
    
    async fn validate_validator_signatures(
        &self,
        block_ref: &BlockRef,
        signatures: &[Vec<u8>],
        metadata: &Option<CosmosMetadata>,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        // Verify block exists in finality client
        let block = self.finality_client.get_block(block_ref).await
            .map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
        if let Some(metadata) = metadata {
            // Verify validator signatures
            let valid_signatures = self.finality_client.verify_validator_signatures(
                block_ref,
                signatures,
            ).await.map_err(|e| PredicateError::FinalityVerificationError(e.to_string()))?;
            
            if !valid_signatures {
                return Err(PredicateError::ValidationFailed(
                    "Invalid validator signatures".into()
                ));
            }
            
            // Calculate voting power confidence
            let power_ratio = metadata.validator_power as f64 / metadata.total_voting_power as f64;
            let confidence = if power_ratio >= 2.0/3.0 { 1.0 } else { power_ratio * 1.5 };
            
            Ok(PredicateResult {
                is_satisfied: power_ratio >= 2.0/3.0,
                confidence,
                evaluation_time: Duration::from_secs(0),
                validation_data: serde_json::json!({
                    "validator_power": metadata.validator_power,
                    "total_voting_power": metadata.total_voting_power,
                    "power_ratio": power_ratio,
                    "app_version": metadata.app_version,
                }),
            })
        } else {
            Err(PredicateError::ValidationFailed("Missing Cosmos metadata".into()))
        }
    }
}

#[async_trait::async_trait]
impl PredicateValidator for CosmosPredicateValidator {
    async fn validate_predicate(
        &self,
        block_ref: &BlockRef,
        signal: &FinalitySignal,
        config: &PredicateConfig,
    ) -> Result<PredicateResult, PredicateError> {
        let start = std::time::Instant::now();
        
        let result = match signal {
            FinalitySignal::Cosmos {
                validator_signatures,
                metadata,
                ..
            } => {
                self.validate_validator_signatures(
                    block_ref,
                    validator_signatures,
                    metadata,
                    config,
                ).await
            }
            _ => Err(PredicateError::InvalidFormat(
                "Not a Cosmos finality signal".into()
            )),
        };
        
        // Check timeout
        let elapsed = start.elapsed();
        if elapsed > config.evaluation_timeout {
            return Err(PredicateError::Timeout(
                format!("Predicate evaluation exceeded timeout: {:?}", elapsed)
            ));
        }
        
        // Update evaluation time in result
        if let Ok(mut predicate_result) = result {
            predicate_result.evaluation_time = elapsed;
            Ok(predicate_result)
        } else {
            result
        }
    }
}

/// Finality verification client interface for cross-chain finality validation
#[async_trait::async_trait]
pub trait FinalityVerificationClient: Send + Sync {
    /// Get block by reference
    async fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError>;
    
    /// Verify block hash
    async fn verify_block_hash(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
    
    /// Get beacon block (Ethereum specific)
    async fn get_beacon_block(&self, block_ref: &BlockRef) -> Result<BeaconBlock, FinalityVerificationError>;
    
    /// Check if block is finalized (Ethereum specific)
    async fn is_block_finalized(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
    
    /// Check if block is justified (Ethereum specific)
    async fn is_block_justified(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError>;
    
    /// Verify vote signatures (Solana specific)
    async fn verify_vote_signatures(
        &self,
        block_ref: &BlockRef,
        signatures: &[Vec<u8>],
    ) -> Result<bool, FinalityVerificationError>;
    
    /// Verify validator signatures (Cosmos specific)
    async fn verify_validator_signatures(
        &self,
        block_ref: &BlockRef,
        signatures: &[Vec<u8>],
    ) -> Result<bool, FinalityVerificationError>;

    /// Get the latest finalized block number
    async fn get_latest_finalized_block(&self) -> Result<u64, FinalityVerificationError>;

    /// Get the current chain head
    async fn get_chain_head(&self) -> Result<BlockRef, FinalityVerificationError>;

    /// Verify block inclusion proof
    async fn verify_block_inclusion(
        &self,
        block_ref: &BlockRef,
        proof: &[u8],
    ) -> Result<bool, FinalityVerificationError>;

    /// Get block finality confidence (0.0 - 1.0)
    async fn get_finality_confidence(
        &self,
        block_ref: &BlockRef,
    ) -> Result<f64, FinalityVerificationError>;

    /// Verify chain-specific rules
    async fn verify_chain_rules(
        &self,
        block_ref: &BlockRef,
        rules: &ChainRules,
    ) -> Result<bool, FinalityVerificationError>;
}

/// Chain-specific validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRules {
    /// Minimum required confirmations
    pub min_confirmations: u32,

    /// Required confidence threshold (0.0 - 1.0)
    pub confidence_threshold: f64,

    /// Maximum allowed fork depth
    pub max_fork_depth: u32,

    /// Required validator participation (0.0 - 1.0)
    pub min_participation: f64,

    /// Chain-specific parameters
    pub chain_params: serde_json::Value,
}

/// Cached block data
#[derive(Debug, Clone)]
struct CachedBlockData {
    block: Block,
    finality_confidence: f64,
    last_updated: std::time::SystemTime,
    verification_count: u64,
}

/// Caching finality verification client implementation
pub struct CachingFinalityClient<C: FinalityVerificationClient> {
    inner: C,
    block_cache: Arc<RwLock<LruCache<BlockRef, CachedBlockData>>>,
    cache_ttl: Duration,
    metrics: Arc<RwLock<VerificationMetrics>>,
}

impl<C: FinalityVerificationClient> CachingFinalityClient<C> {
    pub fn new(inner: C, cache_size: usize, cache_ttl: Duration) -> Self {
        Self {
            inner,
            block_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))),
            cache_ttl,
            metrics: Arc::new(RwLock::new(VerificationMetrics::default())),
        }
    }

    async fn get_cached_block(&self, block_ref: &BlockRef) -> Option<CachedBlockData> {
        let cache = self.block_cache.read().await;
        if let Some(data) = cache.peek(block_ref) {
            if data.last_updated.elapsed().unwrap() < self.cache_ttl {
                return Some(data.clone());
            }
        }
        None
    }

    async fn cache_block(&self, block_ref: BlockRef, data: CachedBlockData) {
        let mut cache = self.block_cache.write().await;
        cache.put(block_ref, data);
    }

    async fn update_metrics(&self, start: std::time::Instant, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.total_verifications += 1;
        if !success {
            metrics.failed_verifications += 1;
        }
        metrics.avg_verification_time = (metrics.avg_verification_time * (metrics.total_verifications - 1) as f64
            + start.elapsed().as_secs_f64()) / metrics.total_verifications as f64;
    }
}

#[async_trait::async_trait]
impl<C: FinalityVerificationClient> FinalityVerificationClient for CachingFinalityClient<C> {
    async fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError> {
        if let Some(cached) = self.get_cached_block(block_ref).await {
            return Ok(cached.block);
        }
        
        let start = std::time::Instant::now();
        let result = self.inner.get_block(block_ref).await;
        
        if let Ok(block) = &result {
            self.cache_block(
                block_ref.clone(),
                CachedBlockData {
                    block: block.clone(),
                    finality_confidence: 0.0,
                    last_updated: std::time::SystemTime::now(),
                    verification_count: 1,
                },
            ).await;
        }
        
        self.update_metrics(start, result.is_ok()).await;
        result
    }

    // ... implement other trait methods with caching ...
}

/// Performance metrics for verification
#[derive(Debug, Default)]
pub struct VerificationMetrics {
    pub total_verifications: u64,
    pub failed_verifications: u64,
    pub avg_verification_time: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
} 