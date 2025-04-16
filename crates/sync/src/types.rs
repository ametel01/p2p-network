use std::fmt::Debug;

/// The state of the block sync pipeline
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// The current block number
    pub current_block: u64,

    /// The target block number
    pub target_block: u64,

    /// Whether the pipeline is syncing
    pub is_syncing: bool,

    /// The number of blocks processed
    pub blocks_processed: u64,

    /// The number of blocks remaining
    pub blocks_remaining: u64,
}
