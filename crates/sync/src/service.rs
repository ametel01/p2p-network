use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use core_crate::{P2PError, StateRoot};
use messages::L2BlockMessage;
use network::P2PService;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::pipeline::BlockSyncPipeline;

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

/// A default block sync service that does nothing
///
/// This is primarily used for testing or as a placeholder.
#[derive(Debug, Clone)]
pub struct DefaultBlockSyncService;

#[async_trait]
impl BlockSyncService for DefaultBlockSyncService {
    async fn start(&self) -> Result<(), P2PError> {
        info!("DefaultBlockSyncService.start() called - this is a no-op implementation");
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("DefaultBlockSyncService.stop() called - this is a no-op implementation");
        Ok(())
    }

    async fn get_state_root(&self) -> Result<StateRoot, P2PError> {
        info!("DefaultBlockSyncService.get_state_root() called - returning default value");
        Ok(StateRoot::new([0; 32]))
    }

    async fn get_block_number(&self) -> Result<u64, P2PError> {
        info!("DefaultBlockSyncService.get_block_number() called - returning 0");
        Ok(0)
    }
}

/// An implementation of BlockSyncService that uses the p2p network
///
/// This service synchronizes blocks from the network by:
/// 1. Requesting block data from connected peers
/// 2. Processing blocks through the pipeline
/// 3. Updating local state based on received blocks
#[derive(Debug)]
pub struct NetworkedBlockSyncService {
    /// The p2p service used for network communication
    #[allow(dead_code)]
    p2p_service: Box<dyn P2PService>,

    /// The block processing pipeline (shared with background tasks)
    pipeline: Arc<dyn BlockSyncPipeline>,

    /// The service running state
    running: Arc<Mutex<bool>>,

    /// The current local block number
    current_block: Arc<Mutex<u64>>,

    /// The target block number to sync to
    target_block: Arc<Mutex<u64>>,
}

impl NetworkedBlockSyncService {
    /// Create a new NetworkedBlockSyncService
    pub fn new(
        p2p_service: Box<dyn P2PService>,
        pipeline: Box<dyn BlockSyncPipeline>,
        start_block: u64,
        target_block: u64,
    ) -> Self {
        info!(
            start_block,
            target_block, "Creating new NetworkedBlockSyncService"
        );

        Self {
            p2p_service,
            // Convert Box to Arc for shared ownership
            pipeline: Arc::from(pipeline),
            running: Arc::new(Mutex::new(false)),
            current_block: Arc::new(Mutex::new(start_block)),
            target_block: Arc::new(Mutex::new(target_block)),
        }
    }

    /// Request blocks from peers
    #[allow(dead_code)]
    async fn request_blocks_from_peers(&self, start: u64, end: u64) -> Result<(), P2PError> {
        // Get list of connected peers
        let peers = self.p2p_service.get_peers().await?;

        if peers.is_empty() {
            warn!("No peers connected, can't request blocks");
            return Ok(());
        }

        info!(
            peer_count = peers.len(),
            start_block = start,
            end_block = end,
            "Requesting blocks from peers"
        );

        // For now, we simply broadcast a request for blocks
        // In a real implementation, we'd split the request across peers
        // and implement a more sophisticated protocol for block requests

        // This is a placeholder method - in a real implementation,
        // we would create a proper block request message type
        self.broadcast_block_request(start, end).await
    }

    /// Broadcast a request for blocks
    #[allow(dead_code)]
    async fn broadcast_block_request(&self, start: u64, end: u64) -> Result<(), P2PError> {
        // This is a simplified implementation
        // In a real system, we would:
        // 1. Create a proper block request message type
        // 2. Handle responses from peers
        // 3. Implement retry logic for failed requests

        debug!(
            start_block = start,
            end_block = end,
            "Broadcasting block request"
        );

        // For now, just log that we would send a request
        // This would be replaced with actual message sending

        warn!("Block request broadcasting not fully implemented");

        Ok(())
    }

    /// Process blocks received from the network
    #[allow(dead_code)]
    async fn process_received_block(&self, block: L2BlockMessage) -> Result<(), P2PError> {
        let block_number = block.header.block_number;

        info!(block_number, "Processing received block");

        // Pass the block to the pipeline for processing
        match self.pipeline.process_block(block).await {
            Ok(()) => {
                debug!(block_number, "Block processed successfully");

                // Update our current block if this is the next block in sequence
                let mut current = self.current_block.lock().unwrap();
                if block_number == *current + 1 {
                    *current = block_number;
                    debug!(new_current = *current, "Updated current block number");
                }

                Ok(())
            }
            Err(e) => {
                error!(
                    block_number,
                    error = %e,
                    "Failed to process block"
                );

                Err(e)
            }
        }
    }
}

#[async_trait]
impl BlockSyncService for NetworkedBlockSyncService {
    async fn start(&self) -> Result<(), P2PError> {
        info!("Starting NetworkedBlockSyncService");

        // Set running flag
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                warn!("NetworkedBlockSyncService already running");
                return Ok(());
            }
            *running = true;
        }

        // Start a background task to sync blocks
        let running_clone = self.running.clone();
        let current_block_clone = self.current_block.clone();
        let target_block_clone = self.target_block.clone();
        // Now we can safely clone the Arc
        let pipeline_clone = self.pipeline.clone();

        // Spawn a background task that will continue running
        tokio::spawn(async move {
            info!("Block sync background task started");

            // Simple sync loop - in a real implementation, this would be more sophisticated
            loop {
                // Check if we should stop
                if !*running_clone.lock().unwrap() {
                    info!("Stopping block sync background task");
                    break;
                }

                let current = *current_block_clone.lock().unwrap();
                let target = *target_block_clone.lock().unwrap();

                // Check if we're caught up
                if current >= target {
                    debug!(
                        current_block = current,
                        target_block = target,
                        "Already at target block, waiting for new blocks"
                    );

                    // Sleep for a while before checking again
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }

                // Get current pipeline state
                match pipeline_clone.get_state().await {
                    Ok(state) => {
                        info!(
                            current_block = state.current_block,
                            target_block = state.target_block,
                            is_syncing = state.is_syncing,
                            blocks_processed = state.blocks_processed,
                            blocks_remaining = state.blocks_remaining,
                            "Current pipeline state"
                        );
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to get pipeline state");
                    }
                }

                // Sleep before next iteration
                sleep(Duration::from_secs(10)).await;
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("Stopping NetworkedBlockSyncService");

        // Clear running flag to stop background task
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(())
    }

    async fn get_state_root(&self) -> Result<StateRoot, P2PError> {
        // For now, just return a placeholder state root
        // In a real implementation, this would come from our storage layer
        warn!("get_state_root not fully implemented, returning placeholder");
        Ok(StateRoot::new([0; 32]))
    }

    async fn get_block_number(&self) -> Result<u64, P2PError> {
        // Return the current block number
        let current = *self.current_block.lock().unwrap();
        Ok(current)
    }
}
