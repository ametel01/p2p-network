use std::fmt::Debug;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::error::P2PError;
use crate::network::PeerInfo;

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

/// Basic message types for the P2P network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Ping message to check if a peer is alive
    Ping { nonce: u64, timestamp: u64 },
    /// Pong response to a ping
    Pong { nonce: u64, timestamp: u64 },
    /// Discover peers request
    DiscoverPeers,
    /// Response with known peers
    PeerList { peers: Vec<PeerInfo> },
    /// Custom application-specific message
    Custom { data: Vec<u8> },
}

impl NetworkMessage for Message {
    fn serialize(&self) -> Result<Bytes, P2PError> {
        let json = serde_json::to_string(self)?;
        Ok(Bytes::from(json))
    }

    fn deserialize(bytes: Bytes) -> Result<Self, P2PError> {
        let json = String::from_utf8_lossy(&bytes);
        let message = serde_json::from_str(&json)?;
        Ok(message)
    }
}
