use std::fmt::Debug;

use bytes::Bytes;
use core_crate::{
    BitcoinTxid, EcdsaSignature, MerkleRoot, NetworkMessage, P2PError, SignedMessage, StateRoot,
    VerifiableMessage,
};
use serde::{Deserialize, Serialize};

/// The main message enum that covers all possible message types
#[derive(Debug, Serialize, Deserialize)]
pub enum CitreaMessage {
    L2Block(L2BlockMessage),
    SequencerCommitment(SequencerCommitmentMessage),
    BatchProof(BatchProofMessage),
    LightClientProof(LightClientProofMessage),
    EVMTransaction(EVMTransactionMessage),
}

/// Message containing L2 block data
#[derive(Debug, Serialize, Deserialize)]
pub struct L2BlockMessage {
    pub header: L2BlockHeader,
    pub transactions_merkle_root: MerkleRoot,
}

/// Header information for an L2 block
#[derive(Debug, Serialize, Deserialize)]
pub struct L2BlockHeader {
    pub block_number: u64,
    pub timestamp: u64,
    pub parent_hash: [u8; 32],
    pub state_root: StateRoot,
}

/// Message containing sequencer commitment data
#[derive(Debug, Serialize, Deserialize)]
pub struct SequencerCommitmentMessage {
    pub index: u64,
    pub l2_end: u64,
    pub merkle_root: MerkleRoot,
    pub bitcoin_txid: BitcoinTxid,
    pub signature: Option<EcdsaSignature>,
}

/// Message containing batch proof data
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchProofMessage {
    pub bitcoin_txid: BitcoinTxid,
    pub groth16_proof: Vec<u8>,
    pub signature: Option<EcdsaSignature>,
}

/// Message containing light client proof data
#[derive(Debug, Serialize, Deserialize)]
pub struct LightClientProofMessage {
    pub zk_proof: Vec<u8>,
    pub state_root: StateRoot,
}

/// Message containing EVM transaction data
#[derive(Debug, Serialize, Deserialize)]
pub struct EVMTransactionMessage {
    pub payload: Vec<u8>,
}

// Implement NetworkMessage for all message types
impl NetworkMessage for CitreaMessage {
    fn serialize(&self) -> Result<Bytes, P2PError> {
        let bytes = serde_json::to_vec(self)?;
        Ok(Bytes::from(bytes))
    }

    fn deserialize(bytes: Bytes) -> Result<Self, P2PError> {
        let message = serde_json::from_slice(&bytes)?;
        Ok(message)
    }
}

// Implement SignedMessage for messages that can be signed
impl SignedMessage for SequencerCommitmentMessage {
    fn verify_signature(&self) -> Result<(), P2PError> {
        // TODO: Implement signature verification
        Ok(())
    }
}

impl SignedMessage for BatchProofMessage {
    fn verify_signature(&self) -> Result<(), P2PError> {
        // TODO: Implement signature verification
        Ok(())
    }
}

// Implement VerifiableMessage for messages that need verification
impl VerifiableMessage for LightClientProofMessage {
    fn verify(&self) -> Result<(), P2PError> {
        // TODO: Implement proof verification
        Ok(())
    }
}

impl VerifiableMessage for L2BlockMessage {
    fn verify(&self) -> Result<(), P2PError> {
        // TODO: Implement block verification
        Ok(())
    }
}
