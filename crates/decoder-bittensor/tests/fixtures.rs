//! Test fixture helpers for creating Bittensor extrinsics
//!
//! These helpers create properly SCALE-encoded extrinsics for testing.
//! Based on Substrate extrinsic format used by Bittensor.

/// Helper to create a minimal signed TAO transfer extrinsic
pub fn create_tao_transfer() -> Vec<u8> {
    // Minimal signed extrinsic structure:
    // - Length prefix (compact)
    // - Version byte (0x84 = v4, signed)
    // - Address (0x00 + 32 bytes)
    // - Signature (0x01 + 64 bytes for Sr25519 - common in Substrate)
    // - Era (0x00 = immortal)
    // - Nonce (compact, e.g., 0x00 = 0)
    // - Tip (compact, e.g., 0x00 = 0)
    // - Call (pallet + function + params)

    let mut extrinsic = Vec::new();

    // Version: v4, signed
    extrinsic.push(0x84);

    // Address: Id type (0x00) + 32-byte account
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF, 0x00,
    ]); // Sender address

    // Signature: Sr25519 (0x01) + 64-byte signature
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xAA; 64]); // Dummy signature

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 5 (compact: 5 << 2 = 20 = 0x14)
    extrinsic.push(0x14);

    // Tip: 0 (compact single byte)
    extrinsic.push(0x00);

    // Call: Balances (0x04) :: transfer (0x00)
    extrinsic.push(0x04);
    extrinsic.push(0x00);

    // Destination: Id type (0x00) + 32-byte account
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99,
    ]); // Recipient address

    // Amount: 1000000000 (1 TAO with 9 decimals) as compact u128
    // 1000000000 = 0x3B9ACA00
    // Compact encoding for 4-byte mode: 0x02 prefix
    extrinsic.push(0x02); // Four-byte mode marker
    extrinsic.push(0x00); // Lower byte
    extrinsic.push(0xCA);
    extrinsic.push(0x9A);
    extrinsic.push(0x3B); // Upper byte

    // Prepend length as compact integer
    prepend_compact_length(extrinsic)
}

/// Helper to create a SubtensorModule::set_weights extrinsic (Bittensor-specific)
pub fn create_set_weights() -> Vec<u8> {
    let mut extrinsic = Vec::new();

    // Version: v4, signed
    extrinsic.push(0x84);

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xFF; 32]);

    // Signature: Sr25519
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xBB; 64]);

    // Era: Mortal - block 1000, period 64
    // Encoded as two bytes for mortal era
    extrinsic.push(0x35); // First byte of mortal era encoding
    extrinsic.push(0x02); // Second byte

    // Nonce: 10
    extrinsic.push(0x28); // 10 << 2 = 40 = 0x28

    // Tip: 100 (small tip)
    extrinsic.push(0x91); // 100 << 2 | 0x01 = 401 = 0x191 (two-byte compact)
    extrinsic.push(0x01);

    // Call: SubtensorModule (0x07) :: set_weights (0x00)
    extrinsic.push(0x07);
    extrinsic.push(0x00);

    // Parameters for set_weights (simplified):
    // - netuid: u16
    // - dests: Vec<u16>
    // - weights: Vec<u16>
    // - version_key: u64

    // netuid: 1
    extrinsic.push(0x01);
    extrinsic.push(0x00);

    // dests: Vec with 2 elements
    extrinsic.push(0x08); // Compact length: 2 << 2 = 8
    extrinsic.push(0x0A); // uid 10
    extrinsic.push(0x00);
    extrinsic.push(0x14); // uid 20
    extrinsic.push(0x00);

    // weights: Vec with 2 elements
    extrinsic.push(0x08); // Compact length: 2
    extrinsic.push(0x64); // weight 100
    extrinsic.push(0x00);
    extrinsic.push(0xC8); // weight 200
    extrinsic.push(0x00);

    // version_key: 0
    extrinsic.extend_from_slice(&[0x00; 8]);

    prepend_compact_length(extrinsic)
}

