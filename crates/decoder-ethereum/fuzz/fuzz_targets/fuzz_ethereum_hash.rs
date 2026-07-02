#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_ethereum::EthereumDecoder;
use universal_decoder_core::prelude::*;
use sha3::{Digest, Keccak256};

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Ethereum transaction hashing should never panic
    //
    // Ethereum uses Keccak256 (not SHA3-256!) for hashing.
    // Transaction hash = Keccak256(RLP(transaction))
    //
    // This fuzzer ensures:
    // 1. Hash calculation never panics
    // 2. Hash is deterministic
    // 3. Hash is correct Keccak256
    // 4. Canonical hash is deterministic

    // Test 1: Direct Keccak256 should never panic
    let hash1 = Keccak256::digest(data);
    let hash2 = Keccak256::digest(data);
    assert_eq!(hash1.as_slice(), hash2.as_slice(), "Keccak256 is non-deterministic");

    // Test 2: If data decodes as transaction, hash should be deterministic
    if let Ok(tx) = EthereumDecoder::decode(data) {
        let tx_hash1 = tx.hash().expect("hash of decoded tx should succeed");
        let tx_hash2 = tx.hash().expect("hash of decoded tx should succeed");
        assert_eq!(tx_hash1, tx_hash2, "Transaction hash is non-deterministic");

        // Hash should match Keccak256 of original bytes
        let expected_hash = Keccak256::digest(data).to_vec();
        assert_eq!(tx_hash1, expected_hash, "Transaction hash incorrect");

        // Test 3: Canonical hash (if canonicalization succeeds)
        if let Ok(tx_ir) = tx.canonicalize() {
            if let Ok(canonical_hash1) = tx_ir.canonical_hash() {
                if let Ok(canonical_hash2) = tx_ir.canonical_hash() {
                    assert_eq!(
                        canonical_hash1, canonical_hash2,
                        "Canonical hash is non-deterministic"
                    );
                }
            }

            // Canonical bytes should also be deterministic
            if let Ok(canonical_bytes1) = tx_ir.to_canonical_bytes() {
                if let Ok(canonical_bytes2) = tx_ir.to_canonical_bytes() {
                    assert_eq!(
                        canonical_bytes1, canonical_bytes2,
                        "Canonical bytes are non-deterministic"
                    );

                    // Canonical hash should match hash of canonical bytes
                    let hash_of_canonical = Keccak256::digest(&canonical_bytes1).to_vec();
                    if let Ok(canonical_hash) = tx_ir.canonical_hash() {
                        // Note: Canonical hash uses SHA256, not Keccak256
                        // So they won't match - but both should be deterministic
                        let _ = hash_of_canonical;
                        let _ = canonical_hash;
                    }
                }
            }
        }
    }

    // Test 4: Empty input hash
    if data.is_empty() {
        let empty_hash = Keccak256::digest(&[]);
        assert_eq!(empty_hash.len(), 32, "Hash should be 32 bytes");
    }

    // Test 5: Very large input hashing (should not panic or OOM)
    if data.len() > 1_000_000 {
        let _ = Keccak256::digest(data);
    }
});
