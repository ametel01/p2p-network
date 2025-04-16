use std::fmt::Debug;

use async_trait::async_trait;
use core_crate::{P2PError, PeerId};
use messages::CitreaMessage;
use tracing::{debug, info, warn};

/// A trait for handling incoming messages in the P2P network.
///
/// Implementors of this trait will receive messages from the network
/// and can process them according to the application's business logic.
#[async_trait]
pub trait MessageHandler: Debug + Send + Sync {
    /// Handle an incoming message from a peer
    async fn handle_message(&self, peer_id: PeerId, message: CitreaMessage)
        -> Result<(), P2PError>;
}

/// A default message handler that logs received messages but doesn't process them.
///
/// This is primarily used as a placeholder and for testing. Real applications
/// should implement their own message handlers with proper business logic.
#[derive(Debug, Clone)]
pub struct DefaultMessageHandler;

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle_message(
        &self,
        peer_id: PeerId,
        message: CitreaMessage,
    ) -> Result<(), P2PError> {
        // Simply log the message receipt but don't do any processing
        info!(
            peer_id = ?peer_id,
            message_type = message.message_type(),
            "Received message, but no handler is configured"
        );

        debug!("Message content: {:?}", message);

        warn!("Using DefaultMessageHandler - this won't process messages");

        Ok(())
    }
}
