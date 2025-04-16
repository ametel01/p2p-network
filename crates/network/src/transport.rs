use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use core_crate::P2PError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Represents the type of network transport to use.
///
/// Different transport types provide different tradeoffs in terms of
/// reliability, latency, and implementation complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// TCP transport - reliable but higher latency
    Tcp,

    /// UDP transport - unreliable but lower latency
    Udp,

    /// WebSocket transport - for browser compatibility
    WebSocket,

    /// WebRTC transport - for peer-to-peer browser connections
    WebRTC,
}

/// Trait defining a network transport layer.
///
/// This abstracts over the specific transport protocol used (TCP, UDP, etc.)
/// and provides a consistent interface for sending and receiving data.
#[async_trait]
pub trait Transport: Debug + Send + Sync {
    /// Start the transport service
    async fn start(&self) -> Result<(), P2PError>;

    /// Stop the transport service
    async fn stop(&self) -> Result<(), P2PError>;

    /// Send data to a specific address
    async fn send_to(&self, addr: SocketAddr, data: Bytes) -> Result<(), P2PError>;

    /// Get the local address this transport is bound to
    fn local_addr(&self) -> Result<SocketAddr, P2PError>;

    /// Get a receiver for incoming messages (address, data)
    fn incoming(&self) -> Option<MessageReceiver>;
}

/// A factory for creating transport implementations.
///
/// This allows for creating different transport implementations based
/// on the configured TransportType.
pub struct TransportFactory;

