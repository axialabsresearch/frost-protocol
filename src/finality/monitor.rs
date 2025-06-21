use async_trait::async_trait;
use std::time::Duration;
use crate::finality::{FinalitySignal, FinalityError};
use crate::state::BlockRef;

/// Monitor for chain finality
#[async_trait]
pub trait FinalityMonitor: Send + Sync {
    /// Wait for finality of a specific block
    async fn wait_for_finality(
        &self,
        block_ref: BlockRef,
        timeout: Option<Duration>,
    ) -> Result<FinalitySignal, FinalityError>;

    /// Verify a finality signal
    async fn verify_finality(
        &self,
        signal: &FinalitySignal,
    ) -> Result<bool, FinalityError>;

    /// Get latest finalized block
    async fn latest_finalized_block(&self) -> Result<BlockRef, FinalityError>;
}
