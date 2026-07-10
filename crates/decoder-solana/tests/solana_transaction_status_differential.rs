//! Differential validation against `solana-transaction-status`.
//!
//! This is the *real* use of the `solana-transaction-status` dev-dep: it was
//! declared as a validation oracle but nothing imported it, so the health
//! report counted it as a dead dependency (Assumption 6). Here we decode the
//! same raw transaction bytes with BOTH our pure-Rust parser and the upstream
//! Solana crate, then assert field-level agreement — not merely "both parsed".
//!
//! The oracle path is `EncodedTransaction::decode()`, which bincode-decodes the
//! wire bytes into a `VersionedTransaction` and runs `sanitize()`. We reach the
//! raw `VersionedTransaction`/`VersionedMessage` types only through the public
//! accessors the upstream crate re-exports transitively; the byte-level
//! comparisons use `AsRef<[u8]>`, which every Solana key/hash/signature
//! implements.
//!
//! Run with:
//!   cargo test -p decoder-solana --test solana_transaction_status_differential

use decoder_primitives::prelude::*;
use decoder_solana::*;
use solana_transaction_status::{EncodedTransaction, TransactionBinaryEncoding};

/// A real mainnet SOL transfer (System Program `Transfer`), base64-encoded.
/// Same fixture the `real_transactions` suite exercises; here it must decode
/// identically under both implementations.
const SOL_TRANSFER_B64: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=";

fn b64_bytes(s: &str) -> Vec<u8> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(s).expect("invalid base64 fixture")
}

/// Decode with the upstream oracle. Returns `None` if the upstream crate
/// rejects the bytes (fails bincode decode or `sanitize()`).
///
/// The concrete return type is `solana-transaction`'s `VersionedTransaction`,
/// but that crate is only a transitive dependency, so we never name it — the
/// value is threaded through type inference from `EncodedTransaction::decode()`.
macro_rules! upstream_decode {
    ($b64:expr) => {
        EncodedTransaction::Binary($b64.to_string(), TransactionBinaryEncoding::Base64).decode()
    };
}

/// Assert our decoder and the upstream oracle agree on every byte-derivable
/// field of a legacy transaction.
fn assert_agreement(b64: &str) {
    let bytes = b64_bytes(b64);

    let ours = SolanaDecoder::decode(&bytes).expect("our decoder rejected the fixture");
    let theirs = upstream_decode!(b64).expect("upstream oracle rejected the fixture");

    // --- signatures ---
    assert_eq!(
        ours.signatures.len(),
        theirs.signatures.len(),
        "signature count disagreement"
    );
    for (i, (o, t)) in ours
        .signatures
        .iter()
        .zip(theirs.signatures.iter())
        .enumerate()
    {
        assert_eq!(o.as_slice(), t.as_ref(), "signature {i} byte disagreement");
    }

    let msg = &theirs.message;

    // --- header ---
    let their_header = msg.header();
    assert_eq!(
        ours.message.header.num_required_signatures, their_header.num_required_signatures,
        "num_required_signatures disagreement"
    );
    assert_eq!(
        ours.message.header.num_readonly_signed_accounts, their_header.num_readonly_signed_accounts,
        "num_readonly_signed_accounts disagreement"
    );
    assert_eq!(
        ours.message.header.num_readonly_unsigned_accounts,
        their_header.num_readonly_unsigned_accounts,
        "num_readonly_unsigned_accounts disagreement"
    );

    // --- account keys ---
    let their_keys = msg.static_account_keys();
    assert_eq!(
        ours.message.account_keys.len(),
        their_keys.len(),
        "account key count disagreement"
    );
    for (i, (o, t)) in ours
        .message
        .account_keys
        .iter()
        .zip(their_keys.iter())
        .enumerate()
    {
        assert_eq!(
            o.as_slice(),
            t.as_ref(),
            "account key {i} byte disagreement"
        );
    }

    // --- recent blockhash ---
    assert_eq!(
        ours.message.recent_blockhash.as_slice(),
        msg.recent_blockhash().as_ref(),
        "recent_blockhash disagreement"
    );

    // --- instructions ---
    let their_ix = msg.instructions();
    assert_eq!(
        ours.message.instructions.len(),
        their_ix.len(),
        "instruction count disagreement"
    );
    for (i, (o, t)) in ours
        .message
        .instructions
        .iter()
        .zip(their_ix.iter())
        .enumerate()
    {
        assert_eq!(
            o.program_id_index, t.program_id_index,
            "instruction {i} program_id_index disagreement"
        );
        assert_eq!(
            o.accounts.as_slice(),
            t.accounts.as_slice(),
            "instruction {i} account index disagreement"
        );
        assert_eq!(
            o.data.as_slice(),
            t.data.as_slice(),
            "instruction {i} data disagreement"
        );
    }
}

#[test]
fn differential_sol_transfer_matches_solana_transaction_status() {
    assert_agreement(SOL_TRANSFER_B64);
}

/// Sanity guard on the oracle itself: it must actually accept and decode the
/// fixture (so a silently-broken oracle can't make `assert_agreement` vacuous).
#[test]
fn oracle_decodes_the_fixture() {
    let theirs = upstream_decode!(SOL_TRANSFER_B64).expect("oracle must decode the fixture");
    assert!(
        !theirs.signatures.is_empty(),
        "decoded transaction should carry at least one signature slot"
    );
    assert!(
        !theirs.message.instructions().is_empty(),
        "SOL transfer should carry at least one instruction"
    );
}
