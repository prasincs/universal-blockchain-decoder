//! Differential validation of our BoC parser against `tonlib-core`.
//!
//! These tests decode real TON Bag-of-Cells with BOTH our pure-Rust parser and
//! the upstream `tonlib-core` library, then assert **field-level agreement** on
//! the reconstructed cell tree: for every cell reachable from a root we compare
//! the bit length, the payload bytes (completion tag stripped), and the
//! reference fan-out — walking references in lockstep. This is a genuine
//! differential test, not a "both parsers returned something" smoke check.

use base64::{engine::general_purpose::STANDARD, Engine};
use decoder_ton::boc::{self, Cell};
use tonlib_core::cell::Cell as TonlibCell;

/// Recursively assert that our cell (addressed by index into the flat cell
/// list) agrees with the corresponding tonlib cell, following references in
/// order. `path` is a human-readable breadcrumb for failure messages.
fn assert_cell_eq(ours: &[Cell], idx: usize, theirs: &TonlibCell, path: &str) {
    let our_cell = &ours[idx];

    assert_eq!(
        our_cell.bit_len as usize,
        theirs.bit_len(),
        "bit_len mismatch at {path} (our cell index {idx})"
    );
    assert_eq!(
        our_cell.data,
        theirs.data(),
        "data mismatch at {path} (our cell index {idx}): ours={} theirs={}",
        hex::encode(&our_cell.data),
        hex::encode(theirs.data())
    );
    assert_eq!(
        our_cell.refs.len(),
        theirs.references().len(),
        "reference count mismatch at {path} (our cell index {idx})"
    );

    for (i, (our_ref, their_ref)) in our_cell
        .refs
        .iter()
        .zip(theirs.references().iter())
        .enumerate()
    {
        assert_cell_eq(ours, *our_ref, their_ref, &format!("{path}/{i}"));
    }
}

/// Decode with both parsers and assert the full cell tree agrees.
fn assert_boc_agrees(boc_bytes: &[u8]) {
    let tonlib_boc =
        tonlib_core::cell::BagOfCells::parse(boc_bytes).expect("tonlib-core failed to parse BoC");
    let (our_cells, our_roots) =
        boc::parse_boc_with_roots(boc_bytes).expect("our parser failed to parse BoC");

    assert_eq!(
        our_roots.len(),
        tonlib_boc.roots.len(),
        "root count mismatch: ours={} tonlib={}",
        our_roots.len(),
        tonlib_boc.roots.len()
    );

    for (r, (our_root_idx, their_root)) in our_roots.iter().zip(tonlib_boc.roots.iter()).enumerate()
    {
        assert_cell_eq(&our_cells, *our_root_idx, their_root, &format!("root{r}"));
    }
}

#[test]
fn test_validate_against_tonlib_simple_boc() {
    // Real TON transfer message BoC (single root, 4 cells, partial + full
    // byte-aligned cells, and an empty cell).
    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";
    let boc_bytes = STANDARD.decode(boc_base64).expect("Invalid base64");
    assert_boc_agrees(&boc_bytes);
}

#[test]
#[ignore = "fixture is malformed: tonlib-core rejects it as truncated \
            (failed to fill whole buffer). See loop/BACKLOG.md TON corpus item."]
fn test_validate_against_tonlib_complex_boc() {
    // The "complex StateInit BoC" base64 used elsewhere in this crate
    // (real_transactions.rs, debug_boc_parsing.rs) is NOT a valid Bag of
    // Cells: tonlib-core fails to deserialize it. It cannot serve as a
    // differential oracle fixture until replaced with a verified mainnet BoC.
    let boc_base64 = "te6cckECGwEAA2sAART/APSkE/S88sgLAQIBIAIDAgFIBAUCAvIMDQ8E9PzoYXX4ZPgfoTjIU7L/AgEgYH74BgYEBIMH/8IAIbGBn4YD4B4f/wQGpCGYfQIBBhITFBUCASAWFwIBSBgZGhsWAAgcPH6BkAIBSBgZAgJ7GhsACNrIULTQjkJwgBDIywVQB88WIfAHy//J0AT0BPQE9ATwAvoA9AQlYfQFPwX/BtQFBg==";
    let boc_bytes = STANDARD.decode(boc_base64).expect("Invalid base64");
    assert_boc_agrees(&boc_bytes);
}
