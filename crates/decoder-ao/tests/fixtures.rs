//! Fixture generator and tests for AO messages

use std::fs;
use std::path::Path;

/// Generate all test fixtures
fn generate_all_fixtures() {
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

    // Signature (65 bytes)
    let sig = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef, 0x01, 0x23, 0x1b,
    ];
    bytes.extend_from_slice(&sig);

    // Owner (65 bytes)
    let mut owner = vec![0x04];
    owner.extend_from_slice(&[0xAB; 32]);
    owner.extend_from_slice(&[0xCD; 32]);
    bytes.extend_from_slice(&owner);

    // Target present
    bytes.push(1);
    let target = [
        0x5a, 0x7b, 0x8c, 0x9d, 0xae, 0xbf, 0xc0, 0xd1, 0xe2, 0xf3, 0x04, 0x15, 0x26, 0x37, 0x48,
        0x59, 0x6a, 0x7b, 0x8c, 0x9d, 0xae, 0xbf, 0xc0, 0xd1, 0xe2, 0xf3, 0x04, 0x15, 0x26, 0x37,
        0x48, 0x59,
    ];
    bytes.extend_from_slice(&target);

    // No anchor
    bytes.push(0);

    // 2 tags
    bytes.extend_from_slice(&2u64.to_be_bytes());

    let mut tag_bytes = Vec::new();
    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"Eval");

    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"ao");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    let data = b"return { result = math.random(1, 100) }";
    bytes.extend_from_slice(data);

    fs::write(dir.join("ao_message_eth_eval.bin"), bytes).unwrap();
}

fn generate_eth_transfer_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&3u16.to_be_bytes());
    bytes.extend_from_slice(&[0x99; 65]);
    bytes.extend_from_slice(&[0x88; 65]);

    bytes.push(1);
    bytes.extend_from_slice(&[0x77; 32]);
    bytes.push(0);

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

    bytes.extend_from_slice(b"{\"recipient\":\"process_xyz\",\"amount\":1000}");

    fs::write(dir.join("ao_message_eth_transfer.bin"), bytes).unwrap();
}

fn generate_solana_spawn_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0xED; 64]);
    bytes.extend_from_slice(&[0x25; 32]);

    bytes.push(1);
    bytes.extend_from_slice(&[0x50; 32]);

    bytes.push(1);
    bytes.extend_from_slice(&[0xAA; 32]);

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

    bytes.extend_from_slice(
        b"-- Lua process code\nHandlers.add('ping', function(msg) return 'pong' end)",
    );

    fs::write(dir.join("ao_message_solana_spawn.bin"), bytes).unwrap();
}

fn generate_solana_minimal_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0x00; 64]);
    bytes.extend_from_slice(&[0x11; 32]);

    bytes.push(0);
    bytes.push(0);

    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(&0u64.to_be_bytes());

    bytes.extend_from_slice(b"ping");

    fs::write(dir.join("ao_message_solana_minimal.bin"), bytes).unwrap();
}

fn generate_message_with_anchor(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&3u16.to_be_bytes());
    bytes.extend_from_slice(&[0xAA; 65]);
    bytes.extend_from_slice(&[0xBB; 65]);

    bytes.push(1);
    bytes.extend_from_slice(&[0xCC; 32]);

    bytes.push(1);
    let anchor = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01,
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

fn generate_multi_tag_message(dir: &Path) {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0xDD; 64]);
    bytes.extend_from_slice(&[0xEE; 32]);

    bytes.push(1);
    bytes.extend_from_slice(&[0xFF; 32]);
    bytes.push(0);

    bytes.extend_from_slice(&5u64.to_be_bytes());

    let mut tag_bytes = Vec::new();

    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(8);
    tag_bytes.extend_from_slice(b"Transfer");

    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"From");
    tag_bytes.push(10);
    tag_bytes.extend_from_slice(b"user_alice");

    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"To");
    tag_bytes.push(8);
    tag_bytes.extend_from_slice(b"user_bob");

    tag_bytes.push(6);
    tag_bytes.extend_from_slice(b"Amount");
    tag_bytes.push(4);
    tag_bytes.extend_from_slice(b"5000");

    tag_bytes.push(13);
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2);
    tag_bytes.extend_from_slice(b"ao");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    bytes.extend_from_slice(b"{\"memo\":\"payment for services\"}");

    fs::write(dir.join("ao_message_multi_tags.bin"), bytes).unwrap();
}

#[test]
#[ignore]
fn generate_fixtures() {
    generate_all_fixtures();
}
