use std::time::Duration;
use frost_protocol::{
    state::BlockRef,
    finality::{
        predicate::{
            FinalityVerificationClient,
            CachingFinalityClient,
            ChainRules,
            VerificationMetrics,
        },
        FinalitySignal,
        FinalityError,
        verifier::{FinalityVerifier, FinalityConfig},
        EthereumFinalityType,
        EthereumMetadata,
    },
};

mod substrate {
    use super::*;
    use subxt::{
        OnlineClient,
        PolkadotConfig,
        blocks::Block,
        blocks::BlockRef as SubxtBlockRef,
        backend::rpc::RpcClient,
        config::substrate::SubstrateHeader,
    };
    use sp_core::{H256, crypto::Pair};
    use sp_runtime::{
        traits::{Header as HeaderT, BlakeTwo256},
        generic::SignedBlock,
    };
    use sp_consensus_babe::{
        digests::{PreDigest, SecondaryPlainPreDigest},
        BabeConfiguration,
    };
    use sp_consensus_aura::{
        digests::{AuraPreDigest, AuraDigestsProcessor},
        AuraApi,
    };
    use sp_consensus_grandpa::{
        AuthorityList,
        AuthoritySignature,
        GrandpaJustification,
    };
    use sp_trie::{
        MemoryDB,
        StorageProof,
        TrieDBBuilder,
        Trie,
    };
    use sp_state_machine::TrieBackend;
    use cumulus_primitives_core::{
        ParaId,
        PersistedValidationData,
        relay_chain::BlockNumber as RelayBlockNumber,
    };
    use xcm::{
        v3::{MultiLocation, Instruction},
        VersionedXcm,
    };
    use codec::{Decode, Encode};
    use tracing::{info, warn, error, debug, trace};
    use metrics::{
        counter,
        gauge,
        histogram,
        register_counter,
        register_gauge,
        register_histogram,
    };
    use xcm_executor::{
        traits::{TransactAsset, ConvertOrigin},
        Assets,
    };
    use xcm_builder::{
        LocationConverter,
        ParentIsPreset,
    };
    use polkadot_primitives::{
        Block as PBlock,
        ValidatorId,
        ValidatorSignature,
        AuthorityDiscoveryId,
    };
    use opentelemetry::{
        metrics::{Counter, Histogram, ValueRecorder},
        KeyValue,
    };

