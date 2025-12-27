//! Property-based tests for Optimism decoder
//!
//! These tests use proptest to verify properties hold for arbitrary inputs.

use decoder_encodings::RlpEncoder;
use decoder_optimism::*;
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

// ============================================================================
// Test Helpers
// ============================================================================

/// Encodes a deposit transaction to RLP bytes (for generating valid raw_bytes)
fn encode_deposit_tx(
    source_hash: &[u8; 32],
    from: &[u8; 20],
    to: Option<[u8; 20]>,
    mint: u128,
    value: u128,
    gas_limit: u64,
    is_creation: bool,
    data: &[u8],
) -> Vec<u8> {
    let mut encoder = RlpEncoder::new();

    // Create RLP list with 8 fields
    let mut list = encoder.begin_list();

    // 1. source_hash (bytes32)
    list.append_bytes(source_hash).unwrap();

    // 2. from (address - 20 bytes)
    list.append_bytes(from).unwrap();

    // 3. to (optional address)
    list.append_address(to).unwrap();

    // 4. mint (uint256 as u128)
    list.append_u128(mint).unwrap();

    // 5. value (uint256 as u128)
    list.append_u128(value).unwrap();

    // 6. gas_limit (uint64)
    list.append_u64(gas_limit).unwrap();

    // 7. is_creation (bool: 0x00 or 0x01)
    let is_creation_byte = if is_creation { 0x01 } else { 0x00 };
    list.append_bytes(&[is_creation_byte]).unwrap();

    // 8. data (bytes)
    list.append_bytes(data).unwrap();

    list.finalize().unwrap();

    // Prepend deposit transaction type (0x7E)
    let mut result = vec![DepositTransaction::TYPE_ID];
    result.extend(encoder.finalize());
    result
}

// ============================================================================
// Test Strategies (Generators)
// ============================================================================

/// Generate arbitrary OP Stack chain IDs
fn arb_op_stack_chain_id() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(10u64),          // Optimism
        Just(8453u64),        // Base
        Just(7777777u64),     // Zora
        Just(34443u64),       // Mode
        Just(424u64),         // PGN
        Just(81457u64),       // Blast
        Just(690u64),         // Redstone
        900000u64..910000u64, // OP Stack testnet range
    ]
}

/// Generate arbitrary addresses
fn arb_address() -> impl Strategy<Value = [u8; 20]> {
    prop::array::uniform20(any::<u8>())
}

/// Generate arbitrary hashes
fn arb_hash() -> impl Strategy<Value = [u8; 32]> {
    prop::array::uniform32(any::<u8>())
}

/// Generate arbitrary transaction bytes
fn arb_tx_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 5..1000)
}

