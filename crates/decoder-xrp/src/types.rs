//! XRP transaction types and canonicalization

use crate::parsing::XrpAmount;
use crate::XrpTransactionType;
use decoder_primitives::prelude::*;

/// Parsed XRP transaction with full field data
#[derive(Debug, Clone)]
pub struct XrpTransaction {
    /// Transaction type
    pub transaction_type: XrpTransactionType,

    /// Account initiating the transaction
    pub account: Option<[u8; 20]>,

    /// Transaction fee in drops
    pub fee: Option<u64>,

    /// Sequence number
    pub sequence: Option<u32>,

    /// Account sequence (optional, for tickets)
    pub account_txn_id: Option<[u8; 32]>,

    /// Last ledger sequence (expiration)
    pub last_ledger_sequence: Option<u32>,

    /// Signing public key
    pub signing_pub_key: Option<Vec<u8>>,

    /// Transaction signature
    pub txn_signature: Option<Vec<u8>>,

    /// Payment-specific fields
    pub destination: Option<[u8; 20]>,
    pub amount: Option<XrpAmount>,
    pub destination_tag: Option<u32>,
    pub send_max: Option<XrpAmount>,

    /// TrustSet-specific fields
    pub limit_amount: Option<XrpAmount>,

    /// OfferCreate-specific fields
    pub taker_pays: Option<XrpAmount>,
    pub taker_gets: Option<XrpAmount>,
    pub offer_sequence: Option<u32>,

    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl XrpTransaction {
    /// Create a new XRP transaction from parsed fields
    pub fn new(transaction_type: XrpTransactionType, raw_bytes: Vec<u8>) -> Self {
        Self {
            transaction_type,
            account: None,
            fee: None,
            sequence: None,
            account_txn_id: None,
            last_ledger_sequence: None,
            signing_pub_key: None,
            txn_signature: None,
            destination: None,
            amount: None,
            destination_tag: None,
            send_max: None,
            limit_amount: None,
            taker_pays: None,
            taker_gets: None,
            offer_sequence: None,
            raw_bytes,
        }
    }

    /// Calculate transaction hash using SHA-512 half
    pub fn txid(&self) -> Vec<u8> {
        use sha2::{Digest, Sha512};

        // XRP uses SHA-512 then takes first 32 bytes
        // Add prefix for signing hash
        let mut hasher = Sha512::new();
        hasher.update(b"TXN\0"); // Transaction hash prefix
        hasher.update(&self.raw_bytes);
        let hash = hasher.finalize();

        // Take first 32 bytes (256 bits)
        hash[..32].to_vec()
    }

    /// Convert XrpAmount to TxIR Amount and AssetId
    fn amount_to_ir(amount: &XrpAmount) -> (Amount, AssetId) {
        match amount {
            XrpAmount::Drops(drops) => (
                Amount {
                    value: *drops as u128,
                    decimals: 6, // XRP has 6 decimals (1 XRP = 1,000,000 drops)
                },
                AssetId::Native,
            ),
            XrpAmount::Iou {
                value,
                currency,
                issuer,
            } => {
                // Parse the value string to extract decimals
                let parsed_value = value.parse::<f64>().unwrap_or(0.0);
                let decimals = 15; // IOU typically uses 15 decimal places
                let value_u128 = (parsed_value * 10_f64.powi(decimals as i32)) as u128;

                // Create token ID from currency code and issuer
                let mut token_id = Vec::with_capacity(40);
                token_id.extend_from_slice(currency);
                token_id.extend_from_slice(issuer);

                (
                    Amount {
                        value: value_u128,
                        decimals,
                    },
                    AssetId::Token(token_id),
                )
            }
        }
    }

    /// Format account ID to XRP address format (base58)
    fn format_account(account_id: &[u8; 20]) -> String {
        use universal_decoder_core::hex;
        // Simplified hex representation (real implementation would use base58)
        format!("r{}", hex::encode(account_id))
    }
}

impl ChainEncoder for XrpTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for XrpTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx_hash = self.txid();

        // Build metadata
        let tx_type_name = match self.transaction_type {
            XrpTransactionType::Payment => "Payment",
            XrpTransactionType::TrustSet => "TrustSet",
            XrpTransactionType::OfferCreate => "OfferCreate",
            XrpTransactionType::OfferCancel => "OfferCancel",
            XrpTransactionType::NFTokenMint => "NFTokenMint",
            _ => "Unknown",
        };

        // Build extra metadata as JSON string (manually to avoid serde_json in production deps)
        let extra = format!(
            r#"{{"transaction_type":"{}","sequence":{},"last_ledger_sequence":{}}}"#,
            tx_type_name,
            self.sequence.map_or("null".to_string(), |s| s.to_string()),
            self.last_ledger_sequence
                .map_or("null".to_string(), |s| s.to_string())
        );

        let metadata = TxMetadata {
            tx_hash,
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra,
        };