impl TransportFactory {
    /// Create a new transport based on the specified type.
    pub fn create(transport_type: TransportType, bind_addr: SocketAddr) -> Box<dyn Transport> {
        info!(
            transport_type = ?transport_type,
            bind_addr = %bind_addr,
            "Creating transport"
        );

        match transport_type {
            TransportType::Tcp => {
                debug!("Creating TCP transport");
                Box::new(TcpTransport::new(bind_addr))
            }
            TransportType::Udp => {
                debug!("Creating UDP transport (dummy implementation)");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
            TransportType::WebSocket => {
                debug!("Creating WebSocket transport (dummy implementation)");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
            TransportType::WebRTC => {
                debug!("Creating WebRTC transport (dummy implementation)");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
        }
    }
}

/// A dummy transport that logs operations but doesn't actually send data.
///
/// This is used for testing and as a placeholder until real transports
/// are implemented.
#[derive(Debug)]
struct DummyTransport {
    transport_type: TransportType,
    bind_addr: SocketAddr,
}

#[async_trait]
impl Transport for DummyTransport {
    async fn start(&self) -> Result<(), P2PError> {
        info!(
            transport_type = ?self.transport_type,
            bind_addr = %self.bind_addr,
            "DummyTransport.start() called"
        );
        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!(
            transport_type = ?self.transport_type,
            "DummyTransport.stop() called"
        );
        Ok(())
    }

    async fn send_to(&self, addr: SocketAddr, data: Bytes) -> Result<(), P2PError> {
        info!(
            transport_type = ?self.transport_type,
            target_addr = %addr,
            data_len = data.len(),
            "DummyTransport.send_to() called"
        );

        debug!("Would send data: {:?}", data);

        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, P2PError> {
        Ok(self.bind_addr)
    }

    fn incoming(&self) -> Option<MessageReceiver> {
        None
    }
}

// Define type aliases for complex types
/// Type alias for Receiver of socket address and byte data
type MessageReceiver = Receiver<(SocketAddr, Bytes)>;
/// Type alias for Sender of socket address and byte data
type MessageSender = Sender<(SocketAddr, Bytes)>;

/// A real TCP-based transport implementation.
///
/// This implementation uses Tokio's TCP networking to provide reliable
/// communication between peers.
#[derive(Debug)]
pub struct TcpTransport {
    /// The address this transport is bound to
    bind_addr: SocketAddr,

    /// Active TCP connections to peers
    connections: Arc<Mutex<HashMap<SocketAddr, TcpConnection>>>,

    /// Listener task handle
    listener_handle: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// Shutdown flag
    shutdown: Arc<Mutex<bool>>,

    /// Receiver for incoming messages
    incoming_rx: Arc<Mutex<Option<MessageReceiver>>>,

    /// Sender for incoming messages
    incoming_tx: Arc<Mutex<Option<MessageSender>>>,
}

/// Information about a TCP connection to a peer
#[derive(Debug)]
struct TcpConnection {
    /// Stream for communication with the peer - use TokioMutex for async safety
    stream: Arc<TokioMutex<TcpStream>>,

    /// Task handle for the connection handler
    task: JoinHandle<()>,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new(bind_addr: SocketAddr) -> Self {
        // Create channel for incoming messages
        let (tx, rx) = mpsc::channel(100);

        Self {
            bind_addr,
            connections: Arc::new(Mutex::new(HashMap::new())),
            listener_handle: Arc::new(Mutex::new(None)),
            shutdown: Arc::new(Mutex::new(false)),
            incoming_rx: Arc::new(Mutex::new(Some(rx))),
            incoming_tx: Arc::new(Mutex::new(Some(tx))),
        }
    }

    /// Handle an incoming TCP connection
    async fn handle_incoming_connection(
        addr: SocketAddr,
        stream: Arc<TokioMutex<TcpStream>>,
        tx: MessageSender,
        shutdown: Arc<Mutex<bool>>,
    ) {
        info!(peer_addr = %addr, "Handling new TCP connection");

        // Buffer for reading data
        let mut buffer = vec![0u8; 65536]; // 64KB buffer

        loop {
            // Check if we should shut down - properly scope the lock and use dereference instead of clone
            let should_shutdown = {
                let shutdown_guard = shutdown.lock().unwrap();
                *shutdown_guard
            };

            if should_shutdown {
                info!(peer_addr = %addr, "Connection handler shutting down");
                break;
            }

            // Lock the stream for reading - Tokio's Mutex is safe across await points
            let mut stream_guard = stream.lock().await;

            // Read from the stream
            match stream_guard.read(&mut buffer).await {
                Ok(0) => {
                    // Connection closed
                    info!(peer_addr = %addr, "Peer disconnected");
                    break;
                }
                Ok(n) => {
                    // Got some data
                    debug!(peer_addr = %addr, bytes_read = n, "Received data");

                    // Drop the lock before sending data (to avoid holding across await)
                    drop(stream_guard);

                    // Extract the data we just read
                    let data = Bytes::copy_from_slice(&buffer[0..n]);

                    // Send the data to the processing pipeline
                    if let Err(e) = tx.send((addr, data)).await {
                        error!(error = %e, "Failed to send incoming message to processing pipeline");
                        break;
                    }
                }
                Err(e) => {
                    // Error reading from stream
                    error!(peer_addr = %addr, error = %e, "Error reading from TCP stream");
                    break;
                }
            }
        }

        info!(peer_addr = %addr, "Connection handler terminated");
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn start(&self) -> Result<(), P2PError> {
        info!(bind_addr = %self.bind_addr, "Starting TCP transport");

        // Create shutdown flag - properly scope the lock
        {
            let mut shutdown = self.shutdown.lock().unwrap();
            *shutdown = false;
        }

        // Start the TCP listener
        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(|e| P2PError::Network(e.into()))?;

        let actual_addr = listener
            .local_addr()
            .map_err(|e| P2PError::Network(e.into()))?;

        info!(local_addr = %actual_addr, "TCP listener bound successfully");

        // Get the message sender
        let tx = {
            let sender_guard = self.incoming_tx.lock().unwrap();
            sender_guard.clone().ok_or_else(|| {
                P2PError::ProtocolError("Message sender not available".to_string())
            })?
        };

        // Clone references for the listener task
        let connections = self.connections.clone();
        let shutdown_flag = self.shutdown.clone();

        // Create listener task
        let handle = tokio::spawn(async move {
            info!("TCP listener task started");

            loop {
                // Check if we should shut down
                let should_shutdown = {
                    let guard = shutdown_flag.lock().unwrap();
                    *guard
                };

                if should_shutdown {
                    info!("TCP listener shutting down");
                    break;
                }

                // Wait for a connection with timeout
                let accept_future = listener.accept();
                let timeout_future = tokio::time::sleep(tokio::time::Duration::from_millis(100));

                let result = tokio::select! {
                    result = accept_future => Some(result),
                    _ = timeout_future => None,
                };

                if let Some(result) = result {
                    match result {
                        Ok((stream, addr)) => {
                            info!(peer_addr = %addr, "Accepted new TCP connection");

                            // Wrap the stream in TokioMutex for async safety
                            let stream_arc = Arc::new(TokioMutex::new(stream));

                            // Setup handler for receiving from this connection
                            let shutdown_clone = shutdown_flag.clone();
                            let stream_clone = stream_arc.clone();
                            let tx_clone = tx.clone();

                            // Spawn a task to handle this connection
                            let handle = tokio::spawn(Self::handle_incoming_connection(
                                addr,
                                stream_clone,
                                tx_clone,
                                shutdown_clone,
                            ));

                            // Store the connection
                            {
                                let mut connections = connections.lock().unwrap();
                                connections.insert(
                                    addr,
                                    TcpConnection {
                                        stream: stream_arc.clone(),
                                        task: handle,
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Error accepting TCP connection");
                        }
                    }
                }
            }

            info!("TCP listener task terminated");
        });

        // Store the listener handle
        {
            let mut listener_handle = self.listener_handle.lock().unwrap();
            *listener_handle = Some(handle);
        }

        Ok(())
    }

    async fn stop(&self) -> Result<(), P2PError> {
        info!("Stopping TCP transport");

        // Set shutdown flag - properly scope the lock
        {
            let mut shutdown = self.shutdown.lock().unwrap();
            *shutdown = true;
        }

        // Wait for listener to terminate - scope the lock
        let listener_handle = {
            let mut guard = self.listener_handle.lock().unwrap();
            guard.take()
        };

        if let Some(handle) = listener_handle {
            handle.abort();
            debug!("TCP listener aborted");
        }

        // Close all connections - scope the lock and extract what we need
        let connections_to_abort = {
            let mut connections = self.connections.lock().unwrap();
            // Extract the address and handle pairs
            connections.drain().collect::<Vec<_>>()
        };

        // Now abort each connection outside the lock
        for (addr, conn) in connections_to_abort {
            conn.task.abort();
            debug!(peer_addr = %addr, "Connection handler aborted");
        }

        info!("TCP transport stopped");
        Ok(())
    }

    async fn send_to(&self, addr: SocketAddr, data: Bytes) -> Result<(), P2PError> {
        debug!(
            target_addr = %addr,
            data_len = data.len(),
            "Sending data over TCP"
        );

        // Get the connection or establish a new one
        let stream_arc = {
            // Scope the lock to ensure it's dropped before any await points
            let existing_connection = {
                let connections_guard = self.connections.lock().unwrap();
                connections_guard.get(&addr).map(|conn| conn.stream.clone())
                // Guard dropped here at end of scope
            };

            // Use existing connection if found
            if let Some(stream) = existing_connection {
                stream
            } else {
                // Create new connection
                info!(peer_addr = %addr, "Establishing new TCP connection for sending");

                let stream = TcpStream::connect(addr)
                    .await
                    .map_err(|e| P2PError::Network(e.into()))?;

                // Wrap in TokioMutex for async safety
                let stream_arc = Arc::new(TokioMutex::new(stream));

                // Get the sender for incoming messages
                let tx = {
                    let sender_guard = self.incoming_tx.lock().unwrap();
                    sender_guard.clone().ok_or_else(|| {
                        P2PError::ProtocolError("Message sender not available".to_string())
                    })?
                };

                // Setup handler for receiving from this connection
                let shutdown_clone = self.shutdown.clone();
                let stream_clone = stream_arc.clone();
                let tx_clone = tx.clone();

                // Spawn a task to handle this connection
                let handle = tokio::spawn(Self::handle_incoming_connection(
                    addr,
                    stream_clone,
                    tx_clone,
                    shutdown_clone,
                ));

                // Store the connection
                {
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(
                        addr,
                        TcpConnection {
                            stream: stream_arc.clone(),
                            task: handle,
                        },
                    );
                }

                stream_arc
            }
        };

        // Lock the stream for writing - Tokio's Mutex is safe across await points
        let mut stream_guard = stream_arc.lock().await;

        // Write data to the stream
        stream_guard
            .write_all(&data)
            .await
            .map_err(|e| P2PError::Network(e.into()))?;

        debug!(
            target_addr = %addr,
            data_len = data.len(),
            "Data sent successfully over TCP"
        );

        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, P2PError> {
        Ok(self.bind_addr)
    }

    fn incoming(&self) -> Option<MessageReceiver> {
        // Properly scope the lock to ensure it's dropped
        let rx = {
            let mut guard = self.incoming_rx.lock().unwrap();
            guard.take()
        };
        rx
    }
}
