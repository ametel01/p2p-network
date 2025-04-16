use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use core_crate::{P2PError, StateRoot};
use messages::{
    BlockRangeRequestMessage, BlockRangeResponseMessage, BlockResponseMessage, CitreaMessage,
    L2BlockMessage,
};
use network::P2PService;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::pipeline::BlockSyncPipeline;

/// Maximum number of blocks to request in a single range request
const MAX_BLOCKS_PER_REQUEST: u32 = 50;

/// Maximum number of parallel block range requests
const MAX_PARALLEL_REQUESTS: usize = 3;

/// Timeout for block requests (in seconds)
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

#[allow(dead_code)]
/// Retry interval for failed requests (in seconds)
const RETRY_INTERVAL_SECONDS: u64 = 5;

/// Information about a pending block request
#[derive(Debug)]
#[allow(dead_code)]
struct PendingRequest {
    /// The start block number
    start_block: u64,

    /// The end block number
    end_block: u64,

    /// The request ID
    request_id: u64,

    /// When the request was sent
    timestamp: Instant,

    /// How many times we've retried this request
    retry_count: u32,
}

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

    /// Manually trigger a synchronization cycle
    async fn trigger_sync(&self) -> Result<(), P2PError>;
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

    async fn trigger_sync(&self) -> Result<(), P2PError> {
        info!("Manually triggering synchronization");
        Ok(())
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
    p2p_service: Arc<Box<dyn P2PService>>,

    /// The block processing pipeline (shared with background tasks)
    pipeline: Arc<dyn BlockSyncPipeline>,

    /// The service running state
    running: Arc<Mutex<bool>>,

    /// The current local block number
    current_block: Arc<Mutex<u64>>,

    /// The target block number to sync to
    target_block: Arc<Mutex<u64>>,

    /// Pending block range requests
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,

    /// Next request ID
    next_request_id: Arc<Mutex<u64>>,

    /// Message receiver for internal communication
    msg_receiver: Arc<Mutex<Option<Receiver<InternalMessage>>>>,

    /// Message sender for internal communication
    msg_sender: Arc<Mutex<Option<Sender<InternalMessage>>>>,

    /// Background sync task handle
    sync_task: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// Background message processor task handle
    processor_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Internal message types for communication between tasks
#[derive(Debug)]
#[allow(dead_code)]
enum InternalMessage {
    /// A block has been received
    BlockReceived(L2BlockMessage),

    /// A block range response has been received
    BlockRangeReceived(BlockRangeResponseMessage),

    /// A request has timed out
    RequestTimeout(u64), // request_id

    /// Trigger synchronization
    TriggerSync,
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

        // Create channel for internal communication
        let (tx, rx) = mpsc::channel(100);

        Self {
            p2p_service: Arc::new(p2p_service),
            // Convert Box to Arc for shared ownership
            pipeline: Arc::from(pipeline),
            running: Arc::new(Mutex::new(false)),
            current_block: Arc::new(Mutex::new(start_block)),
            target_block: Arc::new(Mutex::new(target_block)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            next_request_id: Arc::new(Mutex::new(1)),
            msg_receiver: Arc::new(Mutex::new(Some(rx))),
            msg_sender: Arc::new(Mutex::new(Some(tx))),
            sync_task: Arc::new(Mutex::new(None)),
            processor_task: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the next request ID
    fn get_next_request_id(&self) -> u64 {
        let mut id = self.next_request_id.lock().unwrap();
        let current = *id;
        *id = current + 1;
        current
    }

    /// Request blocks from peers
    async fn request_blocks_from_peers(&self, start: u64, end: u64) -> Result<u64, P2PError> {
        // Get list of connected peers
        let peers = self.p2p_service.get_peers().await?;

        if peers.is_empty() {
            warn!("No peers connected, can't request blocks");
            return Err(P2PError::ProtocolError("No peers connected".to_string()));
        }

        // For simplicity, just pick the first peer
        // In a real implementation, we'd have logic to select the best peer
        // based on reputation, latency, etc.
        let peer_id = peers[0];

        info!(
            peer_id = ?peer_id,
            start_block = start,
            end_block = end,
            "Requesting blocks from peer"
        );

        // Create a request ID
        let request_id = self.get_next_request_id();

        // Calculate how many blocks to request (respect MAX_BLOCKS_PER_REQUEST)
        let block_count = end.saturating_sub(start) + 1;
        let actual_end = if block_count > MAX_BLOCKS_PER_REQUEST as u64 {
            start + MAX_BLOCKS_PER_REQUEST as u64 - 1
        } else {
            end
        };

        // Create the request message
        let request = BlockRangeRequestMessage::new(
            start,
            actual_end,
            MAX_BLOCKS_PER_REQUEST,
            true, // include transactions
            request_id,
        );

        // Store the request
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(
                request_id,
                PendingRequest {
                    start_block: start,
                    end_block: actual_end,
                    request_id,
                    timestamp: Instant::now(),
                    retry_count: 0,
                },
            );
        }

        // Send the request
        let message = CitreaMessage::BlockRangeRequest(request);
        self.p2p_service.send_message(peer_id, message).await?;

        debug!(
            request_id,
            start_block = start,
            end_block = actual_end,
            "Block range request sent"
        );

        Ok(request_id)
    }

    #[allow(dead_code)]
    /// Handle a block response message
    async fn handle_block_response(&self, response: BlockResponseMessage) -> Result<(), P2PError> {
        let block = response.block;
        let block_number = block.header.block_number;

        info!(
            block_number,
            request_id = ?response.request_id,
            "Handling block response"
        );

        // If this response is for a specific request, remove it from pending
        if let Some(request_id) = response.request_id {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.remove(&request_id);
        }

        // Process the block
        self.process_received_block(block).await
    }

    /// Handle a block range response message
    async fn handle_block_range_response(
        &self,
        response: BlockRangeResponseMessage,
    ) -> Result<(), P2PError> {
        let request_id = response.request_id;
        let block_count = response.blocks.len();

        info!(
            request_id,
            block_count,
            is_last = response.is_last,
            next_block = ?response.next_block,
            "Handling block range response"
        );

        // Remove the pending request
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.remove(&request_id);
        }

        // Process each block in the response
        for block in response.blocks {
            if let Err(e) = self.process_received_block(block).await {
                error!(error = ?e, "Failed to process block from range response");
                // Continue processing other blocks even if one fails
            }
        }

        // If there are more blocks to request, do so
        if !response.is_last && response.next_block.is_some() {
            let next_block = response.next_block.unwrap();
            let target = *self.target_block.lock().unwrap();

            // Request the next set of blocks
            if next_block <= target {
                let end_block = (next_block + MAX_BLOCKS_PER_REQUEST as u64 - 1).min(target);

                // Don't await this call to avoid blocking
                tokio::spawn({
                    let service = self.clone();
                    async move {
                        if let Err(e) = service
                            .request_blocks_from_peers(next_block, end_block)
                            .await
                        {
                            error!(
                                error = ?e,
                                next_block,
                                end_block,
                                "Failed to request next blocks in range"
                            );
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// Process a received block
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

    #[allow(dead_code)]
    /// Check for timed out requests
    fn check_request_timeouts(&self) -> Vec<u64> {
        let mut timed_out = Vec::new();
        let now = Instant::now();

        // Get the list of timed out requests
        {
            let pending = self.pending_requests.lock().unwrap();
            for (id, request) in pending.iter() {
                let elapsed = now.duration_since(request.timestamp);
                if elapsed.as_secs() > REQUEST_TIMEOUT_SECONDS {
                    timed_out.push(*id);
                }
            }
        }

        timed_out
    }

    /// Retry a timed out request
    async fn retry_request(&self, request_id: u64) -> Result<(), P2PError> {
        let mut request_to_retry = None;

        // Get the request info and remove it from pending
        {
            let mut pending = self.pending_requests.lock().unwrap();
            if let Some(request) = pending.remove(&request_id) {
                if request.retry_count < 3 {
                    request_to_retry = Some(request);
                } else {
                    warn!(
                        request_id,
                        retry_count = request.retry_count,
                        "Giving up on request after multiple retries"
                    );
                }
            }
        }

        // Retry the request if eligible
        if let Some(mut request) = request_to_retry {
            info!(
                request_id,
                start_block = request.start_block,
                end_block = request.end_block,
                retry_count = request.retry_count,
                "Retrying timed out block request"
            );

            // Increment retry count
            request.retry_count += 1;
            request.timestamp = Instant::now();

            // Get a peer to send to
            let peers = self.p2p_service.get_peers().await?;
            if peers.is_empty() {
                warn!("No peers connected for retry");
                return Err(P2PError::ProtocolError("No peers connected".to_string()));
            }

            // Pick a different peer if possible
            let peer_index = (request.retry_count as usize) % peers.len();
            let peer_id = peers[peer_index];

            // Create a new request message
            let request_msg = BlockRangeRequestMessage::new(
                request.start_block,
                request.end_block,
                MAX_BLOCKS_PER_REQUEST,
                true,       // include transactions
                request_id, // reuse the same ID
            );

            // Store the updated request
            {
                let mut pending = self.pending_requests.lock().unwrap();
                pending.insert(request_id, request);
            }

            // Send the request
            let message = CitreaMessage::BlockRangeRequest(request_msg);
            self.p2p_service.send_message(peer_id, message).await?;

            debug!(
                request_id,
                peer_id = ?peer_id,
                "Retry request sent"
            );
        }

        Ok(())
    }

    /// Start the message processor task
    fn start_message_processor(&self) -> JoinHandle<()> {
        // Take ownership of the receiver
        let mut rx = {
            let mut rx_guard = self.msg_receiver.lock().unwrap();
            rx_guard.take().expect("Message receiver already taken")
        };

        // Clone references for the task
        let _pipeline = self.pipeline.clone();
        let _p2p_service = self.p2p_service.clone();
        let current_block = self.current_block.clone();
        let target_block = self.target_block.clone();
        let pending_requests = self.pending_requests.clone();
        let running = self.running.clone();
        let service = self.clone();

        tokio::spawn(async move {
            info!("Message processor task started");

            while let Some(msg) = rx.recv().await {
                // Check if we should shut down
                {
                    let is_running = *running.lock().unwrap();
                    if !is_running {
                        info!("Message processor shutting down");
                        break;
                    }
                }

                // Handle the message
                match msg {
                    InternalMessage::BlockReceived(block) => {
                        let block_number = block.header.block_number;
                        debug!(block_number, "Received block in processor");

                        if let Err(e) = service.process_received_block(block).await {
                            error!(
                                block_number,
                                error = ?e,
                                "Failed to process received block"
                            );
                        }
                    }
                    InternalMessage::BlockRangeReceived(response) => {
                        debug!(
                            request_id = response.request_id,
                            block_count = response.blocks.len(),
                            "Received block range in processor"
                        );

                        if let Err(e) = service.handle_block_range_response(response).await {
                            error!(
                                error = ?e,
                                "Failed to handle block range response"
                            );
                        }
                    }
                    InternalMessage::RequestTimeout(request_id) => {
                        debug!(request_id, "Request timeout in processor");

                        if let Err(e) = service.retry_request(request_id).await {
                            error!(
                                request_id,
                                error = ?e,
                                "Failed to retry request"
                            );
                        }
                    }
                    InternalMessage::TriggerSync => {
                        debug!("Manual sync trigger in processor");

                        // Get the current and target blocks
                        let current = *current_block.lock().unwrap();
                        let target = *target_block.lock().unwrap();

                        // Check if we need to sync
                        if current < target {
                            let pending_count = pending_requests.lock().unwrap().len();

                            // Only start new requests if we have capacity
                            if pending_count < MAX_PARALLEL_REQUESTS {
                                let next_block = current + 1;
                                let end_block =
                                    (next_block + MAX_BLOCKS_PER_REQUEST as u64 - 1).min(target);

                                if let Err(e) = service
                                    .request_blocks_from_peers(next_block, end_block)
                                    .await
                                {
                                    error!(
                                        error = ?e,
                                        next_block,
                                        end_block,
                                        "Failed to request blocks in sync trigger"
                                    );
                                }
                            }
                        }
                    }
                }
            }

            info!("Message processor task terminated");
        })
    }

    /// Start the background sync task
    fn start_sync_task(&self) -> JoinHandle<()> {
        // Take a clone of the sender
        let tx = {
            let tx_guard = self.msg_sender.lock().unwrap();
            tx_guard
                .as_ref()
                .expect("Message sender not available")
                .clone()
        };

        // Clone references for the task
        let running = self.running.clone();
        let current_block = self.current_block.clone();
        let target_block = self.target_block.clone();
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            info!("Background sync task started");

            // Sync interval
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Check if we should shut down
                {
                    let is_running = *running.lock().unwrap();
                    if !is_running {
                        info!("Background sync task shutting down");
                        break;
                    }
                }

                // Check for timed out requests
                {
                    // Create a vector to store timed out request IDs
                    let mut timed_out_ids = Vec::new();

                    // Scope for MutexGuard to ensure it's dropped before the await
                    {
                        let pending = pending_requests.lock().unwrap();
                        let now = Instant::now();

                        for (id, request) in pending.iter() {
                            let elapsed = now.duration_since(request.timestamp);
                            if elapsed.as_secs() > REQUEST_TIMEOUT_SECONDS {
                                timed_out_ids.push(*id);
                            }
                        }
                    }

                    // Now send timeout messages without holding the MutexGuard
                    for id in timed_out_ids {
                        if let Err(e) = tx.send(InternalMessage::RequestTimeout(id)).await {
                            error!(
                                request_id = id,
                                error = ?e,
                                "Failed to send request timeout message"
                            );
                        }
                    }
                }

                // Check if we need to sync
                let current = *current_block.lock().unwrap();
                let target = *target_block.lock().unwrap();

                if current < target {
                    let pending_count = pending_requests.lock().unwrap().len();

                    // Only start new requests if we have capacity
                    if pending_count < MAX_PARALLEL_REQUESTS {
                        // Send trigger sync message
                        if let Err(e) = tx.send(InternalMessage::TriggerSync).await {
                            error!(
                                error = ?e,
                                "Failed to send trigger sync message"
                            );
                        }
                    }
                }
            }

            info!("Background sync task terminated");
        })
    }

    // Helper for cloning
    fn clone(&self) -> Self {
        // Get the sender for the new instance
        let tx = {
            let tx_guard = self.msg_sender.lock().unwrap();
            tx_guard
                .as_ref()
                .expect("Message sender not available")
                .clone()
        };

        // Create a new receiver
        let (_, rx) = mpsc::channel::<InternalMessage>(100);

        Self {
            p2p_service: self.p2p_service.clone(),
            pipeline: self.pipeline.clone(),
            running: self.running.clone(),
            current_block: self.current_block.clone(),
            target_block: self.target_block.clone(),
            pending_requests: self.pending_requests.clone(),
            next_request_id: self.next_request_id.clone(),
            msg_receiver: Arc::new(Mutex::new(Some(rx))),
            msg_sender: Arc::new(Mutex::new(Some(tx))),
            sync_task: Arc::new(Mutex::new(None)),
            processor_task: Arc::new(Mutex::new(None)),
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

        // Start the message processor task
        {
            let processor = self.start_message_processor();
            let mut processor_task = self.processor_task.lock().unwrap();
            *processor_task = Some(processor);
        }

        // Start the background sync task
        {
            let sync = self.start_sync_task();
            let mut sync_task = self.sync_task.lock().unwrap();
            *sync_task = Some(sync);
        }

        info!("NetworkedBlockSyncService started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("Stopping NetworkedBlockSyncService");

        // Clear running flag
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        // Stop the message processor task
        {
            let mut processor_task = self.processor_task.lock().unwrap();
            if let Some(handle) = processor_task.take() {
                handle.abort();
                debug!("Message processor task aborted");
            }
        }

        // Stop the background sync task
        {
            let mut sync_task = self.sync_task.lock().unwrap();
            if let Some(handle) = sync_task.take() {
                handle.abort();
                debug!("Background sync task aborted");
            }
        }

        info!("NetworkedBlockSyncService stopped successfully");
        Ok(())
    }

    async fn get_state_root(&self) -> Result<StateRoot, P2PError> {
        // Get the current pipeline state
        let _state = self.pipeline.get_state().await?;

        // Request the state root from the appropriate source
        // This is a simplified implementation
        Ok(StateRoot::new([0; 32]))
    }

    async fn get_block_number(&self) -> Result<u64, P2PError> {
        let current = *self.current_block.lock().unwrap();
        Ok(current)
    }

    async fn trigger_sync(&self) -> Result<(), P2PError> {
        info!("Manually triggering synchronization");

        // Send trigger sync message
        let tx = {
            let tx_guard = self.msg_sender.lock().unwrap();
            match tx_guard.as_ref() {
                Some(tx) => tx.clone(),
                None => {
                    return Err(P2PError::ProtocolError(
                        "Message sender not available".to_string(),
                    ))
                }
            }
        };

        tx.send(InternalMessage::TriggerSync).await.map_err(|_| {
            P2PError::ProtocolError("Failed to send trigger sync message".to_string())
        })?;

        Ok(())
    }
}
