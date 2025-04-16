use std::fmt::Debug;

use async_trait::async_trait;
use core_crate::{P2PError, StateRoot};
use messages::L2BlockMessage;
use network::P2PService;

/// A trait for the block sync service
#[async_trait]
pub trait BlockSyncService: Debug + Send + Sync {
    /// Start the block sync service
    async fn start(&self) -> Result<(), P2PError>;

    /// Stop the block sync service
    async fn stop(&self) -> Result<(), P2PError>;

    /// Get the current state root
    async fn get_state_root(&self) -> Result<StateRoot, P2PError>;

    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64, P2PError>;
}

/// A trait for the block sync pipeline
#[async_trait]
pub trait BlockSyncPipeline: Debug + Send + Sync {
    /// Process a new block
    async fn process_block(&self, block: L2BlockMessage) -> Result<(), P2PError>;

    /// Get the current pipeline state
    async fn get_state(&self) -> Result<PipelineState, P2PError>;
}

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

/// Configuration for the block sync service
#[derive(Debug)]
pub struct BlockSyncConfig {
    /// The p2p service to use
    pub p2p_service: Box<dyn P2PService>,

    /// The block sync pipeline to use
    pub pipeline: Box<dyn BlockSyncPipeline>,

    /// The starting block number
    pub start_block: u64,

    /// The target block number
    pub target_block: u64,
}

/// A builder for the block sync service
pub struct BlockSyncServiceBuilder {
    config: BlockSyncConfig,
}

impl BlockSyncServiceBuilder {
    /// Create a new builder
    pub fn new(p2p_service: Box<dyn P2PService>, pipeline: Box<dyn BlockSyncPipeline>) -> Self {
        Self {
            config: BlockSyncConfig {
                p2p_service,
                pipeline,
                start_block: 0,
                target_block: 0,
            },
        }
    }

    /// Set the starting block number
    pub fn with_start_block(mut self, start_block: u64) -> Self {
        self.config.start_block = start_block;
        self
    }

    /// Set the target block number
    pub fn with_target_block(mut self, target_block: u64) -> Self {
        self.config.target_block = target_block;
        self
    }

    /// Build the block sync service
    pub fn build(self) -> impl BlockSyncService {
        // TODO: Implement the actual service
        DefaultBlockSyncService
    }
}

/// A default block sync service that does nothing
#[derive(Debug, Clone)]
struct DefaultBlockSyncService;

#[async_trait]
impl BlockSyncService for DefaultBlockSyncService {
    async fn start(&self) -> Result<(), P2PError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        Ok(())
    }

    async fn get_state_root(&self) -> Result<StateRoot, P2PError> {
        Ok(StateRoot::new([0; 32]))
    }

    async fn get_block_number(&self) -> Result<u64, P2PError> {
        Ok(0)
    }
}

/// A default block sync pipeline that does nothing
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DefaultBlockSyncPipeline;

#[async_trait]
impl BlockSyncPipeline for DefaultBlockSyncPipeline {
    async fn process_block(&self, _block: L2BlockMessage) -> Result<(), P2PError> {
        Ok(())
    }

    async fn get_state(&self) -> Result<PipelineState, P2PError> {
        Ok(PipelineState {
            current_block: 0,
            target_block: 0,
            is_syncing: false,
            blocks_processed: 0,
            blocks_remaining: 0,
        })
    }
}
