//! Differential test: our pure-Rust Solana decoder vs `solana-transaction-status`.
//!
//! This is the real differential suite for Solana (previously `solana-transaction-status`
//! was a declared-but-unused dev-dep — see `loop/BACKLOG.md`, "Solana vs
//! solana-transaction-status"). Each fixture's wire bytes are decoded with BOTH
//! our decoder and the upstream library, then compared field-for-field.
//!
//! The upstream oracle is `EncodedTransaction::decode()`, which base64-decodes,
//! `bincode`-deserializes into a `VersionedTransaction`, and runs `sanitize()`.
//! The contract asserted here: **whenever the upstream library accepts a
//! transaction, our decoder must also accept it and agree on every
//! byte-derivable field** (signatures, header, account keys, recent blockhash,
//! and every instruction's program-id index / accounts / data). Disagreement is
//! a finding, not a tolerated difference.

// The upstream crate carries a crate-level unstable-API notice; we opt into it
// via the `agave-unstable-api` feature in Cargo.toml, so no `allow(deprecated)`
// is needed here.
use base64::{engine::general_purpose::STANDARD, Engine};
use decoder_primitives::prelude::*;
use decoder_solana::SolanaDecoder;
use solana_transaction_status::{EncodedTransaction, TransactionBinaryEncoding};

/// Every base64 fixture shipped under `tests/fixtures/simple/`.
const FIXTURES: &[(&str, &str)] = &[
    (
        "simple_system_transfer",
        include_str!("fixtures/simple/simple_system_transfer.base64"),
    ),
    (
        "multi_instruction_transfer",
        include_str!("fixtures/simple/multi_instruction_transfer.base64"),
    ),
    (
        "multi_signer_transaction",
        include_str!("fixtures/simple/multi_signer_transaction.base64"),
    ),
    (
        "transfer_with_readonly_account",
        include_str!("fixtures/simple/transfer_with_readonly_account.base64"),
    ),
    (
        "durable_nonce_transaction",
        include_str!("fixtures/simple/durable_nonce_transaction.base64"),
    ),
    (
        "transfer_with_nonce",
        include_str!("fixtures/simple/transfer_with_nonce.base64"),
    ),
];

/// Field-level comparison for a single fixture that upstream accepted.
fn assert_agreement(name: &str, raw: &[u8], b64: &str) {
    let ours = SolanaDecoder::decode(raw).unwrap_or_else(|e| {
        panic!(
            "{name}: upstream accepted the transaction but our decoder rejected it: {e:?}\n\
             This is a differential finding — our decoder must accept anything the \
             upstream library accepts."
        )
    });

    let encoded = EncodedTransaction::Binary(b64.to_string(), TransactionBinaryEncoding::Base64);
    let upstream = encoded
        .decode()
        .expect("caller guarantees upstream accepted this fixture");
    let up_msg = &upstream.message;

    // 1. Signature count + bytes.
    assert_eq!(
        ours.num_signatures(),
        upstream.signatures.len(),
        "{name}: signature count differs"
    );
    for (i, up_sig) in upstream.signatures.iter().enumerate() {
        assert_eq!(
            ours.signatures[i].as_slice(),
            up_sig.as_ref(),
            "{name}: signature[{i}] bytes differ"
        );
    }

    // 2. Message header.
    let up_hdr = up_msg.header();
    assert_eq!(
        ours.message.header.num_required_signatures, up_hdr.num_required_signatures,
        "{name}: num_required_signatures differs"
    );
    assert_eq!(
        ours.message.header.num_readonly_signed_accounts, up_hdr.num_readonly_signed_accounts,
        "{name}: num_readonly_signed_accounts differs"
    );
    assert_eq!(
        ours.message.header.num_readonly_unsigned_accounts, up_hdr.num_readonly_unsigned_accounts,
        "{name}: num_readonly_unsigned_accounts differs"
    );

    // 3. Static account keys (count + 32-byte contents, in order).
    let up_keys = up_msg.static_account_keys();
    assert_eq!(
        ours.message.account_keys.len(),
        up_keys.len(),
        "{name}: account key count differs"
    );
    for (i, up_key) in up_keys.iter().enumerate() {
        assert_eq!(
            ours.message.account_keys[i].as_slice(),
            up_key.as_ref(),
            "{name}: account_keys[{i}] differ"
        );
    }

    // 4. Recent blockhash.
    assert_eq!(
        ours.message.recent_blockhash.as_slice(),
        up_msg.recent_blockhash().as_ref(),
        "{name}: recent_blockhash differs"
    );

    // 5. Instructions: count, then program-id index / accounts / data each.
    let up_ixs = up_msg.instructions();
    assert_eq!(
        ours.message.instructions.len(),
        up_ixs.len(),
        "{name}: instruction count differs"
    );
    for (i, up_ix) in up_ixs.iter().enumerate() {
        let our_ix = &ours.message.instructions[i];
        assert_eq!(
            our_ix.program_id_index, up_ix.program_id_index,
            "{name}: instruction[{i}].program_id_index differs"
        );
        assert_eq!(
            our_ix.accounts, up_ix.accounts,
            "{name}: instruction[{i}].accounts differ"
        );
        assert_eq!(
            our_ix.data, up_ix.data,
            "{name}: instruction[{i}].data differ"
        );
    }
}

#[test]
fn differential_against_solana_transaction_status() {
    let mut agreements = 0usize;
    for (name, b64_raw) in FIXTURES {
        let b64 = b64_raw.trim();
        let raw = match STANDARD.decode(b64) {
            Ok(bytes) => bytes,
            Err(e) => panic!("{name}: fixture is not valid base64: {e}"),
        };

        let encoded =
            EncodedTransaction::Binary(b64.to_string(), TransactionBinaryEncoding::Base64);
        match encoded.decode() {
            Some(_) => {
                assert_agreement(name, &raw, b64);
                agreements += 1;
                println!("{name}: upstream accepted; field-level agreement ✓");
            }
            None => {
                // Upstream rejected (e.g. a versioned/ALT transaction our
                // corpus doesn't yet cover, or a sanitize() failure). Not a
                // failure of the differential contract, but record it so a
                // silently-empty run is impossible.
                println!("{name}: upstream rejected (skipped for agreement)");
            }
        }
    }

    // The suite must be non-vacuous: if upstream accepted nothing, we compared
    // nothing and the test proves nothing.
    assert!(
        agreements >= 3,
        "differential suite is vacuous: only {agreements} fixture(s) were \
         accepted by the upstream oracle — expected at least 3"
    );
    println!("Solana differential: {agreements} fixtures agreed field-for-field");
}
