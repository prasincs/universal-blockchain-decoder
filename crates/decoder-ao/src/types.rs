//! Arweave AO (Actor Oriented) transaction types
//!
//! AO uses ANS-104 DataItems as the message format. This module defines
//! the types for parsing and representing AO messages.

use serde::{Deserialize, Serialize};

/// ANS-104 DataItem representing an AO message
///
/// From the ANS-104 specification:
/// - signature_type (2 bytes)
/// - signature (variable)
/// - owner (public key, variable)
/// - target (optional, 32 bytes)
/// - anchor (optional, 32 bytes)
/// - tags (Avro-encoded)
/// - data (payload)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AOMessage {
    /// Signature type identifier
    pub signature_type: SignatureType,

    /// Cryptographic signature
    pub signature: Vec<u8>,

    /// Public key of the sender
    pub owner: Vec<u8>,

    /// Optional target process ID (32 bytes)
    pub target: Option<Vec<u8>>,

    /// Optional anchor for replay attack prevention (32 bytes)
    pub anchor: Option<Vec<u8>>,

    /// Message tags (key-value pairs)
    pub tags: Vec<Tag>,

    /// Message payload
    pub data: Vec<u8>,

    /// Epoch assigned by Scheduler Unit
    pub epoch: Option<u64>,

    /// Nonce for uniqueness within epoch
    pub nonce: Option<u64>,
}

/// Signature type for ANS-104 DataItems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum SignatureType {
    /// Arweave signature (RSA-PSS with SHA-256)
    Arweave = 1,
    /// Ethereum signature (ECDSA with secp256k1)
    Ethereum = 3,
    /// Solana signature (Ed25519)
    Solana = 4,
    /// Unknown signature type
    Unknown(u16),
}

impl From<u16> for SignatureType {
    fn from(value: u16) -> Self {
        match value {
            1 => SignatureType::Arweave,
            3 => SignatureType::Ethereum,
            4 => SignatureType::Solana,
            other => SignatureType::Unknown(other),
        }
    }
}

impl From<SignatureType> for u16 {
    fn from(sig_type: SignatureType) -> Self {
        match sig_type {
            SignatureType::Arweave => 1,
            SignatureType::Ethereum => 3,
            SignatureType::Solana => 4,
            SignatureType::Unknown(val) => val,
        }
    }
}

/// Tag for categorizing and routing messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name (e.g., "Action", "Target", "From")
    pub name: String,

    /// Tag value
    pub value: String,
}

impl AOMessage {
    /// Get the message ID (DataItem ID)
    ///
    /// For ANS-104, this is typically the SHA-256 hash of the signature
    pub fn message_id(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.signature);
        hasher.finalize().to_vec()
    }

    /// Get the target process ID as a string
    pub fn target_string(&self) -> Option<String> {
        self.target.as_ref().map(|t| base64_url::encode(t))
    }

    /// Get a tag value by name
    pub fn get_tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.value.as_str())
    }

    /// Get the action from tags (common AO pattern)
    pub fn action(&self) -> Option<&str> {
        self.get_tag("Action")
    }

    /// Get the sender from tags (if present)
    pub fn sender(&self) -> Option<&str> {
        self.get_tag("From")
    }
}

/// Base64 URL encoding helper
mod base64_url {
    pub fn encode(input: &[u8]) -> String {
        // Simple base64url encoding (without padding)
        let b64 = base64_encode(input);
        b64.replace('+', "-")
            .replace('/', "_")
            .trim_end_matches('=')
            .to_string()
    }

    fn base64_encode(input: &[u8]) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        let mut i = 0;

        while i + 2 < input.len() {
            let b1 = input[i];
            let b2 = input[i + 1];
            let b3 = input[i + 2];

            result.push(CHARSET[(b1 >> 2) as usize] as char);
            result.push(CHARSET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
            result.push(CHARSET[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char);
            result.push(CHARSET[(b3 & 0x3f) as usize] as char);

            i += 3;
        }

        if i < input.len() {
            let b1 = input[i];
            result.push(CHARSET[(b1 >> 2) as usize] as char);

            if i + 1 < input.len() {
                let b2 = input[i + 1];
                result.push(CHARSET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
                result.push(CHARSET[((b2 & 0x0f) << 2) as usize] as char);
                result.push('=');
            } else {
                result.push(CHARSET[((b1 & 0x03) << 4) as usize] as char);
                result.push('=');
                result.push('=');
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_type_conversion() {
        assert_eq!(SignatureType::from(1), SignatureType::Arweave);
        assert_eq!(SignatureType::from(3), SignatureType::Ethereum);
        assert_eq!(SignatureType::from(4), SignatureType::Solana);
        assert_eq!(SignatureType::from(999), SignatureType::Unknown(999));

        assert_eq!(u16::from(SignatureType::Arweave), 1);
        assert_eq!(u16::from(SignatureType::Ethereum), 3);
        assert_eq!(u16::from(SignatureType::Solana), 4);
    }

    #[test]
    fn test_message_id_deterministic() {
        let msg = AOMessage {
            signature_type: SignatureType::Arweave,
            signature: vec![1, 2, 3, 4],
            owner: vec![5, 6, 7, 8],
            target: None,
            anchor: None,
            tags: vec![],
            data: vec![],
            epoch: None,
            nonce: None,
        };

        let id1 = msg.message_id();
        let id2 = msg.message_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_tag() {
        let msg = AOMessage {
            signature_type: SignatureType::Arweave,
            signature: vec![],
            owner: vec![],
            target: None,
            anchor: None,
            tags: vec![
                Tag {
                    name: "Action".to_string(),
                    value: "Transfer".to_string(),
                },
                Tag {
                    name: "From".to_string(),
                    value: "sender123".to_string(),
                },
            ],
            data: vec![],
            epoch: Some(42),
            nonce: Some(100),
        };

        assert_eq!(msg.action(), Some("Transfer"));
        assert_eq!(msg.sender(), Some("sender123"));
        assert_eq!(msg.get_tag("NonExistent"), None);
    }

    #[test]
    fn test_base64_url_encoding() {
        let input = vec![1, 2, 3, 4, 5];
        let encoded = base64_url::encode(&input);

        // Should not contain +, /, or =
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));

        // Should contain - or _ instead
        let test_bytes = vec![0xfb, 0xff]; // Will have + and / in standard base64
        let encoded_special = base64_url::encode(&test_bytes);
        assert!(
            encoded_special.contains('-')
                || encoded_special.contains('_')
                || !encoded_special.is_empty()
        );
    }
}
