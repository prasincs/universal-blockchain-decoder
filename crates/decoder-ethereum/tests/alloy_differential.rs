//! Differential tests: our pure-Rust Ethereum decoder vs alloy (upstream).
//!
//! Every fixture is decoded by BOTH implementations and compared field by
//! field. A disagreement here is a finding, not noise: either our decoder or
//! alloy is wrong, and producing that minimal reproduction is this project's
//! purpose (see docs/SELF_IMPROVEMENT_LOOP.md, anti-Goodhart rule 6).
//!
//! alloy is a dev-dependency ONLY; production code uses pure Rust parsing.

use alloy_consensus::transaction::SignerRecoverable;
use alloy_consensus::{Transaction as _, TxEnvelope, Typed2718 as _};
use alloy_eips::eip2718::Decodable2718;
use alloy_primitives::U256;
use decoder_ethereum::types::EthereumTransaction;
use decoder_ethereum::EthereumDecoder;
use std::fs;
use std::path::Path;
use universal_decoder_core::prelude::*;

fn load_fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let hex_str = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    alloy_primitives::hex::decode(hex_str.trim().trim_start_matches("0x"))
        .unwrap_or_else(|e| panic!("bad hex in {path:?}: {e}"))
}

/// Our decoder stores the raw `v` value; alloy normalizes to a y-parity bool.
fn our_parity(tx: &EthereumTransaction) -> bool {
    match tx.tx_type_u8() {
        0 => {
            if tx.v >= 35 {
                // EIP-155: v = chain_id * 2 + 35 + parity
                (tx.v - 35) % 2 == 1
            } else {
                // pre-EIP-155: v = 27 + parity
                tx.v == 28
            }
        }
        _ => tx.v == 1,
    }
}

/// Both implementations must agree on whether the bytes are a valid
/// transaction at all; if both accept, every field must agree.
fn assert_agreement(name: &str) {
    assert_agreement_bytes(name, &load_fixture(name));
}

fn assert_agreement_bytes(name: &str, raw: &[u8]) {
    match (
        EthereumDecoder::decode(raw),
        TxEnvelope::decode_2718(&mut &raw[..]),
    ) {
        (Ok(ours), Ok(theirs)) => assert_fields_agree(name, &ours, &theirs),
        (Err(_), Err(_)) => {} // agreement: both reject
        (Ok(ours), Err(e)) => {
            panic!("{name}: DISAGREEMENT - we accept, alloy rejects ({e}); ours={ours:?}")
        }
        (Err(e), Ok(theirs)) => {
            panic!("{name}: DISAGREEMENT - alloy accepts, we reject ({e}); alloy={theirs:?}")
        }
    }
}

/// For fixtures known to be invalid: both implementations must reject.
/// If this starts failing, the fixture was repaired - move it to
/// `assert_agreement` and delete the entry here.
fn assert_both_reject(name: &str, why_invalid: &str) {
    let raw = load_fixture(name);
    let ours = EthereumDecoder::decode(&raw);
    let theirs = TxEnvelope::decode_2718(&mut raw.as_slice());
    assert!(
        ours.is_err(),
        "{name}: our decoder accepted a known-invalid fixture ({why_invalid})"
    );
    assert!(
        theirs.is_err(),
        "{name}: alloy accepted a fixture we reject ({why_invalid}) - DISAGREEMENT"
    );
}

