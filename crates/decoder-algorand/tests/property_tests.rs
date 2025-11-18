//! Property-based tests for Algorand decoder
//!
//! These tests use proptest to verify invariants across a wide range of inputs

use decoder_algorand::AlgorandDecoder;
use decoder_primitives::prelude::*;
use proptest::prelude::*;

/// Generate arbitrary byte arrays (limited size for performance)
fn arb_small_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1024)
}

/// Generate arbitrary valid MessagePack maps
fn arb_msgpack_map() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 1..256).prop_map(|mut bytes| {
        // Ensure it starts with a map marker
        if !bytes.is_empty() {
            bytes[0] = 0x80 | (bytes.len() as u8 & 0x0f).min(0x0f);
        }
        bytes
    })
}

proptest! {
    /// Property: Decoder should never panic, regardless of input
    #[test]
    fn prop_decoder_never_panics(bytes in arb_small_bytes()) {
        let _ = AlgorandDecoder::decode(&bytes);
        // If we get here without panicking, test passes
    }

    /// Property: Validation should never panic
    #[test]
    fn prop_validate_format_never_panics(bytes in arb_small_bytes()) {
        let _ = AlgorandDecoder::validate_format(&bytes);
        // If we get here without panicking, test passes
    }

    /// Property: Empty input always fails
    #[test]
    fn prop_empty_input_fails(_seed in any::<u8>()) {
        let result = AlgorandDecoder::decode(&[]);
        prop_assert!(result.is_err());
    }

    /// Property: Canonicalize should be deterministic
    #[test]
    fn prop_canonicalize_deterministic(
        bytes in arb_msgpack_map(),
    ) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            let result1 = tx.canonicalize();
            let result2 = tx.canonicalize();

            // Both should succeed or both should fail
            prop_assert_eq!(result1.is_ok(), result2.is_ok());

            // If both succeed, results should be identical
            if let (Ok(ir1), Ok(ir2)) = (result1, result2) {
                // Compare key fields for determinism
                prop_assert_eq!(ir1.metadata.size, ir2.metadata.size);
                prop_assert_eq!(ir1.authorization.signature_scheme, ir2.authorization.signature_scheme);
                prop_assert_eq!(ir1.operations.len(), ir2.operations.len());
            }
        }
    }

    /// Property: Transaction hash should be deterministic
    #[test]
    fn prop_tx_id_deterministic(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            let hash1 = tx.tx_id();
            let hash2 = tx.tx_id();
            prop_assert_eq!(hash1, hash2, "Transaction ID should be deterministic");
        }
    }

    /// Property: Transaction hash should always be 32 bytes (SHA-512/256)
    #[test]
    fn prop_tx_id_length(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            let hash = tx.tx_id();
            prop_assert_eq!(hash.len(), 32, "SHA-512/256 should produce 32 bytes");
        }
    }

    /// Property: Decoded transaction should preserve raw bytes
    #[test]
    fn prop_preserves_raw_bytes(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            prop_assert_eq!(tx.raw_bytes, bytes, "Raw bytes should be preserved");
        }
    }

    /// Property: Valid transactions have correct signature scheme
    #[test]
    fn prop_signature_scheme_eddsa(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            if let Ok(ir) = tx.canonicalize() {
                prop_assert_eq!(
                    ir.authorization.signature_scheme,
                    SignatureScheme::EdDsa,
                    "Algorand uses Ed25519 (EdDsa)"
                );
            }
        }
    }

    /// Property: Chain identity is always consistent
    #[test]
    fn prop_chain_identity_consistent(_seed in any::<u8>()) {
        let chain = AlgorandDecoder::chain();
        prop_assert_eq!(chain.chain_id(), 4160);
        prop_assert_eq!(chain.chain_name(), "Algorand");
        prop_assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    /// Property: TxIR size should match raw bytes size
    #[test]
    fn prop_txir_size_matches_raw(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            if let Ok(ir) = tx.canonicalize() {
                prop_assert_eq!(
                    ir.metadata.size,
                    bytes.len(),
                    "TxIR size should match raw bytes length"
                );
            }
        }
    }

    /// Property: Address encoding is deterministic
    #[test]
    fn prop_address_encoding_deterministic(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            let addr1 = tx.sender_address();
            let addr2 = tx.sender_address();
            prop_assert_eq!(addr1, addr2, "Address encoding should be deterministic");
        }
    }

    /// Property: Validation errors are consistent
    #[test]
    fn prop_validation_consistent(bytes in arb_msgpack_map()) {
        if let Ok(tx) = AlgorandDecoder::decode(&bytes) {
            let valid1 = tx.validate();
            let valid2 = tx.validate();

            // Both should give same result
            prop_assert_eq!(
                valid1.is_ok(),
                valid2.is_ok(),
                "Validation should be deterministic"
            );
        }
    }
}

/// Additional property tests for specific transaction types
#[cfg(test)]
mod specific_properties {
    use super::*;

    /// Generate arbitrary 32-byte arrays (for addresses/hashes)
    fn arb_32_bytes() -> impl Strategy<Value = Vec<u8>> {
        prop::collection::vec(any::<u8>(), 32..=32)
    }

