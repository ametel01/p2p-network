# Citrea P2P Network

This repository contains the peer-to-peer networking layer for the Citrea L2 blockchain system. The P2P layer enables decentralized propagation of L2 blocks, sequencer commitments, batch proofs, light client proofs, and EVM transactions.

## Architecture

The codebase is organized into a modular workspace structure with the following components:

### Core Module (`crates/core`)

The core module provides fundamental types and traits used throughout the P2P system:

- **Primary Abstractions**: 
  - `PeerId`: Uniquely identifies peers in the network
  - `BitcoinTxid`: Represents Bitcoin transaction IDs
  - `StateRoot`: Represents the Merkle root of a state tree
  - `MerkleRoot`: Represents a Merkle root for transaction verification
  - `EcdsaSignature`: Represents secp256k1 signatures for message validation
  - `P2PError`: Comprehensive error handling system

- **Key Traits**:
  - `SignedMessage`: For messages requiring cryptographic signatures
  - `VerifiableMessage`: For messages that need content verification
  - `NetworkMessage`: For serialization/deserialization of messages

**Rationale**: The core module follows the principle of separation of concerns by isolating the fundamental types and interfaces from their implementations. This approach makes the system more maintainable and testable, allowing each component to evolve independently.

### Messages Module (`crates/messages`)

The messages module defines the structure and validation logic for messages exchanged over the network:

- **Message Types**:
  - `L2BlockMessage`: Contains L2 block headers and validation data
  - `SequencerCommitmentMessage`: Contains sequencer commitments with Bitcoin txids
  - `BatchProofMessage`: Contains batch proofs with Groth16 ZK proofs
  - `LightClientProofMessage`: Contains STARK proofs for light clients
  - `EVMTransactionMessage`: Contains EVM transaction payloads

**Rationale**: Each message type is designed as a serializable data structure with specific validation requirements. This ensures that message exchange follows a well-defined protocol, reducing the risk of data corruption or malicious behavior.

### Network Module (`crates/network`)

The network module provides the actual P2P communication layer:

- **Key Interfaces**:
  - `P2PService`: Primary interface for the P2P network
  - `MessageHandler`: Processes incoming messages
  - `PeerDiscovery`: Manages peer discovery and connection

- **Design Patterns**:
  - Builder pattern for configuring the P2P service
  - Dependency injection for message handlers

**Rationale**: The networking layer uses async traits and tokio for asynchronous communication, which is essential for handling multiple concurrent connections efficiently. The interfaces are designed to be implementation-agnostic, allowing for different underlying networking technologies.

### Sync Module (`crates/sync`)

The sync module handles blockchain synchronization:

- **Key Components**:
  - `BlockSyncService`: Manages the blockchain synchronization process
  - `BlockSyncPipeline`: Processes blocks in sequence
  - `PipelineState`: Tracks synchronization progress

**Rationale**: The sync module separates the block synchronization logic from the networking layer, allowing for more efficient block downloading and validation. The pipeline architecture ensures blocks are processed in the correct order.

## Design Principles

1. **Modularity**: Each crate has a specific responsibility, promoting separation of concerns.
2. **Interface-First Design**: Traits define the behavior, hiding implementation details.
3. **Type Safety**: Rich type system to prevent runtime errors through compile-time checking.
4. **Async First**: Built for asynchronous operation from the ground up.
5. **Builder Pattern**: For constructing complex objects with optional parameters.

## Future Improvements

- Implement concrete networking layer (likely using libp2p)
- Add metrics and monitoring
- Enhance security with additional validation layers
- Optimize block synchronization for large state transitions
- Implement gossip protocol optimizations
