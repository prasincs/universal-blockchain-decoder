//! Stellar transaction decoder
//!
//! This crate provides a pure Rust implementation of Stellar transaction decoding,
//! supporting:
//! - XDR (External Data Representation) parsing
//! - All Stellar operation types
//! - Native (XLM) and issued assets (AlphaNum4, AlphaNum12)
//! - Soroban smart contract operations
//! - Ed25519 signature scheme
//!
//! # Example
//!
//! ```ignore
//! use decoder_stellar::StellarDecoder;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = &[...]; // XDR-encoded Stellar transaction
//! let tx = StellarDecoder::decode(tx_bytes)?;
//! let tx_ir = tx.canonicalize()?;
//! ```

pub mod parsing;
pub mod types;

use decoder_primitives::prelude::*;
use parsing::parse_transaction_envelope;
use types::{StellarAsset, StellarOperation, StellarTransaction};

#[cfg(test)]
use types::StellarMemo;

/// Stellar chain identity
#[derive(Debug, Clone, Copy)]
pub struct StellarChain;

impl ChainIdentity for StellarChain {
    fn chain_id(&self) -> u64 {
        144
    }

    fn chain_name(&self) -> &str {
        "Stellar"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Stellar transaction decoder
pub struct StellarDecoder;

impl ChainDecoder for StellarDecoder {
    type TxSpecific = StellarTransaction;
    type Chain = StellarChain;

    fn chain() -> Self::Chain {
        StellarChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        parse_transaction_envelope(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Stellar transaction cannot be empty",
            ));
        }

        // Basic XDR validation: must have at least envelope type (4 bytes)
        if raw_bytes.len() < 4 {
            return Err(DecoderError::invalid_structure(
                "Stellar transaction too short",
            ));
        }

        Ok(())
    }
}

impl ChainEncoder for StellarTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for StellarTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Build metadata
        let metadata = TxMetadata {
            tx_hash: self.compute_hash(),
            block_height: None, // Not available in transaction itself
            timestamp: self.time_bounds.map(|tb| tb.min_time),
            size: self.raw_bytes.len(),
            extra: serde_json::json!({
                "fee": self.fee,
                "sequence": self.sequence_number,
                "memo": format!("{:?}", self.memo),
                "time_bounds": self.time_bounds.as_ref().map(|tb| {
                    serde_json::json!({
                        "min_time": tb.min_time,
                        "max_time": tb.max_time,
                    })
                }),
            })
            .to_string(),
        };

        // Build authorization package
        let signatures: Vec<Signature> = self
            .signatures
            .iter()
            .enumerate()
            .map(|(idx, sig)| Signature {
                data: sig.signature.clone(),
                key_index: idx,
                metadata: Some(format!(
                    "hint:{}",
                    universal_decoder_core::hex::encode(sig.hint)
                )),
            })
            .collect();

        let public_keys: Vec<PublicKey> = vec![PublicKey {
            data: self.source_account.clone(),
            key_type: KeyType::Ed25519,
        }];

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme: SignatureScheme::EdDsa, // Stellar uses Ed25519
        };

        // Build operations from Stellar operations
        let operations = self.build_operations()?;

        // Build state deltas (account changes)
        let state_deltas = self.build_state_deltas()?;

        Ok(TxIR::new(
            &StellarChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        if !self.is_valid() {
            return Err(DecoderError::invalid_structure(
                "Invalid Stellar transaction structure",
            ));
        }
        Ok(())
    }
}

