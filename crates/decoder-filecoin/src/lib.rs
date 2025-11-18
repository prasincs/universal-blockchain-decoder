//! Filecoin transaction decoder
//!
//! This module provides a decoder for Filecoin transactions, transforming them
//! from their native CBOR format into the universal TxIR representation.
//!
//! Filecoin uses an account-based model similar to Ethereum, with built-in actors
//! providing functionality similar to smart contracts.

use universal_decoder_core::prelude::*;

pub mod parsing;
pub mod types;

use parsing::parse_signed_message;
use types::{FilecoinAddress, FilecoinMessage, FilecoinTransaction};

/// Filecoin chain identity
#[derive(Debug, Clone, Copy)]
pub struct FilecoinChain;

impl ChainIdentity for FilecoinChain {
    fn chain_id(&self) -> u64 {
        461 // Filecoin mainnet chain ID
    }

    fn chain_name(&self) -> &str {
        "Filecoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }

    fn metadata(&self) -> Option<String> {
        Some(r#"{"consensus":"Expected Consensus","native_token":"FIL","decimals":18}"#.to_string())
    }
}

/// Filecoin decoder implementing the ChainDecoder trait
pub struct FilecoinDecoder;

impl ChainDecoder for FilecoinDecoder {
    type TxSpecific = FilecoinTransaction;
    type Chain = FilecoinChain;

    fn chain() -> Self::Chain {
        FilecoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Parse CBOR-encoded signed message
        parse_signed_message(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Filecoin transaction cannot be empty",
            ));
        }

        // Filecoin transactions are CBOR-encoded
        // First byte should be a CBOR array marker (0x82 for 2-element array)
        if raw_bytes[0] != 0x82 {
            return Err(DecoderError::invalid_structure(
                "Filecoin transaction must start with CBOR array marker (0x82)",
            ));
        }

        Ok(())
    }
}

/// Helper function to decode a Filecoin transaction with hooks
pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<FilecoinTransaction> {
    // Execute pre-decode hooks
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    // Perform decoding
    let tx = FilecoinDecoder::decode(raw_bytes)?;

    // Execute post-decode hooks
    let context = HookContext::new(HookStage::PostDecode, raw_bytes).with_chain_specific(&tx);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    Ok(tx)
}

impl ChainEncoder for FilecoinTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for FilecoinTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let msg = &self.signed_message.message;
        let sig = &self.signed_message.signature;

        // Build metadata
        let extra = format!(
            r#"{{"version":{},"sequence":{},"gas_limit":{},"method_num":{}}}"#,
            msg.version, msg.sequence, msg.gas_limit, msg.method_num
        );

        let metadata = TxMetadata {
            tx_hash: self.hash(),
            block_height: None,
            timestamp: None,
            size: self.signed_message.raw_bytes.len(),
            extra,
        };

        // Build authorization package
        let signature_scheme = match sig.sig_type {
            types::SignatureType::Secp256k1 => SignatureScheme::Ecdsa,
            types::SignatureType::Bls => SignatureScheme::Custom(2), // BLS signature type
        };

        let signature = Signature {
            data: sig.data.clone(),
            key_index: 0,
            metadata: Some(format!(r#"{{"type":{:?}}}"#, sig.sig_type)),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![signature],
            public_keys: vec![],
            signature_scheme,
        };

        // Build operations
        let operations = build_operations(msg)?;

        // Build state deltas
        let state_deltas = build_state_deltas(msg)?;

        Ok(TxIR::new(
            &FilecoinChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        let msg = &self.signed_message.message;

        if msg.gas_limit == 0 {
            return Err(DecoderError::invalid_structure("Gas limit cannot be zero"));
        }

        if msg.sequence > u64::MAX / 2 {
            return Err(DecoderError::invalid_structure("Sequence number too large"));
        }

        // Validate signature data is not empty
        if self.signed_message.signature.data.is_empty() {
            return Err(DecoderError::signature_verification(
                "Signature data cannot be empty",
            ));
        }

        Ok(())
    }
}

impl TxHashable for FilecoinTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.signed_message.raw_bytes.clone()
    }

    fn compute_hash(&self) -> Vec<u8> {
        // Filecoin uses CIDs (Content Identifiers) based on Blake2b-256
        self.signed_message.calculate_cid()
    }
}

