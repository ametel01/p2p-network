use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Types of EVM transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EVMTransactionType {
    /// Legacy transaction (pre-EIP-2718)
    Legacy,
    /// EIP-2930 transaction with access list
    AccessList,
    /// EIP-1559 transaction with fee market
    FeeMarket,
}

impl EVMTransactionType {
    /// Get the transaction type identifier byte
    pub fn type_byte(&self) -> Option<u8> {
        match self {
            Self::Legacy => None,        // No type byte for legacy
            Self::AccessList => Some(1), // EIP-2930
            Self::FeeMarket => Some(2),  // EIP-1559
        }
    }

    /// Get transaction type from the first byte
    pub fn from_first_byte(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::Legacy;
        }

        // Check first byte
        match data[0] {
            // If first byte is < 0xc0, it's a typed transaction
            0x01 => Self::AccessList,
            0x02 => Self::FeeMarket,
            // Otherwise it's a legacy transaction
            _ => Self::Legacy,
        }
    }
}

/// Gas pricing information for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPricing {
    /// Gas limit for the transaction
    pub gas_limit: u64,

    /// Gas price (legacy and EIP-2930) or max fee per gas (EIP-1559)
    pub max_fee_per_gas: u64,

    /// Max priority fee per gas (EIP-1559 only)
    pub max_priority_fee_per_gas: Option<u64>,
}

/// Message containing EVM transaction data.
///
/// This message type is used to propagate Ethereum Virtual Machine (EVM)
/// transactions through the Citrea network before they're included in a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EVMTransactionMessage {
    /// The raw transaction payload (RLP encoded EVM transaction)
    pub payload: Vec<u8>,

    /// Transaction type
    pub tx_type: EVMTransactionType,

    /// From address (if extracted)
    pub from: Option<[u8; 20]>,

    /// To address (if extracted, None for contract creation)
    pub to: Option<[u8; 20]>,

    /// Gas pricing information
    pub gas_info: Option<GasPricing>,

    /// Transaction nonce
    pub nonce: Option<u64>,
}

impl EVMTransactionMessage {
    /// Create a new EVM transaction message from raw payload
    pub fn new(payload: Vec<u8>) -> Self {
        debug!(
            payload_size = payload.len(),
            "Creating new EVMTransactionMessage"
        );

        let tx_type = EVMTransactionType::from_first_byte(&payload);

        // In a real implementation, we would extract more metadata here
        // by decoding the RLP payload

        Self {
            payload,
            tx_type,
            from: None,
            to: None,
            gas_info: None,
            nonce: None,
        }
    }

    /// Create a builder for transaction message with metadata
    pub fn builder(payload: Vec<u8>) -> EVMTransactionMessageBuilder {
        EVMTransactionMessageBuilder::new(payload)
    }

    /// Verify basic transaction format
    ///
    /// Note: This is not implementing a trait since it's specific to EVM transactions.
    /// A more comprehensive implementation would decode and validate the transaction.
    pub fn verify_format(&self) -> bool {
        debug!(
            payload_size = self.payload.len(),
            tx_type = ?self.tx_type,
            "Verifying EVMTransactionMessage format"
        );

        // Basic payload verification
        if self.payload.is_empty() {
            error!("EVMTransactionMessage format verification failed: empty payload");
            return false;
        }

        // Verify transaction type consistency with payload
        match self.tx_type {
            EVMTransactionType::Legacy => {
                // Legacy transaction should start with RLP list indicator (0xc0-0xf7)
                if !self.payload.is_empty() && !(0xc0..=0xf7).contains(&self.payload[0]) {
                    error!("Invalid legacy transaction format");
                    return false;
                }
            }
            EVMTransactionType::AccessList => {
                // EIP-2930 transaction should start with 0x01
                if self.payload.is_empty() || self.payload[0] != 0x01 {
                    error!("Invalid EIP-2930 transaction format");
                    return false;
                }
            }
            EVMTransactionType::FeeMarket => {
                // EIP-1559 transaction should start with 0x02
                if self.payload.is_empty() || self.payload[0] != 0x02 {
                    error!("Invalid EIP-1559 transaction format");
                    return false;
                }
            }
        }

        debug!("EVMTransactionMessage format verification successful");
        true
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

/// Builder for creating EVM transaction messages with metadata
#[derive(Debug)]
pub struct EVMTransactionMessageBuilder {
    /// Raw transaction payload
    payload: Vec<u8>,
    /// Transaction type
    tx_type: EVMTransactionType,
    /// Sender address
    from: Option<[u8; 20]>,
    /// Recipient address
    to: Option<[u8; 20]>,
    /// Gas limit
    gas_limit: Option<u64>,
    /// Max fee per gas
    max_fee_per_gas: Option<u64>,
    /// Max priority fee per gas
    max_priority_fee_per_gas: Option<u64>,
    /// Transaction nonce
    nonce: Option<u64>,
}

impl EVMTransactionMessageBuilder {
    /// Create a new transaction builder
    pub fn new(payload: Vec<u8>) -> Self {
        let tx_type = EVMTransactionType::from_first_byte(&payload);

        Self {
            payload,
            tx_type,
            from: None,
            to: None,
            gas_limit: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
        }
    }

    /// Set the transaction type
    pub fn with_type(mut self, tx_type: EVMTransactionType) -> Self {
        self.tx_type = tx_type;
        self
    }

    /// Set the sender address
    pub fn with_from(mut self, from: [u8; 20]) -> Self {
        self.from = Some(from);
        self
    }

    /// Set the recipient address
    pub fn with_to(mut self, to: [u8; 20]) -> Self {
        self.to = Some(to);
        self
    }

    /// Set gas parameters
    pub fn with_gas(mut self, gas_limit: u64, max_fee_per_gas: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self.max_fee_per_gas = Some(max_fee_per_gas);
        self
    }

    /// Set priority fee (EIP-1559 only)
    pub fn with_priority_fee(mut self, max_priority_fee_per_gas: u64) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
        self
    }

    /// Set the transaction nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Build the transaction message
    pub fn build(self) -> EVMTransactionMessage {
        let gas_info = match (self.gas_limit, self.max_fee_per_gas) {
            (Some(gas_limit), Some(max_fee_per_gas)) => Some(GasPricing {
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            }),
            _ => None,
        };

        let nonce_str = self.nonce.map_or("None".to_string(), |n| n.to_string());

        info!(
            payload_size = self.payload.len(),
            tx_type = ?self.tx_type,
            nonce = nonce_str,
            "Building EVMTransactionMessage with metadata"
        );

        EVMTransactionMessage {
            payload: self.payload,
            tx_type: self.tx_type,
            from: self.from,
            to: self.to,
            gas_info,
            nonce: self.nonce,
        }
    }
}
