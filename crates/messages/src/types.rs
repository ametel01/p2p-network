use bytes::Bytes;
use core_crate::{NetworkMessage, P2PError, PeerId, SignedMessage, VerifiableMessage};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::blocks::{BlockAnnouncementMessage, BlockRequestMessage, L2BlockMessage};
use crate::proofs::{BatchProofMessage, LightClientProofMessage, SequencerCommitmentMessage};
use crate::transactions::EVMTransactionMessage;

/// Protocol version for the messaging system
pub const PROTOCOL_VERSION: u16 = 1;

/// Custom message for application-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custom {
    /// Message type identifier
    pub message_type: String,

    /// Custom data payload
    pub data: Vec<u8>,
}

impl Custom {
    /// Create a new custom message
    pub fn new(message_type: String, data: Vec<u8>) -> Self {
        Self { message_type, data }
    }
}

/// The main message envelope that covers all possible message types in the Citrea network.
///
/// This struct serves as the top-level container for all message types that can be
/// sent over the network. It includes protocol versioning and metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Protocol version for compatibility checking
    pub version: u16,

    /// The actual message payload
    pub payload: CitreaMessage,

    /// Optional sender identifier
    pub sender: Option<PeerId>,

    /// Chain ID to prevent cross-chain replay
    pub chain_id: u64,

    /// Timestamp when the message was created
    pub timestamp: u64,
}

impl MessageEnvelope {
    /// Create a new message envelope with the current protocol version
    pub fn new(payload: CitreaMessage, chain_id: u64, sender: Option<PeerId>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: PROTOCOL_VERSION,
            payload,
            sender,
            chain_id,
            timestamp,
        }
    }

    /// Check if this message is for the expected chain
    pub fn is_for_chain(&self, expected_chain_id: u64) -> bool {
        if self.chain_id != expected_chain_id {
            warn!(
                message_type = self.payload.message_type(),
                expected_chain = expected_chain_id,
                actual_chain = self.chain_id,
                "Message for wrong chain received"
            );
            return false;
        }
        true
    }

    /// Check if this message version is compatible with our protocol
    pub fn is_compatible(&self) -> bool {
        // For now, we only accept our exact version
        // In the future, we might support backward compatibility
        if self.version != PROTOCOL_VERSION {
            warn!(
                message_type = self.payload.message_type(),
                our_version = PROTOCOL_VERSION,
                message_version = self.version,
                "Incompatible message version received"
            );
            return false;
        }
        true
    }
}

/// The main message enum that covers all possible message types in the Citrea network.
///
/// This enum serves as the top-level container for all message types that can be
/// sent over the network. Each variant represents a different category of message.
#[derive(Debug, Serialize, Deserialize)]
pub enum CitreaMessage {
    /// Block-related messages containing L2 block data
    L2Block(L2BlockMessage),

    /// Block announcement (lightweight block notification)
    BlockAnnouncement(BlockAnnouncementMessage),

    /// Block request message
    BlockRequest(BlockRequestMessage),

    /// Sequencer commitment messages for L2 state
    SequencerCommitment(SequencerCommitmentMessage),

    /// Proof messages for batches
    BatchProof(BatchProofMessage),

    /// Light client proof messages
    LightClientProof(LightClientProofMessage),

    /// EVM transaction messages
    EVMTransaction(EVMTransactionMessage),

    /// Custom messages
    Custom(Custom),
}

