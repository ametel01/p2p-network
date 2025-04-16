use std::fmt::Debug;
use std::sync::Arc;

use core_crate::P2PConfig;
use tracing::info;

use crate::handler::{DefaultMessageHandler, MessageHandler};
use crate::service::{DefaultP2PService, P2PService};
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

/// Builder for creating P2PService instances.
///
/// This follows the builder pattern to allow flexible configuration
/// of the P2P service with reasonable defaults.
pub struct P2PServiceBuilder {
    config: P2PServiceConfig,
}

impl P2PServiceBuilder {
    /// Create a new builder with default values
    pub fn new(p2p_config: P2PConfig) -> Self {
        info!("Creating new P2PServiceBuilder with default configuration");

        Self {
            config: P2PServiceConfig {
                p2p_config,
                message_handler: Box::new(DefaultMessageHandler),
                enable_discovery: true,
                transport_type: TransportType::Tcp,
            },
        }
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

    /// Build the P2P service with the current configuration
    pub fn build(self) -> impl P2PService {
        info!("Building P2P service");

        // In a real implementation, this would create a proper P2P service
        // with the configured options. For now, we just return the default.

        // Create a real P2PService implementation here
        DefaultP2PService
    }

    /// Build the P2P service and return as a boxed trait object
    pub fn build_boxed(self) -> Box<dyn P2PService> {
        Box::new(self.build())
    }

    /// Build the P2P service and return as an Arc for shared ownership
    pub fn build_shared(self) -> Arc<dyn P2PService> {
        Arc::new(self.build())
    }
}
