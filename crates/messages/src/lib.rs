//! Messages for the P2P network.
//! This crate defines all the message types used in the Citrea network.

// Define modules
mod blocks;
mod proofs;
mod transactions;
mod types;

// Re-export public API
pub use blocks::{BlockAnnouncementMessage, BlockRequestMessage, L2BlockHeader, L2BlockMessage};
pub use proofs::{BatchProofMessage, LightClientProofMessage, SequencerCommitmentMessage};
pub use transactions::{
    EVMTransactionMessage, EVMTransactionMessageBuilder, EVMTransactionType, GasPricing,
};
pub use types::{CitreaMessage, Custom, MessageEnvelope, PROTOCOL_VERSION};

/// Default chain ID for the Citrea network
pub const DEFAULT_CHAIN_ID: u64 = 2424;

/// Create a new message envelope for the default chain
pub fn create_message(
    message: CitreaMessage,
    sender: Option<core_crate::PeerId>,
) -> MessageEnvelope {
    MessageEnvelope::new(message, DEFAULT_CHAIN_ID, sender)
}

/// Create a new message envelope for a specific chain
pub fn create_message_for_chain(
    message: CitreaMessage,
    chain_id: u64,
    sender: Option<core_crate::PeerId>,
) -> MessageEnvelope {
    MessageEnvelope::new(message, chain_id, sender)
}
