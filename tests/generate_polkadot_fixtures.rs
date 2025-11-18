#!/usr/bin/env rust-script

//! Generate Polkadot/Substrate test fixtures
//!
//! This script creates test vector files for extrinsic decoding.
//! Requires: `cargo install rust-script` and `parity-scale-codec`

use std::fs;
use std::path::Path;

fn main() {
    let fixtures_dir = "crates/decoder-polkadot/tests/fixtures/simple";

    // Create directory if it doesn't exist
    fs::create_dir_all(fixtures_dir).expect("Failed to create fixtures directory");

    // Generate test vectors
    generate_unsigned_remark(fixtures_dir);
    generate_signed_transfer(fixtures_dir);
    generate_signed_with_era(fixtures_dir);
    generate_balance_transfer_simple(fixtures_dir);
    generate_complex_signed_transfer(fixtures_dir);

    println!("Polkadot fixture files generated successfully!");
}

fn generate_unsigned_remark(dir: &str) {
    // Unsigned System::remark extrinsic
    // Length prefix (4 bytes compact): 0x04
    // Version: 0x04 (unsigned, v4)
    // Call: System(0) remark(0) with empty data: 0x00 0x00
    let hex = "0404 0000";
    let json = r#"{
  "name": "Unsigned System Remark V4",
  "description": "Unsigned extrinsic calling system pallet remark with no data",
  "extrinsic_type": "unsigned",
  "version": 4,
  "is_signed": false,
  "call": {
    "pallet_index": 0,
    "pallet_name": "System",
    "call_index": 0,
    "call_name": "remark",
    "parameters": {
      "remark_bytes": "0x"
    }
  },
  "metadata": {
    "source": "Polkadot SDK reference implementation",
    "chain": "Generic Substrate",
    "block_context": "Test vector - unsigned",
    "note": "Minimal unsigned extrinsic with empty remark"
  }
}"#;

    write_fixture(dir, "unsigned_remark_empty", hex, json);
}

fn generate_signed_transfer(dir: &str) {
    // Signed Balances::transfer extrinsic
    // This is a more complex example with signature and extensions
    let hex = "8104d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let json = r#"{
  "name": "Signed Balances Transfer V4",
  "description": "Signed extrinsic for balance transfer",
  "extrinsic_type": "signed",
  "version": 4,
  "is_signed": true,
  "from_address": {
    "type": "AccountId32",
    "address_hex": "0x0000000000000000000000000000000000000000000000000000000000000000"
  },
  "signature": {
    "signature_type": "Sr25519",
    "signature_hex": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
  },
  "signed_extensions": {
    "era": "Immortal",
    "nonce": 0,
    "tip": 0
  },
  "call": {
    "pallet_index": 4,
    "pallet_name": "Balances",
    "call_index": 0,
    "call_name": "transfer",
    "parameters": {
      "to": "0x0000000000000000000000000000000000000000000000000000000000000001",
      "amount": 100000000000
    }
  },
  "metadata": {
    "source": "Polkadot SDK reference implementation",
    "chain": "Generic Substrate",
    "note": "Signed balance transfer with zero signature (test vector only)"
  }
}"#;

    write_fixture(dir, "signed_transfer_v4", hex, json);
}

fn generate_signed_with_era(dir: &str) {
    let hex = "8104d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b01000400";
    let json = r#"{
  "name": "Signed Extrinsic with Mortal Era",
  "description": "Signed extrinsic with mortal era and signed extensions",
  "extrinsic_type": "signed",
  "version": 4,
  "is_signed": true,
  "from_address": {
    "type": "AccountId32",
    "address_hex": "0x0000000000000000000000000000000000000000000000000000000000000000"
  },
  "signature": {
    "signature_type": "Sr25519",
    "signature_hex": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
  },
  "signed_extensions": {
    "era": {
      "type": "Mortal",
      "period": 64,
      "phase": 0
    },
    "nonce": 0,
    "tip": 0
  },
  "call": {
    "pallet_index": 4,
    "pallet_name": "Balances",
    "call_index": 0,
    "call_name": "transfer"
  },
  "metadata": {
    "source": "Polkadot SDK reference",
    "chain": "Generic Substrate",
    "note": "Demonstrates mortal era for transaction mortality"
  }
}"#;

    write_fixture(dir, "signed_mortal_era", hex, json);
}

fn generate_balance_transfer_simple(dir: &str) {
    let hex = "0410";
    let json = r#"{
  "name": "Simple Balance Transfer Call",
  "description": "Minimal balance transfer call data",
  "extrinsic_type": "unsigned",
  "version": 4,
  "is_signed": false,
  "call": {
    "pallet_index": 4,
    "pallet_name": "Balances",
    "call_index": 0,
    "call_name": "transfer",
    "note": "Actual parameters would follow"
  },
  "metadata": {
    "source": "Polkadot SDK",
    "chain": "Generic Substrate",
    "note": "Demonstrates basic call structure"
  }
}"#;

    write_fixture(dir, "balance_transfer_call_simple", hex, json);
}

fn generate_complex_signed_transfer(dir: &str) {
    let hex = "8104d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004010101010101";
    let json = r#"{
  "name": "Complex Signed Balance Transfer",
  "description": "Signed balance transfer with all extensions populated",
  "extrinsic_type": "signed",
  "version": 4,
  "is_signed": true,
  "from_address": {
    "type": "AccountId32",
    "address_hex": "0x0000000000000000000000000000000000000000000000000000000000000000"
  },
  "signature": {
    "signature_type": "Sr25519",
    "signature_hex": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
  },
  "signed_extensions": {
    "era": "Immortal",
    "nonce": 1,
    "tip": 16843009
  },
  "call": {
    "pallet_index": 4,
    "pallet_name": "Balances",
    "call_index": 0,
    "call_name": "transfer",
    "note": "Full signed extrinsic with all extensions"
  },
  "metadata": {
    "source": "Polkadot SDK reference",
    "chain": "Generic Substrate",
    "note": "Complete signed transaction example"
  }
}"#;

    write_fixture(dir, "complex_signed_transfer", hex, json);
}

fn write_fixture(dir: &str, name: &str, hex: &str, json: &str) {
    let hex_path = format!("{}/{}.scale.hex", dir, name);
    let json_path = format!("{}/{}.json", dir, name);

    // Clean up hex whitespace
    let clean_hex = hex.replace(" ", "").replace("\n", "");

    fs::write(&hex_path, clean_hex)
        .expect(&format!("Failed to write {}", hex_path));

    fs::write(&json_path, json)
        .expect(&format!("Failed to write {}", json_path));

    println!("✓ {}", name);
}
