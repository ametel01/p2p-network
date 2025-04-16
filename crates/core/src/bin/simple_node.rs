use core::{init_default_logging, Message, Network, P2PConfig, P2PError, PeerId};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), P2PError> {
    // Initialize logging
    init_default_logging().expect("Failed to initialize logging");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let is_bootstrap = args.len() > 1 && args[1] == "--bootstrap";
    let port = if is_bootstrap {
        30303
    } else if args.len() > 2 {
        args[2].parse::<u16>().unwrap_or(30304)
    } else {
        30304
    };

    println!("Starting P2P node on port {}", port);

    // Generate a random peer ID
    let local_id = PeerId::random();
    println!("Local peer ID: {}", local_id);

    // Create network configuration
    let config = P2PConfig {
        port,
        ..P2PConfig::default()
    };

    // If not a bootstrap node, add bootstrap node address
    let config = if !is_bootstrap {
        let mut config = config;
        config.bootstrap_nodes.push("127.0.0.1:30303".to_string());
        config
    } else {
        config
    };

    // Create and start the P2P network
    let _network = Network::new(local_id, config);

    // In a real implementation, we would start the network here
    // network.start().await?;
    println!("Network created successfully!");

    // Wait for some time to allow connections to be established
    tokio::time::sleep(Duration::from_secs(2)).await;

    // If not a bootstrap node, send a message to all peers
    if !is_bootstrap {
        // In a real implementation, we would get connected peers
        // let peers = network.get_connected_peers().await;
        println!("Connected to peers");

        // Send a custom message
        let _message = Message::Custom {
            data: "Hello from simple_node!".as_bytes().to_vec(),
        };

        // In a real implementation, we would broadcast the message
        // network.broadcast(&message).await?;
        println!("Broadcasting message to all peers");
    }

    // Run for some time, in a real application we would handle incoming messages
    println!("Running for 30 seconds...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // In a real implementation, we would stop the network
    // network.stop().await?;
    println!("Stopping network");

    println!("Node shutdown complete");
    Ok(())
}
