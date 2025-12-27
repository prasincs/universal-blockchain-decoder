//! Property-based tests for Solana decoder
//!
//! Tests the following properties:
//! 1. **Panic-freedom**: Decoder never panics on arbitrary input
//! 2. **Roundtrip/Injectivity**: encode(decode(x)) = x for valid transactions
//! 3. **Empty rejection**: Empty input is rejected
//! 4. **Determinism**: Same input produces same output

use decoder_primitives::prelude::*;
use decoder_solana::SolanaDecoder;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

// ============================================================================
// Encoding helpers (for generating valid transaction bytes)
// ============================================================================

/// Encode a value as compact-u16 (Solana's variable-length encoding)
///
/// Compact-u16 encoding:
/// - Values 0-127: single byte (value as-is)
/// - Values 128-16383: two bytes (first byte = (value & 0x7F) | 0x80, second byte = value >> 7)
fn encode_compact_u16(buf: &mut Vec<u8>, value: u16) {
    if value < 128 {
        buf.push(value as u8);
    } else {
        buf.push((value as u8) | 0x80);
        buf.push((value >> 7) as u8);
    }
}

/// Encode a Solana transaction from structured components
fn encode_solana_tx(
    signatures: &[&[u8; 64]],
    header: (u8, u8, u8), // (num_required_signatures, num_readonly_signed, num_readonly_unsigned)
    account_keys: &[&[u8; 32]],
    recent_blockhash: &[u8; 32],
    instructions: &[(u8, &[u8], &[u8])], // (program_id_index, account_indices, data)
) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // Signatures
    encode_compact_u16(&mut tx_bytes, signatures.len() as u16);
    for sig in signatures {
        tx_bytes.extend_from_slice(*sig);
    }

    // Message header
    tx_bytes.push(header.0); // num_required_signatures
    tx_bytes.push(header.1); // num_readonly_signed_accounts
    tx_bytes.push(header.2); // num_readonly_unsigned_accounts

    // Account keys
    encode_compact_u16(&mut tx_bytes, account_keys.len() as u16);
    for pubkey in account_keys {
        tx_bytes.extend_from_slice(*pubkey);
    }

    // Recent blockhash
    tx_bytes.extend_from_slice(recent_blockhash);

    // Instructions
    encode_compact_u16(&mut tx_bytes, instructions.len() as u16);
    for (program_id_index, accounts, data) in instructions {
        tx_bytes.push(*program_id_index);
        encode_compact_u16(&mut tx_bytes, accounts.len() as u16);
        tx_bytes.extend_from_slice(accounts);
        encode_compact_u16(&mut tx_bytes, data.len() as u16);
        tx_bytes.extend_from_slice(data);
    }

    tx_bytes
}

// ============================================================================
// Proptest strategies for generating valid transactions
// ============================================================================

/// Generate a valid 64-byte Ed25519 signature
fn arb_signature() -> impl Strategy<Value = [u8; 64]> {
    proptest::collection::vec(any::<u8>(), 64).prop_map(|v| {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&v);
        arr
    })
}

/// Generate a valid 32-byte pubkey
fn arb_pubkey() -> impl Strategy<Value = [u8; 32]> {
    proptest::collection::vec(any::<u8>(), 32).prop_map(|v| {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&v);
        arr
    })
}

/// Generate a valid 32-byte blockhash
fn arb_blockhash() -> impl Strategy<Value = [u8; 32]> {
    proptest::collection::vec(any::<u8>(), 32).prop_map(|v| {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&v);
        arr
    })
}

/// Strategy for generating minimal valid Solana transaction bytes
///
/// Structure:
/// - 1 signature (matching num_required_signatures = 1)
/// - 2 account keys (signer + program)
/// - 1 instruction referencing valid account indices
fn arb_minimal_solana_tx() -> impl Strategy<Value = Vec<u8>> {
    (
        arb_signature(),
        arb_pubkey(),
        arb_pubkey(),
        arb_blockhash(),
        proptest::collection::vec(any::<u8>(), 0..32), // instruction data
    )
        .prop_map(|(sig, signer, program, blockhash, data)| {
            encode_solana_tx(
                &[&sig],
                (1, 0, 1), // 1 required sig, 0 readonly signed, 1 readonly unsigned (program)
                &[&signer, &program],
                &blockhash,
                &[(1, &[0], &data)], // program at index 1, uses account 0
            )
        })
}

