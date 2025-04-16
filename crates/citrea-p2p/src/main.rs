use std::process::exit;

use core_crate::{init_default_logging, P2PConfig, PeerId};
use network::{P2PServiceBuilder, TransportType};
use structopt::StructOpt;
use sync::BlockSyncServiceBuilder;
use tokio::signal;
use tracing::{error, info};

#[derive(Debug, StructOpt)]
#[structopt(name = "citrea-p2p", about = "Citrea P2P Network Node")]
struct Opt {
    /// P2P port to listen on
    #[structopt(short, long, default_value = "30303", env = "PORT")]
    port: u16,

    /// Network transport type (tcp, udp, websocket, webrtc)
    #[structopt(short, long, default_value = "tcp", env = "TRANSPORT")]
    transport: String,

    /// Maximum number of connections
    #[structopt(long, default_value = "50", env = "MAX_CONNECTIONS")]
    max_connections: usize,

    /// Bootstrap nodes to connect to (comma separated list of addresses)
    #[structopt(long, env = "BOOTSTRAP_NODES")]
    bootstrap_nodes: Option<String>,

    /// Enable peer discovery
    #[structopt(long, env = "ENABLE_DISCOVERY")]
    enable_discovery: bool,

    /// Run as a bootstrap node
    #[structopt(long, env = "BOOTSTRAP_NODE")]
    #[allow(dead_code)]
    bootstrap_node: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Initialize logging
    let _ = init_default_logging();

    // Parse command line arguments
    let opt = Opt::from_args();
    info!("Starting Citrea P2P Network Node with options: {:?}", opt);

    // Generate a random peer ID
    let peer_id = generate_peer_id();
    info!("Generated peer ID: {:?}", peer_id);

    // Create P2P configuration
    let p2p_config = create_p2p_config(&opt);

    // Create transport type from string
    let transport_type = match opt.transport.to_lowercase().as_str() {
        "tcp" => TransportType::Tcp,
        "udp" => TransportType::Udp,
        "websocket" => TransportType::WebSocket,
        "webrtc" => TransportType::WebRTC,
        _ => {
            error!("Invalid transport type: {}", opt.transport);
            exit(1);
        }
    };

    // Build P2P service
    let p2p_service = P2PServiceBuilder::new(p2p_config)
        .with_discovery(opt.enable_discovery)
        .with_transport(transport_type)
        .build_boxed();

    // Build block sync service
    let sync_service = BlockSyncServiceBuilder::new(p2p_service)
        .with_memory_storage()
        .with_start_block(0)
        .with_target_block(0)
        .build_boxed();

    // Create a new P2P service since the previous one was consumed
    let p2p_config = create_p2p_config(&opt);
    let p2p_service = P2PServiceBuilder::new(p2p_config)
        .with_discovery(opt.enable_discovery)
        .with_transport(transport_type)
        .build_boxed();

    // Start services
    info!("Starting P2P service...");
    p2p_service.start().await?;

    info!("Starting block sync service...");
    sync_service.start().await?;

    // Keep the program running until Ctrl+C is pressed
    info!("Services started. Press Ctrl+C to exit.");
    wait_for_shutdown().await;

    // Shutdown services
    info!("Shutting down services...");
    sync_service.stop().await?;
    p2p_service.stop().await?;

    info!("Shutdown complete. Goodbye!");
    Ok(())
}

/// Create P2P configuration from command line options
fn create_p2p_config(opt: &Opt) -> P2PConfig {
    let mut bootstrap_nodes = Vec::new();
    if let Some(nodes) = &opt.bootstrap_nodes {
        bootstrap_nodes = nodes
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    P2PConfig {
        port: opt.port,
        max_connections: opt.max_connections,
        bootstrap_nodes,
        enable_discovery: opt.enable_discovery,
    }
}

/// Generate a random peer ID
fn generate_peer_id() -> PeerId {
    // In a real implementation, this would generate a cryptographically secure ID
    // For now, we just use a simple random number
    let random_bytes: [u8; 32] = rand::random();
    PeerId::new(random_bytes)
}

/// Wait for Ctrl+C signal
async fn wait_for_shutdown() {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C, initiating shutdown...");
        }
        Err(err) => {
            error!("Error waiting for Ctrl+C: {}", err);
        }
    }
}
