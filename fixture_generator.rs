// Fixture generator to extract Aptos transactions from test code
// This generates BCS-encoded transaction hex and JSON metadata

use std::fs;
use std::path::Path;

fn main() {
    // Extract test vectors from the aptos-core source
    // This reads the hardcoded BCS bytes from the test and generates fixtures

    // Test 1: WebAuthn transaction from verify_webauthn_single_key_auth
    let webauthn_txn_bcs_bytes: Vec<u8> = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 13, 97, 112, 116, 111, 115, 95, 97, 99,
        99, 111, 117, 110, 116, 14, 116, 114, 97, 110, 115, 102, 101, 114, 95, 99, 111, 105,
        110, 115, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 10, 97, 112, 116, 111, 115, 95, 99, 111, 105, 110, 9, 65, 112,
        116, 111, 115, 67, 111, 105, 110, 0, 2, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 8, 232, 3, 0, 0, 0, 0, 0, 0, 232,
        3, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 128, 116, 23, 188, 190, 0, 0, 0, 89,
    ];

    let webauthn_hex = hex::encode(&webauthn_txn_bcs_bytes);

    // Create JSON metadata
    let webauthn_json = r#"{
  "name": "webauthn_single_key_transfer",
  "description": "Transaction with WebAuthn single key authentication",
  "transaction_type": "entry_function",
  "sender": "0x0000000000000000000000000000000000000000000000000000000000000001",
  "sequence_number": 0,
  "gas": {
    "max_gas_amount": 1000,
    "gas_unit_price": 100
  },
  "expiration_timestamp_secs": 3016550528,
  "chain_id": 89,
  "payload": {
    "type": "entry_function",
    "function": "0x1::aptos_account::transfer_coins",
    "type_arguments": ["0x1::aptos_coin::AptosCoin"],
    "arguments": [
      "0x0000000000000000000000000000000000000000000000000000000000000001",
      "1000"
    ]
  },
  "authenticator": {
    "type": "single_sender",
    "public_key_type": "secp256r1_ecdsa",
    "signature_type": "webauthn"
  }
}
"#;

    write_fixture("webauthn_single_key_transfer", &webauthn_hex, webauthn_json);

    println!("Generated fixture: webauthn_single_key_transfer");
}

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn write_fixture(name: &str, hex_data: &str, json_data: &str) {
    let base_path = "/home/user/universal-blockchain-decoder/crates/decoder-aptos/tests/fixtures/simple";

    // Write hex file
    let hex_path = format!("{}/{}.bcs.hex", base_path, name);
    fs::write(&hex_path, hex_data).expect(&format!("Failed to write {}", hex_path));

    // Write JSON metadata file
    let json_path = format!("{}/{}.json", base_path, name);
    fs::write(&json_path, json_data).expect(&format!("Failed to write {}", json_path));

    println!("Wrote fixtures to:");
    println!("  {}", hex_path);
    println!("  {}", json_path);
}
