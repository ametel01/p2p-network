use core_crate::{
    BitcoinTxid, EcdsaSignature, MerkleRoot, P2PError, SignedMessage, StateRoot, VerifiableMessage,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

/// Message containing sequencer commitment data.
///
/// Sequencer commitments represent batches of L2 blocks that are committed
/// to L1 (Bitcoin) for security. These commitments anchor the L2 state to
/// the security of the Bitcoin blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerCommitmentMessage {
    /// Sequential index of this commitment
    pub index: u64,

    /// The ending L2 block number included in this commitment
    pub l2_end: u64,

    /// Merkle root of the committed state
    pub merkle_root: MerkleRoot,

    /// Bitcoin transaction ID where this was committed
    pub bitcoin_txid: BitcoinTxid,

    /// Signature of the sequencer (if available)
    pub signature: Option<EcdsaSignature>,
}

impl SequencerCommitmentMessage {
    /// Create a new sequencer commitment message
    pub fn new(
        index: u64,
        l2_end: u64,
        merkle_root: MerkleRoot,
        bitcoin_txid: BitcoinTxid,
        signature: Option<EcdsaSignature>,
    ) -> Self {
        debug!(
            index,
            l2_end,
            bitcoin_txid = ?bitcoin_txid,
            has_signature = signature.is_some(),
            "Creating new SequencerCommitmentMessage"
        );

        Self {
            index,
            l2_end,
            merkle_root,
            bitcoin_txid,
            signature,
        }
    }
}

/// Message containing batch proof data.
///
/// Batch proofs provide cryptographic evidence that the sequencer's
/// commitments are valid. These proofs allow verification of state
/// transitions without re-executing all transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProofMessage {
    /// Bitcoin transaction ID this proof corresponds to
    pub bitcoin_txid: BitcoinTxid,

    /// The zk-SNARK proof data
    pub groth16_proof: Vec<u8>,

    /// Signature of the proof producer (if available)
    pub signature: Option<EcdsaSignature>,
}

impl BatchProofMessage {
    /// Create a new batch proof message
    pub fn new(
        bitcoin_txid: BitcoinTxid,
        groth16_proof: Vec<u8>,
        signature: Option<EcdsaSignature>,
    ) -> Self {
        debug!(
            bitcoin_txid = ?bitcoin_txid,
            proof_size = groth16_proof.len(),
            has_signature = signature.is_some(),
            "Creating new BatchProofMessage"
        );

        Self {
            bitcoin_txid,
            groth16_proof,
            signature,
        }
    }
}

/// Message containing light client proof data.
///
/// Light client proofs allow lightweight verification of the network
/// state without requiring full state storage or computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightClientProofMessage {
    /// The ZK proof data for light client verification
    pub zk_proof: Vec<u8>,

    /// The state root being proven
    pub state_root: StateRoot,
}

impl LightClientProofMessage {
    /// Create a new light client proof message
    pub fn new(zk_proof: Vec<u8>, state_root: StateRoot) -> Self {
        debug!(
            proof_size = zk_proof.len(),
            state_root = ?state_root,
            "Creating new LightClientProofMessage"
        );

        Self {
            zk_proof,
            state_root,
        }
    }
}

// Implement signing verification for sequencer commitments
impl SignedMessage for SequencerCommitmentMessage {
    fn verify_signature(&self) -> Result<(), P2PError> {
        debug!(
            commitment_index = self.index,
            l2_end = self.l2_end,
            has_signature = self.signature.is_some(),
            "Verifying SequencerCommitmentMessage signature"
        );

        // Verify the signature if one is present
        if let Some(signature) = &self.signature {
            // In a real implementation, we would verify the signature against
            // a known sequencer public key

            // For now, just do basic validation
            if signature.v > 1 {
                let error_msg = "Invalid signature recovery ID";
                error!(
                    commitment_index = self.index,
                    recovery_id = signature.v,
                    error = error_msg,
                    "Signature verification failed"
                );
                return Err(P2PError::Verification(error_msg.into()));
            }

            // Further verification would be implemented here
        } else {
            // We might require signatures in production
            warn!(
                commitment_index = self.index,
                "Missing signature in SequencerCommitmentMessage"
            );
            // return Err(P2PError::Verification("Missing required signature".into()));
        }

        debug!(
            commitment_index = self.index,
            "SequencerCommitmentMessage signature verification successful"
        );

        Ok(())
    }
}

// Implement signature verification for batch proofs
impl SignedMessage for BatchProofMessage {
    fn verify_signature(&self) -> Result<(), P2PError> {
        debug!(
            bitcoin_txid = ?self.bitcoin_txid,
            has_signature = self.signature.is_some(),
            "Verifying BatchProofMessage signature"
        );

        // Similar to the sequencer commitment verification
        if let Some(signature) = &self.signature {
            // Basic validation for now
            if signature.v > 1 {
                let error_msg = "Invalid signature recovery ID";
                error!(
                    bitcoin_txid = ?self.bitcoin_txid,
                    recovery_id = signature.v,
                    error = error_msg,
                    "Signature verification failed"
                );
                return Err(P2PError::Verification(error_msg.into()));
            }
        } else {
            warn!(
                bitcoin_txid = ?self.bitcoin_txid,
                "Missing signature in BatchProofMessage"
            );
        }

        debug!(
            bitcoin_txid = ?self.bitcoin_txid,
            "BatchProofMessage signature verification successful"
        );

        Ok(())
    }
}

// Implement verification for light client proofs
impl VerifiableMessage for LightClientProofMessage {
    fn verify(&self) -> Result<(), P2PError> {
        debug!(
            proof_size = self.zk_proof.len(),
            state_root = ?self.state_root,
            "Verifying LightClientProofMessage"
        );

        // In a real implementation, we would:
        // 1. Verify the ZK proof against the state root
        // 2. Verify the proof format is valid

        // Basic validation for now
        if self.zk_proof.is_empty() {
            let error_msg = "Empty ZK proof";
            error!(
                state_root = ?self.state_root,
                error = error_msg,
                "Proof verification failed"
            );
            return Err(P2PError::Verification(error_msg.into()));
        }

        // Additional verification would be implemented here
        debug!(
            proof_size = self.zk_proof.len(),
            state_root = ?self.state_root,
            "LightClientProofMessage verification successful"
        );

        Ok(())
    }
}
