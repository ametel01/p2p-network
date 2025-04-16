use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use core_crate::{NetworkMessage, P2PError, PeerId};
use messages::{CitreaMessage, MessageEnvelope, PROTOCOL_VERSION};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::config::P2PServiceConfig;
use crate::discovery::{PeerDiscovery, PeerDiscoveryFactory};
use crate::handler::MessageHandler;
use crate::transport::{Transport, TransportFactory};

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
#[allow(dead_code)]
pub struct DefaultP2PService {
    local_peer_id: PeerId,
}

impl DefaultP2PService {
    pub fn new(local_peer_id: PeerId) -> Self {
        Self { local_peer_id }
    }
}

#[async_trait]
impl P2PService for DefaultP2PService {
    async fn start(&self) -> Result<(), P2PError> {
        info!("Starting default P2P service");
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("Stopping default P2P service");
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

/// Information about a peer connection
#[derive(Debug)]
#[allow(dead_code)]
struct PeerInfo {
    /// Unique identifier for the peer
    id: PeerId,

    /// Network address of the peer
    addr: SocketAddr,

    /// Last activity timestamp
    last_seen: u64,

    /// Whether this peer was discovered or directly connected
    discovered: bool,
}

/// A complete P2P service implementation.
///
/// This implementation provides actual networking capabilities
/// using the configured transport layer.
#[derive(Debug)]
#[allow(dead_code)]
pub struct RealP2PService {
    /// Our local peer ID
    local_id: PeerId,

    /// The network configuration
    config: P2PServiceConfig,

    /// The transport layer for network communication
    transport: Box<dyn Transport>,

    /// The peer discovery mechanism
    discovery: Box<dyn PeerDiscovery>,

    /// Known peers mapped by their ID
    peers: Arc<Mutex<HashMap<PeerId, PeerInfo>>>,

    /// Addresses mapped to peer IDs
    addr_to_id: Arc<Mutex<HashMap<SocketAddr, PeerId>>>,

    /// Message handler
    message_handler: Box<dyn MessageHandler>,

    /// Task handle for message processing loop
    message_task: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// Task handle for maintenance loop
    maintenance_task: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// Shutdown flag
    shutdown: Arc<Mutex<bool>>,

    /// Chain ID for this network
    chain_id: u64,
}

impl RealP2PService {
    /// Create a new P2P service
    pub fn new(local_id: PeerId, config: P2PServiceConfig, chain_id: u64) -> Self {
        info!(
            local_id = ?local_id,
            port = config.p2p_config.port,
            chain_id = chain_id,
            "Creating new P2P service"
        );

        // Create the bind address from config
        let bind_addr = format!("0.0.0.0:{}", config.p2p_config.port)
            .parse()
            .expect("Invalid bind address");

        // Create transport
        let transport = TransportFactory::create(config.transport_type, bind_addr);

        // Create discovery mechanism
        let discovery = PeerDiscoveryFactory::create();

        Self {
            local_id,
            config: config.clone(),
            transport,
            discovery,
            peers: Arc::new(Mutex::new(HashMap::new())),
            addr_to_id: Arc::new(Mutex::new(HashMap::new())),
            message_handler: config.message_handler,
            message_task: Arc::new(Mutex::new(None)),
            maintenance_task: Arc::new(Mutex::new(None)),
            shutdown: Arc::new(Mutex::new(false)),
            chain_id,
        }
    }

    #[allow(dead_code)]
    /// Handle an incoming message
    async fn handle_incoming_message(&self, addr: SocketAddr, data: Bytes) -> Result<(), P2PError> {
        debug!(
            from_addr = %addr,
            data_len = data.len(),
            "Handling incoming message"
        );

        // Deserialize the message envelope
        let envelope = match MessageEnvelope::deserialize(data) {
            Ok(env) => env,
            Err(e) => {
                error!(
                    from_addr = %addr,
                    error = ?e,
                    "Failed to deserialize message"
                );
                return Err(e);
            }
        };

        // Check protocol version
        if envelope.version != PROTOCOL_VERSION {
            warn!(
                from_addr = %addr,
                message_version = envelope.version,
                our_version = PROTOCOL_VERSION,
                "Received message with incompatible protocol version"
            );
            return Err(P2PError::InvalidProtocolVersion(envelope.version as u32));
        }

        // Check chain ID
        if envelope.chain_id != self.chain_id {
            warn!(
                from_addr = %addr,
                message_chain = envelope.chain_id,
                our_chain = self.chain_id,
                "Received message for wrong chain"
            );
            return Err(P2PError::ProtocolError(format!(
                "Message for wrong chain: got {} expected {}",
                envelope.chain_id, self.chain_id
            )));
        }

        // Get sender ID (either from envelope or map from address)
        let peer_id = if let Some(sender) = envelope.sender {
            // Update our mapping
            let mut addr_to_id = self.addr_to_id.lock().unwrap();
            addr_to_id.insert(addr, sender);

            // Update peer info
            let mut peers = self.peers.lock().unwrap();
            if let Some(info) = peers.get_mut(&sender) {
                info.last_seen = envelope.timestamp;
            } else {
                // This is a new peer
                peers.insert(
                    sender,
                    PeerInfo {
                        id: sender,
                        addr,
                        last_seen: envelope.timestamp,
                        discovered: false,
                    },
                );
            }

            sender
        } else {
            // Try to find the peer ID from address
            let addr_to_id = self.addr_to_id.lock().unwrap();
            if let Some(&id) = addr_to_id.get(&addr) {
                id
            } else {
                warn!(
                    from_addr = %addr,
                    "Received message without sender ID and no known mapping"
                );
                return Err(P2PError::PeerNotFound(PeerId::random())); // Using random as placeholder
            }
        };

        // Handle the message with the configured handler
        self.message_handler
            .handle_message(peer_id, envelope.payload)
            .await
    }

    /// Start the message processing loop
    fn start_message_processor(&self, mut rx: Receiver<(SocketAddr, Bytes)>) -> JoinHandle<()> {
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            info!("Message processor started");

            while let Some((addr, data)) = rx.recv().await {
                // Check if we should shut down
                if *shutdown.lock().unwrap() {
                    info!("Message processor shutting down");
                    break;
                }

                // TODO: Handle the message
                debug!(
                    from_addr = %addr,
                    data_len = data.len(),
                    "Received message"
                );

                // Note: In a full implementation, we would process the message here
            }

            info!("Message processor terminated");
        })
    }

