//! Polkadot transaction types
//!
//! Polkadot uses "Extrinsics" which are transactions or inherent data.
//! This module defines the core structures for parsing extrinsics.

use serde::{Deserialize, Serialize};

/// Polkadot extrinsic version and type
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

/// Polkadot-specific address type (different from core Address)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolkadotAddress {
    /// 32-byte account ID (most common)
    Id(Vec<u8>),
    /// 20-byte account ID
    Index(u32),
    /// Raw bytes
    Raw(Vec<u8>),
    /// 32-byte address
    Address32(Vec<u8>),
    /// 20-byte address
    Address20(Vec<u8>),
}

/// Polkadot-specific signature type (different from core Signature)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolkadotSignature {
    /// Sr25519 signature (64 bytes)
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
    pub from: PolkadotAddress,
    /// Signature
    pub signature: PolkadotSignature,
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

/// Polkadot extrinsic (transaction)
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
    /// Get the pallet name if known
    pub fn pallet_name(&self) -> &str {
        match self.pallet_index {
            0 => "System",
            4 => "Balances",
            5 => "Staking",
            6 => "Session",
            7 => "Democracy",
            8 => "Council",
            9 => "TechnicalCommittee",
            10 => "PhragmenElection",
            11 => "TechnicalMembership",
            12 => "Grandpa",
            13 => "Treasury",
            24 => "Utility",
            25 => "Identity",
            26 => "Proxy",
            27 => "Multisig",
            _ => "Unknown",
        }
    }

    /// Get the call name if known (for Balances pallet)
    pub fn call_name(&self) -> &str {
        if self.pallet_index == 4 {
            // Balances pallet
            match self.call_index {
                0 => "transfer",
                1 => "set_balance",
                2 => "force_transfer",
                3 => "transfer_keep_alive",
                4 => "transfer_all",
                _ => "unknown",
            }
        } else {
            "unknown"
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
    }

    #[test]
    fn test_extrinsic_is_signed() {
        let signed = Extrinsic::Signed(SignedExtrinsic {
            from: PolkadotAddress::Id(vec![0; 32]),
            signature: PolkadotSignature::Sr25519(vec![0; 64]),
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
