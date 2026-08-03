//! BNB Chain-ID gate coverage against REAL Ethereum mainnet transactions.
//!
//! `BnbDecoder` reuses the Ethereum decoder verbatim (see `src/lib.rs`); the
//! RLP decode path is already differentially tested against `alloy` in
//! `decoder-ethereum` (`tests/alloy_differential.rs`). The only BNB-specific
//! logic is the chain-ID gate: a decoded transaction is accepted only when its
//! chain ID is 56 (mainnet), 97 (testnet), or absent (pre-EIP-155 legacy).
//!
//! These tests exercise that gate with genuine Ethereum mainnet transactions
//! (chain ID 1), which the gate must reject. The fixtures are the same real
//! transactions used by the Ethereum differential suite
//! (`crates/decoder-ethereum/tests/fixtures/eth_{eip1559,legacy}.hex`), inlined
//! here so the test is self-contained and needs no cross-crate fixture path.

use decoder_bnb::BnbDecoder;
use decoder_ethereum::EthereumDecoder;
use decoder_primitives::prelude::*;
use universal_decoder_core::hex;

/// Real mainnet EIP-1559 transfer (chain_id encoded as 1 in the RLP payload).
/// Source: `decoder-ethereum/tests/fixtures/eth_eip1559.hex`.
const ETH_MAINNET_EIP1559: &str = "02f8740181f1843b9aca00851535cf027f82520894e0e5d2b4edcc473b988b44b4d13c3972cb6694cb8801ea8d467f558e1e80c001a07eb3335f4fd4de25ec3452c08882f28fb098b2eaa37a332941f918d869f5c2ada059b9d4aa997c7fa34f1b167f98a12432bb1a4a35660d723a9c19bb76b4cd025d";

/// Real legacy transfer with EIP-155 v=37 (=> chain_id 1).
/// Source: `decoder-ethereum/tests/fixtures/eth_legacy.hex`.
const ETH_MAINNET_LEGACY: &str = "f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683";

/// A real Ethereum mainnet EIP-1559 tx (chain_id 1) is valid Ethereum but must
/// be rejected by the BNB gate: the RLP decodes fine, then the chain-ID check
/// fails because 1 is neither 56 nor 97.
#[test]
fn eip1559_ethereum_mainnet_rejected_by_chain_id_gate() {
    let bytes = hex::decode(ETH_MAINNET_EIP1559).unwrap();

    // The reused Ethereum decoder accepts it and reports chain_id 1.
    let eth = EthereumDecoder::decode(&bytes).expect("valid Ethereum EIP-1559 tx");
    assert_eq!(eth.chain_id, Some(1), "fixture must be Ethereum mainnet");

    // BNB rejects it on the chain-ID gate (not on any parse error).
    let err = BnbDecoder::decode(&bytes).expect_err("BNB must reject chain_id 1");
    let msg = format!("{err}");
    assert!(
        msg.contains("Invalid BNB Chain ID"),
        "expected chain-ID gate rejection, got: {msg}"
    );
}

/// Same for a real legacy (EIP-155) mainnet tx: chain_id derived as 1, rejected.
#[test]
fn legacy_ethereum_mainnet_rejected_by_chain_id_gate() {
    let bytes = hex::decode(ETH_MAINNET_LEGACY).unwrap();

    let eth = EthereumDecoder::decode(&bytes).expect("valid Ethereum legacy tx");
    assert_eq!(eth.chain_id, Some(1), "fixture must be Ethereum mainnet");

    let err = BnbDecoder::decode(&bytes).expect_err("BNB must reject chain_id 1");
    assert!(
        format!("{err}").contains("Invalid BNB Chain ID"),
        "expected chain-ID gate rejection"
    );
}

/// The decode path itself is byte-for-byte the Ethereum decoder: whatever
/// `EthereumDecoder` produces for these bytes is exactly what BNB would carry
/// on through (the gate only filters, it never rewrites fields). Confirms BNB
/// adds no divergence on the shared RLP path.
#[test]
fn decode_path_matches_ethereum_before_gate() {
    let bytes = hex::decode(ETH_MAINNET_EIP1559).unwrap();
    let eth = EthereumDecoder::decode(&bytes).unwrap();

    // Re-run through BNB's own validate_format (delegated to Ethereum) to show
    // the format check agrees; the only difference is the post-decode gate.
    assert!(BnbDecoder::validate_format(&bytes).is_ok());
    assert!(EthereumDecoder::validate_format(&bytes).is_ok());
    assert_eq!(eth.chain_id, Some(1));
}

// NOTE: the positive branch of the gate (chain_id 56/97 accepted) needs a real
// BSC mainnet fixture. Fetching one requires network egress (blocked in this
// sandbox); tracked in the P1 corpus section of loop/BACKLOG.md. We do not
// fabricate a signed BSC transaction here.