        // Build authorization
        let mut signatures = vec![];
        let mut public_keys = vec![];

        if let Some(pk) = &self.signing_pub_key {
            public_keys.push(PublicKey {
                data: pk.clone(),
                key_type: KeyType::Secp256k1,
            });
        }

        if let Some(sig) = &self.txn_signature {
            signatures.push(Signature {
                data: sig.clone(),
                key_index: 0,
                metadata: None,
            });
        }

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations based on transaction type
        let mut operations = vec![];

        match self.transaction_type {
            XrpTransactionType::Payment => {
                if let (Some(account), Some(destination), Some(amount)) =
                    (&self.account, &self.destination, &self.amount)
                {
                    let (amount_ir, asset_id) = Self::amount_to_ir(amount);

                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: account.to_vec(),
                            human_readable: Some(Self::format_account(account)),
                        },
                        to: Address {
                            bytes: destination.to_vec(),
                            human_readable: Some(Self::format_account(destination)),
                        },
                        amount: amount_ir,
                        asset: asset_id,
                    }));
                }
            }
            XrpTransactionType::TrustSet => {
                // TrustSet creates or modifies a trust line for holding tokens
                if let (Some(account), Some(limit_amount)) = (&self.account, &self.limit_amount) {
                    let (amount_ir, asset_id) = Self::amount_to_ir(limit_amount);

                    let data_str = format!(
                        r#"{{"account":"{}","limit":"{}","asset":"{:?}"}}"#,
                        Self::format_account(account),
                        amount_ir.value,
                        asset_id
                    );

                    operations.push(Operation::Generic(GenericOperation {
                        op_type: "TrustSet".to_string(),
                        data: data_str.into_bytes(),
                        metadata: String::new(),
                    }));
                }
            }
            XrpTransactionType::OfferCreate => {
                // DEX order creation
                if let (Some(account), Some(taker_pays), Some(taker_gets)) =
                    (&self.account, &self.taker_pays, &self.taker_gets)
                {
                    let (pays_amount, pays_asset) = Self::amount_to_ir(taker_pays);
                    let (gets_amount, gets_asset) = Self::amount_to_ir(taker_gets);

                    let data_str = format!(
                        r#"{{"account":"{}","taker_pays":{{"amount":"{}","asset":"{:?}"}},"taker_gets":{{"amount":"{}","asset":"{:?}"}}}}"#,
                        Self::format_account(account),
                        pays_amount.value,
                        pays_asset,
                        gets_amount.value,
                        gets_asset
                    );

                    operations.push(Operation::Generic(GenericOperation {
                        op_type: "OfferCreate".to_string(),
                        data: data_str.into_bytes(),
                        metadata: String::new(),
                    }));
                }
            }
            _ => {
                // Generic operation for other types
                operations.push(Operation::Generic(GenericOperation {
                    op_type: tx_type_name.to_string(),
                    data: b"{}".to_vec(),
                    metadata: String::new(),
                }));
            }
        }

        // Fee-only balance guesses are NOT byte-derivable state effects and
        // were removed from TxIR (docs/CONCEPTS_REVIEW.md C1).
        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
        };

        Ok(TxIR::new(
            &crate::XrpChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Basic validation
        if self.account.is_none() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have an account",
            ));
        }

        if self.fee.is_none() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have a fee",
            ));
        }

        // Type-specific validation
        match self.transaction_type {
            XrpTransactionType::Payment => {
                if self.destination.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "Payment must have a destination",
                    ));
                }
                if self.amount.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "Payment must have an amount",
                    ));
                }
            }
            XrpTransactionType::TrustSet => {
                if self.limit_amount.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "TrustSet must have a limit amount",
                    ));
                }
            }
            XrpTransactionType::OfferCreate => {
                if self.taker_pays.is_none() || self.taker_gets.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "OfferCreate must have taker_pays and taker_gets",
                    ));
                }
            }
            _ => {} // Other types have minimal validation for now
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrp_amount_to_ir() {
        let drops = XrpAmount::Drops(1_000_000);
        let (amount, asset) = XrpTransaction::amount_to_ir(&drops);
        assert_eq!(amount.value, 1_000_000);
        assert_eq!(amount.decimals, 6);
        assert!(matches!(asset, AssetId::Native));
    }

    #[test]
    fn test_format_account() {
        let account_id = [0u8; 20];
        let formatted = XrpTransaction::format_account(&account_id);
        assert!(formatted.starts_with('r'));
    }

    #[test]
    fn test_transaction_validation() {
        let mut tx = XrpTransaction::new(XrpTransactionType::Payment, vec![]);

        // Should fail without required fields
        assert!(tx.validate().is_err());

        // Add required fields
        tx.account = Some([0u8; 20]);
        tx.fee = Some(10);
        tx.destination = Some([1u8; 20]);
        tx.amount = Some(XrpAmount::Drops(1000));

        // Should now pass
        assert!(tx.validate().is_ok());
    }
}