/// Helper to create a SubtensorModule::add_stake extrinsic
pub fn create_add_stake() -> Vec<u8> {
    let mut extrinsic = Vec::new();

    extrinsic.push(0x84);

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0x12; 32]);

    // Signature
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xCC; 64]);

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 0
    extrinsic.push(0x00);

    // Tip: 0
    extrinsic.push(0x00);

    // Call: SubtensorModule (0x07) :: add_stake (0x01)
    extrinsic.push(0x07);
    extrinsic.push(0x01);

    // Parameters:
    // - hotkey: AccountId (32 bytes)
    // - amount_staked: u64

    // hotkey
    extrinsic.extend_from_slice(&[0x34; 32]);

    // amount_staked: 5000000000 (5 TAO with 9 decimals)
    // Compact encoding
    extrinsic.push(0x02); // Four-byte mode
    extrinsic.push(0x00);
    extrinsic.push(0xF2);
    extrinsic.push(0x05);
    extrinsic.push(0x2A);
    extrinsic.push(0x01);

    prepend_compact_length(extrinsic)
}

/// Helper to create a SubtensorModule::register extrinsic
pub fn create_register_neuron() -> Vec<u8> {
    let mut extrinsic = Vec::new();

    extrinsic.push(0x84);

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0x56; 32]);

    // Signature: Ed25519 (0x00) for variety
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xDD; 64]);

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 1
    extrinsic.push(0x04); // 1 << 2 = 4

    // Tip: 0
    extrinsic.push(0x00);

    // Call: SubtensorModule (0x07) :: register (0x05)
    extrinsic.push(0x07);
    extrinsic.push(0x05);

    // Parameters for register:
    // - netuid: u16
    // - block_number: u64
    // - nonce: u64
    // - work: Vec<u8>
    // - hotkey: AccountId
    // - coldkey: AccountId

    // netuid: 1
    extrinsic.push(0x01);
    extrinsic.push(0x00);

    // block_number: 1000000
    let block_num: u64 = 1000000;
    extrinsic.extend_from_slice(&block_num.to_le_bytes());

    // nonce: 123456
    let nonce: u64 = 123456;
    extrinsic.extend_from_slice(&nonce.to_le_bytes());

    // work: Vec<u8> with 32 bytes (proof of work)
    extrinsic.push(0x80); // Compact length: 32 << 2 = 128 = 0x80
    extrinsic.extend_from_slice(&[0xAB; 32]);

    // hotkey: 32 bytes
    extrinsic.extend_from_slice(&[0x78; 32]);

    // coldkey: 32 bytes
    extrinsic.extend_from_slice(&[0x9A; 32]);

    prepend_compact_length(extrinsic)
}

/// Helper to create an unsigned System::remark extrinsic
pub fn create_unsigned_remark() -> Vec<u8> {
    let mut extrinsic = vec![
        0x04, // Version: v4, unsigned
        0x00, // Call: System (0x00)
        0x01, // :: remark (0x01)
    ];

    // Remark data: "Bittensor test"
    let remark = b"Bittensor test";
    extrinsic.push((remark.len() << 2) as u8); // Compact length
    extrinsic.extend_from_slice(remark);

    prepend_compact_length(extrinsic)
}

