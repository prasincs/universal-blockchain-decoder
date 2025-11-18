//! Filecoin-specific transaction types
//!
//! Pure Rust implementation using CBOR decoder.
//! Supports Filecoin messages (both signed and unsigned).
//!
//! Filecoin uses an account-based model similar to Ethereum, but with
//! built-in actors instead of smart contracts.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use universal_decoder_core::prelude::*;

/// Filecoin address types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressProtocol {
    /// ID address (f0)
    Id = 0,
    /// SECP256K1 address (f1)
    Secp256k1 = 1,
    /// Actor address (f2)
    Actor = 2,
    /// BLS address (f3)
    Bls = 3,
}

impl BorshSerialize for AddressProtocol {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let value: u8 = match self {
            AddressProtocol::Id => 0,
            AddressProtocol::Secp256k1 => 1,
            AddressProtocol::Actor => 2,
            AddressProtocol::Bls => 3,
        };
        BorshSerialize::serialize(&value, writer)
    }
}

impl BorshDeserialize for AddressProtocol {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let value = u8::deserialize_reader(reader)?;
        match value {
            0 => Ok(AddressProtocol::Id),
            1 => Ok(AddressProtocol::Secp256k1),
            2 => Ok(AddressProtocol::Actor),
            3 => Ok(AddressProtocol::Bls),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown address protocol: {}", value),
            )),
        }
    }
}

impl AddressProtocol {
    /// Parse protocol from byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0 => Ok(AddressProtocol::Id),
            1 => Ok(AddressProtocol::Secp256k1),
            2 => Ok(AddressProtocol::Actor),
            3 => Ok(AddressProtocol::Bls),
            _ => Err(DecoderError::invalid_structure(format!(
                "Unknown address protocol: {}",
                byte
            ))),
        }
    }
}

/// Filecoin address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FilecoinAddress {
    /// Address protocol/type
    pub protocol: AddressProtocol,
    /// Address payload (varies by protocol)
    pub payload: Vec<u8>,
}

impl FilecoinAddress {
    /// Create a new Filecoin address
    pub fn new(protocol: AddressProtocol, payload: Vec<u8>) -> Self {
        Self { protocol, payload }
    }

    /// Get address as bytes (protocol byte + payload)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + self.payload.len());
        bytes.push(self.protocol as u8);
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Get human-readable representation
    /// Format: f{network}{protocol}{encoded_payload}
    /// Example: f1ydrh6... (mainnet), t1ydrh6... (testnet)
    pub fn to_string(&self, is_mainnet: bool) -> String {
        let network = if is_mainnet { "f" } else { "t" };
        let protocol = self.protocol as u8;

        // For ID addresses, just use the numeric ID
        if matches!(self.protocol, AddressProtocol::Id) {
            if let Ok(id) = self.parse_id() {
                return format!("{}{}{}", network, protocol, id);
            }
        }

        // For other addresses, encode payload as base32
        // Note: Real implementation would use proper base32 encoding
        // For now, use hex as placeholder
        format!(
            "{}{}{}",
            network,
            protocol,
            universal_decoder_core::hex::encode(&self.payload)
        )
    }

    /// Parse ID from ID address payload
    fn parse_id(&self) -> Result<u64> {
        if !matches!(self.protocol, AddressProtocol::Id) {
            return Err(DecoderError::invalid_structure("Not an ID address"));
        }

        // ID is encoded as LEB128
        let mut result = 0u64;
        let mut shift = 0;

        for &byte in &self.payload {
            if shift >= 64 {
                return Err(DecoderError::invalid_structure("ID too large"));
            }
            result |= ((byte & 0x7f) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }

        Err(DecoderError::invalid_structure(
            "Incomplete LEB128 encoding",
        ))
    }
}

/// Filecoin signature type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    /// SECP256K1 signature
    Secp256k1 = 1,
    /// BLS signature
    Bls = 2,
}