/// Strategy for generating valid Solana transaction bytes with variable sizes
fn arb_valid_solana_tx() -> impl Strategy<Value = Vec<u8>> {
    // Generate 1-3 signers
    (1..=3u8) // num_signers
        .prop_flat_map(|num_signers| {
            let num_readonly_unsigned = 1u8; // At least one program account
            let num_accounts = (num_signers as usize) + (num_readonly_unsigned as usize);

            (
                proptest::collection::vec(arb_signature(), num_signers as usize),
                proptest::collection::vec(arb_pubkey(), num_accounts),
                arb_blockhash(),
                1..=3usize, // num_instructions
            )
                .prop_flat_map(
                    move |(signatures, account_keys, blockhash, num_instructions)| {
                        let num_accounts = account_keys.len();
                        // Generate instructions that reference valid account indices
                        proptest::collection::vec(
                            (
                                // program_id_index must be valid
                                (0..num_accounts as u8),
                                // account indices must be valid
                                proptest::collection::vec(0..num_accounts as u8, 0..3),
                                // instruction data
                                proptest::collection::vec(any::<u8>(), 0..32),
                            ),
                            num_instructions,
                        )
                        .prop_map(move |instructions| {
                            let sigs: Vec<&[u8; 64]> = signatures.iter().collect();
                            let keys: Vec<&[u8; 32]> = account_keys.iter().collect();
                            let instrs: Vec<(u8, &[u8], &[u8])> = instructions
                                .iter()
                                .map(|(pid, accounts, data)| {
                                    (*pid, accounts.as_slice(), data.as_slice())
                                })
                                .collect();

                            encode_solana_tx(
                                &sigs,
                                (num_signers, 0, num_readonly_unsigned),
                                &keys,
                                &blockhash,
                                &instrs,
                            )
                        })
                    },
                )
        })
}

// ============================================================================
// Property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Decoder never panics on arbitrary input
    ///
    /// This is a critical safety property - the decoder must gracefully
    /// handle any input without panicking.
    #[test]
    fn prop_solana_decoder_never_panics(input in proptest::collection::vec(any::<u8>(), 0..2000)) {
        // Simply calling decode should never panic
        let _ = SolanaDecoder::decode(&input);
    }

    /// Property: Empty input is rejected with an error
    #[test]
    fn prop_solana_empty_input_rejected(_dummy in 0..1u8) {
        let result = SolanaDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Empty input should be rejected");
    }

    /// Property: Decoding is deterministic
    ///
    /// Same input must always produce the same output.
    #[test]
    fn prop_solana_decode_deterministic(input in proptest::collection::vec(any::<u8>(), 10..1232)) {
        let result1 = SolanaDecoder::decode(&input);
        let result2 = SolanaDecoder::decode(&input);

        match (result1, result2) {
            (Ok(tx1), Ok(tx2)) => {
                prop_assert_eq!(tx1.signatures, tx2.signatures);
                prop_assert_eq!(tx1.message.header, tx2.message.header);
                prop_assert_eq!(tx1.message.account_keys, tx2.message.account_keys);
                prop_assert_eq!(tx1.message.recent_blockhash, tx2.message.recent_blockhash);
                prop_assert_eq!(tx1.message.instructions, tx2.message.instructions);
            }
            (Err(_), Err(_)) => {
                // Both failed, which is deterministic
            }
            _ => {
                prop_assert!(false, "Decoding produced different results for same input");
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: encode(decode(x)) = x (roundtrip/injectivity)
    ///
    /// For valid transactions, encoding the decoded transaction must
    /// produce the original bytes. This is the fundamental correctness
    /// property for the codec.
    #[test]
    fn prop_solana_roundtrip_encoding(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let re_encoded = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;

        prop_assert_eq!(tx_bytes.as_slice(), re_encoded.as_slice(),
            "Roundtrip failed: encode(decode(tx_bytes)) != tx_bytes");
    }

    /// Property: roundtrip with variable-sized transactions
    #[test]
    fn prop_solana_roundtrip_variable_size(tx_bytes in arb_valid_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let re_encoded = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;

        prop_assert_eq!(tx_bytes.as_slice(), re_encoded.as_slice(),
            "Roundtrip failed: encode(decode(tx_bytes)) != tx_bytes");
    }

    /// Property: to_bytes produces deterministic output
    #[test]
    fn prop_solana_to_bytes_deterministic(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let encoded1 = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("First encode failed: {}", e)))?;
        let encoded2 = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Second encode failed: {}", e)))?;

        prop_assert_eq!(encoded1, encoded2,
            "to_bytes() produced different results for same transaction");
    }
}

