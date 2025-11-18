//! {{CHAIN_NAME}} Transaction Decoder - Multi-Family Template
//!
//! This chain supports multiple transaction families:
//! {{FAMILIES_LIST}}
//!
//! ## Architecture
//!
//! ```text
//! Raw bytes → Format detection → Family decoder → {{CHAIN_NAME}}Transaction
//!              │
//!              ├─ {{FAMILY_1}} format? → {{FAMILY_1}}Decoder
//!              ├─ {{FAMILY_2}} format? → {{FAMILY_2}}Decoder
//!              └─ ...
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use decoder_{{CHAIN_NAME_LOWER}}::*;
//!
//! let tx = {{CHAIN_NAME}}Decoder::decode(&raw_bytes)?;
//!
//! match tx {
//!     {{CHAIN_NAME}}Transaction::{{FAMILY_1_VARIANT}}(tx) => {
//!         // Handle {{FAMILY_1}} transaction
//!     }
//!     {{CHAIN_NAME}}Transaction::{{FAMILY_2_VARIANT}}(tx) => {
//!         // Handle {{FAMILY_2}} transaction
//!     }
//! }
//! ```

use universal_decoder_core::prelude::*;

// Import family modules
{{FAMILY_IMPORTS}}

pub mod chain;
pub mod routing;

pub use chain::{{CHAIN_NAME}}Chain;
pub use routing::{{CHAIN_NAME}}Decoder;

/// {{CHAIN_NAME}} transaction (multi-family enum)
#[derive(Debug, Clone)]
pub enum {{CHAIN_NAME}}Transaction {
    {{TRANSACTION_VARIANTS}}
}

impl {{CHAIN_NAME}}Transaction {
    /// Decode transaction with automatic family detection
    pub fn decode(raw_bytes: &[u8]) -> Result<Self> {
        {{CHAIN_NAME}}Decoder::decode(raw_bytes)
    }

    /// Get transaction family
    pub fn family(&self) -> &str {
        match self {
            {{FAMILY_MATCH_ARMS}}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = {{CHAIN_NAME}}Decoder::chain();
        assert_eq!(chain.chain_id(), {{CHAIN_ID}});
        assert_eq!(chain.chain_name(), "{{CHAIN_NAME}}");
    }

    {{FAMILY_TESTS}}
}

// -----------------------------------------------------------
// IMPLEMENTATION GUIDE
// -----------------------------------------------------------
//
// To use this template:
//
// 1. Replace placeholders:
//    {{CHAIN_NAME}}       → Your chain name (e.g., "Evmos")
//    {{CHAIN_NAME_LOWER}} → Lowercase (e.g., "evmos")
//    {{CHAIN_ID}}         → Chain ID number
//    {{FAMILIES_LIST}}    → List of families (e.g., "Cosmos SDK, EVM")
//    {{FAMILY_1}}, etc.   → Family names
//
// 2. Add family imports:
//    pub mod cosmos;  // Re-exports decoder-cosmos
//    pub mod evm;     // Re-exports decoder-evm
//
// 3. Implement routing.rs (see template_routing.rs)
//
// 4. Implement chain.rs (see template_chain.rs)
//
// 5. Add test fixtures for each family
//
// Example for Evmos:
//   - Replace {{CHAIN_NAME}} with "Evmos"
//   - Families: ["Cosmos", "Evm"]
//   - Routing: Try EVM (0x00-0x7f or 0xc0+), else Cosmos
