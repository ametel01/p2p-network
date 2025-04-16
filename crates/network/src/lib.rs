use std::fmt::Debug;

use async_trait::async_trait;
use core_crate::{P2PConfig, P2PError, PeerId};
use messages::CitreaMessage;

/// A trait for handling incoming messages
#[async_trait]
pub trait MessageHandler: Debug + Send + Sync {
    /// Handle an incoming message
    async fn handle_message(&self, peer_id: PeerId, message: CitreaMessage)
        -> Result<(), P2PError>;
}

/// A trait for the p2p network service
#[async_trait]
pub trait P2PService: Debug + Send + Sync {
    /// Start the p2p service
    async fn start(&self) -> Result<(), P2PError>;

    /// Stop the p2p service
    async fn stop(&self) -> Result<(), P2PError>;

    /// Send a message to a specific peer
    async fn send_message(&self, peer_id: PeerId, message: CitreaMessage) -> Result<(), P2PError>;

    /// Broadcast a message to all connected peers
    async fn broadcast_message(&self, message: CitreaMessage) -> Result<(), P2PError>;

    /// Get the list of connected peers
    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError>;
}

/// A trait for peer discovery
#[async_trait]
pub trait PeerDiscovery: Debug + Send + Sync {
    /// Start the peer discovery process
    async fn start(&self) -> Result<(), P2PError>;

    /// Stop the peer discovery process
    async fn stop(&self) -> Result<(), P2PError>;

    /// Get the list of discovered peers
    async fn get_discovered_peers(&self) -> Result<Vec<PeerId>, P2PError>;
}

/// Configuration for the p2p service
#[derive(Debug)]
pub struct P2PServiceConfig {
    /// The p2p configuration
    pub p2p_config: P2PConfig,

    /// The message handler
    pub message_handler: Box<dyn MessageHandler>,

    /// Whether to enable peer discovery
    pub enable_discovery: bool,
}

/// A builder for the p2p service
pub struct P2PServiceBuilder {
    config: P2PServiceConfig,
}

impl P2PServiceBuilder {
    /// Create a new builder
    pub fn new(p2p_config: P2PConfig) -> Self {
        Self {
            config: P2PServiceConfig {
                p2p_config,
                message_handler: Box::new(DefaultMessageHandler),
                enable_discovery: true,
            },
        }
    }

    /// Set the message handler
    pub fn with_message_handler(mut self, handler: Box<dyn MessageHandler>) -> Self {
        self.config.message_handler = handler;
        self
    }

    /// Enable or disable peer discovery
    pub fn with_discovery(mut self, enable: bool) -> Self {
        self.config.enable_discovery = enable;
        self
    }

    /// Build the p2p service
    pub fn build(self) -> impl P2PService {
        // TODO: Implement the actual service
        DefaultP2PService
    }
}

/// A default message handler that does nothing
#[derive(Debug, Clone)]
struct DefaultMessageHandler;

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle_message(
        &self,
        _peer_id: PeerId,
        _message: CitreaMessage,
    ) -> Result<(), P2PError> {
        Ok(())
    }
}

/// A default p2p service that does nothing
#[derive(Debug, Clone)]
struct DefaultP2PService;

#[async_trait]
impl P2PService for DefaultP2PService {
    async fn start(&self) -> Result<(), P2PError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        Ok(())
    }

    async fn send_message(
        &self,
        _peer_id: PeerId,
        _message: CitreaMessage,
    ) -> Result<(), P2PError> {
        Ok(())
    }

    async fn broadcast_message(&self, _message: CitreaMessage) -> Result<(), P2PError> {
        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        Ok(Vec::new())
    }
}
