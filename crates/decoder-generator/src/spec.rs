//! Decoder specification types
//!
//! Defines the declarative format for specifying blockchain decoders.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete decoder specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoderSpec {
    /// Chain metadata
    pub chain: ChainSpec,

    /// Encoding configuration
    pub encoding: EncodingSpec,

    /// Transaction format
    pub transaction: TransactionSpec,

    /// Optional: extend another chain's implementation
    pub extends: Option<String>,
}

impl DecoderSpec {
    /// Validate the specification
    pub fn validate(&self) -> Result<()> {
        // Validate chain spec
        if self.chain.name.is_empty() {
            return Err(anyhow!("Chain name cannot be empty"));
        }

        // Validate encoding
        if self.encoding.endianness != "little" && self.encoding.endianness != "big" {
            return Err(anyhow!("Endianness must be 'little' or 'big'"));
        }

        // Validate transaction format has at least one field
        if self.transaction.fields.is_empty() {
            return Err(anyhow!("Transaction must have at least one field"));
        }

        Ok(())
    }
}

/// Chain identity specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpec {
    /// Chain name (e.g., "Bitcoin", "Dogecoin")
    pub name: String,

    /// Chain ID number
    pub chain_id: u64,

    /// Chain family (utxo, account, instruction, custom)
    pub family: String,

    /// Optional: human-readable description
    pub description: Option<String>,
}

/// Encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingSpec {
    /// Endianness: "little" or "big"
    pub endianness: String,

    /// Hash algorithm: "sha256", "double-sha256", "keccak256", etc.
    pub hash: String,

    /// Variable-length encoding: "varint", "rlp", "compact-u16", etc.
    pub var_encoding: Option<String>,
}

/// Transaction format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSpec {
    /// Transaction fields in order
    pub fields: Vec<FieldSpec>,

    /// Optional: how to compute transaction ID
    pub txid: Option<TxidSpec>,
}

/// Field specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldSpec {
    /// Fixed-size primitive (u8, u16, u32, u64)
    #[serde(rename = "primitive")]
    Primitive {
        name: String,
        prim_type: String, // "u8", "u16", "u32", "u64", "i32", etc.
    },

    /// Fixed-size byte array
    #[serde(rename = "bytes")]
    Bytes { name: String, size: usize },

    /// Variable-length bytes (prefixed with length)
    #[serde(rename = "var_bytes")]
    VarBytes { name: String, max_size: usize },

    /// Array of items (with variable count)
    #[serde(rename = "array")]
    Array {
        name: String,
        item_type: Box<FieldSpec>,
        max_count: Option<usize>,
    },

    /// Conditional field (only present if condition)
    #[serde(rename = "conditional")]
    Conditional {
        condition: String, // e.g., "is_segwit"
        field: Box<FieldSpec>,
    },
}

/// Transaction ID computation spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxidSpec {
    /// Fields to exclude from hash (e.g., witness data for SegWit)
    pub exclude_fields: Vec<String>,

    /// Hash algorithm (can override chain default)
    pub hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_spec_validation() {
        let spec = DecoderSpec {
            chain: ChainSpec {
                name: "TestChain".to_string(),
                chain_id: 999,
                family: "utxo".to_string(),
                description: Some("Test chain".to_string()),
            },
            encoding: EncodingSpec {
                endianness: "little".to_string(),
                hash: "sha256".to_string(),
                var_encoding: Some("varint".to_string()),
            },
            transaction: TransactionSpec {
                fields: vec![FieldSpec::Primitive {
                    name: "version".to_string(),
                    prim_type: "u32".to_string(),
                }],
                txid: None,
            },
            extends: None,
        };

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_invalid_endianness() {
        let spec = DecoderSpec {
            chain: ChainSpec {
                name: "Test".to_string(),
                chain_id: 1,
                family: "utxo".to_string(),
                description: None,
            },
            encoding: EncodingSpec {
                endianness: "invalid".to_string(),
                hash: "sha256".to_string(),
                var_encoding: None,
            },
            transaction: TransactionSpec {
                fields: vec![FieldSpec::Primitive {
                    name: "version".to_string(),
                    prim_type: "u32".to_string(),
                }],
                txid: None,
            },
            extends: None,
        };

        assert!(spec.validate().is_err());
    }
}