    /// Start the maintenance task
    fn start_maintenance(&self) -> JoinHandle<()> {
        let shutdown = self.shutdown.clone();
        let peers = self.peers.clone();
        let config = self.config.p2p_config.clone();

        tokio::spawn(async move {
            info!("Maintenance task started");

            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config.ping_interval));

            loop {
                interval.tick().await;

                // Check if we should shut down - no need to clone a boolean which is Copy
                let should_shutdown = *shutdown.lock().unwrap();
                if should_shutdown {
                    info!("Maintenance task shutting down");
                    break;
                }

                // Check for timed-out peers
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut peers_to_remove = Vec::new();
                {
                    let peers_lock = peers.lock().unwrap();

                    for (id, info) in peers_lock.iter() {
                        let time_since_last_seen = now.saturating_sub(info.last_seen);

                        if time_since_last_seen > config.connection_timeout {
                            warn!(
                                peer_id = ?id,
                                addr = %info.addr,
                                time_since_last_seen,
                                "Peer connection timed out"
                            );
                            peers_to_remove.push(*id);
                        }
                    }
                }

                // Remove timed-out peers
                if !peers_to_remove.is_empty() {
                    let mut peers_lock = peers.lock().unwrap();
                    for id in peers_to_remove {
                        peers_lock.remove(&id);
                    }
                }
            }

            info!("Maintenance task terminated");
        })
    }
}

