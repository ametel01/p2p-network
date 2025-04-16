use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::P2PError;
use crate::message::{Message, MessageHandler, NetworkMessage, TimestampedMessage};
use crate::types::PeerId;

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    /// Peer ID
    pub id: PeerId,
    /// Network address (IP:port)
    pub address: String,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Protocol version
    pub version: u32,
    /// Capabilities/features supported by this peer
    pub capabilities: Vec<String>,
}

impl PeerInfo {
    /// Create a new peer info
    pub fn new(id: PeerId, address: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            address,
            last_seen: now,
            version: 1, // Default to version 1
            capabilities: vec![],
        }
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Configuration for the p2p network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// The port to listen on
    pub port: u16,

    /// Maximum number of connections
    pub max_connections: usize,

    /// Bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<String>,

    /// Whether to enable discovery
    pub enable_discovery: bool,

    /// How often to ping peers (in seconds)
    pub ping_interval: u64,

    /// How long to wait before considering a peer disconnected (in seconds)
    pub connection_timeout: u64,

    /// Whether to enable encryption for messages
    pub enable_encryption: bool,

    /// Protocol version to use
    pub protocol_version: u32,

    /// Supported capabilities/features
    pub capabilities: Vec<String>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            port: 30303,
            max_connections: 50,
            bootstrap_nodes: Vec::new(),
            enable_discovery: true,
            ping_interval: 30,
            connection_timeout: 120,
            enable_encryption: true,
            protocol_version: 1,
            capabilities: vec!["basic".to_string()],
        }
    }
}

/// Represents a connection to a peer
#[derive(Debug)]
pub struct PeerConnection {
    /// Peer information
    pub info: PeerInfo,
    /// Connection status
    pub connected: bool,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Number of failed attempts to connect/communicate
    pub failure_count: u32,
    /// Last ping nonce (for matching with pong)
    pub last_ping_nonce: Option<u64>,
    /// Latest ping round-trip time in milliseconds
    pub ping_rtt_ms: Option<u64>,
    /// Outbound connection (we initiated) or inbound (peer initiated)
    pub outbound: bool,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(info: PeerInfo, outbound: bool) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            info,
            connected: true,
            last_activity: now,
            failure_count: 0,
            last_ping_nonce: None,
            ping_rtt_ms: None,
            outbound,
        }
    }

    /// Update the last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Record a failure
    pub fn record_failure(&mut self) -> u32 {
        self.failure_count += 1;
        self.failure_count
    }

    /// Check if the connection has timed out
    pub fn is_timed_out(&self, timeout_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.last_activity > timeout_seconds
    }
}

/// Core network functionality for the P2P system
#[derive(Debug)]
pub struct Network {
    /// Our own peer ID
    pub local_id: PeerId,
    /// Network configuration
    pub config: P2PConfig,
    /// Connected peers
    peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
    /// Known peer addresses
    known_peers: Arc<Mutex<HashSet<PeerInfo>>>,
    /// Listener handle
    listener_handle: Option<JoinHandle<()>>,
    /// Maintenance handle (periodic tasks like ping)
    maintenance_handle: Option<JoinHandle<()>>,
    /// Shutdown flag
    shutdown: Arc<Mutex<bool>>,
    /// Running flag (is the network started)
    running: Arc<Mutex<bool>>,
}

impl Network {
    /// Create a new network
    pub fn new(local_id: PeerId, config: P2PConfig) -> Self {
        tracing::info!(
            local_id = ?local_id,
            port = config.port,
            max_connections = config.max_connections,
            "Creating new p2p network"
        );

        let network = Self {
            local_id,
            config,
            peers: Arc::new(Mutex::new(HashMap::new())),
            known_peers: Arc::new(Mutex::new(HashSet::new())),
            listener_handle: None,
            maintenance_handle: None,
            shutdown: Arc::new(Mutex::new(false)),
            running: Arc::new(Mutex::new(false)),
        };

        tracing::debug!("P2P network initialized successfully");
        network
    }

