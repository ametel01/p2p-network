use std::fmt::Debug;

use async_trait::async_trait;
use core_crate::{P2PError, PeerId};
use messages::CitreaMessage;
use tracing::{debug, info, warn};

/// Main service interface for P2P network functionality.
///
/// This trait defines the core operations that any P2P network implementation
/// must provide, including starting and stopping the service, sending messages,
/// and querying the connected peers.
#[async_trait]
pub trait P2PService: Debug + Send + Sync {
    /// Start the P2P service and begin accepting connections
    async fn start(&self) -> Result<(), P2PError>;

    /// Gracefully stop the P2P service
    async fn stop(&self) -> Result<(), P2PError>;

    /// Send a message to a specific peer
    async fn send_message(&self, peer_id: PeerId, message: CitreaMessage) -> Result<(), P2PError>;

    /// Broadcast a message to all connected peers
    async fn broadcast_message(&self, message: CitreaMessage) -> Result<(), P2PError>;

    /// Get the list of currently connected peers
    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError>;
}

/// A default implementation of P2PService that does nothing but log calls.
///
/// This is provided as a placeholder for testing and development purposes.
/// In a real application, this would be replaced with a proper network implementation.
#[derive(Debug, Clone)]
pub struct DefaultP2PService;

#[async_trait]
impl P2PService for DefaultP2PService {
    async fn start(&self) -> Result<(), P2PError> {
        info!("DefaultP2PService.start() called - this is a no-op implementation");
        debug!("Service would start listening for connections here");
        warn!("Using DefaultP2PService - this won't actually create any network connections");
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("DefaultP2PService.stop() called - this is a no-op implementation");
        debug!("Service would close all connections here");
        Ok(())
    }

    async fn send_message(&self, peer_id: PeerId, message: CitreaMessage) -> Result<(), P2PError> {
        info!(
            peer_id = ?peer_id,
            message_type = message.message_type(),
            "DefaultP2PService.send_message() called - message not actually sent"
        );
        debug!("Would send message: {:?}", message);
        warn!("Using DefaultP2PService - no actual network communication will occur");
        Ok(())
    }

    async fn broadcast_message(&self, message: CitreaMessage) -> Result<(), P2PError> {
        info!(
            message_type = message.message_type(),
            "DefaultP2PService.broadcast_message() called - message not actually broadcast"
        );
        debug!("Would broadcast message: {:?}", message);
        warn!("Using DefaultP2PService - no actual network communication will occur");
        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        info!("DefaultP2PService.get_peers() called - returning empty list");
        debug!("Would return actual connected peers here");
        Ok(Vec::new())
    }
}