impl StellarTransaction {
    /// Build TxIR operations from Stellar operations
    fn build_operations(&self) -> Result<Vec<Operation>> {
        let mut operations = Vec::new();

        for stellar_op in &self.operations {
            match stellar_op {
                StellarOperation::Payment {
                    destination,
                    asset,
                    amount,
                } => {
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: self.source_account.clone(),
                            human_readable: None,
                        },
                        to: Address {
                            bytes: destination.clone(),
                            human_readable: None,
                        },
                        amount: Amount {
                            value: (*amount).max(0) as u128, // Convert stroops to u128
                            decimals: 7, // Stellar uses 7 decimals (1 XLM = 10^7 stroops)
                        },
                        asset: self.asset_to_asset_id(asset),
                    }));
                }
                StellarOperation::CreateAccount {
                    destination,
                    starting_balance,
                } => {
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: self.source_account.clone(),
                            human_readable: None,
                        },
                        to: Address {
                            bytes: destination.clone(),
                            human_readable: None,
                        },
                        amount: Amount {
                            value: (*starting_balance).max(0) as u128,
                            decimals: 7,
                        },
                        asset: AssetId::Native, // Always XLM for CreateAccount
                    }));
                }
                StellarOperation::PathPaymentStrictReceive {
                    send_asset,
                    destination,
                    dest_asset,
                    dest_amount,
                    ..
                } => {
                    // Path payments involve swapping, so we represent as a transfer
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: self.source_account.clone(),
                            human_readable: None,
                        },
                        to: Address {
                            bytes: destination.clone(),
                            human_readable: None,
                        },
                        amount: Amount {
                            value: (*dest_amount).max(0) as u128,
                            decimals: 7,
                        },
                        asset: self.asset_to_asset_id(dest_asset),
                    }));

                    // Add metadata about the send asset
                    operations.push(Operation::Generic(GenericOperation {
                        op_type: "PathPaymentStrictReceive".to_string(),
                        data: vec![],
                        metadata: format!("send_asset: {}", self.asset_to_string(send_asset)),
                    }));
                }
                StellarOperation::PathPaymentStrictSend {
                    send_asset,
                    destination,
                    dest_asset,
                    send_amount,
                    ..
                } => {
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: self.source_account.clone(),
                            human_readable: None,
                        },
                        to: Address {
                            bytes: destination.clone(),
                            human_readable: None,
                        },
                        amount: Amount {
                            value: (*send_amount).max(0) as u128,
                            decimals: 7,
                        },
                        asset: self.asset_to_asset_id(send_asset),
                    }));

                    operations.push(Operation::Generic(GenericOperation {
                        op_type: "PathPaymentStrictSend".to_string(),
                        data: vec![],
                        metadata: format!("dest_asset: {}", self.asset_to_string(dest_asset)),
                    }));
                }
                StellarOperation::InvokeHostFunction {
                    function_type,
                    parameters,
                } => {
                    operations.push(Operation::ContractCall(ContractCall {
                        contract: Address {
                            bytes: vec![],
                            human_readable: Some("Soroban".to_string()),
                        },
                        method: function_type.to_be_bytes().to_vec(),
                        data: parameters.clone(),
                        value: None,
                        resource_limits: ResourceLimits {
                            max_units: 0,
                            unit_price: 0,
                            resource_type: ResourceType::ComputeUnits,
                        },
                    }));
                }
                StellarOperation::AccountMerge { destination } => {
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: self.source_account.clone(),
                            human_readable: None,
                        },
                        to: Address {
                            bytes: destination.clone(),
                            human_readable: None,
                        },
                        amount: Amount {
                            value: 0, // Actual amount determined by account balance
                            decimals: 7,
                        },
                        asset: AssetId::Native,
                    }));
                }
                // All other operations represented as Generic
                _ => {
                    operations.push(Operation::Generic(GenericOperation {
                        op_type: stellar_op.operation_type().to_string(),
                        data: vec![],
                        metadata: format!("{:?}", stellar_op),
                    }));
                }
            }
        }

        Ok(operations)
    }

    /// Build state deltas (account changes)
    fn build_state_deltas(&self) -> Result<StateDeltas> {
        // Balance/nonce effect guesses are NOT byte-derivable and were removed
        // from TxIR (docs/CONCEPTS_REVIEW.md C1).
        Ok(StateDeltas {
            inputs: vec![],  // Account-based chain, no UTXOs
            outputs: vec![], // Account-based chain, no UTXOs
        })
    }

    /// Convert Stellar asset to AssetId
    fn asset_to_asset_id(&self, asset: &StellarAsset) -> AssetId {
        match asset {
            StellarAsset::Native => AssetId::Native,
            StellarAsset::CreditAlphanum4 { code, issuer } => {
                let code_str = String::from_utf8_lossy(code);
                let trimmed = code_str.trim_end_matches('\0');
                let asset_str = format!(
                    "{}:{}",
                    trimmed,
                    universal_decoder_core::hex::encode(&issuer[..8])
                );
                AssetId::Token(asset_str.into_bytes())
            }
            StellarAsset::CreditAlphanum12 { code, issuer } => {
                let code_str = String::from_utf8_lossy(code);
                let trimmed = code_str.trim_end_matches('\0');
                let asset_str = format!(
                    "{}:{}",
                    trimmed,
                    universal_decoder_core::hex::encode(&issuer[..8])
                );
                AssetId::Token(asset_str.into_bytes())
            }
        }
    }

    /// Convert Stellar asset to string for metadata
    fn asset_to_string(&self, asset: &StellarAsset) -> String {
        asset.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = StellarDecoder::chain();
        assert_eq!(chain.chain_id(), 144);
        assert_eq!(chain.chain_name(), "Stellar");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format_empty() {
        assert!(StellarDecoder::validate_format(&[]).is_err());
    }

    #[test]
    fn test_validate_format_too_short() {
        assert!(StellarDecoder::validate_format(&[0x01, 0x02]).is_err());
    }

    #[test]
    fn test_validate_format_valid() {
        let dummy = vec![0u8; 100];
        assert!(StellarDecoder::validate_format(&dummy).is_ok());
    }

    #[test]
    fn test_asset_conversion() {
        let tx = StellarTransaction {
            source_account: vec![0; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![],
            signatures: vec![],
            raw_bytes: vec![],
            envelope_type: types::EnvelopeType::Tx,
            network_id: None,
        };

        // Test native asset
        let native = StellarAsset::Native;
        assert_eq!(tx.asset_to_asset_id(&native), AssetId::Native);

        // Test AlphaNum4
        let usdc = StellarAsset::CreditAlphanum4 {
            code: [b'U', b'S', b'D', b'C'],
            issuer: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };
        if let AssetId::Token(bytes) = tx.asset_to_asset_id(&usdc) {
            let asset_str = String::from_utf8(bytes).unwrap();
            assert!(asset_str.starts_with("USDC:"));
        } else {
            panic!("Expected Token asset");
        }
    }
}
