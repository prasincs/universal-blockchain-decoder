//! Fixture generator for AO messages
//!
//! This module generates realistic ANS-104 DataItems for testing.
//! Run with: cargo test --test fixtures -- --nocapture --ignored

use std::fs;
use std::path::Path;

/// Generate all test fixtures
pub fn generate_all_fixtures() {
    let fixtures_dir = Path::new("tests/fixtures");

    // Create directory if it doesn't exist
    fs::create_dir_all(fixtures_dir).unwrap();

    // Generate fixtures
    generate_eth_eval_message(fixtures_dir);
    generate_eth_transfer_message(fixtures_dir);
    generate_solana_spawn_message(fixtures_dir);
    generate_solana_minimal_message(fixtures_dir);
    generate_message_with_anchor(fixtures_dir);
    generate_multi_tag_message(fixtures_dir);

    println!("Generated all fixtures in tests/fixtures/");
}

/// Generate Ethereum signature Eval message
fn generate_eth_eval_message(dir: &Path) {
    let mut bytes = Vec::new();

    // Signature type: Ethereum (3)
    bytes.extend_from_slice(&3u16.to_be_bytes());

    // Signature (65 bytes) - realistic Ethereum signature pattern
    let sig = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x01, 0x23,
        0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
        0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
        0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
        0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
        0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
        0x1b // recovery byte
    ];
    bytes.extend_from_slice(&sig);

    // Owner (65 bytes) - Ethereum public key
    let owner = [0x04].iter()
        .chain(&[0xAB; 32])
        .chain(&[0xCD; 32])
        .copied()
        .collect::<Vec<u8>>();
    bytes.extend_from_slice(&owner);

    // Target present = 1
    bytes.push(1);
    // Target: realistic process ID
    let target = [
        0x5a, 0x7b, 0x8c, 0x9d, 0xae, 0xbf, 0xc0, 0xd1,
        0xe2, 0xf3, 0x04, 0x15, 0x26, 0x37, 0x48, 0x59,
        0x6a, 0x7b, 0x8c, 0x9d, 0xae, 0xbf, 0xc0, 0xd1,
        0xe2, 0xf3, 0x04, 0x15, 0x26, 0x37, 0x48, 0x59,
    ];
    bytes.extend_from_slice(&target);

    // No anchor
    bytes.push(0);

    // Tags: Action=Eval, Data-Protocol=ao
    bytes.extend_from_slice(&2u64.to_be_bytes());

    let mut tag_bytes = Vec::new();
    // Tag 1: Action = Eval
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"Eval");

    // Tag 2: Data-Protocol = ao
    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"ao");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    // Data: Lua code
    let data = b"return { result = math.random(1, 100) }";
    bytes.extend_from_slice(data);

    fs::write(dir.join("ao_message_eth_eval.bin"), bytes).unwrap();
}

/// Generate Ethereum signature Transfer message
fn generate_eth_transfer_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&3u16.to_be_bytes());
    bytes.extend_from_slice(&[0x99; 65]); // Signature
    bytes.extend_from_slice(&[0x88; 65]); // Owner

    // Target present
    bytes.push(1);
    bytes.extend_from_slice(&[0x77; 32]);

    // No anchor
    bytes.push(0);

    // Tags: Action=Transfer, Amount=1000
    bytes.extend_from_slice(&2u64.to_be_bytes());

    let mut tag_bytes = Vec::new();
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(8);
    tag_bytes.extend_from_slice(b"Transfer");

    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Amount");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"1000");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    let data = b"{\"recipient\":\"process_xyz\",\"amount\":1000}";
    bytes.extend_from_slice(data);

    fs::write(dir.join("ao_message_eth_transfer.bin"), bytes).unwrap();
}

/// Generate Solana signature Spawn-Process message
fn generate_solana_spawn_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes()); // Solana
    bytes.extend_from_slice(&[0xED; 64]); // Ed25519 signature
    bytes.extend_from_slice(&[0x25; 32]); // Ed25519 public key

    // Target present
    bytes.push(1);
    bytes.extend_from_slice(&[0x50; 32]);

    // Anchor present (replay protection)
    bytes.push(1);
    bytes.extend_from_slice(&[0xAN; 32]);

    // Tags: Action=Spawn-Process, Data-Protocol=ao, Type=Process
    bytes.extend_from_slice(&3u64.to_be_bytes());

    let mut tag_bytes = Vec::new();
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Spawn-Process");

    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"ao");

    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"Type");
    tag_bytes.push(7);
    tag_bytes.extend_from_slice(b"Process");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    let data = b"-- Lua process code\nHandlers.add('ping', function(msg) return 'pong' end)";
    bytes.extend_from_slice(data);

    fs::write(dir.join("ao_message_solana_spawn.bin"), bytes).unwrap();
}

/// Generate minimal Solana message
fn generate_solana_minimal_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0x00; 64]);
    bytes.extend_from_slice(&[0x11; 32]);

    // No target
    bytes.push(0);

    // No anchor
    bytes.push(0);

    // No tags
    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(&0u64.to_be_bytes());

    let data = b"ping";
    bytes.extend_from_slice(data);

    fs::write(dir.join("ao_message_solana_minimal.bin"), bytes).unwrap();
}

/// Generate message with anchor
fn generate_message_with_anchor(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&3u16.to_be_bytes());
    bytes.extend_from_slice(&[0xAA; 65]);
    bytes.extend_from_slice(&[0xBB; 65]);

    bytes.push(1);
    bytes.extend_from_slice(&[0xCC; 32]);

    // Anchor present
    bytes.push(1);
    let anchor = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
        0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
        0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
    ];
    bytes.extend_from_slice(&anchor);

    bytes.extend_from_slice(&1u64.to_be_bytes());

    let mut tag_bytes = Vec::new();
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"Eval");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    bytes.extend_from_slice(b"return 42");

    fs::write(dir.join("ao_message_with_anchor.bin"), bytes).unwrap();
}

/// Generate message with multiple tags
fn generate_multi_tag_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0xDD; 64]);
    bytes.extend_from_slice(&[0xEE; 32]);

    bytes.push(1);
    bytes.extend_from_slice(&[0xFF; 32]);

    bytes.push(0);

    // 5 tags
    bytes.extend_from_slice(&5u64.to_be_bytes());

    let mut tag_bytes = Vec::new();

    // Action
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(8);
    tag_bytes.extend_from_slice(b"Transfer");

    // From
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"From");
    tag_bytes.push(9);
    tag_bytes.extend_from_slice(b"user_alice");

    // To
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"To");
    tag_bytes.push(7);
    tag_bytes.extend_from_slice(b"user_bob");

    // Amount
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Amount");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"5000");

    // Data-Protocol
    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"ao");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    bytes.extend_from_slice(b"{\"memo\":\"payment for services\"}");

    fs::write(dir.join("ao_message_multi_tags.bin"), bytes).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run with: cargo test generate_fixtures -- --ignored
    fn generate_fixtures() {
        generate_all_fixtures();
    }
}