/// Build operations from Filecoin message
fn build_operations(msg: &FilecoinMessage) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    if msg.is_transfer() {
        // Method 0 = simple value transfer
        operations.push(Operation::Transfer(Transfer {
            from: filecoin_address_to_universal(&msg.from),
            to: filecoin_address_to_universal(&msg.to),
            amount: Amount {
                value: msg.value_as_u128()?,
                decimals: 18, // FIL has 18 decimals (attoFIL)
            },
            asset: AssetId::Native,
        }));
    } else {
        // Method != 0 = actor method call
        operations.push(Operation::ContractCall(ContractCall {
            contract: filecoin_address_to_universal(&msg.to),
            method: msg.method_num.to_le_bytes().to_vec(),
            data: msg.params.clone(),
            value: Some(Amount {
                value: msg.value_as_u128()?,
                decimals: 18,
            }),
            resource_limits: ResourceLimits {
                max_units: msg.gas_limit,
                unit_price: msg.gas_premium_as_u128()? as u64,
                resource_type: ResourceType::Gas,
            },
        }));
    }

    Ok(operations)
}

/// Build state deltas from Filecoin message
fn build_state_deltas(msg: &FilecoinMessage) -> Result<StateDeltas> {
    let value = msg.value_as_u128()? as i128;

    let mut account_changes = vec![
        // Sender account
        AccountChange {
            address: filecoin_address_to_universal(&msg.from),
            nonce: Some(msg.sequence),
            balance_change: -value,
            storage_changes: vec![],
        },
        // Recipient account
        AccountChange {
            address: filecoin_address_to_universal(&msg.to),
            nonce: None,
            balance_change: value,
            storage_changes: vec![],
        },
    ];

    // If it's a method call, add a note about potential state changes
    if !msg.is_transfer() {
        // For actor calls, state changes depend on the actor implementation
        // We can only track the explicit value transfer here
        // Store the method parameters as a storage change indicator
        account_changes[1].storage_changes.push(StorageChange {
            key: vec![],
            value: Some(msg.params.clone()),
        });
    }

    Ok(StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes,
    })
}

/// Convert Filecoin address to universal Address type
fn filecoin_address_to_universal(addr: &FilecoinAddress) -> Address {
    Address {
        bytes: addr.to_bytes(),
        human_readable: Some(addr.to_string(true)), // mainnet = true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = FilecoinDecoder::chain();
        assert_eq!(chain.chain_id(), 461);
        assert_eq!(chain.chain_name(), "Filecoin");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
        assert_eq!(chain.network(), Some("mainnet"));
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(FilecoinDecoder::validate_format(&[]).is_err());

        // Invalid first byte should fail
        assert!(FilecoinDecoder::validate_format(&[0x01]).is_err());

        // Valid CBOR array marker should pass
        let valid_start = vec![0x82, 0x00, 0x00]; // Array of 2 elements
        assert!(FilecoinDecoder::validate_format(&valid_start).is_ok());
    }

    #[test]
    fn test_filecoin_transaction_version() {
        assert_eq!(FilecoinTransaction::VERSION, 1);
    }

    #[test]
    fn test_decode_with_hooks() {
        let registry = HookRegistryBuilder::new().with_size_limit(10000).build();

        // This would need a valid Filecoin transaction
        // For now, we just test the hook mechanism with placeholder data
        let tx_bytes = vec![0x82]; // CBOR array marker
        let _result = decode_with_hooks(&tx_bytes, &registry);
    }

    #[test]
    fn test_filecoin_address_conversion() {
        use types::AddressProtocol;

        let addr = FilecoinAddress::new(AddressProtocol::Id, vec![0x01]);
        let universal_addr = filecoin_address_to_universal(&addr);

        assert_eq!(universal_addr.bytes[0], 0); // Protocol ID
        assert_eq!(universal_addr.bytes[1], 0x01); // Payload
        assert!(universal_addr.human_readable.is_some());
    }
}
