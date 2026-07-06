//! Differential tests: our pure-Rust Solana decoder vs. `solana-transaction-status`.
//!
//! Until now `solana-transaction-status` was a declared-but-unused dev-dep
//! (health report flagged it as a dead validation dep). These tests make it a
//! real oracle: each fixture is decoded by BOTH our decoder and the upstream
//! crate, and every byte-derivable field is asserted to agree — not merely
//! "both parsed".
//!
//! Oracle path: `EncodedTransaction::Binary(base64, Base64).decode()` runs
//! `bincode::deserialize::<VersionedTransaction>` followed by `sanitize()`,
//! i.e. the exact deserializer Solana's RPC layer uses. If the oracle and our
//! decoder ever disagree on a real fixture, that is a finding to capture — not
//! something to paper over.

use decoder_primitives::prelude::*;
use decoder_solana::*;

use solana_transaction_status::{EncodedTransaction, TransactionBinaryEncoding};

/// Every legacy-transaction fixture shipped in `tests/fixtures/simple/`.
/// These are complete, self-contained transactions (header + accounts +
/// blockhash + instructions), which is what a field-level differential needs.
const FIXTURES: &[&str] = &[
    "simple_system_transfer",
    "multi_instruction_transfer",
    "multi_signer_transaction",
    "transfer_with_readonly_account",
    "durable_nonce_transaction",
    "transfer_with_nonce",
];

fn fixture_base64(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/simple/{}.base64",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {path}: {e}"))
        .trim()
        .to_string()
}

fn to_bytes<T: AsRef<[u8]>>(v: T) -> Vec<u8> {
    v.as_ref().to_vec()
}

#[test]
fn differential_all_fixtures_agree_field_by_field() {
    for name in FIXTURES {
        let base64_tx = fixture_base64(name);
        let raw = {
            use base64::{engine::general_purpose::STANDARD, Engine};
            STANDARD
                .decode(&base64_tx)
                .expect("fixture is valid base64")
        };

        // Our decoder.
        let ours = SolanaDecoder::decode(&raw)
            .unwrap_or_else(|e| panic!("[{name}] our decoder rejected the fixture: {e:?}"));

        // Upstream oracle: bincode::deserialize::<VersionedTransaction> + sanitize().
        let encoded =
            EncodedTransaction::Binary(base64_tx.clone(), TransactionBinaryEncoding::Base64);
        let theirs = encoded.decode().unwrap_or_else(|| {
            panic!("[{name}] solana-transaction-status rejected a fixture our decoder accepted")
        });
        let msg = &theirs.message;

        // --- signatures ---
        assert_eq!(
            ours.signatures.len(),
            theirs.signatures.len(),
            "[{name}] signature count"
        );
        for (i, (a, b)) in ours
            .signatures
            .iter()
            .zip(theirs.signatures.iter())
            .enumerate()
        {
            assert_eq!(a, &to_bytes(b.as_array()), "[{name}] signature[{i}] bytes");
        }

        // --- header ---
        let oh = msg.header();
        assert_eq!(
            ours.message.header.num_required_signatures, oh.num_required_signatures,
            "[{name}] num_required_signatures"
        );
        assert_eq!(
            ours.message.header.num_readonly_signed_accounts, oh.num_readonly_signed_accounts,
            "[{name}] num_readonly_signed_accounts"
        );
        assert_eq!(
            ours.message.header.num_readonly_unsigned_accounts, oh.num_readonly_unsigned_accounts,
            "[{name}] num_readonly_unsigned_accounts"
        );

        // --- account keys ---
        let keys = msg.static_account_keys();
        assert_eq!(
            ours.message.account_keys.len(),
            keys.len(),
            "[{name}] account key count"
        );
        for (i, (a, b)) in ours
            .message
            .account_keys
            .iter()
            .zip(keys.iter())
            .enumerate()
        {
            assert_eq!(a, &to_bytes(b), "[{name}] account_keys[{i}] bytes");
        }

        // --- recent blockhash ---
        assert_eq!(
            ours.message.recent_blockhash,
            msg.recent_blockhash().to_bytes().to_vec(),
            "[{name}] recent_blockhash"
        );

        // --- instructions ---
        let ins = msg.instructions();
        assert_eq!(
            ours.message.instructions.len(),
            ins.len(),
            "[{name}] instruction count"
        );
        for (i, (a, b)) in ours.message.instructions.iter().zip(ins.iter()).enumerate() {
            assert_eq!(
                a.program_id_index, b.program_id_index,
                "[{name}] instruction[{i}] program_id_index"
            );
            assert_eq!(a.accounts, b.accounts, "[{name}] instruction[{i}] accounts");
            assert_eq!(a.data, b.data, "[{name}] instruction[{i}] data");
        }
    }
}

/// Guard against a silently-shrinking oracle: the upstream `decode()` applies
/// `sanitize()`, so a legacy fixture that both sides accept exercises the full
/// path. This test fails loudly if a fixture stops decoding on either side
/// rather than skipping it.
#[test]
fn differential_every_fixture_decodes_on_both_sides() {
    for name in FIXTURES {
        let base64_tx = fixture_base64(name);
        let raw = {
            use base64::{engine::general_purpose::STANDARD, Engine};
            STANDARD
                .decode(&base64_tx)
                .expect("fixture is valid base64")
        };
        assert!(
            SolanaDecoder::decode(&raw).is_ok(),
            "[{name}] our decoder must accept this fixture"
        );
        let encoded =
            EncodedTransaction::Binary(base64_tx.clone(), TransactionBinaryEncoding::Base64);
        assert!(
            encoded.decode().is_some(),
            "[{name}] solana-transaction-status must accept this fixture"
        );
    }
}
