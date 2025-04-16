// Sync module for blockchain synchronization
//
// This crate provides functionality for syncing blockchain data
// across the p2p network, including block synchronization and state management.

mod config;
mod pipeline;
mod service;
mod storage;
mod types;

// Re-export public API
pub use config::{BlockSyncConfig, BlockSyncServiceBuilder};
pub use pipeline::{BlockSyncPipeline, DefaultBlockSyncPipeline, StandardBlockPipeline};
pub use service::{BlockSyncService, DefaultBlockSyncService, NetworkedBlockSyncService};
pub use storage::{BlockStorage, BlockStorageFactory, InMemoryBlockStorage};
pub use types::PipelineState;