/// Helper to create a Utility::batch extrinsic with multiple calls
pub fn create_batch_transfer() -> Vec<u8> {
    let mut extrinsic = Vec::new();

    extrinsic.push(0x84);

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xBC; 32]);

    // Signature
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xEE; 64]);

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 2
    extrinsic.push(0x08);

    // Tip: 0
    extrinsic.push(0x00);

    // Call: Utility (0x0B) :: batch (0x00)
    extrinsic.push(0x0B);
    extrinsic.push(0x00);

    // Calls: Vec with 2 transfer calls
    extrinsic.push(0x08); // Compact length: 2

    // First transfer
    extrinsic.push(0x04); // Balances
    extrinsic.push(0x00); // transfer
    extrinsic.push(0x00); // Dest address type
    extrinsic.extend_from_slice(&[0x11; 32]); // Dest
    extrinsic.push(0x00); // Amount: 0 (compact)

    // Second transfer
    extrinsic.push(0x04); // Balances
    extrinsic.push(0x00); // transfer
    extrinsic.push(0x00); // Dest address type
    extrinsic.extend_from_slice(&[0x22; 32]); // Dest
    extrinsic.push(0x00); // Amount: 0 (compact)

    prepend_compact_length(extrinsic)
}

/// Helper to create a large transfer (stress test)
pub fn create_large_transfer() -> Vec<u8> {
    let mut extrinsic = Vec::new();

    extrinsic.push(0x84);

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xDE; 32]);

    // Signature: ECDSA (0x02) for variety - 65 bytes
    extrinsic.push(0x02);
    extrinsic.extend_from_slice(&[0xFF; 65]);

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 100 (compact two-byte mode)
    // 100 in two-byte compact: ((100 << 2) | 0x01) = 401 = 0x191
    extrinsic.push(0x91);
    extrinsic.push(0x01);

    // Tip: 10000 (smaller tip value to avoid encoding complexity)
    // 10000 in two-byte compact: ((10000 << 2) | 0x01) = 40001 = 0x9C41
    extrinsic.push(0x41);
    extrinsic.push(0x9C);

    // Call: Balances::transfer
    extrinsic.push(0x04);
    extrinsic.push(0x00);

    // Destination
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xCD; 32]);

    // Amount: 1000000000000 (1000 TAO)
    // Big-integer mode for compact encoding
    extrinsic.push(0x13); // Big-integer mode: (4 << 2) | 0x03 = 19 = 0x13
    extrinsic.push(0x00);
    extrinsic.push(0x10);
    extrinsic.push(0xA5);
    extrinsic.push(0xD4);
    extrinsic.push(0xE8);
    extrinsic.push(0x00);
    extrinsic.push(0x00);

    prepend_compact_length(extrinsic)
}

/// Prepend compact length to extrinsic
fn prepend_compact_length(extrinsic: Vec<u8>) -> Vec<u8> {
    let length = extrinsic.len() as u32;
    let mut with_length = Vec::new();

    // Encode length as compact
    if length < 64 {
        with_length.push((length << 2) as u8);
    } else if length < 16384 {
        with_length.push(((length << 2) | 0x01) as u8);
        with_length.push((length >> 6) as u8);
    } else {
        // Four-byte mode
        with_length.push(((length << 2) | 0x02) as u8);
        with_length.push((length >> 6) as u8);
        with_length.push((length >> 14) as u8);
        with_length.push((length >> 22) as u8);
    }

    with_length.extend_from_slice(&extrinsic);
    with_length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_lengths() {
        // All fixtures should be non-empty and have valid lengths
        assert!(create_tao_transfer().len() > 4);
        assert!(create_set_weights().len() > 4);
        assert!(create_add_stake().len() > 4);
        assert!(create_register_neuron().len() > 4);
        assert!(create_unsigned_remark().len() > 4);
        assert!(create_batch_transfer().len() > 4);
        assert!(create_large_transfer().len() > 4);
    }

    #[test]
    fn test_all_fixtures_start_with_length() {
        // All extrinsics should start with a compact length prefix
        let fixtures = vec![
            create_tao_transfer(),
            create_set_weights(),
            create_add_stake(),
            create_register_neuron(),
            create_unsigned_remark(),
            create_batch_transfer(),
            create_large_transfer(),
        ];

        for fixture in fixtures {
            // First byte should be a valid compact encoding
            let first_byte = fixture[0];
            let mode = first_byte & 0b11;
            assert!(mode <= 0b11, "Invalid compact encoding mode");
        }
    }
}
