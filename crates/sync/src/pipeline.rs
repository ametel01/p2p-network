use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use core_crate::P2PError;
use messages::L2BlockMessage;
use tracing::{debug, error, info, warn};

use crate::storage::BlockStorage;
use crate::types::PipelineState;

/// A trait for the block sync pipeline
///
/// The pipeline handles the processing and validation of blocks
/// as they move through the sync system.
#[async_trait]
pub trait BlockSyncPipeline: Debug + Send + Sync {
    /// Process a new block
    async fn process_block(&self, block: L2BlockMessage) -> Result<(), P2PError>;

    /// Get the current pipeline state
    async fn get_state(&self) -> Result<PipelineState, P2PError>;
}

/// A default block sync pipeline that does nothing
///
/// This is primarily used for testing or as a placeholder.
#[derive(Debug, Clone)]
pub struct DefaultBlockSyncPipeline;

#[async_trait]
impl BlockSyncPipeline for DefaultBlockSyncPipeline {
    async fn process_block(&self, block: L2BlockMessage) -> Result<(), P2PError> {
        info!(
            block_number = block.header.block_number,
            "DefaultBlockSyncPipeline.process_block() called - this is a no-op implementation"
        );
        Ok(())
    }

    async fn get_state(&self) -> Result<PipelineState, P2PError> {
        info!("DefaultBlockSyncPipeline.get_state() called - returning default state");
        Ok(PipelineState {
            current_block: 0,
            target_block: 0,
            is_syncing: false,
            blocks_processed: 0,
            blocks_remaining: 0,
        })
    }
}

/// A block processing pipeline implementation that validates and stores blocks
///
/// This pipeline:
/// 1. Validates block format and data integrity
/// 2. Checks if the block is part of the canonical chain
/// 3. Stores the block in persistent storage
/// 4. Updates state based on processed blocks
#[derive(Debug, Clone)]
pub struct StandardBlockPipeline {
    /// The storage backend for blocks
    storage: Arc<dyn BlockStorage>,

    /// The current processing state
    state: Arc<Mutex<PipelineState>>,
}

impl StandardBlockPipeline {
    /// Create a new StandardBlockPipeline
    pub fn new(storage: Arc<dyn BlockStorage>, initial_block: u64, target_block: u64) -> Self {
        info!(
            initial_block,
            target_block, "Creating new StandardBlockPipeline"
        );

        let state = PipelineState {
            current_block: initial_block,
            target_block,
            is_syncing: initial_block < target_block,
            blocks_processed: 0,
            blocks_remaining: target_block.saturating_sub(initial_block),
        };

        Self {
            storage,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Validate a block
    fn validate_block(&self, block: &L2BlockMessage) -> Result<(), P2PError> {
        debug!(block_number = block.header.block_number, "Validating block");

        // In a real implementation, we would:
        // 1. Verify the block format
        // 2. Verify the block's cryptographic integrity
        // 3. Verify the block's parent hash points to the previous block
        // 4. Verify the state transition is valid

        // Simplified validation for now
        if block.header.block_number == 0 {
            // Allow genesis block
            return Ok(());
        }

        // Check if the block number is sequential (simplified)
        let current_block = self.state.lock().unwrap().current_block;
        if block.header.block_number != current_block + 1 {
            warn!(
                current_block,
                received_block = block.header.block_number,
                "Received non-sequential block"
            );

            // In a real implementation, we might queue this for later
            // or request missing blocks, but for now just reject it
            return Err(P2PError::InvalidMessage(format!(
                "Non-sequential block: expected {}, got {}",
                current_block + 1,
                block.header.block_number
            )));
        }

        // All checks passed
        Ok(())
    }

    /// Update the pipeline state after processing a block
    fn update_state_after_block(&self, block: &L2BlockMessage) {
        let mut state = self.state.lock().unwrap();

        // Update current block if this is the next block
        if block.header.block_number == state.current_block + 1 {
            state.current_block = block.header.block_number;
            state.blocks_processed += 1;

            // Update remaining blocks
            if state.current_block < state.target_block {
                state.blocks_remaining = state.target_block - state.current_block;
            } else {
                state.blocks_remaining = 0;
                state.is_syncing = false;
            }

            debug!(
                new_current = state.current_block,
                target = state.target_block,
                remaining = state.blocks_remaining,
                "Updated pipeline state after processing block"
            );
        }
    }
}

#[async_trait]
impl BlockSyncPipeline for StandardBlockPipeline {
    async fn process_block(&self, block: L2BlockMessage) -> Result<(), P2PError> {
        let block_number = block.header.block_number;

        info!(block_number, "Processing block in StandardBlockPipeline");

        // Step 1: Validate the block
        if let Err(e) = self.validate_block(&block) {
            error!(
                block_number,
                error = %e,
                "Block validation failed"
            );
            return Err(e);
        }

        // Step 2: Store the block
        match self.storage.store_block(&block).await {
            Ok(()) => {
                debug!(block_number, "Block stored successfully");
            }
            Err(e) => {
                error!(
                    block_number,
                    error = %e,
                    "Failed to store block"
                );
                return Err(e);
            }
        }

        // Step 3: Update state
        self.update_state_after_block(&block);

        // All done
        info!(block_number, "Block processed successfully");
        Ok(())
    }

    async fn get_state(&self) -> Result<PipelineState, P2PError> {
        // Return a clone of the current state
        let state = self.state.lock().unwrap().clone();
        Ok(state)
    }
}
