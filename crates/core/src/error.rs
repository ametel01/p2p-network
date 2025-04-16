use thiserror::Error;

use crate::types::PeerId;

/// Errors that can occur in the p2p system
#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Network error: {0}")]
    Network(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Peer not found: {0:?}")]
    PeerNotFound(PeerId),

    #[error("Already connected to peer: {0:?}")]
    AlreadyConnected(PeerId),
}
