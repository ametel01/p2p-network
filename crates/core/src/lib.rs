// Core module for P2P network functionality
mod error;
mod logging;
mod message;
mod network;
mod types;

// Re-export public API
pub use error::P2PError;
pub use logging::{init_default_logging, init_logging, init_subscriber};
pub use message::{Message, NetworkMessage, SignedMessage, VerifiableMessage};
pub use network::{Network, P2PConfig, PeerConnection, PeerInfo};
pub use types::{BitcoinTxid, EcdsaSignature, MerkleRoot, PeerId, StateRoot};
