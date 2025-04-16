use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use core_crate::{P2PError, StateRoot};
use messages::L2BlockMessage;
use tracing::{debug, info, warn};

/// A trait for block storage operations
///
/// This trait defines operations for storing and retrieving blocks
/// from a persistent storage layer.
#[async_trait]
pub trait BlockStorage: Debug + Send + Sync {
    /// Store a block
    async fn store_block(&self, block: &L2BlockMessage) -> Result<(), P2PError>;

    /// Retrieve a block by number
    async fn get_block(&self, block_number: u64) -> Result<Option<L2BlockMessage>, P2PError>;

    /// Get the latest block number
    async fn get_latest_block_number(&self) -> Result<u64, P2PError>;

    /// Get the state root for a specific block
    async fn get_state_root(&self, block_number: u64) -> Result<Option<StateRoot>, P2PError>;

    /// Get the latest state root
    async fn get_latest_state_root(&self) -> Result<StateRoot, P2PError>;
}

/// In-memory block storage implementation
///
/// This is a simple implementation that stores blocks in memory.
/// Useful for testing and development, but not suitable for production
/// as it doesn't persist data across restarts.
#[derive(Debug)]
pub struct InMemoryBlockStorage {
    /// Blocks stored by block number
    blocks: Arc<Mutex<HashMap<u64, L2BlockMessage>>>,

    /// The latest block number
    latest_block: Arc<Mutex<u64>>,
}

impl InMemoryBlockStorage {
    /// Create a new in-memory block storage
    pub fn new() -> Self {
        info!("Creating new InMemoryBlockStorage");
        Self {
            blocks: Arc::new(Mutex::new(HashMap::new())),
            latest_block: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new in-memory storage with pre-populated blocks
    pub fn with_blocks(blocks: Vec<L2BlockMessage>) -> Self {
        info!(
            block_count = blocks.len(),
            "Creating InMemoryBlockStorage with pre-populated blocks"
        );

        let mut blocks_map = HashMap::new();
        let mut latest_block = 0;

        for block in blocks {
            if block.header.block_number > latest_block {
                latest_block = block.header.block_number;
            }
            blocks_map.insert(block.header.block_number, block);
        }

        Self {
            blocks: Arc::new(Mutex::new(blocks_map)),
            latest_block: Arc::new(Mutex::new(latest_block)),
        }
    }
}

impl Default for InMemoryBlockStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockStorage for InMemoryBlockStorage {
    async fn store_block(&self, block: &L2BlockMessage) -> Result<(), P2PError> {
        let block_number = block.header.block_number;

        debug!(block_number, "Storing block in memory");

        // Lock the blocks map and insert the block
        let mut blocks = self.blocks.lock().unwrap();
        blocks.insert(block_number, block.clone());

        // Update latest block if necessary
        let mut latest = self.latest_block.lock().unwrap();
        if block_number > *latest {
            *latest = block_number;
            debug!(new_latest = *latest, "Updated latest block number");
        }

        Ok(())
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<L2BlockMessage>, P2PError> {
        debug!(block_number, "Retrieving block from memory");

        // Get the block from the map
        let blocks = self.blocks.lock().unwrap();
        let block = blocks.get(&block_number).cloned();

        Ok(block)
    }

    async fn get_latest_block_number(&self) -> Result<u64, P2PError> {
        let latest = *self.latest_block.lock().unwrap();
        debug!(latest_block = latest, "Retrieved latest block number");
        Ok(latest)
    }

    async fn get_state_root(&self, block_number: u64) -> Result<Option<StateRoot>, P2PError> {
        debug!(block_number, "Retrieving state root for block");

        // Get the block
        let blocks = self.blocks.lock().unwrap();
        let state_root = blocks
            .get(&block_number)
            .map(|block| block.header.state_root);

        Ok(state_root)
    }

    async fn get_latest_state_root(&self) -> Result<StateRoot, P2PError> {
        let latest = *self.latest_block.lock().unwrap();

        debug!(
            latest_block = latest,
            "Retrieving state root for latest block"
        );

        // Get the latest block's state root
        let blocks = self.blocks.lock().unwrap();
        match blocks.get(&latest) {
            Some(block) => Ok(block.header.state_root),
            None => {
                warn!("No blocks in storage, returning default state root");
                Ok(StateRoot::new([0; 32]))
            }
        }
    }
}

/// Factory for creating block storage instances
#[derive(Debug)]
pub struct BlockStorageFactory;

impl BlockStorageFactory {
    /// Create a new block storage instance
    pub fn create_memory_storage() -> Arc<dyn BlockStorage> {
        Arc::new(InMemoryBlockStorage::new())
    }

    /// Create a new block storage with pre-populated blocks
    pub fn create_memory_storage_with_blocks(blocks: Vec<L2BlockMessage>) -> Arc<dyn BlockStorage> {
        Arc::new(InMemoryBlockStorage::with_blocks(blocks))
    }

    // In a real implementation, we would have methods to create
    // disk-based storage, database storage, etc.
}