    /// Connect to a peer
    pub async fn connect(&self, peer_info: PeerInfo) -> Result<(), P2PError> {
        tracing::info!(
            peer_id = ?peer_info.id,
            address = %peer_info.address,
            "Attempting to connect to peer"
        );

        let mut peers = self.peers.lock().await;

        // Check if we're already connected
        if peers.contains_key(&peer_info.id) {
            tracing::warn!(peer_id = ?peer_info.id, "Already connected to peer");
            return Err(P2PError::AlreadyConnected(peer_info.id));
        }

        // Check if we've reached maximum connections
        if peers.len() >= self.config.max_connections {
            tracing::warn!(
                current = peers.len(),
                max = self.config.max_connections,
                "Maximum connections reached"
            );
            return Err(P2PError::MaxConnectionsReached);
        }

        // In a real implementation, we would establish a TCP connection here
        // For now, just add to our list of peers

        let connection = PeerConnection::new(peer_info.clone(), true);

        peers.insert(peer_info.id, connection);

        // Also add to known peers
        let mut known = self.known_peers.lock().await;
        known.insert(peer_info.clone());

        tracing::info!(
            peer_id = ?peer_info.id,
            address = %peer_info.address,
            "Successfully connected to peer"
        );

        Ok(())
    }

    /// Disconnect from a peer
    pub async fn disconnect(&self, peer_id: PeerId) -> Result<(), P2PError> {
        tracing::info!(peer_id = ?peer_id, "Disconnecting from peer");

        let mut peers = self.peers.lock().await;

        if let Some(connection) = peers.get_mut(&peer_id) {
            // In a real implementation, we would close the TCP connection here
            connection.connected = false;
            tracing::info!(
                peer_id = ?peer_id,
                address = %connection.info.address,
                "Peer disconnected successfully"
            );
            Ok(())
        } else {
            tracing::warn!(peer_id = ?peer_id, "Attempted to disconnect from unknown peer");
            Err(P2PError::PeerNotFound(peer_id))
        }
    }

