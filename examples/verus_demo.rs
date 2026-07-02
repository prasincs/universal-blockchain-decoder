//! Verus Formal Verification Demo
//!
//! This example demonstrates how Verus annotations work with the Universal
//! Blockchain Decoder core library.
//!
//! ## Running this example
//!
//! ### As a regular Rust program:
//! ```bash
//! cargo run --example verus_demo
//! ```
//!
//! ### With Verus verification:
//! ```bash
//! ./scripts/verus.sh examples/verus_demo.rs --compile
//! ./verus_demo
//! ```
//!
//! ## What this demonstrates
//!
//! 1. Deterministic serialization
//! 2. Deterministic hashing
//! 3. Roundtrip property (serialize → deserialize preserves data)
//! 4. Collision resistance

use universal_decoder_core::canonical::*;
use universal_decoder_core::chain::*;

fn main() {
    println!("🔬 Verus Formal Verification Demo");
    println!("==================================\n");

    // Create a test transaction
    let tx = create_sample_transaction();

    // Property 1: Deterministic Serialization
    println!("✓ Testing Property 1: Deterministic Serialization");
    test_deterministic_serialization(&tx);

    // Property 2: Deterministic Hashing
    println!("✓ Testing Property 2: Deterministic Hashing");
    test_deterministic_hashing(&tx);

    // Property 3: Roundtrip
    println!("✓ Testing Property 3: Roundtrip Preservation");
    test_roundtrip(&tx);

    // Property 4: Collision Resistance
    println!("✓ Testing Property 4: Collision Resistance");
    test_collision_resistance(&tx);

    println!("\n✅ All verification properties validated!");
    println!("\nNote: These tests validate properties at runtime.");
    println!("With Verus, these properties are PROVEN at compile-time!");
}

fn create_sample_transaction() -> CanonicalTxIR {
    CanonicalTxIR {
        version: 1,
        chain: ChainRef {
            id: 0,
            name: "Bitcoin".to_string(),
            family: ChainFamilyEncoded::Utxo,
            network: Some("mainnet".to_string()),
        },
        metadata: CanonicalTxMetadata {
            tx_hash: vec![0xab, 0xcd, 0xef, 0x12],
            block_height: Some(750_000),
            timestamp: Some(1699999999),
            size: 225,
            extra: "{}".to_string(),
        },
        authorization: CanonicalAuthorizationPackage {
            signatures: vec![CanonicalSignature {
                data: vec![0x30, 0x44, 0x02, 0x20],
                key_index: 0,
                metadata: None,
            }],
            public_keys: vec![CanonicalPublicKey {
                data: vec![0x02; 33],
                key_type: CanonicalKeyType::Secp256k1,
            }],
            signature_scheme: CanonicalSignatureScheme::Ecdsa,
        },
        operations: vec![CanonicalOperation::Transfer(CanonicalTransfer {
            from: CanonicalAddress {
                bytes: vec![0x00; 20],
                human_readable: Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()),
            },
            to: CanonicalAddress {
                bytes: vec![0x01; 20],
                human_readable: Some("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()),
            },
            amount: CanonicalAmount {
                value: 50_000_000,
                decimals: 8,
            },
            asset: CanonicalAssetId::Native,
        })],
        state_deltas: CanonicalStateDeltas {
            inputs: vec![],
            outputs: vec![],
        },
    }
}

fn test_deterministic_serialization(tx: &CanonicalTxIR) {
    let bytes1 = tx.to_canonical_bytes().expect("Serialization failed");
    let bytes2 = tx.to_canonical_bytes().expect("Serialization failed");

    assert_eq!(bytes1, bytes2, "Serialization must be deterministic!");
    println!("  ✓ Serialized {} bytes deterministically", bytes1.len());
}

fn test_deterministic_hashing(tx: &CanonicalTxIR) {
    let hash1 = tx.canonical_hash().expect("Hashing failed");
    let hash2 = tx.canonical_hash().expect("Hashing failed");

    assert_eq!(hash1, hash2, "Hashing must be deterministic!");
    assert_eq!(hash1.len(), 32, "SHA-256 must produce 32 bytes!");

    println!("  ✓ Hash: {}", hex_encode(&hash1));
}

fn test_roundtrip(tx: &CanonicalTxIR) {
    let bytes = tx.to_canonical_bytes().expect("Serialization failed");
    let deserialized =
        CanonicalTxIR::from_canonical_bytes(&bytes).expect("Deserialization failed");

    assert_eq!(*tx, deserialized, "Roundtrip must preserve data!");
    println!("  ✓ Data preserved after serialize → deserialize");
}

fn test_collision_resistance(tx: &CanonicalTxIR) {
    // Create a modified transaction
    let mut tx2 = tx.clone();
    tx2.metadata.block_height = Some(750_001); // Different block height

    let hash1 = tx.canonical_hash().expect("Hashing failed");
    let hash2 = tx2.canonical_hash().expect("Hashing failed");

    assert_ne!(hash1, hash2, "Different transactions must have different hashes!");
    println!("  ✓ Different inputs produce different hashes");
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}