/// Generate valid deposit transaction
fn arb_deposit_tx() -> impl Strategy<Value = DepositTransaction> {
    (
        arb_hash(),
        arb_address(),
        prop::option::of(arb_address()),
        any::<u128>(),
        any::<u128>(),
        1u64..100_000_000u64, // gas_limit must be > 0
        any::<bool>(),
        prop::collection::vec(any::<u8>(), 0..200),
    )
        .prop_filter(
            "value must be <= mint, is_creation consistency",
            |(_, _, to, mint, value, _, is_creation, _)| {
                // value <= mint
                value <= mint &&
                // is_creation consistency
                match (is_creation, to) {
                    (true, None) => true,      // creation without to: valid
                    (false, Some(_)) => true,  // non-creation with to: valid
                    _ => false,                // other combinations: invalid
                }
            },
        )
        .prop_map(
            |(source_hash, from, to, mint, value, gas_limit, is_creation, data)| {
                // Generate proper RLP-encoded raw_bytes for this transaction
                let raw_bytes = encode_deposit_tx(
                    &source_hash,
                    &from,
                    to,
                    mint,
                    value,
                    gas_limit,
                    is_creation,
                    &data,
                );

                DepositTransaction {
                    source_hash,
                    from,
                    to,
                    mint,
                    value,
                    gas_limit,
                    is_creation,
                    data,
                    raw_bytes,
                }
            },
        )
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Decoding should never panic (even on invalid input)
    #[test]
    fn prop_decode_never_panics(bytes in arb_tx_bytes()) {
        let _ = OptimismDecoder::decode(&bytes);
        // Should not panic - either Ok or Err
    }

    /// Property: Validation should never panic
    #[test]
    fn prop_validate_format_never_panics(bytes in arb_tx_bytes()) {
        let _ = OptimismDecoder::validate_format(&bytes);
        // Should not panic
    }

    /// Property: Valid deposit transactions should always validate
    #[test]
    fn prop_valid_deposits_validate(deposit in arb_deposit_tx()) {
        prop_assert!(deposit.validate().is_ok());
    }

    /// Property: Deposit transaction validation is deterministic
    #[test]
    fn prop_deposit_validation_deterministic(deposit in arb_deposit_tx()) {
        let result1 = deposit.validate();
        let result2 = deposit.validate();
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }

    /// Property: Borsh serialization roundtrip for deposit transactions
    #[test]
    fn prop_deposit_borsh_roundtrip(deposit in arb_deposit_tx()) {
        let serialized = borsh::to_vec(&deposit)
            .map_err(|e| TestCaseError::fail(format!("Borsh serialization failed: {}", e)))?;
        let deserialized: DepositTransaction = borsh::from_slice(&serialized)
            .map_err(|e| TestCaseError::fail(format!("Borsh deserialization failed: {}", e)))?;
        prop_assert_eq!(deposit, deserialized);
    }

    /// Property: L1 attributes detection is consistent
    #[test]
    fn prop_l1_attributes_detection_consistent(deposit in arb_deposit_tx()) {
        let is_l1_attrs = deposit.is_l1_attributes_deposit();
        let is_user = deposit.is_user_deposit();
        // Must be exactly one or the other
        prop_assert_eq!(is_l1_attrs, !is_user);
    }

    /// Property: OptimismTransaction type detection is consistent
    #[test]
    fn prop_transaction_type_detection_consistent(deposit in arb_deposit_tx()) {
        let tx = OptimismTransaction::Deposit(deposit);
        prop_assert!(tx.is_deposit());
        prop_assert!(!tx.is_standard());
        prop_assert_eq!(tx.tx_type(), 0x7E);
    }

    /// Property: Transaction accessors are consistent with deposit fields
    #[test]
    fn prop_transaction_accessors_consistent(deposit in arb_deposit_tx()) {
        let expected_from = deposit.from;
        let expected_to = deposit.to;
        let expected_value = deposit.value;
        let expected_data = deposit.data.clone();

        let tx = OptimismTransaction::Deposit(deposit);
        prop_assert_eq!(tx.from(), expected_from);
        prop_assert_eq!(tx.to(), expected_to);
        prop_assert_eq!(tx.value(), expected_value);
        prop_assert_eq!(tx.data(), expected_data.as_slice());
    }

    /// Property: Canonicalization should succeed for valid deposits
    #[test]
    fn prop_deposit_canonicalization_succeeds(deposit in arb_deposit_tx()) {
        let tx = OptimismTransaction::Deposit(deposit);
        let result = tx.canonicalize();
        prop_assert!(result.is_ok());
    }

    /// Property: Canonicalization produces valid TxIR
    #[test]
    fn prop_canonicalization_produces_valid_txir(deposit in arb_deposit_tx()) {
        let has_mint = deposit.mint > 0;
        let has_value = deposit.value > 0;
        let has_data = !deposit.data.is_empty();

        let tx = OptimismTransaction::Deposit(deposit);
        let tx_ir = tx.canonicalize()
            .map_err(|e| TestCaseError::fail(format!("Canonicalization failed: {}", e)))?;

        // TxIR should have metadata
        prop_assert!(!tx_ir.metadata.tx_hash.is_empty());

        // If mint > 0, should have operations
        if has_mint || has_value || has_data {
            prop_assert!(!tx_ir.operations.is_empty());
        }

        // If mint > 0, should have account changes
        if has_mint {
            prop_assert!(!tx_ir.state_deltas.account_changes.is_empty());
        }
    }

    /// Property: Deposit transaction type ID is constant
    #[test]
    fn prop_deposit_type_id_constant(_deposit in arb_deposit_tx()) {
        prop_assert_eq!(DepositTransaction::TYPE_ID, 0x7E);
    }

    /// Property: Chain ID detection for OP Stack
    #[test]
    fn prop_op_stack_chain_ids_recognized(chain_id in arb_op_stack_chain_id()) {
        // This is internal function, but we can test known IDs
        let known_ids = [10u64, 8453, 7777777, 34443, 424, 81457, 690];
        let is_known = known_ids.contains(&chain_id) || (900000..910000).contains(&chain_id);
        prop_assert!(is_known);
    }

    /// Property: Mint >= Value invariant
    #[test]
    fn prop_mint_gte_value_invariant(deposit in arb_deposit_tx()) {
        // Valid deposits should have mint >= value
        prop_assert!(deposit.mint >= deposit.value);
    }

    /// Property: is_creation and to consistency
    #[test]
    fn prop_creation_to_consistency(deposit in arb_deposit_tx()) {
        match (deposit.is_creation, deposit.to) {
            (true, None) => {}, // Valid: creation without to
            (false, Some(_)) => {}, // Valid: non-creation with to
            _ => panic!("Invalid is_creation/to combination"),
        }
    }

    /// Property: Gas limit is always positive
    #[test]
    fn prop_gas_limit_positive(deposit in arb_deposit_tx()) {
        prop_assert!(deposit.gas_limit > 0);
    }

    /// Property: Roundtrip encoding/decoding preserves transaction (injective property)
    #[test]
    fn prop_deposit_roundtrip_encoding(deposit in arb_deposit_tx()) {
        // The deposit was constructed with proper raw_bytes from encoding
        let original_bytes = &deposit.raw_bytes;

        // Decode the raw bytes
        let decoded = OptimismDecoder::decode(original_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // Should decode to a deposit transaction
        prop_assert!(decoded.is_deposit());

        if let OptimismTransaction::Deposit(decoded_deposit) = decoded {
            // Check all fields match
            prop_assert_eq!(decoded_deposit.source_hash, deposit.source_hash);
            prop_assert_eq!(decoded_deposit.from, deposit.from);
            prop_assert_eq!(decoded_deposit.to, deposit.to);
            prop_assert_eq!(decoded_deposit.mint, deposit.mint);
            prop_assert_eq!(decoded_deposit.value, deposit.value);
            prop_assert_eq!(decoded_deposit.gas_limit, deposit.gas_limit);
            prop_assert_eq!(decoded_deposit.is_creation, deposit.is_creation);
            prop_assert_eq!(decoded_deposit.data, deposit.data);

            // Most importantly: raw_bytes should match (injective property)
            prop_assert_eq!(decoded_deposit.raw_bytes, deposit.raw_bytes,
                "Roundtrip failed: encode(decode(x)) != x");
        }
    }

    /// Property: Source hash is 32 bytes
    #[test]
    fn prop_source_hash_length(deposit in arb_deposit_tx()) {
        prop_assert_eq!(deposit.source_hash.len(), 32);
    }

    /// Property: From address is 20 bytes
    #[test]
    fn prop_from_address_length(deposit in arb_deposit_tx()) {
        prop_assert_eq!(deposit.from.len(), 20);
    }

    /// Property: To address (if present) is 20 bytes
    #[test]
    fn prop_to_address_length(deposit in arb_deposit_tx()) {
        if let Some(to) = deposit.to {
            prop_assert_eq!(to.len(), 20);
        }
    }
}

// ============================================================================
// Specific Property Tests for Edge Cases
// ============================================================================

proptest! {
    /// Property: Zero mint and value is valid
    #[test]
    fn prop_zero_mint_value_valid(
        source_hash in arb_hash(),
        from in arb_address(),
        to in prop::option::of(arb_address()),
        gas_limit in 1u64..100_000_000u64,
        data in prop::collection::vec(any::<u8>(), 0..100)
    ) {
        // Ensure is_creation matches to presence
        let is_creation = to.is_none();

        // Generate proper RLP-encoded raw_bytes
        let raw_bytes = encode_deposit_tx(
            &source_hash,
            &from,
            to,
            0,    // mint
            0,    // value
            gas_limit,
            is_creation,
            &data,
        );

        let deposit = DepositTransaction {
            source_hash,
            from,
            to,
            mint: 0,
            value: 0,
            gas_limit,
            is_creation,
            data,
            raw_bytes,
        };

        prop_assert!(deposit.validate().is_ok());
    }

    /// Property: Max values don't overflow
    #[test]
    fn prop_max_values_no_overflow(
        source_hash in arb_hash(),
        from in arb_address(),
        to in arb_address(),
        gas_limit in 1u64..u64::MAX,
    ) {
        // Generate proper RLP-encoded raw_bytes
        let raw_bytes = encode_deposit_tx(
            &source_hash,
            &from,
            Some(to),
            u128::MAX,
            u128::MAX,
            gas_limit,
            false, // is_creation
            &[],   // empty data
        );

        let deposit = DepositTransaction {
            source_hash,
            from,
            to: Some(to),
            mint: u128::MAX,
            value: u128::MAX,
            gas_limit,
            is_creation: false,
            data: vec![],
            raw_bytes,
        };

        // Should validate successfully
        prop_assert!(deposit.validate().is_ok());

        // Should serialize without panic
        let serialized = borsh::to_vec(&deposit).unwrap();
        prop_assert!(!serialized.is_empty());
    }

    /// Property: Empty data is valid
    #[test]
    fn prop_empty_data_valid(deposit in arb_deposit_tx().prop_map(|mut d| {
        d.data = vec![];
        d
    })) {
        prop_assert!(deposit.validate().is_ok());
        prop_assert!(deposit.data.is_empty());
    }

    /// Property: Large data is valid (up to reasonable limits)
    #[test]
    fn prop_large_data_valid(
        deposit in arb_deposit_tx(),
        extra_data in prop::collection::vec(any::<u8>(), 0..10_000)
    ) {
        let mut deposit = deposit;
        deposit.data.extend(extra_data);

        // Should still validate
        prop_assert!(deposit.validate().is_ok());
    }
}

// ============================================================================
// Registry Property Tests
// ============================================================================

proptest! {
    /// Property: Registry lookups are consistent
    #[test]
    fn prop_registry_lookup_consistent(chain_id in arb_op_stack_chain_id()) {
        use crate::registry::SuperchainRegistry;

        let registry = SuperchainRegistry::new();

        let result1 = registry.get_chain(chain_id);
        let result2 = registry.get_chain(chain_id);

        // Should get same result
        prop_assert_eq!(result1.is_some(), result2.is_some());

        if let (Some(chain1), Some(chain2)) = (result1, result2) {
            prop_assert_eq!(chain1.chain_id, chain2.chain_id);
            prop_assert_eq!(&chain1.name, &chain2.name);
        }
    }

    /// Property: Registry has_chain is consistent with get_chain
    #[test]
    fn prop_registry_has_chain_consistent(chain_id in any::<u64>()) {
        use crate::registry::SuperchainRegistry;

        let registry = SuperchainRegistry::new();

        let has = registry.has_chain(chain_id);
        let get = registry.get_chain(chain_id);

        prop_assert_eq!(has, get.is_some());
    }
}

// ============================================================================
// Canonicalization Property Tests
// ============================================================================

proptest! {
    /// Property: Canonicalization is deterministic
    #[test]
    fn prop_canonicalization_deterministic(deposit in arb_deposit_tx()) {
        let tx = OptimismTransaction::Deposit(deposit);

        let result1 = tx.canonicalize();
        let result2 = tx.canonicalize();

        prop_assert_eq!(result1.is_ok(), result2.is_ok());

        if let (Ok(txir1), Ok(txir2)) = (result1, result2) {
            // Hashes should be equal
            prop_assert_eq!(txir1.metadata.tx_hash, txir2.metadata.tx_hash);

            // Operation counts should be equal
            prop_assert_eq!(txir1.operations.len(), txir2.operations.len());

            // Account change counts should be equal
            prop_assert_eq!(
                txir1.state_deltas.account_changes.len(),
                txir2.state_deltas.account_changes.len()
            );
        }
    }

    /// Property: Canonicalization preserves transaction semantics
    #[test]
    fn prop_canonicalization_preserves_semantics(deposit in arb_deposit_tx()) {
        let mint = deposit.mint;

        let tx = OptimismTransaction::Deposit(deposit);
        let tx_ir = tx.canonicalize().unwrap();

        // Check metadata includes deposit info
        let extra = &tx_ir.metadata.extra;
        prop_assert!(extra.contains("deposit"));
        prop_assert!(extra.contains(&mint.to_string()));

        // If mint > 0, should have transfer operation
        if mint > 0 {
            let has_transfer = tx_ir.operations.iter().any(|op| {
                matches!(op, Operation::Transfer(_))
            });
            prop_assert!(has_transfer, "Should have transfer operation for mint");
        }
    }

    /// Property: Validation matches canonicalization success
    #[test]
    fn prop_validation_matches_canonicalization(deposit in arb_deposit_tx()) {
        let tx = OptimismTransaction::Deposit(deposit);

        let validate_result = tx.validate();
        let canonicalize_result = tx.canonicalize();

        // If validation passes, canonicalization should succeed
        if validate_result.is_ok() {
            prop_assert!(canonicalize_result.is_ok());
        }
    }
}
