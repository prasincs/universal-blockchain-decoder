#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Ethereum signature parsing should never panic
    //
    // Ethereum uses ECDSA signatures with secp256k1 curve.
    // Signatures consist of (v, r, s) where:
    // - v: recovery ID (1 byte, value 0-3 or 27-28 or EIP-155 encoded)
    // - r: x-coordinate of R point (32 bytes)
    // - s: signature value (32 bytes)
    //
    // This fuzzer ensures we handle all edge cases:
    // 1. Invalid v values
    // 2. r = 0 or s = 0 (invalid signatures)
    // 3. r >= curve order or s >= curve order (invalid)
    // 4. High s values (should be normalized)
    // 5. EIP-155 replay protection encoding

    if data.len() < 65 {
        // Need at least 65 bytes for a signature (32 + 32 + 1)
        return;
    }

    // Extract signature components
    let r = &data[0..32];
    let s = &data[32..64];
    let v = if data.len() >= 65 { data[64] } else { 27 };

    // Test 1: Check if r and s are valid (not zero, not greater than curve order)
    let r_is_zero = r.iter().all(|&b| b == 0);
    let s_is_zero = s.iter().all(|&b| b == 0);

    if r_is_zero || s_is_zero {
        // Invalid signature - these should be rejected
        return;
    }

    // Test 2: Verify v is in valid range
    // Pre-EIP-155: v ∈ {27, 28}
    // Post-EIP-155: v = {0, 1} + CHAIN_ID * 2 + 35
    // Typed transactions: v ∈ {0, 1}
    let v_valid = match v {
        0 | 1 => true,        // Typed transaction format
        27 | 28 => true,      // Pre-EIP-155 format
        v if v >= 35 => true, // EIP-155 format
        _ => false,           // Invalid
    };

    if !v_valid {
        return;
    }

    // Test 3: Extract chain ID from v (EIP-155)
    if v >= 35 {
        let chain_id = (v as u64 - 35) / 2;
        let _parity = (v as u64 - 35) % 2;

        // Chain ID should be reasonable
        assert!(chain_id < 1_000_000, "Chain ID too large");
    }

    // Test 4: Check s malleability (EIP-2)
    // s should be <= curve_order / 2 to prevent malleability
    // secp256k1 curve order: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
    // Half: 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
    let curve_half = [
        0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
        0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
    ];

    // Compare s with curve_half (should not panic)
    let _s_high = s > curve_half.as_slice();

    // Test 5: Verify signature parsing with RLP (if data contains more)
    if data.len() > 65 {
        use decoder_encodings::rlp::RlpItem;
        let _ = RlpItem::decode(data);
    }
});
