use serde::{Deserialize, Serialize};

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
        // Simple random implementation for now
        let mut id = [0u8; 32];
        for byte in id.iter_mut() {
            *byte = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
                % 255) as u8;
        }
        Self(id)
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
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