    /// Send a message to a peer
    pub async fn send_message(&self, peer_id: PeerId, message: &Message) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            message_type = ?std::any::type_name::<Message>(),
            "Sending message to peer"
        );

        let peers = self.peers.lock().await;

        if let Some(connection) = peers.get(&peer_id) {
            if !connection.connected {
                tracing::warn!(
                    peer_id = ?peer_id,
                    "Cannot send message to disconnected peer"
                );
                return Err(P2PError::Network(anyhow::anyhow!("Peer not connected")));
            }

            // Create a timestamped message
            let timestamped = TimestampedMessage::new(message.clone(), self.local_id);

            // Serialize the message
            let bytes = NetworkMessage::serialize(&timestamped)?;

            // In a real implementation, we would send the bytes over TCP here
            // For now, just log that we would send it
            tracing::info!(
                peer_id = ?peer_id,
                address = %connection.info.address,
                message = ?message,
                message_size = bytes.len(),
                "Message sent to peer"
            );

            Ok(())
        } else {
            tracing::warn!(
                peer_id = ?peer_id,
                "Attempted to send message to unknown peer"
            );
            Err(P2PError::PeerNotFound(peer_id))
        }
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast(&self, message: &Message) -> Result<(), P2PError> {
        let peers = self.peers.lock().await;
        let peer_count = peers.len();

        tracing::info!(
            message_type = ?std::any::type_name::<Message>(),
            message = ?message,
            peer_count,
            "Broadcasting message to all connected peers"
        );

        let connected_count = peers.values().filter(|conn| conn.connected).count();

        tracing::debug!(
            total_peers = peers.len(),
            connected_peers = connected_count,
            "Preparing broadcast"
        );

        // Create a timestamped message once for all peers
        let timestamped = TimestampedMessage::new(message.clone(), self.local_id);
        let bytes = NetworkMessage::serialize(&timestamped)?;

        for (peer_id, connection) in peers.iter() {
            if connection.connected {
                // In a real implementation, we would send the message over TCP here
                tracing::debug!(
                    peer_id = ?peer_id,
                    address = %connection.info.address,
                    message_size = bytes.len(),
                    "Broadcasting message to peer"
                );
            }
        }

        tracing::info!(
            connected_peers = connected_count,
            "Message broadcast complete"
        );

        Ok(())
    }

    /// Get a list of all connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.lock().await;

        let connected = peers
            .values()
            .filter(|conn| conn.connected)
            .map(|conn| conn.info.clone())
            .collect::<Vec<_>>();

        tracing::debug!(
            total_peers = peers.len(),
            connected_count = connected.len(),
            "Retrieved connected peers"
        );

        connected
    }

    /// Get all known peers
    pub async fn get_known_peers(&self) -> Vec<PeerInfo> {
        let known = self.known_peers.lock().await;
        let peers = known.iter().cloned().collect::<Vec<_>>();

        tracing::debug!(known_peers_count = peers.len(), "Retrieved known peers");

        peers
    }

    /// Start the network
    pub async fn start(&mut self) -> Result<(), P2PError> {
        tracing::info!(
            local_id = ?self.local_id,
            port = self.config.port,
            "Starting p2p network"
        );

        // Check if already running
        let mut running = self.running.lock().await;
        if *running {
            tracing::warn!("Network already started");
            return Ok(());
        }

        // Reset shutdown flag
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = false;
        drop(shutdown);

        // Mark as running
        *running = true;
        drop(running);

        // Set up the network listener
        let listener_handle = self.start_listener()?;
        self.listener_handle = Some(listener_handle);

        // Set up maintenance tasks
        let maintenance_handle = self.start_maintenance()?;
        self.maintenance_handle = Some(maintenance_handle);

        // Connect to bootstrap nodes
        self.connect_to_bootstrap_nodes().await?;

        tracing::info!("P2P network started successfully");
        Ok(())
    }

    /// Stop the network
    pub async fn stop(&mut self) -> Result<(), P2PError> {
        tracing::info!("Stopping p2p network");

        // Check if not running
        let mut running = self.running.lock().await;
        if !*running {
            tracing::warn!("Network not started");
            return Ok(());
        }

        // Set shutdown flag
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = true;
        drop(shutdown);

        // Wait for tasks to complete
        if let Some(handle) = self.listener_handle.take() {
            handle.abort();
            tracing::debug!("Listener task aborted");
        }

        if let Some(handle) = self.maintenance_handle.take() {
            handle.abort();
            tracing::debug!("Maintenance task aborted");
        }

        // Disconnect all peers
        let peers = self.peers.lock().await;
        let peer_ids: Vec<PeerId> = peers.keys().cloned().collect();
        drop(peers);

        for peer_id in peer_ids {
            if let Err(e) = self.disconnect(peer_id).await {
                tracing::warn!(
                    peer_id = ?peer_id,
                    error = ?e,
                    "Failed to disconnect from peer during shutdown"
                );
            }
        }

        // Mark as not running
        *running = false;

        tracing::info!("P2P network stopped successfully");
        Ok(())
    }

    /// Start the TCP listener
    fn start_listener(&self) -> Result<JoinHandle<()>, P2PError> {
        let port = self.config.port;
        let peers = self.peers.clone();
        let known_peers = self.known_peers.clone();
        let shutdown = self.shutdown.clone();
        let local_id = self.local_id;
        let config = self.config.clone();

        // Start listener in a background task
        let handle = tokio::spawn(async move {
            tracing::info!(port, "Starting TCP listener");

            let addr = format!("0.0.0.0:{}", port);
            let listener = match TcpListener::bind(&addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    tracing::error!(
                        error = ?e,
                        address = %addr,
                        "Failed to bind TCP listener"
                    );
                    return;
                }
            };

            tracing::info!(address = %addr, "TCP listener bound successfully");

            loop {
                // Check shutdown flag
                if *shutdown.lock().await {
                    tracing::info!("Shutting down TCP listener");
                    break;
                }

                // Accept new connections with timeout
                let accept_future = listener.accept();
                let timeout_future = tokio::time::sleep(Duration::from_secs(1));

                let result = tokio::select! {
                    res = accept_future => Some(res),
                    _ = timeout_future => None,
                };

                if let Some(result) = result {
                    match result {
                        Ok((socket, addr)) => {
                            tracing::info!(
                                address = %addr,
                                "Accepted new connection"
                            );

                            // Check if we've reached maximum connections
                            let peer_count = peers.lock().await.len();
                            if peer_count >= config.max_connections {
                                tracing::warn!(
                                    address = %addr,
                                    current = peer_count,
                                    max = config.max_connections,
                                    "Maximum connections reached, rejecting connection"
                                );
                                continue;
                            }

                            // Handle connection in a new task
                            let peers_clone = peers.clone();
                            let known_peers_clone = known_peers.clone();
                            let local_id_clone = local_id;

                            tokio::spawn(async move {
                                // Here, you would implement the handshake protocol
                                // For now, just log the connection
                                tracing::info!(
                                    address = %addr,
                                    "Handling new inbound connection"
                                );

                                // Unused variables - will be used in full implementation
                                let _ = peers_clone;
                                let _ = known_peers_clone;
                                let _ = local_id_clone;
                                let _ = socket;

                                // In a real implementation, we would:
                                // 1. Perform handshake to get the peer's ID
                                // 2. Validate the handshake
                                // 3. Create a PeerConnection
                                // 4. Add to the peers map
                                // 5. Start reading messages from the socket
                            });
                        }
                        Err(e) => {
                            tracing::error!(
                                error = ?e,
                                "Failed to accept connection"
                            );
                        }
                    }
                }
            }

            tracing::info!("TCP listener stopped");
        });

        Ok(handle)
    }

    /// Start maintenance tasks like peer pinging
    fn start_maintenance(&self) -> Result<JoinHandle<()>, P2PError> {
        let peers = self.peers.clone();
        let shutdown = self.shutdown.clone();
        let ping_interval = self.config.ping_interval;
        let connection_timeout = self.config.connection_timeout;
        let local_id = self.local_id;

        // Start maintenance in a background task
        let handle = tokio::spawn(async move {
            tracing::info!("Starting maintenance tasks");

            loop {
                // Check shutdown flag
                if *shutdown.lock().await {
                    tracing::info!("Shutting down maintenance tasks");
                    break;
                }

                // Sleep for a while
                tokio::time::sleep(Duration::from_secs(ping_interval)).await;

                // Check peer timeouts and send pings
                let mut peers_to_ping = Vec::new();
                let mut peers_to_disconnect = Vec::new();

                {
                    let mut peers_lock = peers.lock().await;

                    for (id, connection) in peers_lock.iter_mut() {
                        if !connection.connected {
                            continue;
                        }

                        // Check if peer has timed out
                        if connection.is_timed_out(connection_timeout) {
                            tracing::warn!(
                                peer_id = ?id,
                                address = %connection.info.address,
                                last_seen = connection.last_activity,
                                timeout = connection_timeout,
                                "Peer connection timed out"
                            );

                            peers_to_disconnect.push(*id);
                            continue;
                        }

                        // If it's time to ping this peer
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        if now - connection.last_activity > ping_interval / 2 {
                            peers_to_ping.push((*id, connection.info.clone()));

                            // Generate a new ping nonce
                            let mut rng = rand::thread_rng();
                            connection.last_ping_nonce = Some(rng.gen());
                        }
                    }
                }

                // Disconnect timed-out peers
                for peer_id in peers_to_disconnect {
                    let mut peers_lock = peers.lock().await;
                    if let Some(connection) = peers_lock.get_mut(&peer_id) {
                        connection.connected = false;
                        tracing::info!(
                            peer_id = ?peer_id,
                            address = %connection.info.address,
                            "Disconnected timed-out peer"
                        );
                    }
                }

                // Send pings to peers
                for (peer_id, info) in peers_to_ping {
                    let nonce = {
                        let peers_lock = peers.lock().await;
                        if let Some(connection) = peers_lock.get(&peer_id) {
                            connection.last_ping_nonce
                        } else {
                            None
                        }
                    };

                    if let Some(nonce) = nonce {
                        // Create a ping message
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        let ping = Message::Ping {
                            nonce,
                            timestamp: now,
                        };

                        // Create a timestamped message
                        let _timestamped = TimestampedMessage::new(ping, local_id);

                        // In a real implementation, we would send this to the peer
                        tracing::debug!(
                            peer_id = ?peer_id,
                            address = %info.address,
                            nonce = nonce,
                            "Sending ping to peer"
                        );
                    }
                }
            }

            tracing::info!("Maintenance tasks stopped");
        });

        Ok(handle)
    }

    /// Connect to bootstrap nodes
    async fn connect_to_bootstrap_nodes(&self) -> Result<(), P2PError> {
        if self.config.bootstrap_nodes.is_empty() {
            tracing::info!("No bootstrap nodes configured");
            return Ok(());
        }

        tracing::info!(
            count = self.config.bootstrap_nodes.len(),
            "Connecting to bootstrap nodes"
        );

        for address in &self.config.bootstrap_nodes {
            // In a real implementation, we would:
            // 1. Establish a TCP connection to the address
            // 2. Perform handshake to get the peer's ID
            // 3. Create a PeerInfo
            // 4. Call self.connect with the PeerInfo

            tracing::info!(
                address = %address,
                "Would connect to bootstrap node"
            );

            // For now, just simulate a PeerInfo
            let peer_id = PeerId::random();
            let peer_info = PeerInfo::new(peer_id, address.clone());

            match self.connect(peer_info).await {
                Ok(_) => {
                    tracing::info!(
                        address = %address,
                        "Successfully connected to bootstrap node"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        address = %address,
                        error = ?e,
                        "Failed to connect to bootstrap node"
                    );
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl MessageHandler for Network {
    async fn handle_message(&self, peer_id: PeerId, message: Message) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            message_type = ?std::any::type_name::<Message>(),
            "Handling incoming message"
        );

        // Update the peer's last activity timestamp
        {
            let mut peers = self.peers.lock().await;
            if let Some(connection) = peers.get_mut(&peer_id) {
                connection.update_activity();
            }
        }

        // Process message based on type
        match message {
            Message::Ping { nonce, timestamp } => self.handle_ping(peer_id, nonce, timestamp).await,
            Message::Pong { nonce, timestamp } => self.handle_pong(peer_id, nonce, timestamp).await,
            Message::DiscoverPeers => self.handle_discover_peers(peer_id).await,
            Message::PeerList { peers } => self.handle_peer_list(peer_id, peers).await,
            Message::Custom { data } => self.handle_custom(peer_id, data).await,
            Message::DirectMessage { target, data } => {
                self.handle_direct_message(peer_id, target, data).await
            }
            Message::Handshake {
                version,
                peer_id: sender_id,
                listen_port,
                capabilities,
            } => {
                self.handle_handshake(peer_id, version, sender_id, listen_port, capabilities)
                    .await
            }
            Message::Gossip { topic, data, ttl } => {
                self.handle_gossip(peer_id, topic, data, ttl).await
            }
            Message::Subscribe { topic } => self.handle_subscribe(peer_id, topic).await,
            Message::Unsubscribe { topic } => self.handle_unsubscribe(peer_id, topic).await,
        }
    }

    async fn handle_ping(
        &self,
        peer_id: PeerId,
        nonce: u64,
        timestamp: u64,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            nonce = nonce,
            timestamp = timestamp,
            "Handling ping message"
        );

        // Create a pong response
        let pong = Message::Pong {
            nonce,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Send the pong response back to the peer
        self.send_message(peer_id, &pong).await
    }

    async fn handle_pong(
        &self,
        peer_id: PeerId,
        nonce: u64,
        timestamp: u64,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            nonce = nonce,
            timestamp = timestamp,
            "Handling pong message"
        );

        // Check if this is a response to our ping
        let mut peers = self.peers.lock().await;
        if let Some(connection) = peers.get_mut(&peer_id) {
            if let Some(ping_nonce) = connection.last_ping_nonce {
                if ping_nonce == nonce {
                    // Calculate round-trip time
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    // Very rough calculation, assumes timestamps are synchronized
                    let rtt_ms = (now - timestamp).saturating_mul(1000);

                    connection.ping_rtt_ms = Some(rtt_ms);
                    connection.last_ping_nonce = None;

                    tracing::debug!(
                        peer_id = ?peer_id,
                        rtt_ms = rtt_ms,
                        "Pong matches previous ping, updated RTT"
                    );
                } else {
                    tracing::warn!(
                        peer_id = ?peer_id,
                        received_nonce = nonce,
                        expected_nonce = ping_nonce,
                        "Received pong with unexpected nonce"
                    );
                }
            } else {
                tracing::debug!(
                    peer_id = ?peer_id,
                    "Received pong but no ping was sent"
                );
            }
        }

        Ok(())
    }

    async fn handle_discover_peers(&self, peer_id: PeerId) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            "Handling discover peers request"
        );

        // Get the list of connected peers
        let peers = self.get_connected_peers().await;

        // Create a peer list message
        let peer_list = Message::PeerList { peers };

        // Send the peer list back to the requesting peer
        self.send_message(peer_id, &peer_list).await
    }

    async fn handle_peer_list(
        &self,
        peer_id: PeerId,
        peers: Vec<PeerInfo>,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            peer_count = peers.len(),
            "Handling peer list message"
        );

        // Add the received peers to our known peers
        let mut known = self.known_peers.lock().await;
        let mut new_count = 0;

        for peer in peers {
            if !known.contains(&peer) && peer.id != self.local_id {
                known.insert(peer);
                new_count += 1;
            }
        }

        tracing::debug!(
            peer_id = ?peer_id,
            new_peers = new_count,
            total_known = known.len(),
            "Added new peers to known peers list"
        );

        Ok(())
    }

    async fn handle_custom(&self, peer_id: PeerId, data: Vec<u8>) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            data_size = data.len(),
            "Handling custom message"
        );

        // In a real implementation, this would be handled by the application layer
        if let Ok(text) = String::from_utf8(data.clone()) {
            tracing::info!(
                peer_id = ?peer_id,
                text = %text,
                "Received text message"
            );
        } else {
            tracing::info!(
                peer_id = ?peer_id,
                data_size = data.len(),
                "Received binary message"
            );
        }

        Ok(())
    }

    async fn handle_direct_message(
        &self,
        peer_id: PeerId,
        target: PeerId,
        data: Vec<u8>,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            from_peer = ?peer_id,
            to_peer = ?target,
            data_size = data.len(),
            "Handling direct message"
        );

        // Check if we are the target
        if target == self.local_id {
            tracing::info!(
                from_peer = ?peer_id,
                "Received direct message for us"
            );

            // Process the message as a custom message
            self.handle_custom(peer_id, data).await
        } else {
            // Forward the message to the target
            tracing::debug!(
                from_peer = ?peer_id,
                to_peer = ?target,
                "Forwarding direct message"
            );

            let message = Message::DirectMessage { target, data };

            self.send_message(target, &message).await
        }
    }

    async fn handle_handshake(
        &self,
        peer_id: PeerId,
        version: u32,
        sender_id: PeerId,
        listen_port: u16,
        capabilities: Vec<String>,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            sender_id = ?sender_id,
            version = version,
            port = listen_port,
            capabilities = ?capabilities,
            "Handling handshake message"
        );

        // Verify protocol version
        if version != self.config.protocol_version {
            tracing::warn!(
                peer_id = ?peer_id,
                their_version = version,
                our_version = self.config.protocol_version,
                "Protocol version mismatch"
            );

            // In a real implementation, we might disconnect or negotiate version
        }

        // Update peer ID and info
        let mut peers = self.peers.lock().await;

        if let Some(connection) = peers.get_mut(&peer_id) {
            // Update the peer info with the real peer ID and capabilities
            let address = connection.info.address.clone();
            let info = PeerInfo {
                id: sender_id,
                address,
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                version,
                capabilities: capabilities.clone(),
            };

            // If the peer ID changed, we need to update the map
            if peer_id != sender_id {
                let mut new_connection = PeerConnection::new(info.clone(), connection.outbound);
                new_connection.last_activity = connection.last_activity;
                new_connection.connected = connection.connected;

                // Replace in the map
                peers.remove(&peer_id);
                peers.insert(sender_id, new_connection);

                tracing::info!(
                    old_peer_id = ?peer_id,
                    new_peer_id = ?sender_id,
                    "Updated peer ID after handshake"
                );
            } else {
                // Just update the existing connection
                connection.info = info.clone();
                tracing::info!(
                    peer_id = ?peer_id,
                    "Updated peer info after handshake"
                );
            }

            // Add to known peers
            let mut known = self.known_peers.lock().await;
            known.insert(info);
        }

        // Send our handshake response if this was an inbound connection
        let peers_lock = peers;
        if let Some(connection) = peers_lock.get(&sender_id) {
            if !connection.outbound {
                // Create a handshake response
                let handshake = Message::Handshake {
                    version: self.config.protocol_version,
                    peer_id: self.local_id,
                    listen_port: self.config.port,
                    capabilities: self.config.capabilities.clone(),
                };

                // Send the handshake response
                drop(peers_lock);
                self.send_message(sender_id, &handshake).await?;
            }
        }

        Ok(())
    }

    async fn handle_gossip(
        &self,
        peer_id: PeerId,
        topic: String,
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            topic = %topic,
            data_size = data.len(),
            ttl = ttl,
            "Handling gossip message"
        );

        // Process the message locally
        tracing::info!(
            peer_id = ?peer_id,
            topic = %topic,
            "Received gossip message"
        );

        // If TTL > 0, forward to other peers
        if ttl > 0 {
            // Decrement TTL
            let new_ttl = ttl - 1;

            // Create a new gossip message
            let gossip = Message::Gossip {
                topic,
                data,
                ttl: new_ttl,
            };

            // Get connected peers
            let peers = self.peers.lock().await;
            let peers_to_forward: Vec<PeerId> = peers
                .iter()
                .filter(|(id, conn)| **id != peer_id && conn.connected)
                .map(|(id, _)| *id)
                .collect();

            drop(peers);

            // Forward to other peers
            for forward_peer_id in peers_to_forward {
                if let Err(e) = self.send_message(forward_peer_id, &gossip).await {
                    tracing::warn!(
                        peer_id = ?forward_peer_id,
                        error = ?e,
                        "Failed to forward gossip message"
                    );
                }
            }
        }

        Ok(())
    }

    async fn handle_subscribe(&self, peer_id: PeerId, topic: String) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            topic = %topic,
            "Handling subscribe message"
        );

        // In a real implementation, we would track subscriptions
        tracing::info!(
            peer_id = ?peer_id,
            topic = %topic,
            "Peer subscribed to topic"
        );

        Ok(())
    }

    async fn handle_unsubscribe(&self, peer_id: PeerId, topic: String) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            topic = %topic,
            "Handling unsubscribe message"
        );

        // In a real implementation, we would track subscriptions
        tracing::info!(
            peer_id = ?peer_id,
            topic = %topic,
            "Peer unsubscribed from topic"
        );

        Ok(())
    }
}
