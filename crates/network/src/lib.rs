//! Network implementation for P2P communication.
//!
//! This crate provides the network layer for the P2P system, handling
//! connections, message passing, and peer discovery.

// Define the modules
mod config;
mod discovery;
mod handler;
mod service;
mod transport;

// Re-export the public API
pub use config::{P2PServiceBuilder, P2PServiceConfig};
pub use discovery::{PeerDiscovery, PeerDiscoveryFactory};
pub use handler::{DefaultMessageHandler, MessageHandler};
pub use service::{DefaultP2PService, P2PService};
pub use transport::{Transport, TransportFactory, TransportType};