impl BorshSerialize for SignatureType {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let value: u8 = match self {
            SignatureType::Secp256k1 => 1,
            SignatureType::Bls => 2,
        };
        BorshSerialize::serialize(&value, writer)
    }
}

impl BorshDeserialize for SignatureType {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let value = u8::deserialize_reader(reader)?;
        match value {
            1 => Ok(SignatureType::Secp256k1),
            2 => Ok(SignatureType::Bls),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown signature type: {}", value),
            )),
        }
    }
}

impl SignatureType {
    /// Parse signature type from byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            1 => Ok(SignatureType::Secp256k1),
            2 => Ok(SignatureType::Bls),
            _ => Err(DecoderError::invalid_structure(format!(
                "Unknown signature type: {}",
                byte
            ))),
        }
    }
}

/// Filecoin signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FilecoinSignature {
    /// Signature type
    pub sig_type: SignatureType,
    /// Signature data
    pub data: Vec<u8>,
}

impl FilecoinSignature {
    /// Create a new signature
    pub fn new(sig_type: SignatureType, data: Vec<u8>) -> Self {
        Self { sig_type, data }
    }
}

/// Filecoin unsigned message
///
/// This is the message that gets signed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FilecoinMessage {
    /// Message version
    pub version: u64,
    /// Sender address
    pub from: FilecoinAddress,
    /// Recipient address
    pub to: FilecoinAddress,
    /// Message sequence number (nonce)
    pub sequence: u64,
    /// Amount to transfer in attoFIL (10^-18 FIL)
    pub value: Vec<u8>, // BigInt as bytes
    /// Gas limit
    pub gas_limit: u64,
    /// Maximum fee per gas unit
    pub gas_fee_cap: Vec<u8>, // BigInt as bytes
    /// Priority fee (tip)
    pub gas_premium: Vec<u8>, // BigInt as bytes
    /// Method number (0 = simple transfer, >0 = actor method)
    pub method_num: u64,
    /// Method parameters (CBOR-encoded)
    pub params: Vec<u8>,
}

impl FilecoinMessage {
    /// Check if this is a simple value transfer (method 0)
    pub fn is_transfer(&self) -> bool {
        self.method_num == 0
    }

    /// Get value as u128 (if it fits)
    pub fn value_as_u128(&self) -> Result<u128> {
        bigint_bytes_to_u128(&self.value)
    }

    /// Get gas_fee_cap as u128 (if it fits)
    pub fn gas_fee_cap_as_u128(&self) -> Result<u128> {
        bigint_bytes_to_u128(&self.gas_fee_cap)
    }

    /// Get gas_premium as u128 (if it fits)
    pub fn gas_premium_as_u128(&self) -> Result<u128> {
        bigint_bytes_to_u128(&self.gas_premium)
    }
}

/// Filecoin signed message
///
/// This is what gets broadcast to the network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FilecoinSignedMessage {
    /// The unsigned message
    pub message: FilecoinMessage,
    /// Signature over the message
    pub signature: FilecoinSignature,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl FilecoinSignedMessage {
    /// Create a new signed message
    pub fn new(message: FilecoinMessage, signature: FilecoinSignature, raw_bytes: Vec<u8>) -> Self {
        Self {
            message,
            signature,
            raw_bytes,
        }
    }

    /// Calculate message CID (Content Identifier)
    ///
    /// Filecoin uses CIDs for transaction hashes, which are based on Blake2b-256
    pub fn calculate_cid(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        // Note: Real implementation would use Blake2b-256 and proper CID encoding
        // For now, using SHA-256 as placeholder
        // TODO: Add blake2 dependency and implement proper CID calculation
        Sha256::digest(&self.raw_bytes).to_vec()
    }
}

/// Filecoin-specific transaction representation
///
/// This is the main type that implements the decoder traits.
/// It wraps a FilecoinSignedMessage and provides the interface
/// expected by universal-decoder-core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FilecoinTransaction {
    /// The signed message
    pub signed_message: FilecoinSignedMessage,
}

