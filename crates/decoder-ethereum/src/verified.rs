//! Verified Ethereum transaction types.
//!
//! This module provides type-safe Ethereum transaction types that enforce
//! actual parsing through the type system by requiring reconstruction
//! from parsed fields.
//!
//! # Design
//!
//! The key insight is separating parsed fields from raw bytes:
//!
//! - `EthereumParsedFields`: Contains all semantic fields (no raw_bytes)
//! - `ReconstructableTransaction`: Requires RLP reconstruction from fields
//! - `VerifiedEthereumDecoder`: Returns `VerifiedTransaction<EthereumParsedFields>`
//!
//! This ensures the injective property is satisfied through actual parsing,
//! not just storing and replaying bytes.

use borsh::{BorshDeserialize, BorshSerialize};
use decoder_encodings::rlp_encoder::RlpEncoder;
use serde::{Deserialize, Serialize};
use universal_decoder_core::prelude::*;

use crate::types::{AccessListItem, TxType};

/// Ethereum transaction parsed fields (no raw_bytes).
///
/// This struct contains ONLY the semantic fields parsed from a transaction.
/// It does NOT contain raw bytes, ensuring that `reconstruct_bytes()` must
/// actually serialize from the parsed fields.
///
/// # Type-Safety
///
/// By not having a `raw_bytes` field, the implementation of
/// `ReconstructableTransaction::reconstruct_bytes()` is forced to
/// reconstruct bytes from the semantic fields, guaranteeing actual parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EthereumParsedFields {
    /// Transaction type
    pub tx_type: TxType,
    /// Nonce
    pub nonce: u64,
    /// Gas price (legacy transactions)
    pub gas_price: Option<u128>,
    /// Gas limit
    pub gas_limit: u128,
    /// Recipient address (None for contract creation)
    pub to: Option<[u8; 20]>,
    /// Value in Wei
    pub value: u128,
    /// Call data or contract bytecode
    pub data: Vec<u8>,
    /// Chain ID
    pub chain_id: Option<u64>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<u128>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<u128>,
    /// Access list (EIP-2930)
    pub access_list: Vec<AccessListItem>,
    /// Signature v component
    pub v: u64,
    /// Signature r component (32 bytes)
    pub r: [u8; 32],
    /// Signature s component (32 bytes)
    pub s: [u8; 32],
    // NOTE: No raw_bytes field! This is intentional.
}

impl EthereumParsedFields {
    /// Encode signature r component with proper RLP handling.
    ///
    /// Signature components are encoded as minimal big-endian integers,
    /// stripping leading zeros.
    fn encode_signature_r(&self) -> Vec<u8> {
        strip_leading_zeros(&self.r)
    }

    /// Encode signature s component with proper RLP handling.
    fn encode_signature_s(&self) -> Vec<u8> {
        strip_leading_zeros(&self.s)
    }

    /// Reconstruct a legacy transaction's RLP bytes.
    fn reconstruct_legacy(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;
        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;
        Ok(encoder.finalize())
    }

