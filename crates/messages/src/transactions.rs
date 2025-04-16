use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// Message containing EVM transaction data.
///
/// This message type is used to propagate Ethereum Virtual Machine (EVM)
/// transactions through the Citrea network before they're included in a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EVMTransactionMessage {
    /// The raw transaction payload (RLP encoded EVM transaction)
    pub payload: Vec<u8>,
}

impl EVMTransactionMessage {
    /// Create a new EVM transaction message
    pub fn new(payload: Vec<u8>) -> Self {
        debug!(
            payload_size = payload.len(),
            "Creating new EVMTransactionMessage"
        );

        Self { payload }
    }

    /// Verify basic transaction format
    ///
    /// Note: This is not implementing a trait since it's specific to EVM transactions.
    /// A more comprehensive implementation would decode and validate the transaction.
    pub fn verify_format(&self) -> bool {
        debug!(
            payload_size = self.payload.len(),
            "Verifying EVMTransactionMessage format"
        );

        // In a real implementation we would:
        // 1. Check that the transaction is valid RLP
        // 2. Check that the signature is valid
        // 3. Check that the transaction format matches expected EVM format

        // For now, just ensure we have some minimal data
        let valid = !self.payload.is_empty();

        if valid {
            debug!("EVMTransactionMessage format verification successful");
        } else {
            error!(
                payload_size = self.payload.len(),
                "EVMTransactionMessage format verification failed: empty payload"
            );
        }

        valid
    }

    /// Get the size of the transaction payload in bytes
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    /// Get a hex-encoded prefix of the transaction for logging purposes
    pub fn hex_prefix(&self) -> String {
        if self.payload.is_empty() {
            return "empty".to_string();
        }

        // Show first 8 bytes as hex, or all if there are fewer than 8
        let len = std::cmp::min(8, self.payload.len());
        let prefix: Vec<String> = self.payload[..len]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        format!("0x{}...", prefix.join(""))
    }
}