    proptest! {
        /// Property: Addresses should round-trip through encoding
        #[test]
        fn prop_address_format_valid(pubkey in arb_32_bytes()) {
            use decoder_algorand::*;

            // Create a minimal transaction with this pubkey
            let tx_data = create_minimal_payment_msgpack(&pubkey);

            if let Ok(tx) = AlgorandDecoder::decode(&tx_data) {
                let addr = tx.sender_address();

                // Valid Algorand addresses are base32 encoded
                prop_assert!(
                    addr.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
                    "Address should be base32: {}",
                    addr
                );

                // Length should be around 58 characters (36 bytes * 8/5 ≈ 58)
                prop_assert!(
                    addr.len() >= 55 && addr.len() <= 62,
                    "Address length should be ~58 chars, got {}",
                    addr.len()
                );
            }
        }

        /// Property: Payment transactions should have receiver
        #[test]
        fn prop_payment_has_receiver(
            sender in arb_32_bytes(),
            receiver in arb_32_bytes(),
            amount in 1u64..1_000_000_000u64,
        ) {
            let tx_data = create_payment_msgpack(&sender, &receiver, amount);

            if let Ok(tx) = AlgorandDecoder::decode(&tx_data) {
                if let Ok(ir) = tx.canonicalize() {
                    // Payment transactions should have at least one operation
                    prop_assert!(
                        !ir.operations.is_empty(),
                        "Payment transaction should have operations"
                    );

                    // Should have account changes for sender and receiver
                    prop_assert!(
                        !ir.state_deltas.account_changes.is_empty(),
                        "Payment should have account changes"
                    );
                }
            }
        }

        /// Property: Sender should always lose at least the fee
        #[test]
        fn prop_sender_loses_fee(
            sender in arb_32_bytes(),
            receiver in arb_32_bytes(),
            amount in 1u64..1_000_000u64,
            fee in 1000u64..10_000u64,
        ) {
            let tx_data = create_payment_with_fee(&sender, &receiver, amount, fee);

            if let Ok(tx) = AlgorandDecoder::decode(&tx_data) {
                if let Ok(ir) = tx.canonicalize() {
                    // Find sender's account change
                    let sender_change = ir.state_deltas.account_changes.iter()
                        .find(|ac| ac.address.bytes == sender);

                    if let Some(change) = sender_change {
                        prop_assert!(
                            change.balance_change < 0,
                            "Sender should lose balance (fee + amount)"
                        );

                        prop_assert!(
                            change.balance_change <= -(fee as i128),
                            "Sender should lose at least the fee"
                        );
                    }
                }
            }
        }
    }

    /// Helper: Create minimal payment transaction MessagePack
    fn create_minimal_payment_msgpack(sender: &[u8]) -> Vec<u8> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct MinimalSignedTx<'a> {
            #[serde(rename = "txn")]
            txn: MinimalTx<'a>,
        }

        #[derive(Serialize)]
        struct MinimalTx<'a> {
            #[serde(rename = "type")]
            tx_type: &'static str,
            #[serde(rename = "snd")]
            sender: &'a [u8],
            #[serde(rename = "fee")]
            fee: u64,
            #[serde(rename = "fv")]
            first_valid: u64,
            #[serde(rename = "lv")]
            last_valid: u64,
            #[serde(rename = "gh")]
            genesis_hash: Vec<u8>,
            #[serde(rename = "rcv")]
            receiver: Vec<u8>,
            #[serde(rename = "amt")]
            amount: u64,
        }

        let tx = MinimalSignedTx {
            txn: MinimalTx {
                tx_type: "pay",
                sender,
                fee: 1000,
                first_valid: 1000,
                last_valid: 2000,
                genesis_hash: vec![0u8; 32],
                receiver: vec![1u8; 32],
                amount: 1_000_000,
            },
        };

        rmp_serde::to_vec(&tx).unwrap_or_default()
    }

    /// Helper: Create payment transaction with specific receiver and amount
    fn create_payment_msgpack(sender: &[u8], receiver: &[u8], amount: u64) -> Vec<u8> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct SignedTx<'a> {
            #[serde(rename = "txn")]
            txn: PaymentTx<'a>,
        }

        #[derive(Serialize)]
        struct PaymentTx<'a> {
            #[serde(rename = "type")]
            tx_type: &'static str,
            #[serde(rename = "snd")]
            sender: &'a [u8],
            #[serde(rename = "fee")]
            fee: u64,
            #[serde(rename = "fv")]
            first_valid: u64,
            #[serde(rename = "lv")]
            last_valid: u64,
            #[serde(rename = "gh")]
            genesis_hash: Vec<u8>,
            #[serde(rename = "rcv")]
            receiver: &'a [u8],
            #[serde(rename = "amt")]
            amount: u64,
        }

        let tx = SignedTx {
            txn: PaymentTx {
                tx_type: "pay",
                sender,
                fee: 1000,
                first_valid: 1000,
                last_valid: 2000,
                genesis_hash: vec![0u8; 32],
                receiver,
                amount,
            },
        };

        rmp_serde::to_vec(&tx).unwrap_or_default()
    }

    /// Helper: Create payment with specific fee
    fn create_payment_with_fee(sender: &[u8], receiver: &[u8], amount: u64, fee: u64) -> Vec<u8> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct SignedTx<'a> {
            #[serde(rename = "txn")]
            txn: PaymentTx<'a>,
        }

        #[derive(Serialize)]
        struct PaymentTx<'a> {
            #[serde(rename = "type")]
            tx_type: &'static str,
            #[serde(rename = "snd")]
            sender: &'a [u8],
            #[serde(rename = "fee")]
            fee: u64,
            #[serde(rename = "fv")]
            first_valid: u64,
            #[serde(rename = "lv")]
            last_valid: u64,
            #[serde(rename = "gh")]
            genesis_hash: Vec<u8>,
            #[serde(rename = "rcv")]
            receiver: &'a [u8],
            #[serde(rename = "amt")]
            amount: u64,
        }

        let tx = SignedTx {
            txn: PaymentTx {
                tx_type: "pay",
                sender,
                fee,
                first_valid: 1000,
                last_valid: 2000,
                genesis_hash: vec![0u8; 32],
                receiver,
                amount,
            },
        };

        rmp_serde::to_vec(&tx).unwrap_or_default()
    }
}
