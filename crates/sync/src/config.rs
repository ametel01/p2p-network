use std::fmt::Debug;
use std::sync::Arc;

use messages::L2BlockMessage;
use network::P2PService;
use tracing::{debug, info};

use crate::pipeline::{BlockSyncPipeline, DefaultBlockSyncPipeline, StandardBlockPipeline};
use crate::service::{BlockSyncService, NetworkedBlockSyncService};
use crate::storage::{BlockStorage, BlockStorageFactory};

/// Configuration for the block sync service
///
/// This struct contains all the configuration parameters needed to
/// initialize a block sync service.
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
///
/// This follows the builder pattern to allow flexible configuration
/// with reasonable defaults.
pub struct BlockSyncServiceBuilder {
    /// The configuration being built
    config: BlockSyncConfig,

    /// Whether to use in-memory storage
    use_memory_storage: bool,

    /// Optional pre-populated blocks for memory storage
    memory_blocks: Option<Vec<L2BlockMessage>>,

    /// Whether this is using the default pipeline
    using_default_pipeline: bool,
}

impl BlockSyncServiceBuilder {
    /// Create a new builder with default values
    pub fn new(p2p_service: Box<dyn P2PService>) -> Self {
        info!("Creating new BlockSyncServiceBuilder with default configuration");

        Self {
            config: BlockSyncConfig {
                p2p_service,
                pipeline: Box::new(DefaultBlockSyncPipeline),
                start_block: 0,
                target_block: 0,
            },
            use_memory_storage: true,
            memory_blocks: None,
            using_default_pipeline: true,
        }
    }

    /// Set a custom pipeline
    pub fn with_pipeline(mut self, pipeline: Box<dyn BlockSyncPipeline>) -> Self {
        debug!("Setting custom pipeline");
        self.config.pipeline = pipeline;
        self.using_default_pipeline = false;
        self
    }

    /// Set the starting block number
    pub fn with_start_block(mut self, start_block: u64) -> Self {
        debug!(start_block, "Setting start block");
        self.config.start_block = start_block;
        self
    }

    /// Set the target block number
    pub fn with_target_block(mut self, target_block: u64) -> Self {
        debug!(target_block, "Setting target block");
        self.config.target_block = target_block;
        self
    }

    /// Use in-memory storage
    pub fn with_memory_storage(mut self) -> Self {
        debug!("Using in-memory block storage");
        self.use_memory_storage = true;
        self
    }

    /// Use in-memory storage with pre-populated blocks
    pub fn with_memory_blocks(mut self, blocks: Vec<L2BlockMessage>) -> Self {
        debug!(
            block_count = blocks.len(),
            "Using in-memory storage with pre-populated blocks"
        );
        self.use_memory_storage = true;
        self.memory_blocks = Some(blocks);
        self
    }

    /// Create a standard pipeline with the configured options
    fn create_standard_pipeline(&self) -> Box<dyn BlockSyncPipeline> {
        // Create storage based on configuration
        let storage: Arc<dyn BlockStorage> = if self.use_memory_storage {
            if let Some(blocks) = &self.memory_blocks {
                BlockStorageFactory::create_memory_storage_with_blocks(blocks.clone())
            } else {
                BlockStorageFactory::create_memory_storage()
            }
        } else {
            // Default to memory storage for now
            BlockStorageFactory::create_memory_storage()
        };

        // Create the pipeline
        Box::new(StandardBlockPipeline::new(
            storage,
            self.config.start_block,
            self.config.target_block,
        ))
    }

    /// Build the block sync service
    pub fn build(self) -> impl BlockSyncService {
        info!("Building BlockSyncService");

        // Create the standard pipeline if default is being used
        let pipeline = if self.using_default_pipeline {
            info!("Creating standard pipeline to replace default");
            self.create_standard_pipeline()
        } else {
            self.config.pipeline
        };

        // Create the actual service
        NetworkedBlockSyncService::new(
            self.config.p2p_service,
            pipeline,
            self.config.start_block,
            self.config.target_block,
        )
    }

    /// Build and return as boxed trait object
    pub fn build_boxed(self) -> Box<dyn BlockSyncService> {
        Box::new(self.build())
    }

    /// Build and return as Arc for shared ownership
    pub fn build_shared(self) -> Arc<dyn BlockSyncService> {
        Arc::new(self.build())
    }
}
