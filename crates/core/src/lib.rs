use std::fmt::Debug;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A unique identifier for a peer in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

/// A Bitcoin transaction ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BitcoinTxid([u8; 32]);

/// A state root hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateRoot([u8; 32]);

impl StateRoot {
    /// Create a new state root
    pub fn new(value: [u8; 32]) -> Self {
        Self(value)
    }
}

/// A Merkle root hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleRoot([u8; 32]);

/// An ECDSA signature using secp256k1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcdsaSignature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

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
}

/// A trait for messages that can be signed
pub trait SignedMessage: Debug + Send + Sync {
    /// Verify the signature of the message
    fn verify_signature(&self) -> Result<(), P2PError>;
}

/// A trait for messages that can be verified
pub trait VerifiableMessage: Debug + Send + Sync {
    /// Verify the contents of the message
    fn verify(&self) -> Result<(), P2PError>;
}

/// A trait for messages that can be serialized and deserialized
pub trait NetworkMessage: Debug + Send + Sync {
    /// Serialize the message to bytes
    fn serialize(&self) -> Result<Bytes, P2PError>;

    /// Deserialize the message from bytes
    fn deserialize(bytes: Bytes) -> Result<Self, P2PError>
    where
        Self: Sized;
}

/// Configuration for the p2p network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// The port to listen on
    pub port: u16,

    /// Maximum number of connections
    pub max_connections: usize,

    /// Bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<String>,

    /// Whether to enable discovery
    pub enable_discovery: bool,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            port: 30303,
            max_connections: 50,
            bootstrap_nodes: Vec::new(),
            enable_discovery: true,
        }
    }
}
