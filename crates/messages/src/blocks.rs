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
        }
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

        // Additional verification would be implemented here
        debug!(
            block_number = self.header.block_number,
            "L2BlockMessage verification successful"
        );

        Ok(())
    }
}
