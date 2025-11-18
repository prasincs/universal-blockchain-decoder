//! Basic tests for TON decoder

use decoder_primitives::prelude::*;
use decoder_ton::*;

#[test]
fn test_ton_chain_identity() {
    let chain = TonDecoder::chain();
    assert_eq!(chain.chain_id(), 607);
    assert_eq!(chain.chain_name(), "TON");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
    assert_eq!(chain.network(), Some("mainnet"));
}

#[test]
fn test_validate_format_standard_boc_magic() {
    // Standard BoC magic: 0xb5ee9c72
    let valid = vec![0xb5, 0xee, 0x9c, 0x72, 0x01, 0x02, 0x03, 0x04];
    assert!(TonDecoder::validate_format(&valid).is_ok());
}

#[test]
fn test_validate_format_idx_boc_magic() {
    // BoC with index magic: 0x68ff65f3
    let valid = vec![0x68, 0xff, 0x65, 0xf3, 0x01, 0x02, 0x03, 0x04];
    assert!(TonDecoder::validate_format(&valid).is_ok());
}

#[test]
fn test_validate_format_crc32c_boc_magic() {
    // BoC with CRC32C magic: 0xacc3a728
    let valid = vec![0xac, 0xc3, 0xa7, 0x28, 0x01, 0x02, 0x03, 0x04];
    assert!(TonDecoder::validate_format(&valid).is_ok());
}

#[test]
fn test_validate_format_invalid_magic() {
    let invalid = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x02];
    assert!(TonDecoder::validate_format(&invalid).is_err());
}

#[test]
fn test_validate_format_empty() {
    assert!(TonDecoder::validate_format(&[]).is_err());
}

#[test]
fn test_validate_format_too_small() {
    assert!(TonDecoder::validate_format(&[0x01]).is_err());
}

/// Test a minimal valid BoC structure
///
/// This creates a minimal BoC with one cell containing dummy data.
/// Real TON transactions would be more complex, but this validates
/// the basic parsing logic.
#[test]
fn test_minimal_boc_parsing() {
    let mut boc = Vec::new();

    // Magic number (standard BoC): 0xb5ee9c72
    boc.extend_from_slice(&[0xb5, 0xee, 0x9c, 0x72]);

    // Flags byte: no idx, no crc32c, no cache bits, size=0
    boc.push(0x00);

    // Off bytes: 1 (using 1-byte offsets)
    boc.push(0x01);

    // Cell count: 1
    boc.push(0x01);

    // Root count: 1
    boc.push(0x01);

    // Absent count: 0
    boc.push(0x00);

    // Total cells size: placeholder (we'll calculate)
    boc.push(0x20); // 32 bytes for simplicity

    // Root list: cell 0
    boc.push(0x00);

    // Cell descriptor (2 bytes):
    // - d1: refs_count=0, not exotic, no hashes, level_mask=0
    boc.push(0x00);
    // - d2: bit_len = 0 (will use size byte)
    boc.push(0x00);

    // Data size: 32 bytes
    boc.push(0x20);

    // Cell data: 32 bytes of dummy data
    boc.extend_from_slice(&[0xAAu8; 32]);

    // Try to parse the BoC
    let cells = decoder_ton::boc::parse_boc(&boc).expect("Failed to parse minimal BoC");

    assert_eq!(cells.len(), 1);
    assert_eq!(cells[0].data.len(), 32);
    assert_eq!(cells[0].refs.len(), 0);
}

#[test]
fn test_transaction_validation() {
    use decoder_ton::types::TonTransaction;

    // Create a transaction with valid structure
    use decoder_ton::types::{AccountStatus, CurrencyCollection};

    let tx = TonTransaction {
        raw_bytes: vec![0u8; 100],
        cells: vec![],
        account_addr: vec![1u8; 32],
        lt: 12345,
        prev_trans_hash: vec![2u8; 32],
        prev_trans_lt: 12344,
        now: 1700000000,
        outmsg_cnt: 5,
        orig_status: AccountStatus::Active,
        end_status: AccountStatus::Active,
        total_fees: CurrencyCollection {
            grams: 1000,
            extra: vec![],
        },
        in_msg: None,
        out_msgs: vec![],
    };

    let result = tx.canonicalize();
    assert!(result.is_ok());

    let tx_ir = result.unwrap();
    assert_eq!(tx_ir.chain.id, 607);
    assert_eq!(tx_ir.chain.name, "TON");
    assert_eq!(tx_ir.metadata.size, 100);
    assert_eq!(tx_ir.metadata.timestamp, Some(1700000000));
}

#[test]
fn test_transaction_validation_invalid_account_addr() {
    use decoder_ton::types::TonTransaction;

    // Create a transaction with invalid account address (wrong size)
    use decoder_ton::types::{AccountStatus, CurrencyCollection};

    let tx = TonTransaction {
        raw_bytes: vec![0u8; 100],
        cells: vec![],
        account_addr: vec![1u8; 20], // Invalid: should be 32 bytes
        lt: 12345,
        prev_trans_hash: vec![2u8; 32],
        prev_trans_lt: 12344,
        now: 1700000000,
        outmsg_cnt: 5,
        orig_status: AccountStatus::Active,
        end_status: AccountStatus::Active,
        total_fees: CurrencyCollection {
            grams: 1000,
            extra: vec![],
        },
        in_msg: None,
        out_msgs: vec![],
    };

    let result = tx.validate();
    assert!(result.is_err());
}

#[test]
fn test_transaction_validation_invalid_prev_hash() {
    use decoder_ton::types::TonTransaction;

    // Create a transaction with invalid prev_trans_hash (wrong size)
    use decoder_ton::types::{AccountStatus, CurrencyCollection};

    let tx = TonTransaction {
        raw_bytes: vec![0u8; 100],
        cells: vec![],
        account_addr: vec![1u8; 32],
        lt: 12345,
        prev_trans_hash: vec![2u8; 16], // Invalid: should be 32 bytes
        prev_trans_lt: 12344,
        now: 1700000000,
        outmsg_cnt: 5,
        orig_status: AccountStatus::Active,
        end_status: AccountStatus::Active,
        total_fees: CurrencyCollection {
            grams: 1000,
            extra: vec![],
        },
        in_msg: None,
        out_msgs: vec![],
    };

    let result = tx.validate();
    assert!(result.is_err());
}