impl FilecoinTransaction {
    /// Create from raw CBOR-encoded bytes
    pub fn from_raw_bytes(raw_bytes: &[u8]) -> Result<Self> {
        // Implemented in parsing.rs via parse_signed_message
        // This is just a convenience wrapper
        crate::parsing::parse_signed_message(raw_bytes)
    }

    /// Get the hash (CID) of this transaction
    pub fn hash(&self) -> Vec<u8> {
        self.signed_message.calculate_cid()
    }

    /// Get the message
    pub fn message(&self) -> &FilecoinMessage {
        &self.signed_message.message
    }

    /// Get the signature
    pub fn signature(&self) -> &FilecoinSignature {
        &self.signed_message.signature
    }
}

/// Helper: Convert BigInt bytes (big-endian) to u128
///
/// Filecoin uses big integers for values and gas prices.
/// This helper converts them to u128 for the TxIR representation.
fn bigint_bytes_to_u128(bytes: &[u8]) -> Result<u128> {
    if bytes.is_empty() {
        return Ok(0);
    }

    if bytes.len() > 16 {
        return Err(DecoderError::invalid_structure("BigInt too large for u128"));
    }

    let mut value = 0u128;
    for &byte in bytes {
        value = value
            .checked_mul(256)
            .ok_or_else(|| DecoderError::invalid_structure("Overflow in BigInt conversion"))?;
        value = value
            .checked_add(byte as u128)
            .ok_or_else(|| DecoderError::invalid_structure("Overflow in BigInt conversion"))?;
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_protocol_from_byte() {
        assert_eq!(AddressProtocol::from_byte(0).unwrap(), AddressProtocol::Id);
        assert_eq!(
            AddressProtocol::from_byte(1).unwrap(),
            AddressProtocol::Secp256k1
        );
        assert_eq!(
            AddressProtocol::from_byte(2).unwrap(),
            AddressProtocol::Actor
        );
        assert_eq!(AddressProtocol::from_byte(3).unwrap(), AddressProtocol::Bls);
        assert!(AddressProtocol::from_byte(99).is_err());
    }

    #[test]
    fn test_signature_type_from_byte() {
        assert_eq!(
            SignatureType::from_byte(1).unwrap(),
            SignatureType::Secp256k1
        );
        assert_eq!(SignatureType::from_byte(2).unwrap(), SignatureType::Bls);
        assert!(SignatureType::from_byte(99).is_err());
    }

    #[test]
    fn test_bigint_bytes_to_u128() {
        // Empty bytes = 0
        assert_eq!(bigint_bytes_to_u128(&[]).unwrap(), 0);

        // Single byte
        assert_eq!(bigint_bytes_to_u128(&[42]).unwrap(), 42);

        // Multiple bytes (big-endian)
        assert_eq!(bigint_bytes_to_u128(&[0x01, 0x00]).unwrap(), 256);
        assert_eq!(bigint_bytes_to_u128(&[0x01, 0x02, 0x03]).unwrap(), 66051);

        // Too large (more than 16 bytes)
        assert!(bigint_bytes_to_u128(&[0u8; 17]).is_err());
    }

    #[test]
    fn test_filecoin_address_to_bytes() {
        let addr = FilecoinAddress::new(AddressProtocol::Id, vec![0x01, 0x02]);
        let bytes = addr.to_bytes();
        assert_eq!(bytes[0], 0); // Protocol
        assert_eq!(&bytes[1..], &[0x01, 0x02]); // Payload
    }

    #[test]
    fn test_filecoin_message_is_transfer() {
        let addr = FilecoinAddress::new(AddressProtocol::Id, vec![0x01]);
        let msg = FilecoinMessage {
            version: 0,
            from: addr.clone(),
            to: addr.clone(),
            sequence: 0,
            value: vec![],
            gas_limit: 1000000,
            gas_fee_cap: vec![],
            gas_premium: vec![],
            method_num: 0,
            params: vec![],
        };

        assert!(msg.is_transfer());

        let mut msg2 = msg;
        msg2.method_num = 1;
        assert!(!msg2.is_transfer());
    }
}
