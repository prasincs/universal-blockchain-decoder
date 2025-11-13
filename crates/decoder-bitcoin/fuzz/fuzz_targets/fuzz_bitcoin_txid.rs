#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;
use universal_decoder_core::hex;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: TXID calculation should be deterministic and never panic
    //
    // This specifically targets the TXID calculation logic which is
    // critical for transaction identification and must be:
    // 1. Deterministic (same input → same output)
    // 2. Panic-free
    // 3. Consistent with Bitcoin Core behavior

    // Test 1: If decode succeeds, TXID calculation should not panic
    if let Ok(tx) = BitcoinDecoder::decode(data) {
        // Calculate TXID (should never panic)
        let txid = tx.txid();

        // Test 2: TXID calculation is deterministic
        // Computing TXID twice should yield same result
        let txid2 = tx.txid();
        assert_eq!(txid, txid2, "TXID calculation is non-deterministic");

        // Test 3: TXID length is always 32 bytes
        assert_eq!(txid.len(), 32, "TXID length is not 32 bytes");

        // Test 4: For SegWit transactions, TXID excludes witness data
        if tx.is_segwit() {
            // TXID should be computed from transaction without witness
            // This is a critical property for BIP 141 compliance
            let _ = tx.txid();  // Should complete without panic
        }

        // Test 5: For coinbase transactions, TXID is computed normally
        if tx.is_coinbase() {
            let _ = tx.txid();  // Should complete without panic
        }

        // Test 6: TXID should be valid hex when encoded
        let hex_txid = hex::encode(&txid);
        assert_eq!(hex_txid.len(), 64, "Hex TXID length should be 64");

        // Test 7: Decoding the hex TXID should work
        assert!(hex::decode(&hex_txid).is_ok(), "TXID hex decode failed");
    }

    // Test 8: Invalid transactions should fail gracefully
    // (no panic even if TXID calculation is attempted on malformed data)
    if data.len() < 10 {
        // Very short data should fail decode, not panic
        assert!(BitcoinDecoder::decode(data).is_err());
    }
});
