use std::fmt::Debug;
use std::sync::Arc;

use core_crate::{P2PConfig, P2PError, PeerId};
use tracing::info;

use crate::handler::{DefaultMessageHandler, MessageHandler};
use crate::service::{DefaultP2PService, P2PService, RealP2PService};
use crate::transport::TransportType;

/// Configuration for the P2P service.
///
/// This struct contains all the configuration parameters needed to
/// initialize a P2P network service.
#[derive(Debug)]
pub struct P2PServiceConfig {
    /// The base P2P configuration (ports, addresses, etc.)
    pub p2p_config: P2PConfig,

    /// The message handler to process incoming messages
    pub message_handler: Box<dyn MessageHandler>,

    /// Whether to enable peer discovery
    pub enable_discovery: bool,

    /// The type of transport to use (TCP, UDP, etc.)
    pub transport_type: TransportType,
}

// We can't derive Clone because Box<dyn MessageHandler> doesn't implement Clone,
// so we implement it manually
impl Clone for P2PServiceConfig {
    fn clone(&self) -> Self {
        // Use a standard DefaultMessageHandler for the clone
        // In practice, we replace it with our own handler right away
        Self {
            p2p_config: self.p2p_config.clone(),
            message_handler: Box::new(DefaultMessageHandler),
            enable_discovery: self.enable_discovery,
            transport_type: self.transport_type,
        }
    }
}

/// Builder for creating P2PService instances.
///
/// This follows the builder pattern to allow flexible configuration
/// of the P2P service with reasonable defaults.
pub struct P2PServiceBuilder {
    config: P2PServiceConfig,
    local_id: PeerId,
    chain_id: u64,
    use_mock: bool,
}

impl P2PServiceBuilder {
    /// Create a new builder with default values
    pub fn new(p2p_config: P2PConfig) -> Self {
        info!("Creating new P2PServiceBuilder with default configuration");

        // Generate a random local peer ID if not provided
        let local_id = PeerId::random();

        Self {
            config: P2PServiceConfig {
                p2p_config,
                message_handler: Box::new(DefaultMessageHandler),
                enable_discovery: true,
                transport_type: TransportType::Tcp,
            },
            local_id,
            chain_id: 1, // Default chain ID
            use_mock: false,
        }
    }

    /// Set a specific local peer ID
    pub fn with_local_id(mut self, id: PeerId) -> Self {
        info!(peer_id = ?id, "Setting local peer ID");
        self.local_id = id;
        self
    }

    /// Set the chain ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        info!(chain_id, "Setting chain ID");
        self.chain_id = chain_id;
        self
    }

    /// Set a custom message handler
    pub fn with_message_handler(mut self, handler: Box<dyn MessageHandler>) -> Self {
        info!("Setting custom message handler");
        self.config.message_handler = handler;
        self
    }

    /// Enable or disable peer discovery
    pub fn with_discovery(mut self, enable: bool) -> Self {
        info!(enable, "Setting peer discovery");
        self.config.enable_discovery = enable;
        self
    }

    /// Set the transport type
    pub fn with_transport(mut self, transport_type: TransportType) -> Self {
        info!(transport = ?transport_type, "Setting transport type");
        self.config.transport_type = transport_type;
        self
    }

    /// Use a mock implementation instead of a real one
    pub fn with_mock(mut self) -> Self {
        info!("Setting to use mock implementation");
        self.use_mock = true;
        self
    }

    /// Build the P2P service with the current configuration
    pub fn build(self) -> Arc<dyn P2PService> {
        info!("Building P2P service");

        if self.use_mock {
            info!("Using mock P2P service implementation");
            Arc::new(DefaultP2PService::new(self.local_id))
        } else {
            info!(
                local_id = ?self.local_id,
                chain_id = self.chain_id,
                "Creating real P2P service implementation"
            );

            // When cloning config, we need to replace the message handler
            let mut config = self.config.clone();
            config.message_handler = self.config.message_handler;

            Arc::new(RealP2PService::new(self.local_id, config, self.chain_id))
        }
    }

    /// Build the P2P service and return as a boxed trait object
    pub fn build_boxed(self) -> Box<dyn P2PService> {
        let service = self.build();
        Box::new(ServiceWrapper(service))
    }

    /// Build the P2P service and return as an Arc for shared ownership
    pub fn build_shared(self) -> Arc<dyn P2PService> {
        self.build()
    }
}

/// A wrapper struct that forwards P2PService trait methods to an Arc<dyn P2PService>
#[derive(Debug)]
struct ServiceWrapper(Arc<dyn P2PService>);

#[async_trait::async_trait]
impl P2PService for ServiceWrapper {
    async fn start(&self) -> Result<(), P2PError> {
        self.0.start().await
    }

    async fn stop(&self) -> Result<(), P2PError> {
        self.0.stop().await
    }

    async fn send_message(
        &self,
        peer_id: PeerId,
        message: messages::CitreaMessage,
    ) -> Result<(), P2PError> {
        self.0.send_message(peer_id, message).await
    }

    async fn broadcast_message(&self, message: messages::CitreaMessage) -> Result<(), P2PError> {
        self.0.broadcast_message(message).await
    }

    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        self.0.get_peers().await
    }
}
