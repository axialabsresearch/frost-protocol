use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::network::{Peer, NodeIdentity, P2PEvent};
use crate::Result;

/// Reputation score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    /// Base score (0-100)
    pub base_score: f64,
    /// Participation score (0-100)
    pub participation_score: f64,
    /// Performance score (0-100)
    pub performance_score: f64,
    /// Reliability score (0-100)
    pub reliability_score: f64,
    /// Stake weight (0-1)
    pub stake_weight: f64,
    /// Time weight (0-1)
    pub time_weight: f64,
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self {
            base_score: 50.0,
            participation_score: 50.0,
            performance_score: 50.0,
            reliability_score: 50.0,
            stake_weight: 0.0,
            time_weight: 0.0,
        }
    }
}

impl ReputationScore {
    /// Calculate weighted total score
    pub fn total_score(&self) -> f64 {
        let base_weight = 0.2;
        let participation_weight = 0.3;
        let performance_weight = 0.3;
        let reliability_weight = 0.2;

        let raw_score = 
            self.base_score * base_weight +
            self.participation_score * participation_weight +
            self.performance_score * performance_weight +
            self.reliability_score * reliability_weight;

        // Apply stake and time weights
        raw_score * (1.0 + self.stake_weight) * (1.0 + self.time_weight)
    }
}

/// Reputation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationMetrics {
    /// Total messages processed
    pub messages_processed: u64,
    /// Successful validations
    pub successful_validations: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Uptime percentage
    pub uptime_percentage: f64,
    /// Resource contribution
    pub resource_contribution: ResourceMetrics,
    /// Protocol violations
    pub protocol_violations: u32,
    /// Last update time
    pub last_update: SystemTime,
}

/// Resource contribution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU contribution (0-100)
    pub cpu_score: f64,
    /// Memory contribution (0-100)
    pub memory_score: f64,
    /// Bandwidth contribution (0-100)
    pub bandwidth_score: f64,
    /// Storage contribution (0-100)
    pub storage_score: f64,
}

/// Incentive rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewards {
    /// Base rewards
    pub base_rewards: u64,
    /// Performance multiplier
    pub performance_multiplier: f64,
    /// Stake multiplier
    pub stake_multiplier: f64,
    /// Time multiplier
    pub time_multiplier: f64,
    /// Bonus rewards
    pub bonus_rewards: u64,
}

impl Default for Rewards {
    fn default() -> Self {
        Self {
            base_rewards: 100,
            performance_multiplier: 1.0,
            stake_multiplier: 1.0,
            time_multiplier: 1.0,
            bonus_rewards: 0,
        }
    }
}

/// Reputation manager
pub struct ReputationManager {
    /// Node identity
    identity: NodeIdentity,
    /// Peer reputations
    reputations: RwLock<HashMap<Uuid, ReputationScore>>,
    /// Peer metrics
    metrics: RwLock<HashMap<Uuid, ReputationMetrics>>,
    /// Reward distribution
    rewards: RwLock<HashMap<Uuid, Rewards>>,
    /// Configuration
    config: ReputationConfig,
}

/// Reputation system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// Minimum score threshold
    pub min_score_threshold: f64,
    /// Score decay rate
    pub score_decay_rate: f64,
    /// Update interval
    pub update_interval: Duration,
    /// Reward distribution interval
    pub reward_interval: Duration,
    /// Maximum rewards per interval
    pub max_rewards_per_interval: u64,
    /// Minimum stake requirement
    pub min_stake_requirement: u64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            min_score_threshold: 25.0,
            score_decay_rate: 0.01,
            update_interval: Duration::from_secs(300),
            reward_interval: Duration::from_secs(3600),
            max_rewards_per_interval: 1000,
            min_stake_requirement: 100,
        }
    }
}

