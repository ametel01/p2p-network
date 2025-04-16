use bytes::Bytes;
use core_crate::{NetworkMessage, P2PError};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::blocks::L2BlockMessage;
use crate::proofs::{BatchProofMessage, LightClientProofMessage, SequencerCommitmentMessage};
use crate::transactions::EVMTransactionMessage;

/// The main message enum that covers all possible message types in the Citrea network.
///
/// This enum serves as the top-level container for all message types that can be
/// sent over the network. Each variant represents a different category of message.
#[derive(Debug, Serialize, Deserialize)]
pub enum CitreaMessage {
    /// Block-related messages containing L2 block data
    L2Block(L2BlockMessage),

    /// Sequencer commitment messages for L2 state
    SequencerCommitment(SequencerCommitmentMessage),

    /// Proof messages for batches
    BatchProof(BatchProofMessage),

    /// Light client proof messages
    LightClientProof(LightClientProofMessage),

    /// EVM transaction messages
    EVMTransaction(EVMTransactionMessage),
}

impl CitreaMessage {
    /// Get a string description of the message type for logging
    pub fn message_type(&self) -> &'static str {
        match self {
            CitreaMessage::L2Block(_) => "L2Block",
            CitreaMessage::SequencerCommitment(_) => "SequencerCommitment",
            CitreaMessage::BatchProof(_) => "BatchProof",
            CitreaMessage::LightClientProof(_) => "LightClientProof",
            CitreaMessage::EVMTransaction(_) => "EVMTransaction",
        }
    }
}

// Implement NetworkMessage for the main message type
impl NetworkMessage for CitreaMessage {
    /// Serialize the message to bytes for network transmission
    fn serialize(&self) -> Result<Bytes, P2PError> {
        // Log serialization attempt with message type
        debug!(
            message_type = self.message_type(),
            "Serializing CitreaMessage"
        );

        // Use JSON serialization for simplicity
        // In a production system, a more efficient binary format might be used
        let bytes = match serde_json::to_vec(self).map_err(P2PError::Serialization) {
            Ok(b) => {
                debug!(
                    message_type = self.message_type(),
                    size_bytes = b.len(),
                    "Successfully serialized message"
                );
                b
            }
            Err(e) => {
                error!(
                    message_type = self.message_type(),
                    error = %e,
                    "Failed to serialize message"
                );
                return Err(e);
            }
        };

        Ok(Bytes::from(bytes))
    }

    /// Deserialize bytes into a message
    fn deserialize(bytes: Bytes) -> Result<Self, P2PError> {
        // Log deserialization attempt
        debug!(
            bytes_len = bytes.len(),
            "Attempting to deserialize CitreaMessage"
        );

        // Parse the incoming bytes as JSON
        let message = match serde_json::from_slice::<CitreaMessage>(&bytes)
            .map_err(P2PError::Serialization)
        {
            Ok(m) => {
                debug!(
                    message_type = m.message_type(),
                    "Successfully deserialized message"
                );
                m
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Failed to deserialize message"
                );
                return Err(e);
            }
        };

        Ok(message)
    }
}
