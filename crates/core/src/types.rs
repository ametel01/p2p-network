use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A unique identifier for a peer in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Create a new peer ID from bytes
    pub fn new(id: [u8; 32]) -> Self {
        Self(id)
    }

    /// Generate a random peer ID (useful for testing)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut id = [0u8; 32];
        rng.fill(&mut id);
        Self(id)
    }

    /// Create a peer ID from a public key
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        Self(id)
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string for display
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// A Bitcoin transaction ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BitcoinTxid([u8; 32]);

impl BitcoinTxid {
    /// Create a new txid
    pub fn new(id: [u8; 32]) -> Self {
        Self(id)
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A state root hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateRoot([u8; 32]);

impl StateRoot {
    /// Create a new state root
    pub fn new(value: [u8; 32]) -> Self {
        Self(value)
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A Merkle root hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleRoot([u8; 32]);

impl MerkleRoot {
    /// Create a new merkle root
    pub fn new(value: [u8; 32]) -> Self {
        Self(value)
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// An ECDSA signature using secp256k1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcdsaSignature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

impl EcdsaSignature {
    /// Create a new signature
    pub fn new(r: [u8; 32], s: [u8; 32], v: u8) -> Self {
        Self { r, s, v }
    }
}
