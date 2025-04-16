use core_crate::{MerkleRoot, P2PError, StateRoot, VerifiableMessage};
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing::error;

/// Message containing L2 block data.
///
/// L2 blocks contain the transactions and state updates for the Citrea Layer 2
/// network. This message type is used to propagate new blocks through the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BlockMessage {
    /// The block header containing metadata
    pub header: L2BlockHeader,

    /// Merkle root of all transactions in the block
    pub transactions_merkle_root: MerkleRoot,

    /// Optional block body (may be omitted for lightweight announcements)
    pub transactions: Option<Vec<Vec<u8>>>,

    /// Optional block receipts root
    pub receipts_root: Option<MerkleRoot>,
}

impl L2BlockMessage {
    /// Create a new L2 block message
    pub fn new(header: L2BlockHeader, transactions_merkle_root: MerkleRoot) -> Self {
        debug!(
            block_number = header.block_number,
            timestamp = header.timestamp,
            "Creating new L2BlockMessage"
        );

        Self {
            header,
            transactions_merkle_root,
            transactions: None,
            receipts_root: None,
        }
    }

    /// Create a new L2 block message with full transaction data
    pub fn new_with_transactions(
        header: L2BlockHeader,
        transactions_merkle_root: MerkleRoot,
        transactions: Vec<Vec<u8>>,
        receipts_root: Option<MerkleRoot>,
    ) -> Self {
        debug!(
            block_number = header.block_number,
            timestamp = header.timestamp,
            tx_count = transactions.len(),
            "Creating new L2BlockMessage with transactions"
        );

        Self {
            header,
            transactions_merkle_root,
            transactions: Some(transactions),
            receipts_root,
        }
    }

    /// Check if this block is a full block (with transactions)
    pub fn is_full_block(&self) -> bool {
        self.transactions.is_some()
    }

    /// Get the block number of this block
    pub fn block_number(&self) -> u64 {
        self.header.block_number
    }

    /// Get the parent hash of this block
    pub fn parent_hash(&self) -> &[u8; 32] {
        &self.header.parent_hash
    }
}

/// Header information for an L2 block.
///
/// The block header contains essential metadata about a block,
/// similar to headers in other blockchain systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BlockHeader {
    /// Sequential block number
    pub block_number: u64,

    /// Unix timestamp when the block was created
    pub timestamp: u64,

    /// Hash of the parent block (creates the chain)
    pub parent_hash: [u8; 32],

    /// Root hash of the state tree after applying this block
    pub state_root: StateRoot,
}

impl L2BlockHeader {
    /// Create a new L2 block header
    pub fn new(
        block_number: u64,
        timestamp: u64,
        parent_hash: [u8; 32],
        state_root: StateRoot,
    ) -> Self {
        debug!(block_number, timestamp, "Creating new L2BlockHeader");

        Self {
            block_number,
            timestamp,
            parent_hash,
            state_root,
        }
    }
}

/// Block announcement message for lightweight propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAnnouncementMessage {
    /// The block number being announced
    pub block_number: u64,

    /// The block's parent hash
    pub parent_hash: [u8; 32],

    /// The block's hash
    pub block_hash: [u8; 32],

    /// Whether the sender has the full block data available
    pub has_block: bool,
}

impl BlockAnnouncementMessage {
    /// Create a new block announcement
    pub fn new(
        block_number: u64,
        parent_hash: [u8; 32],
        block_hash: [u8; 32],
        has_block: bool,
    ) -> Self {
        debug!(
            block_number,
            has_full_block = has_block,
            "Creating new BlockAnnouncementMessage"
        );

        Self {
            block_number,
            parent_hash,
            block_hash,
            has_block,
        }
    }

    /// Create an announcement from a block message
    pub fn from_block(block: &L2BlockMessage, block_hash: [u8; 32]) -> Self {
        Self {
            block_number: block.header.block_number,
            parent_hash: block.header.parent_hash,
            block_hash,
            has_block: block.is_full_block(),
        }
    }
}

/// Block request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequestMessage {
    /// The hash of the requested block
    pub block_hash: [u8; 32],

    /// Whether to include full transaction data
    pub include_transactions: bool,
}

impl BlockRequestMessage {
    /// Create a new block request
    pub fn new(block_hash: [u8; 32], include_transactions: bool) -> Self {
        debug!(include_transactions, "Creating new BlockRequestMessage");

        Self {
            block_hash,
            include_transactions,
        }
    }
}

