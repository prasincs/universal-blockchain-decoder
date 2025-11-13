//! # Decoder Encodings
//!
//! Shared encoding utilities for blockchain transaction decoders.
//!
//! This crate provides common encoding/decoding functionality used across
//! multiple blockchain decoders, extracted to eliminate code duplication
//! and provide a single source of truth for critical encoding logic.
//!
//! ## Modules
//!
//! - `bcs`: Binary Canonical Serialization (Move/Aptos/Sui)
//! - `varint`: Bitcoin-style variable-length integer encoding
//! - `compact_u16`: Solana-style compact-u16 encoding
//! - `rlp`: Ethereum's Recursive Length Prefix encoding
//!
//! ## Design Principles
//!
//! 1. **Zero production dependencies**: Only depends on `universal-decoder-core` for error types
//! 2. **Pure Rust implementations**: No external blockchain libraries
//! 3. **Security-focused**: Explicit bounds checking, overflow protection
//! 4. **Well-tested**: Comprehensive unit tests for all encodings
//!
//! ## Usage
//!
//! ```rust
//! use decoder_encodings::varint::encode_varint;
//! use decoder_encodings::compact_u16::read_compact_u16;
//! use decoder_encodings::rlp::RlpItem;
//!
//! // Bitcoin VarInt
//! let mut buf = Vec::new();
//! encode_varint(&mut buf, 1000);
//!
//! // Solana compact-u16
//! use std::io::Cursor;
//! let data = vec![0x80, 0x01]; // 128 in compact-u16
//! let mut cursor = Cursor::new(data.as_slice());
//! let value = read_compact_u16(&mut cursor).unwrap();
//!
//! // Ethereum RLP
//! let rlp_data = vec![0x83, b'd', b'o', b'g'];
//! let item = RlpItem::decode(&rlp_data).unwrap();
//! ```

pub mod bcs;
pub mod compact_u16;
pub mod rlp;
pub mod varint;

// Re-export commonly used types
pub use bcs::{read_bytes as read_bcs_bytes, read_uleb128};
pub use compact_u16::read_compact_u16;
pub use rlp::RlpItem;
pub use varint::encode_varint;