fn assert_fields_agree(name: &str, ours: &EthereumTransaction, theirs: &TxEnvelope) {
    assert_eq!(ours.tx_type_u8(), theirs.tx_type().ty(), "{name}: tx type");
    assert_eq!(ours.chain_id, theirs.chain_id(), "{name}: chain_id");
    assert_eq!(ours.nonce, theirs.nonce(), "{name}: nonce");
    assert_eq!(
        ours.gas_limit,
        theirs.gas_limit() as u128,
        "{name}: gas_limit"
    );
    assert_eq!(ours.gas_price, theirs.gas_price(), "{name}: gas_price");
    if ours.tx_type_u8() >= 2 {
        assert_eq!(
            ours.max_fee_per_gas,
            Some(theirs.max_fee_per_gas()),
            "{name}: max_fee_per_gas"
        );
        assert_eq!(
            ours.max_priority_fee_per_gas,
            theirs.max_priority_fee_per_gas(),
            "{name}: max_priority_fee_per_gas"
        );
    }
    assert_eq!(U256::from(ours.value), theirs.value(), "{name}: value");
    assert_eq!(
        ours.to.as_ref().map(|a| &a[..]),
        theirs.to().as_ref().map(|a| a.as_slice()),
        "{name}: to address"
    );
    assert_eq!(
        ours.data.as_slice(),
        theirs.input().as_ref(),
        "{name}: input data"
    );

    // Access list (EIP-2930 / EIP-1559)
    let their_al = theirs.access_list().cloned().unwrap_or_default();
    assert_eq!(
        ours.access_list.len(),
        their_al.len(),
        "{name}: access list length"
    );
    for (i, (a, b)) in ours.access_list.iter().zip(their_al.iter()).enumerate() {
        assert_eq!(
            &a.address[..],
            b.address.as_slice(),
            "{name}: access_list[{i}].address"
        );
        let our_keys: Vec<&[u8]> = a.storage_keys.iter().map(|k| &k[..]).collect();
        let their_keys: Vec<&[u8]> = b.storage_keys.iter().map(|k| k.as_slice()).collect();
        assert_eq!(
            our_keys, their_keys,
            "{name}: access_list[{i}].storage_keys"
        );
    }

    // Blob fields (EIP-4844)
    assert_eq!(
        ours.max_fee_per_blob_gas,
        theirs.max_fee_per_blob_gas(),
        "{name}: max_fee_per_blob_gas"
    );
    let our_blob_hashes: Vec<&[u8]> = ours.blob_versioned_hashes.iter().map(|h| &h[..]).collect();
    let their_blob_hashes: Vec<&[u8]> = theirs
        .blob_versioned_hashes()
        .unwrap_or_default()
        .iter()
        .map(|h| h.as_slice())
        .collect();
    assert_eq!(
        our_blob_hashes, their_blob_hashes,
        "{name}: blob_versioned_hashes"
    );

    // Signature components and the signed-transaction hash.
    let (r, s, parity, hash) = match theirs {
        TxEnvelope::Legacy(t) => (
            t.signature().r(),
            t.signature().s(),
            t.signature().v(),
            *t.hash(),
        ),
        TxEnvelope::Eip2930(t) => (
            t.signature().r(),
            t.signature().s(),
            t.signature().v(),
            *t.hash(),
        ),
        TxEnvelope::Eip1559(t) => (
            t.signature().r(),
            t.signature().s(),
            t.signature().v(),
            *t.hash(),
        ),
        TxEnvelope::Eip4844(t) => (
            t.signature().r(),
            t.signature().s(),
            t.signature().v(),
            *t.hash(),
        ),
        other => panic!("{name}: unexpected envelope variant: {other:?}"),
    };
    assert_eq!(ours.r, r.to_be_bytes::<32>(), "{name}: signature r");
    assert_eq!(ours.s, s.to_be_bytes::<32>(), "{name}: signature s");
    assert_eq!(our_parity(ours), parity, "{name}: signature y-parity");
    // hash() is now computed from RE-ENCODED bytes (raw_bytes was removed
    // upstream), so agreement here also validates our reconstruction.
    let our_hash = ours
        .hash()
        .unwrap_or_else(|e| panic!("{name}: hash reconstruction failed: {e}"));
    assert_eq!(our_hash.as_slice(), hash.as_slice(), "{name}: tx hash");

    // Sender recovery: both must agree on accept/reject, and on the address
    // when both accept. (Fixtures with synthetic signatures are rejected by
    // both - that is agreement too.)
    match (ours.recover_sender(), theirs.recover_signer()) {
        (Ok(a), Ok(b)) => assert_eq!(&a[..], b.as_slice(), "{name}: recovered sender"),
        (Err(_), Err(_)) => {}
        (a, b) => panic!("{name}: sender recovery disagreement: ours={a:?}, alloy={b:?}"),
    }
}

/// EIP-4844 blob transaction, GENERATED by alloy's encoder (no mainnet
/// type-3 fixture exists yet; fetching one needs RPC egress - see backlog).
/// Upstream's encoder is the oracle: whatever alloy emits, we must decode
/// to identical fields. This caught two bugs in 2026-06: parse_eip4844 used
/// the 12-field EIP-1559 layout (v/r/s read from the wrong positions), and
/// signing_hash() used type byte 0x02 without blob fields for type-3 txs
/// (wrong recovered sender).
#[test]
fn differential_eip4844_alloy_generated() {
    use alloy_consensus::{SignableTransaction, TxEip4844};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_eips::eip2930::{AccessList, AccessListItem};
    use alloy_primitives::{address, b256, Bytes, Signature};

    let tx = TxEip4844 {
        chain_id: 1,
        nonce: 42,
        gas_limit: 210_000,
        max_fee_per_gas: 50_000_000_000,
        max_priority_fee_per_gas: 1_500_000_000,
        to: address!("dac17f958d2ee523a2206206994597c13d831ec7"),
        value: U256::from(123_456_789u64),
        access_list: AccessList(vec![AccessListItem {
            address: address!("0000000000000000000000000000000000000102"),
            storage_keys: vec![b256!(
                "00000000000000000000000000000000000000000000000000000000000060a7"
            )],
        }]),
        blob_versioned_hashes: vec![
            b256!("01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            b256!("01bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        ],
        max_fee_per_blob_gas: 3_000_000_000,
        input: Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]),
    };
    let signature = Signature::new(
        U256::from_be_bytes([0x11u8; 32]),
        U256::from_be_bytes([0x22u8; 32]),
        false,
    );
    let envelope = TxEnvelope::from(tx.into_signed(signature));
    let raw = envelope.encoded_2718();
    assert_agreement_bytes("alloy_generated_eip4844", &raw);
}

#[test]
fn differential_legacy() {
    assert_agreement("eth_legacy.hex");
}

#[test]
fn differential_eip1559() {
    assert_agreement("eth_eip1559.hex");
}

/// Corpus finding (2026-06): this fixture is an UNSIGNED 8-field EIP-2930
/// payload (the spec example, no v/r/s), not a signed transaction. Both
/// implementations reject it. Backlog: replace with a real signed mainnet
/// EIP-2930 transaction.
#[test]
fn differential_eip2930_unsigned_fixture_rejected_by_both() {
    assert_both_reject("eth_eip2930.hex", "unsigned 8-field EIP-2930 payload");
}

/// Corpus finding (2026-06): this fixture is CORRUPT - the outer RLP list
/// declares 176 bytes of payload but only 175 follow (truncated signature
/// `s`), despite the .json sidecar describing it as a real transfer. Both
/// implementations reject it. Backlog: replace with a verified mainnet
/// ERC-20 transfer fetched by txid.
#[test]
fn differential_erc20_corrupt_fixture_rejected_by_both() {
    assert_both_reject(
        "eth_erc20_transfer.hex",
        "RLP list length 176 != 175 actual (truncated s)",
    );
}

#[test]
fn differential_contract_creation() {
    assert_agreement("eth_contract_creation.hex");
}

#[test]
fn differential_large_data() {
    assert_agreement("eth_large_data.hex");
}
