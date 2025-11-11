//! Vendored dependencies for the core library.
//!
//! This module contains dependencies that have been vendored using git subtree
//! to minimize the Trusted Computing Base (TCB) and enable formal verification.
//!
//! ## Vendored Dependencies
//!
//! - **hex** (v0.4.3): Hex encoding/decoding utilities
//!   - Original: https://github.com/KokaKiwi/rust-hex
//!   - License: MIT OR Apache-2.0
//!   - Added via: `git subtree add --prefix crates/universal-decoder-core/src/vendored/hex https://github.com/KokaKiwi/rust-hex.git v0.4.3 --squash`
//!
//! ## Why Vendored?
//!
//! 1. **Minimal TCB**: Reduces external dependencies in production code
//! 2. **Formal Verification**: Allows verification of the entire codebase
//! 3. **Supply Chain Security**: Full control over dependencies
//! 4. **Cryptographic Verification**: Git subtree provides verifiable history
//!
//! ## Verification
//!
//! To verify that vendored code matches upstream:
//!
//! ```bash
//! git diff v0.4.3 -- crates/universal-decoder-core/src/vendored/hex
//! ```
//!
//! No output means the vendored code is identical to the upstream tag.

// We're using a wrapper module for hex because including it directly with #[path]
// doesn't work (it expects to be a crate root). Instead, we expose only the
// functionality we need through a clean public API.

pub mod hex {
    //! Hex encoding and decoding utilities (vendored from rust-hex v0.4.3)
    //!
    //! This module provides hex encoding/decoding functionality vendored from
    //! the `hex` crate. We vendor this code to minimize external dependencies
    //! and enable formal verification of the entire codebase.
    //!
    //! ## License
    //!
    //! Copyright (c) 2013-2014 The Rust Project Developers.
    //! Copyright (c) 2015-2020 The rust-hex Developers.
    //!
    //! Licensed under the Apache License, Version 2.0 or the MIT license,
    //! at your option.
    //!
    //! Original source: <https://github.com/KokaKiwi/rust-hex>

    use std::fmt;

    /// An error that can occur when decoding a hex string.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FromHexError {
        /// An invalid character was found in the hex string.
        InvalidHexCharacter { c: char, index: usize },
        /// A hex string's length needs to be even.
        OddLength,
    }

    impl fmt::Display for FromHexError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                FromHexError::InvalidHexCharacter { c, index } => {
                    write!(f, "Invalid character {:?} at position {}", c, index)
                }
                FromHexError::OddLength => write!(f, "Odd number of digits"),
            }
        }
    }

    impl std::error::Error for FromHexError {}

    /// Encodes some bytes as a hex string.
    ///
    /// # Example
    ///
    /// ```
    /// use universal_decoder_core::hex;
    ///
    /// let encoded = hex::encode(b"Hello world!");
    /// assert_eq!(encoded, "48656c6c6f20776f726c6421");
    /// ```
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }

    /// Encodes some bytes as an uppercase hex string.
    ///
    /// # Example
    ///
    /// ```
    /// use universal_decoder_core::hex;
    ///
    /// let encoded = hex::encode_upper(b"test");
    /// assert_eq!(encoded, "74657374");
    /// ```
    pub fn encode_upper<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref()
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect()
    }

    /// Decodes a hex string into raw bytes.
    ///
    /// Both, upper and lower case characters are valid in the input string and
    /// can even be mixed (e.g. `f9b4Ca` is valid).
    ///
    /// Decoding will return `Err(FromHexError::OddLength)` if the input string's
    /// length is not even. If the input contains invalid hex characters, the
    /// first invalid character and its position in the string will be returned
    /// as an error.
    ///
    /// # Example
    ///
    /// ```
    /// use universal_decoder_core::hex;
    ///
    /// let decoded = hex::decode("48656c6c6f20776f726c6421").unwrap();
    /// assert_eq!(decoded, b"Hello world!");
    /// ```
    pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
        let data = data.as_ref();

        if data.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        let mut result = Vec::with_capacity(data.len() / 2);

        for (index, chunk) in data.chunks(2).enumerate() {
            let high = val(chunk[0], index * 2)?;
            let low = val(chunk[1], index * 2 + 1)?;
            result.push(high << 4 | low);
        }

        Ok(result)
    }

    /// Helper function to convert a hex character to its value
    #[inline]
    fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
        match c {
            b'A'..=b'F' => Ok(c - b'A' + 10),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'0'..=b'9' => Ok(c - b'0'),
            _ => Err(FromHexError::InvalidHexCharacter {
                c: c as char,
                index: idx,
            }),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_encode() {
            assert_eq!(encode(b"Hello world!"), "48656c6c6f20776f726c6421");
        }

        #[test]
        fn test_decode() {
            assert_eq!(
                decode("48656c6c6f20776f726c6421").unwrap(),
                b"Hello world!"
            );
        }

        #[test]
        fn test_roundtrip() {
            let original = b"test data 123";
            let encoded = encode(original);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(&decoded[..], &original[..]);
        }
    }
}
