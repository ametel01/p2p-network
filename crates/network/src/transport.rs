use std::fmt::Debug;
use std::net::SocketAddr;

use async_trait::async_trait;
use bytes::Bytes;
use core_crate::P2PError;
use tracing::{debug, info};

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

        // For now, we'll just return a dummy implementation
        // In a real implementation, we would create actual transport instances
        match transport_type {
            TransportType::Tcp => {
                debug!("Creating TCP transport");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
            TransportType::Udp => {
                debug!("Creating UDP transport");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
            TransportType::WebSocket => {
                debug!("Creating WebSocket transport");
                Box::new(DummyTransport {
                    transport_type,
                    bind_addr,
                })
            }
            TransportType::WebRTC => {
                debug!("Creating WebRTC transport");
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
}