impl ReputationManager {
    /// Create new reputation manager
    pub fn new(identity: NodeIdentity, config: ReputationConfig) -> Self {
        Self {
            identity,
            reputations: RwLock::new(HashMap::new()),
            metrics: RwLock::new(HashMap::new()),
            rewards: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Start reputation management
    pub async fn start(&self) -> Result<()> {
        tokio::spawn(self.update_loop());
        tokio::spawn(self.reward_loop());
        Ok(())
    }

    /// Main update loop
    async fn update_loop(&self) {
        loop {
            tokio::time::sleep(self.config.update_interval).await;
            if let Err(e) = self.update_all_scores().await {
                eprintln!("Error updating reputation scores: {}", e);
            }
        }
    }

    /// Reward distribution loop
    async fn reward_loop(&self) {
        loop {
            tokio::time::sleep(self.config.reward_interval).await;
            if let Err(e) = self.distribute_rewards().await {
                eprintln!("Error distributing rewards: {}", e);
            }
        }
    }

    /// Update reputation for a peer
    pub async fn update_reputation(
        &self,
        peer_id: Uuid,
        event: ReputationEvent,
    ) -> Result<()> {
        let mut reputations = self.reputations.write().await;
        let mut metrics = self.metrics.write().await;

        let score = reputations.entry(peer_id).or_default();
        let metric = metrics.entry(peer_id).or_default();

        match event {
            ReputationEvent::SuccessfulValidation { response_time } => {
                score.performance_score += 1.0;
                metric.successful_validations += 1;
                metric.avg_response_time = update_average_duration(
                    metric.avg_response_time,
                    response_time,
                    metric.messages_processed
                );
            }
            ReputationEvent::FailedValidation => {
                score.performance_score -= 2.0;
                metric.failed_validations += 1;
            }
            ReputationEvent::ResourceContribution(resources) => {
                score.participation_score += 1.0;
                metric.resource_contribution = resources;
            }
            ReputationEvent::ProtocolViolation => {
                score.reliability_score -= 5.0;
                metric.protocol_violations += 1;
            }
            ReputationEvent::StakeUpdate(stake) => {
                score.stake_weight = calculate_stake_weight(stake);
            }
        }

        // Update metrics
        metric.messages_processed += 1;
        metric.last_update = SystemTime::now();

        // Normalize scores
        normalize_score(&mut score.performance_score);
        normalize_score(&mut score.participation_score);
        normalize_score(&mut score.reliability_score);

        Ok(())
    }

    /// Update all peer scores
    async fn update_all_scores(&self) -> Result<()> {
        let mut reputations = self.reputations.write().await;
        let metrics = self.metrics.read().await;

        for (peer_id, score) in reputations.iter_mut() {
            // Apply score decay
            score.base_score *= 1.0 - self.config.score_decay_rate;
            score.participation_score *= 1.0 - self.config.score_decay_rate;
            score.performance_score *= 1.0 - self.config.score_decay_rate;
            score.reliability_score *= 1.0 - self.config.score_decay_rate;

            // Update time weight
            if let Some(metric) = metrics.get(peer_id) {
                score.time_weight = calculate_time_weight(metric.last_update);
            }

            // Check minimum threshold
            if score.total_score() < self.config.min_score_threshold {
                // Emit low score warning event
            }
        }

        Ok(())
    }

    /// Distribute rewards
    async fn distribute_rewards(&self) -> Result<()> {
        let reputations = self.reputations.read().await;
        let mut rewards = self.rewards.write().await;

        let total_score: f64 = reputations.values()
            .map(|score| score.total_score())
            .sum();

        if total_score <= 0.0 {
            return Ok(());
        }

        let reward_pool = self.config.max_rewards_per_interval;

        for (peer_id, score) in reputations.iter() {
            let peer_score = score.total_score();
            let reward_share = (peer_score / total_score) * reward_pool as f64;

            let reward = rewards.entry(*peer_id).or_default();
            reward.base_rewards += reward_share as u64;
            reward.performance_multiplier = calculate_performance_multiplier(score);
            reward.stake_multiplier = 1.0 + score.stake_weight;
            reward.time_multiplier = 1.0 + score.time_weight;

            // Calculate bonus rewards
            let bonus = calculate_bonus_rewards(score, &self.config);
            reward.bonus_rewards += bonus;
        }

        Ok(())
    }

    /// Get peer reputation
    pub async fn get_reputation(&self, peer_id: &Uuid) -> Option<ReputationScore> {
        self.reputations.read().await.get(peer_id).cloned()
    }

    /// Get peer metrics
    pub async fn get_metrics(&self, peer_id: &Uuid) -> Option<ReputationMetrics> {
        self.metrics.read().await.get(peer_id).cloned()
    }

    /// Get peer rewards
    pub async fn get_rewards(&self, peer_id: &Uuid) -> Option<Rewards> {
        self.rewards.read().await.get(peer_id).cloned()
    }
}

/// Reputation events
#[derive(Debug, Clone)]
pub enum ReputationEvent {
    /// Successful validation
    SuccessfulValidation {
        response_time: Duration,
    },
    /// Failed validation
    FailedValidation,
    /// Resource contribution update
    ResourceContribution(ResourceMetrics),
    /// Protocol violation
    ProtocolViolation,
    /// Stake update
    StakeUpdate(u64),
}

// Helper functions

fn normalize_score(score: &mut f64) {
    *score = score.max(0.0).min(100.0);
}

fn update_average_duration(
    current: Duration,
    new: Duration,
    count: u64
) -> Duration {
    if count == 0 {
        new
    } else {
        let weight = 1.0 / (count + 1) as f64;
        Duration::from_secs_f64(
            current.as_secs_f64() * (1.0 - weight) +
            new.as_secs_f64() * weight
        )
    }
}

fn calculate_stake_weight(stake: u64) -> f64 {
    // Logarithmic stake weight calculation
    (stake as f64).log10() / 6.0
}

fn calculate_time_weight(last_update: SystemTime) -> f64 {
    match SystemTime::now().duration_since(last_update) {
        Ok(duration) => {
            let days = duration.as_secs() as f64 / 86400.0;
            (days / 30.0).min(1.0)
        }
        Err(_) => 0.0,
    }
}

fn calculate_performance_multiplier(score: &ReputationScore) -> f64 {
    let base_multiplier = score.performance_score / 50.0;
    1.0 + base_multiplier.max(0.0).min(1.0)
}

fn calculate_bonus_rewards(
    score: &ReputationScore,
    config: &ReputationConfig,
) -> u64 {
    let performance_bonus = if score.performance_score > 90.0 { 100 } else { 0 };
    let reliability_bonus = if score.reliability_score > 95.0 { 100 } else { 0 };
    let participation_bonus = if score.participation_score > 85.0 { 100 } else { 0 };

    performance_bonus + reliability_bonus + participation_bonus
} 