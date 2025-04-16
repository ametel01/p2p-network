//! Messages for the P2P network.
//! This crate defines all the message types used in the Citrea network.

// Define modules
mod blocks;
mod proofs;
mod transactions;
mod types;

// Re-export public API
pub use blocks::{L2BlockHeader, L2BlockMessage};
pub use proofs::{BatchProofMessage, LightClientProofMessage, SequencerCommitmentMessage};
pub use transactions::EVMTransactionMessage;
pub use types::CitreaMessage;