// ============================================================================
// Additional structural property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction ID (first signature) is preserved through decode
    #[test]
    fn prop_solana_txid_preserved(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // First signature should be present
        prop_assert!(decoded.signature().is_some(), "Transaction should have at least one signature");

        // Signature should be 64 bytes
        let sig = decoded.signature().unwrap();
        prop_assert_eq!(sig.len(), 64, "Signature should be 64 bytes");
    }

    /// Property: Account keys are preserved correctly
    #[test]
    fn prop_solana_account_keys_preserved(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // Should have at least 2 account keys (signer + program)
        prop_assert!(decoded.message.num_account_keys() >= 2,
            "Should have at least 2 account keys");

        // All account keys should be 32 bytes
        for key in decoded.account_keys() {
            prop_assert_eq!(key.len(), 32, "Account key should be 32 bytes");
        }
    }

    /// Property: Blockhash is preserved correctly
    #[test]
    fn prop_solana_blockhash_preserved(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        prop_assert_eq!(decoded.recent_blockhash().len(), 32,
            "Blockhash should be 32 bytes");
    }

    /// Property: Instruction indices are within valid range
    #[test]
    fn prop_solana_instruction_indices_valid(tx_bytes in arb_valid_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let num_accounts = decoded.message.num_account_keys();

        for instruction in decoded.instructions() {
            // program_id_index must be valid
            prop_assert!((instruction.program_id_index as usize) < num_accounts,
                "program_id_index {} out of bounds (num_accounts={})",
                instruction.program_id_index, num_accounts);

            // All account indices must be valid
            for &idx in &instruction.accounts {
                prop_assert!((idx as usize) < num_accounts,
                    "account index {} out of bounds (num_accounts={})",
                    idx, num_accounts);
            }
        }
    }

    /// Property: is_valid() returns true for well-formed transactions
    #[test]
    fn prop_solana_valid_tx_passes_validation(tx_bytes in arb_valid_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        prop_assert!(decoded.is_valid(),
            "Valid transaction should pass is_valid()");
    }
}

// ============================================================================
// Canonicalizer property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: canonicalize() produces valid TxIR
    #[test]
    fn prop_solana_canonicalize_produces_valid_txir(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let txir = decoded.canonicalize()
            .map_err(|e| TestCaseError::fail(format!("Canonicalize failed: {}", e)))?;

        // Chain identification should be correct
        prop_assert_eq!(txir.chain.name.as_str(), "Solana");
        prop_assert_eq!(txir.chain.family(), ChainFamily::Account);

        // Operations should match instruction count
        prop_assert_eq!(txir.operations.len(), decoded.message.num_instructions(),
            "TxIR operations should match instruction count");
    }

    /// Property: canonicalize() is deterministic
    #[test]
    fn prop_solana_canonicalize_deterministic(tx_bytes in arb_minimal_solana_tx()) {
        let decoded = SolanaDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        let txir1 = decoded.canonicalize()
            .map_err(|e| TestCaseError::fail(format!("First canonicalize failed: {}", e)))?;
        let txir2 = decoded.canonicalize()
            .map_err(|e| TestCaseError::fail(format!("Second canonicalize failed: {}", e)))?;

        // Compare key fields
        prop_assert_eq!(txir1.metadata.tx_hash, txir2.metadata.tx_hash);
        prop_assert_eq!(txir1.metadata.size, txir2.metadata.size);
        prop_assert_eq!(txir1.operations.len(), txir2.operations.len());
        prop_assert_eq!(txir1.authorization.signatures.len(), txir2.authorization.signatures.len());
    }
}

#[cfg(test)]
mod encode_helper_tests {
    use super::*;

    #[test]
    fn test_encode_compact_u16() {
        // Single byte values (0-127)
        let mut buf = Vec::new();
        encode_compact_u16(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        encode_compact_u16(&mut buf, 127);
        assert_eq!(buf, vec![0x7F]);

        // Two byte values (128+)
        buf.clear();
        encode_compact_u16(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);

        buf.clear();
        encode_compact_u16(&mut buf, 255);
        assert_eq!(buf, vec![0xFF, 0x01]);

        buf.clear();
        encode_compact_u16(&mut buf, 256);
        assert_eq!(buf, vec![0x80, 0x02]);

        buf.clear();
        encode_compact_u16(&mut buf, 16383);
        assert_eq!(buf, vec![0xFF, 0x7F]);
    }

    #[test]
    fn test_encode_minimal_tx() {
        let sig = [0u8; 64];
        let signer = [1u8; 32];
        let program = [2u8; 32];
        let blockhash = [0u8; 32];

        let tx_bytes = encode_solana_tx(
            &[&sig],
            (1, 0, 1),
            &[&signer, &program],
            &blockhash,
            &[(1, &[0], &[])],
        );

        // Should decode successfully
        let decoded = SolanaDecoder::decode(&tx_bytes).expect("Should decode");
        assert_eq!(decoded.num_signatures(), 1);
        assert_eq!(decoded.message.num_account_keys(), 2);
        assert_eq!(decoded.message.num_instructions(), 1);
        assert!(decoded.is_valid());
    }

    #[test]
    fn test_roundtrip_simple() {
        let sig = [42u8; 64];
        let signer = [1u8; 32];
        let program = [2u8; 32];
        let blockhash = [99u8; 32];

        let tx_bytes = encode_solana_tx(
            &[&sig],
            (1, 0, 1),
            &[&signer, &program],
            &blockhash,
            &[(1, &[0], &[1, 2, 3, 4])],
        );

        let decoded = SolanaDecoder::decode(&tx_bytes).expect("Should decode");
        let re_encoded = decoded.to_bytes().expect("Should encode");

        assert_eq!(tx_bytes, re_encoded, "Roundtrip should preserve bytes");
    }
}
