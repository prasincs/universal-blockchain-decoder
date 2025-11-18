//! Bittensor transaction types
//!
//! Bittensor uses Substrate "Extrinsics" like Polkadot.
//! This module defines the core structures for parsing Bittensor extrinsics.

use serde::{Deserialize, Serialize};

/// Bittensor extrinsic version and type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtrinsicVersion {
    /// Version number (lower 7 bits)
    pub version: u8,
    /// Is signed? (bit 7)
    pub is_signed: bool,
}

impl ExtrinsicVersion {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            version: byte & 0x7F,
            is_signed: (byte & 0x80) != 0,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut byte = self.version & 0x7F;
        if self.is_signed {
            byte |= 0x80;
        }
        byte
    }
}

/// Bittensor-specific address type (Substrate SS58 encoding)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BittensorAddress {
    /// 32-byte account ID (most common)
    Id(Vec<u8>),
    /// Account index
    Index(u32),
    /// Raw bytes
    Raw(Vec<u8>),
    /// 32-byte address
    Address32(Vec<u8>),
    /// 20-byte address
    Address20(Vec<u8>),
}

/// Bittensor-specific signature type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BittensorSignature {
    /// Sr25519 signature (64 bytes) - most common in Substrate
    Sr25519(Vec<u8>),
    /// Ed25519 signature (64 bytes)
    Ed25519(Vec<u8>),
    /// ECDSA signature (65 bytes)
    Ecdsa(Vec<u8>),
}

/// Era specification for transaction mortality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Era {
    /// Transaction is immortal (never expires)
    Immortal,
    /// Transaction expires after a certain period
    /// (period, phase)
    Mortal(u64, u64),
}

/// Extra data attached to signed extrinsics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedExtension {
    /// Era (mortality)
    pub era: Era,
    /// Nonce
    pub nonce: u64,
    /// Tip for block producer
    pub tip: u128,
}

/// Signed extrinsic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedExtrinsic {
    /// Sender address
    pub from: BittensorAddress,
    /// Signature
    pub signature: BittensorSignature,
    /// Signed extension (era, nonce, tip)
    pub extension: SignedExtension,
    /// Call data (pallet + function + parameters)
    pub call: Vec<u8>,
}

/// Unsigned extrinsic (inherents, unsigned transactions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedExtrinsic {
    /// Call data (pallet + function + parameters)
    pub call: Vec<u8>,
}

/// Bittensor extrinsic (transaction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Extrinsic {
    /// Signed transaction
    Signed(SignedExtrinsic),
    /// Unsigned transaction or inherent
    Unsigned(UnsignedExtrinsic),
}

impl Extrinsic {
    pub fn is_signed(&self) -> bool {
        matches!(self, Extrinsic::Signed(_))
    }

    pub fn call_data(&self) -> &[u8] {
        match self {
            Extrinsic::Signed(s) => &s.call,
            Extrinsic::Unsigned(u) => &u.call,
        }
    }
}

/// Call within an extrinsic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    /// Pallet index (which pallet/module)
    pub pallet_index: u8,
    /// Call index (which function in the pallet)
    pub call_index: u8,
    /// Call parameters (SCALE-encoded)
    pub parameters: Vec<u8>,
}

impl Call {
    /// Get the pallet name if known (Bittensor-specific pallets)
    pub fn pallet_name(&self) -> &str {
        match self.pallet_index {
            0 => "System",
            1 => "Scheduler",
            2 => "Preimage",
            3 => "Timestamp",
            4 => "Balances",
            5 => "TransactionPayment",
            7 => "SubtensorModule", // Bittensor's core pallet
            8 => "Triumvirate",
            9 => "TriumvirateMembers",
            10 => "SenateMembers",
            11 => "Utility",
            12 => "Sudo",
            13 => "Multisig",
            14 => "Proxy",
            15 => "Registry", // Bittensor subnet registry
            _ => "Unknown",
        }
    }

    /// Get the call name if known (for common pallets)
    pub fn call_name(&self) -> &str {
        match self.pallet_index {
            4 => {
                // Balances pallet
                match self.call_index {
                    0 => "transfer",
                    1 => "set_balance",
                    2 => "force_transfer",
                    3 => "transfer_keep_alive",
                    4 => "transfer_all",
                    5 => "force_unreserve",
                    _ => "unknown",
                }
            }
            7 => {
                // SubtensorModule (Bittensor-specific)
                match self.call_index {
                    0 => "set_weights",
                    1 => "add_stake",
                    2 => "remove_stake",
                    3 => "serve_axon",
                    4 => "serve_prometheus",
                    5 => "register",
                    6 => "sudo_register",
                    7 => "burned_register",
                    _ => "unknown",
                }
            }
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrinsic_version() {
        let version = ExtrinsicVersion::from_byte(0x84); // v4, signed
        assert_eq!(version.version, 4);
        assert!(version.is_signed);
        assert_eq!(version.to_byte(), 0x84);

        let version = ExtrinsicVersion::from_byte(0x04); // v4, unsigned
        assert_eq!(version.version, 4);
        assert!(!version.is_signed);
        assert_eq!(version.to_byte(), 0x04);
    }

    #[test]
    fn test_call_pallet_name() {
        let call = Call {
            pallet_index: 4,
            call_index: 0,
            parameters: vec![],
        };
        assert_eq!(call.pallet_name(), "Balances");
        assert_eq!(call.call_name(), "transfer");

        let call = Call {
            pallet_index: 7,
            call_index: 0,
            parameters: vec![],
        };
        assert_eq!(call.pallet_name(), "SubtensorModule");
        assert_eq!(call.call_name(), "set_weights");
    }

    #[test]
    fn test_extrinsic_is_signed() {
        let signed = Extrinsic::Signed(SignedExtrinsic {
            from: BittensorAddress::Id(vec![0; 32]),
            signature: BittensorSignature::Sr25519(vec![0; 64]),
            extension: SignedExtension {
                era: Era::Immortal,
                nonce: 0,
                tip: 0,
            },
            call: vec![],
        });
        assert!(signed.is_signed());

        let unsigned = Extrinsic::Unsigned(UnsignedExtrinsic { call: vec![] });
        assert!(!unsigned.is_signed());
    }
}
