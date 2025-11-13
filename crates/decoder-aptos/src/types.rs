//! Aptos-specific transaction types
//!
//! Aptos uses Move VM with BCS (Binary Canonical Serialization) encoding.
//! Transactions are account-based with a sequence number system.

/// Aptos account address (32 bytes)
pub type AccountAddress = [u8; 32];

/// Aptos transaction payload types
#[derive(Debug, Clone)]
pub enum TransactionPayload {
    /// Script payload (deprecated in Aptos)
    Script {
        code: Vec<u8>,
        type_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    },
    /// Entry function payload (most common)
    EntryFunction {
        module: ModuleId,
        function: String,
        type_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    },
    /// Multisig payload
    Multisig {
        multisig_address: AccountAddress,
        transaction_payload: Option<Box<TransactionPayload>>,
    },
}

/// Module identifier
#[derive(Debug, Clone)]
pub struct ModuleId {
    pub address: AccountAddress,
    pub name: String,
}

/// Type tag for Move type system
#[derive(Debug, Clone)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(StructTag),
}

/// Struct tag for Move structs
#[derive(Debug, Clone)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: String,
    pub name: String,
    pub type_params: Vec<TypeTag>,
}

/// Aptos signature types
#[derive(Debug, Clone)]
pub enum TransactionAuthenticator {
    /// Ed25519 single signature
    Ed25519 {
        public_key: [u8; 32],
        signature: [u8; 64],
    },
    /// Multi-Ed25519 signature
    MultiEd25519 {
        public_keys: Vec<[u8; 32]>,
        signatures: Vec<[u8; 64]>,
        bitmap: Vec<u8>,
    },
    /// Multi-agent signature
    MultiAgent {
        sender: Box<TransactionAuthenticator>,
        secondary_signer_addresses: Vec<AccountAddress>,
        secondary_signers: Vec<TransactionAuthenticator>,
    },
}

/// Raw Aptos transaction (before signing)
#[derive(Debug, Clone)]
pub struct RawTransaction {
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub payload: TransactionPayload,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
    pub chain_id: u8,
}

/// Signed Aptos transaction
#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub raw_txn: RawTransaction,
    pub authenticator: TransactionAuthenticator,
}

/// Complete Aptos transaction representation
#[derive(Debug, Clone)]
pub struct AptosTransaction {
    pub signed_txn: SignedTransaction,
    pub raw_bytes: Vec<u8>,
}

impl AptosTransaction {
    /// Get the sender address
    pub fn sender(&self) -> &AccountAddress {
        &self.signed_txn.raw_txn.sender
    }

    /// Get the sequence number
    pub fn sequence_number(&self) -> u64 {
        self.signed_txn.raw_txn.sequence_number
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u8 {
        self.signed_txn.raw_txn.chain_id
    }

    /// Get max gas amount
    pub fn max_gas_amount(&self) -> u64 {
        self.signed_txn.raw_txn.max_gas_amount
    }

    /// Get gas unit price
    pub fn gas_unit_price(&self) -> u64 {
        self.signed_txn.raw_txn.gas_unit_price
    }

    /// Check if transaction is an entry function call
    pub fn is_entry_function(&self) -> bool {
        matches!(
            self.signed_txn.raw_txn.payload,
            TransactionPayload::EntryFunction { .. }
        )
    }

    /// Get entry function details if applicable
    pub fn entry_function(&self) -> Option<(&ModuleId, &str)> {
        if let TransactionPayload::EntryFunction {
            module, function, ..
        } = &self.signed_txn.raw_txn.payload
        {
            Some((module, function.as_str()))
        } else {
            None
        }
    }

    /// Calculate transaction hash using SHA3-256
    ///
    /// Aptos uses SHA3-256 with a domain separator prefix
    pub fn hash(&self) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};

        // Aptos transaction hash prefix
        const TRANSACTION_HASH_PREFIX: &[u8] = b"APTOS::Transaction";

        let mut hasher = Sha3_256::new();
        hasher.update(TRANSACTION_HASH_PREFIX);
        hasher.update(&self.raw_bytes);

        hasher.finalize().to_vec()
    }
}

impl TransactionPayload {
    /// Get a human-readable description of the payload type
    pub fn payload_type(&self) -> &str {
        match self {
            TransactionPayload::Script { .. } => "Script",
            TransactionPayload::EntryFunction { .. } => "EntryFunction",
            TransactionPayload::Multisig { .. } => "Multisig",
        }
    }
}

impl std::fmt::Display for ModuleId {
    /// Format as "address::module_name"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{}::{}",
            universal_decoder_core::hex::encode(self.address),
            self.name
        )
    }
}

impl std::fmt::Display for StructTag {
    /// Format as "address::module::name<type_params>"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{}::{}::{}",
            universal_decoder_core::hex::encode(self.address),
            self.module,
            self.name
        )?;

        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for TypeTag {
    /// Convert to a human-readable string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeTag::Bool => write!(f, "bool"),
            TypeTag::U8 => write!(f, "u8"),
            TypeTag::U64 => write!(f, "u64"),
            TypeTag::U128 => write!(f, "u128"),
            TypeTag::Address => write!(f, "address"),
            TypeTag::Signer => write!(f, "signer"),
            TypeTag::Vector(inner) => write!(f, "vector<{}>", inner),
            TypeTag::Struct(s) => write!(f, "{}", s),
        }
    }
}