/// Block range request message for requesting multiple sequential blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRangeRequestMessage {
    /// The start block number of the range (inclusive)
    pub start_block: u64,

    /// The end block number of the range (inclusive)
    pub end_block: u64,

    /// Maximum number of blocks to return in a single response
    pub max_blocks: u32,

    /// Whether to include full transaction data
    pub include_transactions: bool,

    /// A unique request ID to match responses to requests
    pub request_id: u64,
}

impl BlockRangeRequestMessage {
    /// Create a new block range request
    pub fn new(
        start_block: u64,
        end_block: u64,
        max_blocks: u32,
        include_transactions: bool,
        request_id: u64,
    ) -> Self {
        debug!(
            start_block,
            end_block,
            max_blocks,
            include_transactions,
            request_id,
            "Creating new BlockRangeRequestMessage"
        );

        Self {
            start_block,
            end_block,
            max_blocks,
            include_transactions,
            request_id,
        }
    }
}

/// Block response message to fulfill a block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponseMessage {
    /// The block being returned
    pub block: L2BlockMessage,

    /// Optional request ID this response is for
    pub request_id: Option<u64>,

    /// Whether this is the last block in a range response
    pub is_last: bool,
}

impl BlockResponseMessage {
    /// Create a new block response
    pub fn new(block: L2BlockMessage, request_id: Option<u64>, is_last: bool) -> Self {
        debug!(
            block_number = block.header.block_number,
            request_id = ?request_id,
            is_last,
            "Creating new BlockResponseMessage"
        );

        Self {
            block,
            request_id,
            is_last,
        }
    }
}

/// Block range response message to fulfill a block range request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRangeResponseMessage {
    /// The blocks being returned
    pub blocks: Vec<L2BlockMessage>,

    /// The request ID this response is for
    pub request_id: u64,

    /// Whether this is the last response in the range
    pub is_last: bool,

    /// The next block number to request (if !is_last)
    pub next_block: Option<u64>,
}

impl BlockRangeResponseMessage {
    /// Create a new block range response
    pub fn new(
        blocks: Vec<L2BlockMessage>,
        request_id: u64,
        is_last: bool,
        next_block: Option<u64>,
    ) -> Self {
        debug!(
            block_count = blocks.len(),
            request_id,
            is_last,
            next_block = ?next_block,
            "Creating new BlockRangeResponseMessage"
        );

        Self {
            blocks,
            request_id,
            is_last,
            next_block,
        }
    }

    /// Get the highest block number in this response
    pub fn highest_block_number(&self) -> Option<u64> {
        self.blocks
            .iter()
            .map(|block| block.header.block_number)
            .max()
    }

    /// Get the lowest block number in this response
    pub fn lowest_block_number(&self) -> Option<u64> {
        self.blocks
            .iter()
            .map(|block| block.header.block_number)
            .min()
    }
}

// Implement verification for L2 blocks
impl VerifiableMessage for L2BlockMessage {
    fn verify(&self) -> Result<(), P2PError> {
        debug!(
            block_number = self.header.block_number,
            timestamp = self.header.timestamp,
            "Verifying L2BlockMessage"
        );

        // In a real implementation, we would:
        // 1. Verify the parent hash exists in our chain
        // 2. Verify the timestamp is reasonable
        // 3. Verify the block number follows the parent
        // 4. Verify the state root is correct after applying transactions

        // Basic validation for now
        if self.header.timestamp == 0 {
            let error_msg = "Block timestamp cannot be zero";
            error!(
                block_number = self.header.block_number,
                error = error_msg,
                "Block verification failed"
            );
            return Err(P2PError::Verification(error_msg.into()));
        }

        if self.header.block_number == 0 && !self.header.parent_hash.iter().all(|&b| b == 0) {
            // Genesis block should have zero parent hash
            let error_msg = "Invalid parent hash for genesis block";
            error!(
                block_number = self.header.block_number,
                error = error_msg,
                "Block verification failed"
            );
            return Err(P2PError::Verification(error_msg.into()));
        }

        // If we have transactions, verify the merkle root matches
        if let Some(transactions) = &self.transactions {
            // In a real implementation, we would:
            // 1. Calculate the merkle root from the transactions
            // 2. Verify it matches the transactions_merkle_root

            // For now, just check that we have at least one transaction
            if transactions.is_empty() {
                let error_msg = "Block with transactions has empty transaction list";
                error!(
                    block_number = self.header.block_number,
                    error = error_msg,
                    "Block verification failed"
                );
                return Err(P2PError::Verification(error_msg.into()));
            }
        }

        debug!(
            block_number = self.header.block_number,
            "L2BlockMessage verification successful"
        );

        Ok(())
    }
}