#[async_trait]
impl P2PService for RealP2PService {
    async fn start(&self) -> Result<(), P2PError> {
        info!("Starting P2P service");

        // Reset shutdown flag - scope the mutex lock
        {
            let mut shutdown = self.shutdown.lock().unwrap();
            *shutdown = false;
        }

        // Start the transport
        self.transport.start().await?;

        // Get the incoming message receiver
        let rx = if let Some(rx) = self.transport.incoming() {
            rx
        } else {
            error!("Transport doesn't provide an incoming message stream");
            return Err(P2PError::ProtocolError(
                "No incoming message stream".to_string(),
            ));
        };

        // Start the message processor
        let handle = self.start_message_processor(rx);

        // Store the handle - scope the mutex lock
        {
            let mut message_task = self.message_task.lock().unwrap();
            *message_task = Some(handle);
        }

        // Start peer discovery if enabled
        if self.config.enable_discovery {
            self.discovery.start().await?;
        }

        // Start maintenance task
        let handle = self.start_maintenance();

        // Store the maintenance handle - scope the mutex lock
        {
            let mut maintenance_task = self.maintenance_task.lock().unwrap();
            *maintenance_task = Some(handle);
        }

        // Connect to bootstrap nodes
        for addr_str in &self.config.p2p_config.bootstrap_nodes {
            match addr_str.parse::<SocketAddr>() {
                Ok(addr) => {
                    info!(address = %addr, "Connecting to bootstrap node");

                    // Send a simple hello message
                    // In a real implementation, we would implement a proper handshake
                    let hello = CitreaMessage::Custom(messages::Custom::new(
                        "hello".into(),
                        format!("Hello from peer {}", self.local_id).into_bytes(),
                    ));

                    let envelope = MessageEnvelope::new(hello, self.chain_id, Some(self.local_id));

                    let data = NetworkMessage::serialize(&envelope)?;

                    // Send the message
                    if let Err(e) = self.transport.send_to(addr, data).await {
                        warn!(
                            address = %addr,
                            error = ?e,
                            "Failed to connect to bootstrap node"
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        address = %addr_str,
                        error = %e,
                        "Invalid bootstrap node address"
                    );
                }
            }
        }

        info!("P2P service started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("Stopping P2P service");

        // Set shutdown flag
        {
            let mut shutdown = self.shutdown.lock().unwrap();
            *shutdown = true;
        }

        // Stop message processor
        {
            let mut message_task = self.message_task.lock().unwrap();
            if let Some(handle) = message_task.take() {
                handle.abort();
                debug!("Message processor aborted");
            }
        }

        // Stop maintenance task
        {
            let mut maintenance_task = self.maintenance_task.lock().unwrap();
            if let Some(handle) = maintenance_task.take() {
                handle.abort();
                debug!("Maintenance task aborted");
            }
        }

        // Stop discovery
        if self.config.enable_discovery {
            self.discovery.stop().await?;
        }

        // Stop transport
        self.transport.stop().await?;

        info!("P2P service stopped successfully");
        Ok(())
    }

    async fn send_message(&self, peer_id: PeerId, message: CitreaMessage) -> Result<(), P2PError> {
        debug!(
            peer_id = ?peer_id,
            message_type = message.message_type(),
            "Sending message to peer"
        );

        // Find the peer's address
        let addr = {
            let peers = self.peers.lock().unwrap();

            if let Some(info) = peers.get(&peer_id) {
                info.addr
            } else {
                return Err(P2PError::PeerNotFound(peer_id));
            }
        };

        // Create message envelope
        let envelope = MessageEnvelope::new(message, self.chain_id, Some(self.local_id));

        // Serialize
        let data = NetworkMessage::serialize(&envelope)?;

        // Send via transport
        self.transport.send_to(addr, data).await?;

        debug!(
            peer_id = ?peer_id,
            addr = %addr,
            "Message sent successfully"
        );

        Ok(())
    }

    async fn broadcast_message(&self, message: CitreaMessage) -> Result<(), P2PError> {
        // Get peer info first
        let peer_data = {
            let peers = self.peers.lock().unwrap();
            // Clone relevant data to avoid holding the lock across await points
            if peers.is_empty() {
                debug!("No peers to broadcast to");
                return Ok(());
            }

            // Collect peer addresses to avoid holding the lock during network operations
            peers
                .iter()
                .map(|(id, info)| (*id, info.addr))
                .collect::<Vec<(PeerId, SocketAddr)>>()
        };

        let peer_count = peer_data.len();

        info!(
            message_type = message.message_type(),
            peer_count, "Broadcasting message to all peers"
        );

        // Create message envelope
        let envelope = MessageEnvelope::new(message, self.chain_id, Some(self.local_id));

        // Serialize (do it once for all peers)
        let data = NetworkMessage::serialize(&envelope)?;

        // Send to each peer
        for (id, addr) in peer_data {
            debug!(
                peer_id = ?id,
                addr = %addr,
                "Sending broadcast message to peer"
            );

            if let Err(e) = self.transport.send_to(addr, data.clone()).await {
                warn!(
                    peer_id = ?id,
                    addr = %addr,
                    error = ?e,
                    "Failed to send broadcast message to peer"
                );
            }
        }

        info!(peer_count, "Broadcast complete");

        Ok(())
    }

    async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        // Scope the lock
        let peer_ids = {
            let peers = self.peers.lock().unwrap();
            peers.keys().cloned().collect::<Vec<_>>()
        };

        debug!(peer_count = peer_ids.len(), "Retrieved peer list");

        Ok(peer_ids)
    }
}
