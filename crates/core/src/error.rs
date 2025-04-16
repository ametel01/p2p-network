use std::io;
use thiserror::Error;

use crate::types::PeerId;

/// Errors that can occur in the p2p system
#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Network error: {0}")]
    Network(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IO(#[from] io::Error),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),

    #[error("Already connected to peer: {0}")]
    AlreadyConnected(PeerId),

    #[error("Connection timeout with peer: {0}")]
    ConnectionTimeout(PeerId),

    #[error("Handshake failed with peer: {0}")]
    HandshakeFailed(PeerId),

    #[error("Maximum connections reached")]
    MaxConnectionsReached,

    #[error("Invalid protocol version: {0}")]
    InvalidProtocolVersion(u32),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Address already in use: {0}")]
    AddressInUse(String),
}
