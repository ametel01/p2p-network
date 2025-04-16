# Citrea P2P Network Node

This is the binary executable for running a Citrea P2P network node. It integrates the core, messages, network, and sync crates to provide a complete P2P node implementation.

## Features

- Configurable P2P network transport (TCP, UDP, WebSocket, WebRTC)
- Command-line and environment variable configuration
- Block synchronization
- Peer discovery
- Bootstrap node functionality

## Usage

### Building

```bash
cargo build --release
```

The binary will be located at `target/release/citrea-p2p`.

### Running

```bash
# Basic usage
./target/release/citrea-p2p

# With command line arguments
./target/release/citrea-p2p --port 30303 --transport tcp --enable-discovery

# As a bootstrap node
./target/release/citrea-p2p --bootstrap-node
```

### Configuration

The node can be configured through command-line arguments or environment variables. You can also use a `.env` file for configuration.

Copy the example configuration file:

```bash
cp crates/citrea-p2p/.env.example .env
```

Then edit the `.env` file to customize your configuration.

#### Available Options

| Option | ENV Variable | Default | Description |
|--------|--------------|---------|-------------|
| `--port` | `PORT` | 30303 | P2P port to listen on |
| `--transport` | `TRANSPORT` | tcp | Network transport type (tcp, udp, websocket, webrtc) |
| `--max-connections` | `MAX_CONNECTIONS` | 50 | Maximum number of connections |
| `--bootstrap-nodes` | `BOOTSTRAP_NODES` | none | Bootstrap nodes to connect to (comma-separated list) |
| `--enable-discovery` | `ENABLE_DISCOVERY` | false | Enable peer discovery |
| `--bootstrap-node` | `BOOTSTRAP_NODE` | false | Run as a bootstrap node |

## Example Network Setup

To set up a small test network on a local machine:

1. Start a bootstrap node:
   ```bash
   ./target/release/citrea-p2p --port 30303 --bootstrap-node
   ```

2. Start additional nodes that connect to the bootstrap node:
   ```bash
   ./target/release/citrea-p2p --port 30304 --bootstrap-nodes 127.0.0.1:30303
   ./target/release/citrea-p2p --port 30305 --bootstrap-nodes 127.0.0.1:30303
   ```

## Development

For development, you can run the node with logging enabled:

```bash
RUST_LOG=debug cargo run --bin citrea-p2p
``` 