    /// Reconstruct an EIP-2930 transaction's bytes.
    fn reconstruct_eip2930(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, v, r, s]
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        self.append_access_list(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        // Prepend type byte
        let mut result = vec![0x01];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Reconstruct an EIP-1559 transaction's bytes.
    fn reconstruct_eip1559(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, v, r, s]
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.max_priority_fee_per_gas)?;
        list.append_optional_u128(self.max_fee_per_gas)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        self.append_access_list(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        // Prepend type byte
        let mut result = vec![0x02];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Reconstruct an EIP-4844 transaction's bytes.
    ///
    /// Note: EIP-4844 has additional blob fields that we don't fully support yet.
    /// This implementation is equivalent to EIP-1559 with type byte 0x03.
    fn reconstruct_eip4844(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // Same as EIP-1559 for now (blob fields not yet supported)
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.max_priority_fee_per_gas)?;
        list.append_optional_u128(self.max_fee_per_gas)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        self.append_access_list(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        // Prepend type byte
        let mut result = vec![0x03];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Append access list to RLP encoder.
    fn append_access_list(
        &self,
        list: &mut decoder_encodings::rlp_encoder::ListEncoder<'_>,
    ) -> Result<()> {
        list.append_list(|access_list_encoder| {
            for item in &self.access_list {
                access_list_encoder.append_list(|entry| {
                    entry.append_bytes(&item.address)?;
                    entry.append_list(|keys| {
                        for key in &item.storage_keys {
                            keys.append_bytes(key)?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl ReconstructableTransaction for EthereumParsedFields {
    /// Reconstruct the RLP-encoded transaction bytes from parsed fields.
    ///
    /// This method MUST NOT rely on any stored raw bytes - it reconstructs
    /// the transaction purely from the semantic fields.
    ///
    /// # Formal Property
    ///
    /// For valid transactions:
    /// ```text
    /// ∀ tx_bytes: parse(tx_bytes)?.reconstruct_bytes()? == tx_bytes
    /// ```
    fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
        match self.tx_type {
            TxType::Legacy => self.reconstruct_legacy(),
            TxType::Eip2930 => self.reconstruct_eip2930(),
            TxType::Eip1559 => self.reconstruct_eip1559(),
            TxType::Eip4844 => self.reconstruct_eip4844(),
        }
    }
}

/// Strip leading zeros from a byte array.
fn strip_leading_zeros(bytes: &[u8]) -> Vec<u8> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

/// Verified Ethereum decoder that enforces actual parsing.
///
/// Unlike `EthereumDecoder` which returns `EthereumTransaction` (with raw_bytes),
/// this decoder returns `VerifiedTransaction<EthereumParsedFields>` which
/// separates parsed fields from raw bytes and enforces reconstruction.
pub struct VerifiedEthereumDecoder;

impl VerifiedChainDecoder for VerifiedEthereumDecoder {
    type ParsedFields = EthereumParsedFields;

    fn decode_verified(raw_bytes: &[u8]) -> Result<VerifiedTransaction<Self::ParsedFields>> {
        use crate::types::EthereumTransaction;

        // Parse using existing decoder
        let tx = EthereumTransaction::from_raw_bytes(raw_bytes)?;

        // Convert to parsed fields (no raw_bytes)
        let parsed = EthereumParsedFields {
            tx_type: tx.tx_type,
            nonce: tx.nonce,
            gas_price: tx.gas_price,
            gas_limit: tx.gas_limit,
            to: tx.to,
            value: tx.value,
            data: tx.data,
            chain_id: tx.chain_id,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            access_list: tx.access_list,
            v: tx.v,
            r: tx.r,
            s: tx.s,
        };

        Ok(VerifiedTransaction::new(parsed, raw_bytes.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_fields_no_raw_bytes() {
        // Verify that EthereumParsedFields does NOT have a raw_bytes field
        // by checking its size and fields
        let parsed = EthereumParsedFields {
            tx_type: TxType::Legacy,
            nonce: 0,
            gas_price: Some(0),
            gas_limit: 21000,
            to: None,
            value: 0,
            data: vec![],
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: vec![],
            v: 27,
            r: [0u8; 32],
            s: [0u8; 32],
        };

        // This compiles because there's no raw_bytes field
        assert_eq!(parsed.nonce, 0);
        assert_eq!(parsed.tx_type, TxType::Legacy);
    }

    #[test]
    fn test_strip_leading_zeros() {
        assert_eq!(strip_leading_zeros(&[0, 0, 1, 2, 3]), vec![1, 2, 3]);
        assert_eq!(strip_leading_zeros(&[1, 2, 3]), vec![1, 2, 3]);
        assert_eq!(strip_leading_zeros(&[0, 0, 0]), Vec::<u8>::new());
        assert_eq!(strip_leading_zeros(&[]), Vec::<u8>::new());
    }

    #[test]
    fn test_reconstruct_legacy_minimal() {
        let parsed = EthereumParsedFields {
            tx_type: TxType::Legacy,
            nonce: 0,
            gas_price: Some(20_000_000_000u128), // 20 Gwei
            gas_limit: 21000,
            to: Some([0xAB; 20]),
            value: 1_000_000_000_000_000_000u128, // 1 ETH
            data: vec![],
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: vec![],
            v: 37, // EIP-155 signature (chain_id = 1)
            r: [0x12; 32],
            s: [0x34; 32],
        };

        // Verify reconstruction produces valid RLP
        let bytes = parsed.reconstruct_bytes().unwrap();
        assert!(!bytes.is_empty());
        assert!(bytes[0] >= 0xc0); // RLP list prefix
    }

    #[test]
    fn test_field_mutation_changes_output() {
        let parsed = EthereumParsedFields {
            tx_type: TxType::Legacy,
            nonce: 5,
            gas_price: Some(20_000_000_000u128),
            gas_limit: 21000,
            to: Some([0xAB; 20]),
            value: 1_000_000_000_000_000_000u128,
            data: vec![],
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: vec![],
            v: 37,
            r: [0x12; 32],
            s: [0x34; 32],
        };

        let original_bytes = parsed.reconstruct_bytes().unwrap();

        // Mutate nonce
        let mut mutated = parsed.clone();
        mutated.nonce = 999;
        let mutated_bytes = mutated.reconstruct_bytes().unwrap();

        assert_ne!(
            original_bytes, mutated_bytes,
            "Changing nonce should change output bytes"
        );
    }

    #[test]
    fn test_verified_transaction_detects_mutation() {
        use universal_decoder_core::verified::testing::verify_field_affects_output;

        let parsed = EthereumParsedFields {
            tx_type: TxType::Legacy,
            nonce: 5,
            gas_price: Some(20_000_000_000u128),
            gas_limit: 21000,
            to: Some([0xAB; 20]),
            value: 1_000_000_000_000_000_000u128,
            data: vec![1, 2, 3],
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: vec![],
            v: 37,
            r: [0x12; 32],
            s: [0x34; 32],
        };

        let original_bytes = parsed.reconstruct_bytes().unwrap();
        let tx = VerifiedTransaction::new(parsed, original_bytes);

        // Verify critical fields affect output
        verify_field_affects_output(&tx, |p| p.nonce = 999).unwrap();
        verify_field_affects_output(&tx, |p| p.value = 0).unwrap();
        verify_field_affects_output(&tx, |p| p.gas_limit = 100000).unwrap();
        verify_field_affects_output(&tx, |p| p.data = vec![9, 8, 7]).unwrap();
    }
}
