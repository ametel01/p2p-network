use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::error::P2PError;
use crate::message::Message;
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
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            port: 30303,
            max_connections: 50,
            bootstrap_nodes: Vec::new(),
            enable_discovery: true,
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
        };

        tracing::debug!("P2P network initialized successfully");
        network
    }

    /// Connect to a peer
    pub fn connect(&self, peer_info: PeerInfo) -> Result<(), P2PError> {
        tracing::info!(
            peer_id = ?peer_info.id,
            address = %peer_info.address,
            "Attempting to connect to peer"
        );

        let mut peers = self.peers.lock().unwrap();

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
            return Err(P2PError::Network(anyhow::anyhow!(
                "Maximum connections reached"
            )));
        }

        // In a real implementation, we would establish a network connection here
        // For now, just add to our list of peers
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let connection = PeerConnection {
            info: peer_info.clone(),
            connected: true,
            last_activity: now,
        };

        peers.insert(peer_info.id, connection);

        // Also add to known peers
        let mut known = self.known_peers.lock().unwrap();
        known.insert(peer_info.clone());

        tracing::info!(
            peer_id = ?peer_info.id,
            address = %peer_info.address,
            "Successfully connected to peer"
        );

        Ok(())
    }

    /// Disconnect from a peer
    pub fn disconnect(&self, peer_id: PeerId) -> Result<(), P2PError> {
        tracing::info!(peer_id = ?peer_id, "Disconnecting from peer");

        let mut peers = self.peers.lock().unwrap();

        if let Some(connection) = peers.get_mut(&peer_id) {
            // In a real implementation, we would close the network connection here
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
    pub fn send_message(&self, peer_id: PeerId, message: &Message) -> Result<(), P2PError> {
        tracing::debug!(
            peer_id = ?peer_id,
            message_type = ?std::any::type_name::<Message>(),
            "Sending message to peer"
        );

        let peers = self.peers.lock().unwrap();

        if let Some(connection) = peers.get(&peer_id) {
            if !connection.connected {
                tracing::warn!(
                    peer_id = ?peer_id,
                    "Cannot send message to disconnected peer"
                );
                return Err(P2PError::Network(anyhow::anyhow!("Peer not connected")));
            }

            // In a real implementation, we would send the message over the network here
            // For now, just log that we would send it
            tracing::info!(
                peer_id = ?peer_id,
                address = %connection.info.address,
                message = ?message,
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
    pub fn broadcast(&self, message: &Message) -> Result<(), P2PError> {
        let peer_count = self.peers.lock().unwrap().len();
        tracing::info!(
            message_type = ?std::any::type_name::<Message>(),
            message = ?message,
            peer_count,
            "Broadcasting message to all connected peers"
        );

        let peers = self.peers.lock().unwrap();
        let connected_count = peers.values().filter(|conn| conn.connected).count();

        tracing::debug!(
            total_peers = peers.len(),
            connected_peers = connected_count,
            "Preparing broadcast"
        );

        for (peer_id, connection) in peers.iter() {
            if connection.connected {
                // In a real implementation, we would send the message over the network here
                tracing::debug!(
                    peer_id = ?peer_id,
                    address = %connection.info.address,
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
    pub fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.lock().unwrap();

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
    pub fn get_known_peers(&self) -> Vec<PeerInfo> {
        let known = self.known_peers.lock().unwrap();
        let peers = known.iter().cloned().collect::<Vec<_>>();

        tracing::debug!(known_peers_count = peers.len(), "Retrieved known peers");

        peers
    }
}
