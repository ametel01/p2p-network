use std::fmt::Debug;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::error::P2PError;
use crate::network::PeerInfo;
use crate::types::PeerId;

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

/// A trait for handling incoming messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message from a peer
    async fn handle_message(&self, peer_id: PeerId, message: Message) -> Result<(), P2PError>;

    /// Handle a ping message
    async fn handle_ping(
        &self,
        peer_id: PeerId,
        nonce: u64,
        timestamp: u64,
    ) -> Result<(), P2PError>;

    /// Handle a pong message
    async fn handle_pong(
        &self,
        peer_id: PeerId,
        nonce: u64,
        timestamp: u64,
    ) -> Result<(), P2PError>;

    /// Handle a discover peers request
    async fn handle_discover_peers(&self, peer_id: PeerId) -> Result<(), P2PError>;

    /// Handle a peer list message
    async fn handle_peer_list(&self, peer_id: PeerId, peers: Vec<PeerInfo>)
        -> Result<(), P2PError>;

    /// Handle a custom message
    async fn handle_custom(&self, peer_id: PeerId, data: Vec<u8>) -> Result<(), P2PError>;

    /// Handle a direct message
    async fn handle_direct_message(
        &self,
        peer_id: PeerId,
        target: PeerId,
        data: Vec<u8>,
    ) -> Result<(), P2PError>;

    /// Handle a handshake message
    async fn handle_handshake(
        &self,
        peer_id: PeerId,
        version: u32,
        sender_id: PeerId,
        listen_port: u16,
        capabilities: Vec<String>,
    ) -> Result<(), P2PError>;

    /// Handle a gossip message
    async fn handle_gossip(
        &self,
        peer_id: PeerId,
        topic: String,
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<(), P2PError>;

    /// Handle a subscribe message
    async fn handle_subscribe(&self, peer_id: PeerId, topic: String) -> Result<(), P2PError>;

    /// Handle an unsubscribe message
    async fn handle_unsubscribe(&self, peer_id: PeerId, topic: String) -> Result<(), P2PError>;
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

    /// Direct message to a specific peer
    DirectMessage { target: PeerId, data: Vec<u8> },

    /// Handshake message for initial connection
    Handshake {
        version: u32,
        peer_id: PeerId,
        listen_port: u16,
        capabilities: Vec<String>,
    },

    /// Gossip message for network-wide propagation
    Gossip {
        topic: String,
        data: Vec<u8>,
        ttl: u8, // Time-to-live (decrements at each hop)
    },

    /// Subscribe to a topic
    Subscribe { topic: String },

    /// Unsubscribe from a topic
    Unsubscribe { topic: String },
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

/// Message with timestamp information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedMessage {
    /// The actual message
    pub message: Message,
    /// Timestamp when the message was created
    pub timestamp: u64,
    /// Sender peer ID
    pub sender: PeerId,
}

impl TimestampedMessage {
    /// Create a new timestamped message
    pub fn new(message: Message, sender: PeerId) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            message,
            timestamp,
            sender,
        }
    }
}

impl NetworkMessage for TimestampedMessage {
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
