// Build script for decoder-evm
//
// Since we use Borsh binary format (chains.borsh) instead of parsing JSON at build time,
// this build script is minimal and just verifies the binary exists.

use std::path::Path;

fn main() {
    // Verify Borsh binary exists
    let borsh_path = Path::new("data/chains.borsh");

    if !borsh_path.exists() {
        eprintln!("\n========================================");
        eprintln!("ERROR: Chain registry binary not found!");
        eprintln!("========================================");
        eprintln!();
        eprintln!("Expected: data/chains.borsh");
        eprintln!();
        eprintln!("To generate the binary, run:");
        eprintln!("  cargo run -p chain-registry-generator");
        eprintln!();
        eprintln!("Or use the update script:");
        eprintln!("  ./scripts/decoder-evm/update-chains.sh");
        eprintln!();
        panic!("Missing chain registry binary");
    }

    // Rerun build script if binary changes
    println!("cargo:rerun-if-changed=data/chains.borsh");
    println!("cargo:rerun-if-changed=data/chains.metadata.txt");

    // Note: We no longer parse JSON at build time!
    // The vendored JSON files are kept for verification purposes only.
    // See docs/BORSH_REGISTRY_MIGRATION.md for details.
}