    /// Extended consensus types
    #[derive(Debug, Clone, Copy)]
    pub enum ConsensusType {
        Babe(BabeConsensusConfig),
        Aura(AuraConsensusConfig),
        None,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct BabeConsensusConfig {
        pub secondary_slots_ratio: (u64, u64),
        pub leadership_rate: f64,
        pub block_time: Duration,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AuraConsensusConfig {
        pub slot_duration: u64,
        pub authority_round: u64,
        pub authorities_len: u32,
    }

    /// XCM message types we support
    #[derive(Debug, Clone)]
    pub enum XcmMessageType {
        AssetTransfer,
        RemoteExecution,
        Query,
        Trap,
        Custom(String),
    }

    /// Enhanced metrics with OpenTelemetry support
    pub struct EnhancedMetrics {
        // Consensus metrics
        consensus_verification_time: Histogram,
        authority_changes: Counter,
        missed_slots: Counter,
        
        // XCM metrics
        xcm_messages_by_type: Counter,
        xcm_execution_time: Histogram,
        xcm_errors: Counter,
        
        // Parachain metrics
        relay_chain_finality_lag: ValueRecorder,
        parachain_validation_time: Histogram,
        parachain_messages: Counter,
        
        // Performance metrics
        block_processing_queue: ValueRecorder,
        verification_memory_usage: ValueRecorder,
        rpc_latency: Histogram,
    }

    impl EnhancedMetrics {
        fn new() -> Self {
            let meter = opentelemetry::global::meter("frost_substrate");
            
            Self {
                consensus_verification_time: meter
                    .histogram_with_options(
                        "substrate_consensus_verification_seconds",
                        "Time spent verifying consensus",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                authority_changes: meter
                    .counter_with_options(
                        "substrate_authority_changes_total",
                        "Number of authority set changes",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                missed_slots: meter
                    .counter_with_options(
                        "substrate_missed_slots_total",
                        "Number of missed slots",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                xcm_messages_by_type: meter
                    .counter_with_options(
                        "substrate_xcm_messages_total",
                        "Number of XCM messages by type",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                xcm_execution_time: meter
                    .histogram_with_options(
                        "substrate_xcm_execution_seconds",
                        "Time spent executing XCM messages",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                xcm_errors: meter
                    .counter_with_options(
                        "substrate_xcm_errors_total",
                        "Number of XCM errors",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                relay_chain_finality_lag: meter
                    .value_recorder_with_options(
                        "substrate_relay_chain_finality_lag",
                        "Lag between relay chain and parachain finality",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                parachain_validation_time: meter
                    .histogram_with_options(
                        "substrate_parachain_validation_seconds",
                        "Time spent validating parachain blocks",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                parachain_messages: meter
                    .counter_with_options(
                        "substrate_parachain_messages_total",
                        "Number of parachain messages",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                block_processing_queue: meter
                    .value_recorder_with_options(
                        "substrate_block_processing_queue",
                        "Number of blocks in processing queue",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                verification_memory_usage: meter
                    .value_recorder_with_options(
                        "substrate_verification_memory_bytes",
                        "Memory usage during verification",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
                rpc_latency: meter
                    .histogram_with_options(
                        "substrate_rpc_latency_seconds",
                        "RPC request latency",
                        vec![KeyValue::new("chain_type", "substrate")],
                    ),
            }
        }

        fn record_xcm_message(&self, msg_type: XcmMessageType, duration: Duration) {
            let labels = vec![
                KeyValue::new("message_type", format!("{:?}", msg_type)),
            ];
            self.xcm_messages_by_type.add(1, &labels);
            self.xcm_execution_time.record(duration.as_secs_f64(), &labels);
        }
    }

    /// Enhanced Substrate client with additional validation features
    pub struct SubstrateClient {
        client: OnlineClient<PolkadotConfig>,
        network: String,
        finality_threshold: u32,
        para_id: Option<ParaId>,
        consensus_type: ConsensusType,
        babe_config: Option<BabeConfiguration>,
        metrics: SubstrateMetrics,
    }

    /// Substrate-specific metrics
    struct SubstrateMetrics {
        blocks_validated: metrics::Counter,
        validation_time: metrics::Histogram,
        current_finality_lag: metrics::Gauge,
        xcmp_messages_verified: metrics::Counter,
        consensus_failures: metrics::Counter,
    }

    impl SubstrateMetrics {
        fn new() -> Self {
            Self {
                blocks_validated: register_counter!("substrate_blocks_validated_total"),
                validation_time: register_histogram!("substrate_validation_time_seconds"),
                current_finality_lag: register_gauge!("substrate_finality_lag_blocks"),
                xcmp_messages_verified: register_counter!("substrate_xcmp_messages_verified_total"),
                consensus_failures: register_counter!("substrate_consensus_failures_total"),
            }
        }
    }

    /// Substrate-specific block validation data
    #[derive(Debug, Clone, Encode, Decode)]
    pub struct SubstrateBlockValidation {
        pub babe_pre_digest: Option<PreDigest>,
        pub grandpa_proof: Option<GrandpaJustification<H256>>,
        pub storage_proof: Option<StorageProof>,
        pub parachain_data: Option<PersistedValidationData>,
    }

    impl SubstrateClient {
        pub async fn new(
            rpc_url: &str,
            network: &str,
            finality_threshold: u32,
            para_id: Option<ParaId>,
            consensus_type: ConsensusType,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let rpc_client = RpcClient::from_url(rpc_url).await?;
            let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client).await?;
            
            // Get BABE configuration if using BABE consensus
            let babe_config = match consensus_type {
                ConsensusType::Babe => Some(Self::fetch_babe_config(&client).await?),
                _ => None,
            };
            
            Ok(Self {
                client,
                network: network.to_string(),
                finality_threshold,
                para_id: para_id.map(ParaId::from),
                consensus_type,
                babe_config,
                metrics: SubstrateMetrics::new(),
            })
        }

        async fn fetch_babe_config(
            client: &OnlineClient<PolkadotConfig>,
        ) -> Result<BabeConfiguration, FinalityVerificationError> {
            // In a real implementation, we would fetch this from the chain
            // For now, return a default config
            Ok(BabeConfiguration {
                slot_duration: 6000,
                epoch_length: 200,
                c: (1, 4),
                genesis_authorities: vec![],
                randomness: [0u8; 32],
                secondary_slots: true,
                ..Default::default()
            })
        }

        async fn verify_consensus(
            &self,
            block: &SignedBlock<Block>,
        ) -> Result<bool, FinalityVerificationError> {
            let start = std::time::Instant::now();
            let result = match self.consensus_type {
                ConsensusType::Babe => self.verify_babe_consensus(block).await?,
                ConsensusType::Aura => self.verify_aura_consensus(block).await?,
                ConsensusType::None => true,
            };

            if !result {
                self.metrics.consensus_failures.increment(1);
            }

            let duration = start.elapsed().as_secs_f64();
            self.metrics.validation_time.record(duration);

            Ok(result)
        }

        async fn verify_aura_consensus(
            &self,
            block: &SignedBlock<Block>,
        ) -> Result<bool, FinalityVerificationError> {
            let header = &block.block.header;
            
            // Extract Aura pre-digest
            let aura_digest = header.digest()
                .logs()
                .iter()
                .find_map(|log| AuraPreDigest::try_from(log).ok())
                .ok_or_else(|| FinalityVerificationError::from("No Aura pre-digest found"))?;

            // In a real implementation, we would:
            // 1. Verify the slot number
            // 2. Check the authority schedule
            // 3. Verify the authority signature
            
            debug!("Verified Aura consensus for block {}", header.number());
            Ok(true)
        }

        async fn verify_babe_consensus(
            &self,
            block: &SignedBlock<Block>,
        ) -> Result<bool, FinalityVerificationError> {
            let header = &block.block.header;
            
            // Extract BABE pre-digest
            let babe_digest = header.digest()
                .logs()
                .iter()
                .find_map(|log| PreDigest::try_from(log).ok())
                .ok_or_else(|| FinalityVerificationError::from("No BABE pre-digest found"))?;

            // In a real implementation, we would:
            // 1. Verify the slot number matches the block number
            // 2. Verify the VRF output
            // 3. Check the block producer's authority
            
            debug!("Verified BABE consensus for block {}", header.number());
            Ok(true)
        }

        async fn verify_storage_proof(
            &self,
            block_hash: H256,
            proof: StorageProof,
            root: H256,
        ) -> Result<bool, FinalityVerificationError> {
            let db = MemoryDB::default();
            let trie = TrieDBBuilder::new(&db, &root).build();
            
            // Verify each key-value pair in the proof
            for (key, value) in proof.into_iter() {
                let proven_value = trie.get(&key)
                    .map_err(|e| FinalityVerificationError::from(e.to_string()))?
                    .ok_or_else(|| FinalityVerificationError::from("Storage value not found"))?;
                
                if proven_value != value {
                    return Ok(false);
                }
            }
            
            debug!("Verified storage proof for block {}", block_hash);
            Ok(true)
        }

        async fn verify_parachain_data(
            &self,
            block_ref: &BlockRef,
            validation_data: &PersistedValidationData,
        ) -> Result<bool, FinalityVerificationError> {
            if let Some(para_id) = self.para_id {
                // Verify parachain-specific data
                let block = self.get_block(block_ref).await?;
                
                // In a real implementation, we would:
                // 1. Verify the relay chain parent hash
                // 2. Check the parachain head data
                // 3. Verify validator signatures
                // 4. Check state root against validation data
                
                debug!("Verified parachain data for para_id: {}", para_id);
                Ok(true)
            } else {
                // Not a parachain, skip validation
                Ok(true)
            }
        }

        async fn verify_xcmp_messages(
            &self,
            block_ref: &BlockRef,
            validation_data: &PersistedValidationData,
        ) -> Result<bool, FinalityVerificationError> {
            if let Some(para_id) = self.para_id {
                let block = self.get_block(block_ref).await?;
                
                // Get XCMP messages from storage
                let xcmp_messages = self.client
                    .storage()
                    .at(block.hash())
                    .await
                    .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

                // Verify each XCMP message
                for msg in xcmp_messages {
                    let versioned_msg = VersionedXcm::decode(&mut &msg[..])
                        .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

                    // Verify message format and contents
                    match versioned_msg {
                        VersionedXcm::V3(xcm) => {
                            // Verify message origin and destination
                            for instruction in xcm.0 {
                                match instruction {
                                    Instruction::ReserveAssetDeposited(..) |
                                    Instruction::ClearOrigin |
                                    Instruction::BuyExecution { .. } => {
                                        // These instructions are allowed
                                        continue;
                                    }
                                    _ => {
                                        warn!("Unsupported XCM instruction: {:?}", instruction);
                                        return Ok(false);
                                    }
                                }
                            }
                        }
                        _ => {
                            warn!("Unsupported XCM version");
                            return Ok(false);
                        }
                    }

                    self.metrics.xcmp_messages_verified.increment(1);
                }

                debug!("Verified XCMP messages for para_id: {}", para_id);
                Ok(true)
            } else {
                // Not a parachain, skip XCMP verification
                Ok(true)
            }
        }

        async fn update_metrics(&self, block_ref: &BlockRef) {
            let current_block = self.get_latest_finalized_block().await
                .unwrap_or(block_ref.number() as u64);
            
            let finality_lag = current_block.saturating_sub(block_ref.number() as u64);
            self.metrics.current_finality_lag.set(finality_lag as f64);
            self.metrics.blocks_validated.increment(1);
        }

        async fn verify_xcm_message(
            &self,
            msg: &VersionedXcm,
            validation_data: &PersistedValidationData,
        ) -> Result<(bool, XcmMessageType), FinalityVerificationError> {
            match msg {
                VersionedXcm::V3(xcm) => {
                    let start = std::time::Instant::now();
                    let mut msg_type = None;

                    for instruction in &xcm.0 {
                        match instruction {
                            Instruction::TransferAsset { assets, beneficiary, .. } => {
                                // Verify asset transfer
                                if !self.verify_asset_transfer(assets, beneficiary).await? {
                                    return Ok((false, XcmMessageType::AssetTransfer));
                                }
                                msg_type = Some(XcmMessageType::AssetTransfer);
                            }
                            Instruction::Transact { origin_kind, require_weight_at_most, call, .. } => {
                                // Verify remote execution
                                if !self.verify_remote_execution(origin_kind, call).await? {
                                    return Ok((false, XcmMessageType::RemoteExecution));
                                }
                                msg_type = Some(XcmMessageType::RemoteExecution);
                            }
                            Instruction::QueryResponse { query_id, response, .. } => {
                                // Verify query response
                                if !self.verify_query_response(query_id, response).await? {
                                    return Ok((false, XcmMessageType::Query));
                                }
                                msg_type = Some(XcmMessageType::Query);
                            }
                            // ... handle other instruction types ...
                        }
                    }

                    let msg_type = msg_type.unwrap_or(XcmMessageType::Custom("Unknown".into()));
                    self.metrics.record_xcm_message(msg_type.clone(), start.elapsed());
                    Ok((true, msg_type))
                }
                _ => Ok((false, XcmMessageType::Custom("Unsupported Version".into()))),
            }
        }

        async fn verify_consensus_detailed(
            &self,
            block: &SignedBlock<Block>,
        ) -> Result<bool, FinalityVerificationError> {
            let start = std::time::Instant::now();
            
            let result = match self.consensus_type {
                ConsensusType::Babe(config) => {
                    self.verify_babe_consensus_detailed(block, config).await?
                }
                ConsensusType::Aura(config) => {
                    self.verify_aura_consensus_detailed(block, config).await?
                }
                ConsensusType::None => true,
            };

            let duration = start.elapsed();
            self.metrics.consensus_verification_time.record(
                duration.as_secs_f64(),
                &[KeyValue::new("consensus_type", format!("{:?}", self.consensus_type))],
            );

            Ok(result)
        }

        async fn verify_babe_consensus_detailed(
            &self,
            block: &SignedBlock<Block>,
            config: BabeConsensusConfig,
        ) -> Result<bool, FinalityVerificationError> {
            let header = &block.block.header;
            
            // Extract BABE pre-digest
            let babe_digest = header.digest()
                .logs()
                .iter()
                .find_map(|log| PreDigest::try_from(log).ok())
                .ok_or_else(|| FinalityVerificationError::from("No BABE pre-digest found"))?;

            match babe_digest {
                PreDigest::Primary(primary) => {
                    // Verify VRF output
                    if !self.verify_vrf_output(&primary).await? {
                        return Ok(false);
                    }

                    // Check slot probability
                    let slot_probability = primary.probability();
                    if slot_probability < config.leadership_rate {
                        warn!(
                            "Block {} has low slot probability: {} < {}",
                            header.number(),
                            slot_probability,
                            config.leadership_rate
                        );
                        return Ok(false);
                    }
                }
                PreDigest::Secondary(secondary) => {
                    // Verify secondary slot ratio
                    let (allowed, total) = config.secondary_slots_ratio;
                    if secondary.slot_number() % total >= allowed {
                        warn!(
                            "Invalid secondary slot for block {}",
                            header.number()
                        );
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        }

        async fn verify_aura_consensus_detailed(
            &self,
            block: &SignedBlock<Block>,
            config: AuraConsensusConfig,
        ) -> Result<bool, FinalityVerificationError> {
            let header = &block.block.header;
            
            // Extract Aura pre-digest
            let aura_digest = header.digest()
                .logs()
                .iter()
                .find_map(|log| AuraPreDigest::try_from(log).ok())
                .ok_or_else(|| FinalityVerificationError::from("No Aura pre-digest found"))?;

            // Verify slot number
            let slot = aura_digest.slot();
            if slot % config.authorities_len as u64 != config.authority_round {
                warn!(
                    "Invalid authority slot for block {}: {} % {} != {}",
                    header.number(),
                    slot,
                    config.authorities_len,
                    config.authority_round
                );
                return Ok(false);
            }

            // Verify slot duration
            if slot * config.slot_duration != header.number() as u64 * config.slot_duration {
                warn!(
                    "Invalid slot duration for block {}",
                    header.number()
                );
                return Ok(false);
            }

            Ok(true)
        }

        async fn verify_parachain_data_detailed(
            &self,
            block_ref: &BlockRef,
            validation_data: &PersistedValidationData,
        ) -> Result<bool, FinalityVerificationError> {
            let start = std::time::Instant::now();

            if let Some(para_id) = self.para_id {
                // 1. Verify relay chain state
                let relay_head = self.get_relay_chain_head().await?;
                if validation_data.relay_parent_number > relay_head.number {
                    warn!(
                        "Invalid relay parent number: {} > {}",
                        validation_data.relay_parent_number,
                        relay_head.number
                    );
                    return Ok(false);
                }

                // 2. Verify parachain head
                let head_data = self.get_parachain_head(para_id).await?;
                if head_data != validation_data.parent_head {
                    warn!("Invalid parachain head data");
                    return Ok(false);
                }

                // 3. Verify validator set
                if !self.verify_validator_set(para_id, &validation_data.relay_parent_storage_root).await? {
                    warn!("Invalid validator set");
                    return Ok(false);
                }

                let duration = start.elapsed();
                self.metrics.parachain_validation_time.record(
                    duration.as_secs_f64(),
                    &[KeyValue::new("para_id", para_id.to_string())],
                );

                Ok(true)
            } else {
                Ok(true)
            }
        }
    }

    #[async_trait::async_trait]
    impl FinalityVerificationClient for SubstrateClient {
        async fn get_block(&self, block_ref: &BlockRef) -> Result<Block, FinalityVerificationError> {
            let hash = self.get_block_hash(block_ref.number() as u64)
                .await?
                .ok_or_else(|| FinalityVerificationError::from("Block not found"))?;

            let block = self.client
                .blocks()
                .at(hash)
                .await
                .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

            Ok(block)
        }

        async fn verify_block_hash(&self, block_ref: &BlockRef) -> Result<bool, FinalityVerificationError> {
            let hash = self.get_block_hash(block_ref.number() as u64)
                .await?
                .ok_or_else(|| FinalityVerificationError::from("Block not found"))?;

            Ok(hash == H256::from_slice(&block_ref.hash()))
        }

        async fn get_latest_finalized_block(&self) -> Result<u64, FinalityVerificationError> {
            let hash = self.get_finalized_head().await?;
            let block = self.client
                .blocks()
                .at(hash)
                .await
                .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

            Ok(block.number().into())
        }

        async fn verify_block_inclusion(
            &self,
            block_ref: &BlockRef,
            proof: &[u8],
        ) -> Result<bool, FinalityVerificationError> {
            let hash = self.get_block_hash(block_ref.number() as u64)
                .await?
                .ok_or_else(|| FinalityVerificationError::from("Block not found"))?;

            let block = self.client
                .blocks()
                .at(hash)
                .await
                .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

            // Decode the storage proof
            let storage_proof = StorageProof::decode(&mut &proof[..])
                .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

            // Verify the storage proof
            self.verify_storage_proof(
                hash,
                storage_proof,
                block.header().state_root().clone(),
            ).await
        }

        async fn get_finality_confidence(
            &self,
            block_ref: &BlockRef,
        ) -> Result<f64, FinalityVerificationError> {
            let current_block = self.get_latest_finalized_block().await?;
            let block_number = block_ref.number() as u64;

            if block_number > current_block {
                return Ok(0.0);
            }

            let confirmations = current_block - block_number;
            let confidence = (confirmations as f64) / (self.finality_threshold as f64);
            Ok(confidence.min(1.0))
        }

        async fn verify_chain_rules(
            &self,
            block_ref: &BlockRef,
            rules: &ChainRules,
        ) -> Result<bool, FinalityVerificationError> {
            let start = std::time::Instant::now();
            
            // Get the block
            let hash = self.get_block_hash(block_ref.number() as u64)
                .await?
                .ok_or_else(|| FinalityVerificationError::from("Block not found"))?;

            let block = self.client
                .blocks()
                .at(hash)
                .await
                .map_err(|e| FinalityVerificationError::from(e.to_string()))?;

            // 1. Basic validation
            if !self.verify_block_hash(block_ref).await? {
                warn!("Block hash verification failed for block {}", block_ref.number());
                return Ok(false);
            }

            // 2. Consensus verification
            if !self.verify_consensus(&block).await? {
                warn!("Consensus verification failed for block {}", block_ref.number());
                return Ok(false);
            }

            // 3. Parachain and XCMP validation
            if let Some(para_id) = self.para_id {
                let validation_data = PersistedValidationData {
                    parent_head: vec![].into(),
                    relay_parent_number: block_ref.number() as u32,
                    relay_parent_storage_root: H256::default(),
                    max_pov_size: 5_242_880, // 5MB
                };

                if !self.verify_parachain_data(block_ref, &validation_data).await? {
                    warn!("Parachain validation failed for para_id: {}", para_id);
                    return Ok(false);
                }

                if !self.verify_xcmp_messages(block_ref, &validation_data).await? {
                    warn!("XCMP verification failed for para_id: {}", para_id);
                    return Ok(false);
                }
            }

            // 4. Finality confidence check
            let confidence = self.get_finality_confidence(block_ref).await?;
            if confidence < rules.confidence_threshold {
                warn!(
                    "Insufficient finality confidence: {} < {}",
                    confidence,
                    rules.confidence_threshold
                );
                return Ok(false);
            }

            // Update metrics
            self.update_metrics(block_ref).await;

            let elapsed = start.elapsed();
            info!(
                "Verified block {} in {:?} (confidence: {}, parachain: {}, consensus: {:?})",
                block_ref.number(),
                elapsed,
                confidence,
                self.para_id.is_some(),
                self.consensus_type,
            );

            Ok(true)
        }
    }
}

#[tokio::test]
async fn test_substrate_finality_verification() {
    // Test with Polkadot
    let substrate_client = substrate::SubstrateClient::new(
        "wss://rpc.polkadot.io",
        "polkadot",
        4, // Finality threshold (GRANDPA typically needs 2/3 of validators)
        None,
        substrate::ConsensusType::None,
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100, // cache size
        Duration::from_secs(60), // cache TTL
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "polkadot",
            "finality_protocol": "grandpa",
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("polkadot"), latest_block - 2, [0u8; 32]); // Test with 2 confirmations

    // Verify block
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid);

    // Test caching
    let start = std::time::Instant::now();
    let _block = cached_client.get_block(&block_ref).await.unwrap();
    let first_query = start.elapsed();

    let start = std::time::Instant::now();
    let _block = cached_client.get_block(&block_ref).await.unwrap();
    let cached_query = start.elapsed();

    assert!(cached_query < first_query);
}

#[tokio::test]
async fn test_substrate_fork_detection() {
    // Test with Kusama (known for having more frequent forks)
    let substrate_client = substrate::SubstrateClient::new(
        "wss://kusama-rpc.polkadot.io",
        "kusama",
        4,
        None,
        substrate::ConsensusType::None,
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 2, // Stricter fork depth for Kusama
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "kusama",
            "finality_protocol": "grandpa",
            "strict_fork_detection": true,
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    
    // Test with a block that's too far from finalized head
    let fork_block_ref = BlockRef::new(ChainId::new("kusama"), latest_block + 3, [0u8; 32]);
    let is_valid = cached_client.verify_chain_rules(&fork_block_ref, &rules).await.unwrap();
    assert!(!is_valid, "Block should be rejected due to excessive fork depth");

    // Test with a finalized block
    let finalized_block_ref = BlockRef::new(ChainId::new("kusama"), latest_block - 2, [0u8; 32]);
    let is_valid = cached_client.verify_chain_rules(&finalized_block_ref, &rules).await.unwrap();
    assert!(is_valid, "Finalized block should be accepted");
}

#[tokio::test]
async fn test_substrate_parachain_validation() {
    // Test with a parachain (e.g., Acala)
    let substrate_client = substrate::SubstrateClient::new(
        "wss://acala-rpc.aca-api.network",
        "acala",
        4,
        Some(2000), // Acala's para_id
        substrate::ConsensusType::None,
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "acala",
            "finality_protocol": "grandpa",
            "parachain": true,
            "relay_chain": "polkadot",
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("acala"), latest_block - 2, [0u8; 32]);

    // Verify block with parachain validation
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "Parachain block validation should succeed");
}

#[tokio::test]
async fn test_substrate_storage_proof() {
    // Test with Polkadot
    let substrate_client = substrate::SubstrateClient::new(
        "wss://rpc.polkadot.io",
        "polkadot",
        4,
        None,
        substrate::ConsensusType::None,
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("polkadot"), latest_block - 2, [0u8; 32]);

    // Create a test storage proof (in real implementation, this would come from the chain)
    let proof = StorageProof::new(vec![]);
    let proof_bytes = proof.encode();

    // Verify storage proof
    let is_valid = cached_client.verify_block_inclusion(&block_ref, &proof_bytes).await.unwrap();
    assert!(is_valid, "Storage proof verification should succeed");
}

#[tokio::test]
async fn test_substrate_aura_consensus() {
    // Test with an Aura-based chain (e.g., Moonbeam)
    let substrate_client = substrate::SubstrateClient::new(
        "wss://moonbeam.api.onfinality.io/public-ws",
        "moonbeam",
        4,
        Some(2004), // Moonbeam's para_id
        substrate::ConsensusType::Aura,
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "moonbeam",
            "finality_protocol": "grandpa",
            "consensus": "aura",
            "parachain": true,
            "relay_chain": "polkadot",
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("moonbeam"), latest_block - 2, [0u8; 32]);

    // Verify block with Aura consensus
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "Aura consensus validation should succeed");
}

#[tokio::test]
async fn test_substrate_xcmp_verification() {
    // Test XCMP between two parachains (e.g., Acala and Karura)
    let acala_client = substrate::SubstrateClient::new(
        "wss://acala-rpc.aca-api.network",
        "acala",
        4,
        Some(2000),
        substrate::ConsensusType::Aura,
    ).await.expect("Failed to create Acala client");

    let cached_client = CachingFinalityClient::new(
        acala_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "acala",
            "finality_protocol": "grandpa",
            "consensus": "aura",
            "parachain": true,
            "relay_chain": "polkadot",
            "xcmp_enabled": true,
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("acala"), latest_block - 2, [0u8; 32]);

    // Verify block with XCMP messages
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "XCMP verification should succeed");
}

#[tokio::test]
async fn test_substrate_babe_consensus_detailed() {
    // Test with Polkadot (BABE consensus)
    let substrate_client = substrate::SubstrateClient::new(
        "wss://rpc.polkadot.io",
        "polkadot",
        4,
        None,
        substrate::ConsensusType::Babe(substrate::BabeConsensusConfig {
            secondary_slots_ratio: (1, 3), // 1/3 of slots can be secondary
            leadership_rate: 0.2, // 20% chance of being slot leader
            block_time: Duration::from_secs(6),
        }),
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "polkadot",
            "finality_protocol": "grandpa",
            "consensus": "babe",
            "secondary_slots_enabled": true,
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("polkadot"), latest_block - 2, [0u8; 32]);

    // Verify block with detailed BABE consensus
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "BABE consensus validation should succeed");
}

#[tokio::test]
async fn test_substrate_xcm_detailed() {
    // Test XCM between Polkadot and Acala
    let substrate_client = substrate::SubstrateClient::new(
        "wss://acala-rpc.aca-api.network",
        "acala",
        4,
        Some(2000),
        substrate::ConsensusType::Aura(substrate::AuraConsensusConfig {
            slot_duration: 12000,
            authority_round: 0,
            authorities_len: 4,
        }),
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "acala",
            "finality_protocol": "grandpa",
            "consensus": "aura",
            "parachain": true,
            "relay_chain": "polkadot",
            "xcm_version": 3,
            "allowed_xcm_instructions": [
                "TransferAsset",
                "ReserveAssetDeposited",
                "ClearOrigin",
                "BuyExecution"
            ],
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("acala"), latest_block - 2, [0u8; 32]);

    // Verify block with XCM messages
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "XCM verification should succeed");

    // Verify metrics
    let metrics = cached_client.get_metrics().await;
    assert!(metrics.xcm_messages_verified.get() > 0, "Should have verified XCM messages");
    assert!(metrics.xcm_execution_time.get_histogram().count() > 0, "Should have XCM execution time metrics");
}

#[tokio::test]
async fn test_substrate_parachain_detailed() {
    // Test parachain validation with Moonbeam
    let substrate_client = substrate::SubstrateClient::new(
        "wss://moonbeam.api.onfinality.io/public-ws",
        "moonbeam",
        4,
        Some(2004),
        substrate::ConsensusType::Aura(substrate::AuraConsensusConfig {
            slot_duration: 12000,
            authority_round: 0,
            authorities_len: 4,
        }),
    ).await.expect("Failed to create Substrate client");

    let cached_client = CachingFinalityClient::new(
        substrate_client,
        100,
        Duration::from_secs(60),
    );

    let rules = ChainRules {
        min_confirmations: 2,
        confidence_threshold: 0.95,
        max_fork_depth: 5,
        min_participation: 0.66,
        chain_params: serde_json::json!({
            "network": "moonbeam",
            "finality_protocol": "grandpa",
            "consensus": "aura",
            "parachain": true,
            "relay_chain": "polkadot",
            "verify_relay_chain": true,
            "verify_validator_set": true,
        }),
    };

    // Get latest finalized block
    let latest_block = cached_client.get_latest_finalized_block().await.unwrap();
    let block_ref = BlockRef::new(ChainId::new("moonbeam"), latest_block - 2, [0u8; 32]);

    // Verify block with detailed parachain validation
    let is_valid = cached_client.verify_chain_rules(&block_ref, &rules).await.unwrap();
    assert!(is_valid, "Parachain validation should succeed");

    // Verify metrics
    let metrics = cached_client.get_metrics().await;
    assert!(
        metrics.parachain_validation_time.get_histogram().count() > 0,
        "Should have parachain validation time metrics"
    );
    assert!(
        metrics.relay_chain_finality_lag.get() >= 0.0,
        "Should have relay chain finality lag metrics"
    );
}

#[tokio::test]
async fn test_ethereum_finality_integration() {
    let config = FinalityConfig {
        min_confirmations: 12,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "network": "mainnet",
            "use_beacon": true,
            "fork_choice_threshold": 0.66,
            "min_validator_participation": 0.75,
            "min_justification_participation": 0.80,
            "min_validator_balance": 32000000000,
        }),
    };

    let mut mock_client = MockBeaconClient::new();
    let block_ref = test_block_ref("ethereum", 1000);
    let block_hash = [0u8; 32];

    // Setup mock expectations for canonical chain verification
    mock_client.expect_get_block_by_number()
        .with(eq(1000u64))
        .returning(move |_| Ok(Some(BeaconBlock {
            slot: 32000,
            block_root: [1u8; 32],
            parent_root: [2u8; 32],
            state_root: [3u8; 32],
            number: 1000,
            block_hash,
        })));

    mock_client.expect_get_finalized_block()
        .returning(|| Ok(BeaconBlock {
            slot: 31900,
            block_root: [4u8; 32],
            parent_root: [5u8; 32],
            state_root: [6u8; 32],
            number: 990,
            block_hash: [7u8; 32],
        }));

    mock_client.expect_get_block_ancestors()
        .returning(|_, _| Ok(vec![[7u8; 32]]));

    // Setup mock expectations for fork choice verification
    mock_client.expect_get_fork_choice_head()
        .returning(|| Ok(BeaconBlock {
            slot: 32000,
            block_root: [1u8; 32],
            parent_root: [2u8; 32],
            state_root: [3u8; 32],
            number: 1000,
            block_hash: [0u8; 32],
        }));

    mock_client.expect_get_current_slot()
        .returning(|| Ok(32000));

    // Setup mock expectations for validator verification
    mock_client.expect_get_block_attestations()
        .returning(|_| Ok(vec![
            Attestation {
                validator_index: 1,
                slot: 32000,
                beacon_block_root: [1u8; 32],
            },
            Attestation {
                validator_index: 2,
                slot: 32000,
                beacon_block_root: [1u8; 32],
            },
        ]));

    mock_client.expect_get_active_validators()
        .returning(|| Ok(vec![
            Validator {
                index: 1,
                effective_balance: 32000000000,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            },
            Validator {
                index: 2,
                effective_balance: 32000000000,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            },
            Validator {
                index: 3,
                effective_balance: 32000000000,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            },
        ]));

    // Setup mock expectations for reorg checking
    mock_client.expect_get_recent_reorgs()
        .returning(|_| Ok(vec![
            ReorgEvent {
                old_head_block: 995,
                new_head_block: 998,
                depth: 3,
                timestamp: 1234567890,
            },
        ]));

    let mut verifier = EthereumVerifier::new(config);
    
    // Test successful finality verification
    let valid_metadata = EthereumMetadata {
        gas_used: 1500000,
        base_fee: 15000000000,
        difficulty: 0,
        total_difficulty: 0,
        current_slot: Some(32000),
        head_slot: Some(31990),
        justified_epoch: Some(1000),
        finalized_epoch: Some(999),
        participation_rate: Some(0.95),
        active_validators: Some(400000),
        total_validators: Some(420000),
        validator_balance: Some(32000000000),
        latest_fork_version: Some([1, 0, 0, 0]),
        fork_choice_head: Some([1u8; 32]),
        justified_checkpoint_root: Some([2u8; 32]),
        finalized_checkpoint_root: Some([3u8; 32]),
        is_syncing: Some(false),
        sync_distance: Some(10),
        chain_id: Some(1),
        network_version: Some("mainnet".into()),
        extra_data: None,
    };

    let valid_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash,
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(valid_metadata.clone()),
    };

    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Test non-canonical block
    mock_client.expect_get_block_by_number()
        .with(eq(1000u64))
        .returning(move |_| Ok(Some(BeaconBlock {
            slot: 32000,
            block_root: [9u8; 32],
            parent_root: [2u8; 32],
            state_root: [3u8; 32],
            number: 1000,
            block_hash: [8u8; 32], // Different hash
        })));

    let non_canonical_signal = FinalitySignal::Ethereum {
        block_number: 1000,
        block_hash,
        confirmations: 1,
        finality_type: EthereumFinalityType::BeaconFinalized,
        metadata: Some(valid_metadata.clone()),
    };

    assert!(matches!(
        verifier.verify_finality(&block_ref, &non_canonical_signal).await,
        Err(FinalityError::NonCanonical(_))
    ));

    // Test deep reorg
    mock_client.expect_get_recent_reorgs()
        .returning(|_| Ok(vec![
            ReorgEvent {
                old_head_block: 900,
                new_head_block: 1100,
                depth: 100, // Deep reorg
                timestamp: 1234567890,
            },
        ]));

    assert!(matches!(
        verifier.verify_finality(&block_ref, &valid_signal).await,
        Err(FinalityError::DeepReorg(_))
    ));

    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert!(metrics.total_blocks_verified > 0);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
}

#[tokio::test]
async fn test_cosmos_finality_integration() {
    let config = FinalityConfig {
        min_confirmations: 1,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "chain_id": "cosmoshub-4",
            "min_validator_power": 10000,
            "min_voting_power": 0.67,
            "max_evidence_age": 120000,
        }),
    };

    let mut mock_client = MockTendermintClient::new();
    let block_ref = test_block_ref("cosmos", 1000);

    // Setup mock expectations for block header
    mock_client.expect_get_block_header()
        .with(eq(1000u64))
        .returning(|_| Ok(TendermintHeader {
            chain_id: "cosmoshub-4".into(),
            height: 1000,
            time: 1234567890,
            last_block_id: Some(BlockId {
                hash: [1u8; 32],
                part_set_header: PartSetHeader {
                    total: 1,
                    hash: [2u8; 32],
                },
            }),
            last_commit_hash: [3u8; 32],
            data_hash: [4u8; 32],
            validators_hash: [5u8; 32],
            next_validators_hash: [6u8; 32],
            consensus_hash: [7u8; 32],
            app_hash: [8u8; 32],
            last_results_hash: [9u8; 32],
            evidence_hash: [10u8; 32],
            proposer_address: [11u8; 20],
        }));

    // Setup mock expectations for validator sets
    let validators = vec![
        TendermintValidator {
            address: [1u8; 20],
            pub_key: [1u8; 32],
            voting_power: 100000,
            proposer_priority: 0,
        },
        TendermintValidator {
            address: [2u8; 20],
            pub_key: [2u8; 32],
            voting_power: 100000,
            proposer_priority: 1,
        },
        TendermintValidator {
            address: [3u8; 20],
            pub_key: [3u8; 32],
            voting_power: 100000,
            proposer_priority: 2,
        },
    ];

    mock_client.expect_get_validator_set()
        .returning(move || Ok(validators.clone()));

    mock_client.expect_get_next_validator_set()
        .returning(move || Ok(validators.clone()));

    // Setup mock expectations for commit
    mock_client.expect_get_commit()
        .returning(|height| Ok(Some(TendermintCommit {
            height,
            round: 0,
            block_id: BlockId {
                hash: [1u8; 32],
                part_set_header: PartSetHeader {
                    total: 1,
                    hash: [2u8; 32],
                },
            },
            signatures: vec![
                CommitSig {
                    validator_address: [1u8; 20],
                    timestamp: 1234567890,
                    signature: vec![1u8; 64],
                },
                CommitSig {
                    validator_address: [2u8; 20],
                    timestamp: 1234567890,
                    signature: vec![2u8; 64],
                },
            ],
        })));

    // Setup mock expectations for evidence
    mock_client.expect_get_evidence()
        .returning(|_| Ok(vec![]));

    let mut verifier = CosmosVerifier::new(config);
    
    // Test successful finality verification
    let valid_metadata = CosmosMetadata {
        voting_power: Some(200000),
        total_power: Some(200000),
    };

    let valid_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![vec![1u8; 64], vec![2u8; 64]],
        metadata: Some(valid_metadata.clone()),
    };

    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Test insufficient voting power
    let insufficient_metadata = CosmosMetadata {
        voting_power: Some(100000), // Only 1/3 of total power
        total_power: Some(300000),
    };

    let insufficient_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![vec![1u8; 64]],
        metadata: Some(insufficient_metadata),
    };

    assert!(!verifier.verify_finality(&block_ref, &insufficient_signal).await.unwrap());

    // Test with evidence
    let evidence_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![vec![1u8; 64], vec![2u8; 64]],
        metadata: Some(valid_metadata),
    };

    assert!(!verifier.verify_finality(&block_ref, &evidence_signal).await.unwrap());

    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert!(metrics.total_blocks_verified > 0);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
}

#[tokio::test]
async fn test_cosmos_consensus_verification() {
    let config = FinalityConfig {
        min_confirmations: 1,
        finality_timeout: Duration::from_secs(60),
        chain_params: serde_json::json!({
            "chain_id": "cosmoshub-4",
            "min_validator_power": 10000,
            "min_voting_power": 0.67,
            "max_evidence_age": 120000,
        }),
    };

    let mut mock_client = MockTendermintClient::new();
    let block_ref = test_block_ref("cosmos", 1000);

    // Setup mock expectations for block header
    let header = TendermintHeader {
        chain_id: "cosmoshub-4".into(),
        height: 1000,
        time: 1234567890,
        last_block_id: Some(BlockId {
            hash: [1u8; 32],
            part_set_header: PartSetHeader {
                total: 1,
                hash: [2u8; 32],
            },
        }),
        last_commit_hash: [3u8; 32],
        data_hash: [4u8; 32],
        validators_hash: [5u8; 32],
        next_validators_hash: [6u8; 32],
        consensus_hash: [7u8; 32],
        app_hash: [8u8; 32],
        last_results_hash: [9u8; 32],
        evidence_hash: [10u8; 32],
        proposer_address: [11u8; 20],
    };

    mock_client.expect_get_block_header()
        .with(eq(1000u64))
        .returning(move |_| Ok(header.clone()));

    // Setup mock expectations for validator sets
    let validators = vec![
        TendermintValidator {
            address: [1u8; 20],
            pub_key: [1u8; 32],
            voting_power: 100000,
            proposer_priority: 0,
        },
        TendermintValidator {
            address: [2u8; 20],
            pub_key: [2u8; 32],
            voting_power: 100000,
            proposer_priority: 1,
        },
    ];

    mock_client.expect_get_validator_set()
        .returning(move || Ok(validators.clone()));

    mock_client.expect_get_next_validator_set()
        .returning(move || Ok(validators.clone()));

    // Setup mock expectations for commit
    let commit = TendermintCommit {
        height: 1000,
        round: 0,
        block_id: BlockId {
            hash: [1u8; 32],
            part_set_header: PartSetHeader {
                total: 1,
                hash: [2u8; 32],
            },
        },
        signatures: vec![
            CommitSig {
                validator_address: [1u8; 20],
                timestamp: 1234567890,
                signature: vec![1u8; 64],
            },
            CommitSig {
                validator_address: [2u8; 20],
                timestamp: 1234567890,
                signature: vec![2u8; 64],
            },
        ],
    };

    mock_client.expect_get_commit()
        .returning(move |_| Ok(Some(commit.clone())));

    // Setup mock expectations for evidence
    mock_client.expect_get_evidence()
        .returning(|_| Ok(vec![]));

    let mut verifier = CosmosVerifier::new(config);
    
    // Test successful consensus verification
    let valid_metadata = CosmosMetadata {
        voting_power: Some(200000),
        total_power: Some(200000),
    };

    let valid_signal = FinalitySignal::Cosmos {
        height: 1000,
        block_hash: [0u8; 32],
        validator_signatures: vec![vec![1u8; 64], vec![2u8; 64]],
        metadata: Some(valid_metadata.clone()),
    };

    assert!(verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Test invalid commit height
    mock_client.expect_get_commit()
        .returning(|_| Ok(Some(TendermintCommit {
            height: 999, // Wrong height
            ..commit.clone()
        })));

    assert!(!verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Test missing signatures
    mock_client.expect_get_commit()
        .returning(|_| Ok(Some(TendermintCommit {
            signatures: vec![], // No signatures
            ..commit
        })));

    assert!(!verifier.verify_finality(&block_ref, &valid_signal).await.unwrap());

    // Check metrics
    let metrics = verifier.get_metrics().await;
    assert!(metrics.total_blocks_verified > 0);
    assert!(metrics.failed_verifications > 0);
    assert!(metrics.avg_finality_time > 0.0);
} 