impl CitreaMessage {
    /// Get a string description of the message type for logging
    pub fn message_type(&self) -> &'static str {
        match self {
            CitreaMessage::L2Block(_) => "L2Block",
            CitreaMessage::BlockAnnouncement(_) => "BlockAnnouncement",
            CitreaMessage::BlockRequest(_) => "BlockRequest",
            CitreaMessage::SequencerCommitment(_) => "SequencerCommitment",
            CitreaMessage::BatchProof(_) => "BatchProof",
            CitreaMessage::LightClientProof(_) => "LightClientProof",
            CitreaMessage::EVMTransaction(_) => "EVMTransaction",
            CitreaMessage::Custom(_) => "Custom",
        }
    }

    /// Try to get this message as an L2BlockMessage
    pub fn as_l2_block(&self) -> Option<&L2BlockMessage> {
        match self {
            CitreaMessage::L2Block(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as a BlockAnnouncementMessage
    pub fn as_block_announcement(&self) -> Option<&BlockAnnouncementMessage> {
        match self {
            CitreaMessage::BlockAnnouncement(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as a BlockRequestMessage
    pub fn as_block_request(&self) -> Option<&BlockRequestMessage> {
        match self {
            CitreaMessage::BlockRequest(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as a SequencerCommitmentMessage
    pub fn as_sequencer_commitment(&self) -> Option<&SequencerCommitmentMessage> {
        match self {
            CitreaMessage::SequencerCommitment(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as a BatchProofMessage
    pub fn as_batch_proof(&self) -> Option<&BatchProofMessage> {
        match self {
            CitreaMessage::BatchProof(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as a LightClientProofMessage
    pub fn as_light_client_proof(&self) -> Option<&LightClientProofMessage> {
        match self {
            CitreaMessage::LightClientProof(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to get this message as an EVMTransactionMessage
    pub fn as_evm_transaction(&self) -> Option<&EVMTransactionMessage> {
        match self {
            CitreaMessage::EVMTransaction(msg) => Some(msg),
            _ => None,
        }
    }

    /// Verify the message based on its type
    pub fn verify(&self) -> Result<(), P2PError> {
        match self {
            CitreaMessage::L2Block(msg) => msg.verify(),
            CitreaMessage::BlockAnnouncement(_) => Ok(()), // Simple announcement, no verification needed
            CitreaMessage::BlockRequest(_) => Ok(()),      // Simple request, no verification needed
            CitreaMessage::SequencerCommitment(msg) => msg.verify_signature(),
            CitreaMessage::BatchProof(msg) => msg.verify_signature(),
            CitreaMessage::LightClientProof(msg) => msg.verify(),
            CitreaMessage::EVMTransaction(msg) => {
                if !msg.verify_format() {
                    Err(P2PError::Verification(
                        "Invalid EVM transaction format".into(),
                    ))
                } else {
                    Ok(())
                }
            }
            CitreaMessage::Custom(_msg) => Ok(()), // Custom messages don't need verification
        }
    }
}

// Implement NetworkMessage for the envelope rather than the raw message
impl NetworkMessage for MessageEnvelope {
    /// Serialize the message envelope to bytes for network transmission
    fn serialize(&self) -> Result<Bytes, P2PError> {
        // Log serialization attempt with message type
        debug!(
            message_type = self.payload.message_type(),
            version = self.version,
            "Serializing MessageEnvelope"
        );

        // Use JSON serialization for simplicity
        // In a production system, a more efficient binary format might be used
        let bytes = match serde_json::to_vec(self).map_err(P2PError::Serialization) {
            Ok(b) => {
                debug!(
                    message_type = self.payload.message_type(),
                    size_bytes = b.len(),
                    "Successfully serialized message"
                );
                b
            }
            Err(e) => {
                error!(
                    message_type = self.payload.message_type(),
                    error = %e,
                    "Failed to serialize message"
                );
                return Err(e);
            }
        };

        Ok(Bytes::from(bytes))
    }

    /// Deserialize bytes into a message envelope
    fn deserialize(bytes: Bytes) -> Result<Self, P2PError> {
        // Log deserialization attempt
        debug!(
            bytes_len = bytes.len(),
            "Attempting to deserialize MessageEnvelope"
        );

        // Parse the incoming bytes as JSON
        let envelope = match serde_json::from_slice::<MessageEnvelope>(&bytes)
            .map_err(P2PError::Serialization)
        {
            Ok(e) => {
                debug!(
                    message_type = e.payload.message_type(),
                    version = e.version,
                    "Successfully deserialized message envelope"
                );
                e
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Failed to deserialize message envelope"
                );
                return Err(e);
            }
        };

        // Check protocol compatibility
        if !envelope.is_compatible() {
            return Err(P2PError::InvalidProtocolVersion(envelope.version as u32));
        }

        Ok(envelope)
    }
}
