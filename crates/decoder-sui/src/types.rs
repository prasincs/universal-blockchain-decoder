//! Sui-specific transaction types
//!
//! Sui uses Move VM with an object-centric model. Transactions are composed of
//! commands that operate on objects and can call Move functions.

/// Sui object ID (32 bytes, using BLAKE2b-256)
pub type ObjectID = [u8; 32];

/// Sui account address (32 bytes)
pub type SuiAddress = [u8; 32];

/// Object reference consisting of (object_id, version, digest)
#[derive(Debug, Clone)]
pub struct ObjectRef {
    pub object_id: ObjectID,
    pub version: u64,
    pub digest: [u8; 32],
}

/// Sui transaction command types
#[derive(Debug, Clone)]
pub enum Command {
    /// Transfer objects to an address
    TransferObjects {
        objects: Vec<Argument>,
        address: Argument,
    },
    /// Split coins
    SplitCoins {
        coin: Argument,
        amounts: Vec<Argument>,
    },
    /// Merge coins
    MergeCoins {
        destination: Argument,
        sources: Vec<Argument>,
    },
    /// Publish a Move package
    Publish {
        modules: Vec<Vec<u8>>,
        dependencies: Vec<ObjectID>,
    },
    /// Call a Move function
    MoveCall {
        package: ObjectID,
        module: String,
        function: String,
        type_arguments: Vec<TypeTag>,
        arguments: Vec<Argument>,
    },
    /// Make objects mutable/shared
    MakeMoveVec {
        type_tag: Option<TypeTag>,
        elements: Vec<Argument>,
    },
}

/// Argument to a transaction command
#[derive(Debug, Clone)]
pub enum Argument {
    /// Gas coin
    GasCoin,
    /// Input object at index
    Input(u16),
    /// Result from previous command
    Result(u16),
    /// Nested result from previous command
    NestedResult(u16, u16),
}

/// Type tag for Move type system (similar to Aptos but Sui-specific)
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
    U16,
    U32,
    U256,
}

/// Struct tag for Move structs
#[derive(Debug, Clone)]
pub struct StructTag {
    pub address: SuiAddress,
    pub module: String,
    pub name: String,
    pub type_params: Vec<TypeTag>,
}

/// Transaction input (objects or pure values)
#[derive(Debug, Clone)]
pub enum CallArg {
    /// Pure value (BCS-encoded)
    Pure(Vec<u8>),
    /// Object reference
    Object(ObjectArg),
}

/// Object argument types
#[derive(Debug, Clone)]
pub enum ObjectArg {
    /// Immutable or owned object
    ImmOrOwnedObject(ObjectRef),
    /// Shared object
    SharedObject {
        object_id: ObjectID,
        initial_shared_version: u64,
        mutable: bool,
    },
    /// Object received from another transaction
    Receiving(ObjectRef),
}

/// Transaction kind
#[derive(Debug, Clone)]
pub enum TransactionKind {
    /// Programmable transaction (most common in Sui)
    ProgrammableTransaction(ProgrammableTransaction),
    /// Change epoch (system transaction)
    ChangeEpoch {
        epoch: u64,
        storage_charge: u64,
        computation_charge: u64,
    },
    /// Genesis transaction
    Genesis { objects: Vec<ObjectID> },
}

/// Programmable transaction with commands
#[derive(Debug, Clone)]
pub struct ProgrammableTransaction {
    pub inputs: Vec<CallArg>,
    pub commands: Vec<Command>,
}

/// Transaction data (before signing)
#[derive(Debug, Clone)]
pub struct TransactionData {
    pub kind: TransactionKind,
    pub sender: SuiAddress,
    pub gas_data: GasData,
    pub expiration: TransactionExpiration,
}

/// Gas payment data
#[derive(Debug, Clone)]
pub struct GasData {
    pub payment: Vec<ObjectRef>,
    pub owner: SuiAddress,
    pub price: u64,
    pub budget: u64,
}

/// Transaction expiration
#[derive(Debug, Clone)]
pub enum TransactionExpiration {
    /// No expiration
    None,
    /// Expires at epoch
    Epoch(u64),
}

/// Sui signature schemes
#[derive(Debug, Clone)]
pub enum SuiSignature {
    Ed25519 {
        signature: [u8; 64],
        public_key: [u8; 32],
    },
    Secp256k1 {
        signature: Vec<u8>,
        public_key: Vec<u8>,
    },
    Secp256r1 {
        signature: Vec<u8>,
        public_key: Vec<u8>,
    },
}

/// Complete Sui transaction representation
#[derive(Debug, Clone)]
pub struct SuiTransaction {
    pub data: TransactionData,
    pub signatures: Vec<SuiSignature>,
    pub raw_bytes: Vec<u8>,
}

impl SuiTransaction {
    /// Get the sender address
    pub fn sender(&self) -> &SuiAddress {
        &self.data.sender
    }

    /// Get gas budget
    pub fn gas_budget(&self) -> u64 {
        self.data.gas_data.budget
    }

    /// Get gas price
    pub fn gas_price(&self) -> u64 {
        self.data.gas_data.price
    }

    /// Check if transaction is programmable
    pub fn is_programmable(&self) -> bool {
        matches!(self.data.kind, TransactionKind::ProgrammableTransaction(_))
    }

    /// Get programmable transaction if applicable
    pub fn programmable_transaction(&self) -> Option<&ProgrammableTransaction> {
        if let TransactionKind::ProgrammableTransaction(pt) = &self.data.kind {
            Some(pt)
        } else {
            None
        }
    }

    /// Calculate transaction digest using BLAKE2b-256
    ///
    /// Sui uses BLAKE2b with a domain separator
    pub fn digest(&self) -> Vec<u8> {
        use blake2::{Blake2b, Digest};

        const TRANSACTION_DIGEST_PREFIX: &[u8] = b"TransactionData::";

        let mut hasher = Blake2b::<blake2::digest::consts::U32>::new();
        hasher.update(TRANSACTION_DIGEST_PREFIX);
        hasher.update(&self.raw_bytes);

        hasher.finalize().to_vec()
    }

    /// Count the number of commands
    pub fn command_count(&self) -> usize {
        match &self.data.kind {
            TransactionKind::ProgrammableTransaction(pt) => pt.commands.len(),
            _ => 0,
        }
    }
}

impl Command {
    /// Get a human-readable description of the command
    pub fn command_type(&self) -> &str {
        match self {
            Command::TransferObjects { .. } => "TransferObjects",
            Command::SplitCoins { .. } => "SplitCoins",
            Command::MergeCoins { .. } => "MergeCoins",
            Command::Publish { .. } => "Publish",
            Command::MoveCall { .. } => "MoveCall",
            Command::MakeMoveVec { .. } => "MakeMoveVec",
        }
    }
}

impl std::fmt::Display for TypeTag {
    /// Convert to a human-readable string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeTag::Bool => write!(f, "bool"),
            TypeTag::U8 => write!(f, "u8"),
            TypeTag::U16 => write!(f, "u16"),
            TypeTag::U32 => write!(f, "u32"),
            TypeTag::U64 => write!(f, "u64"),
            TypeTag::U128 => write!(f, "u128"),
            TypeTag::U256 => write!(f, "u256"),
            TypeTag::Address => write!(f, "address"),
            TypeTag::Signer => write!(f, "signer"),
            TypeTag::Vector(inner) => write!(f, "vector<{}>", inner),
            TypeTag::Struct(s) => write!(f, "{}", s),
        }
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
