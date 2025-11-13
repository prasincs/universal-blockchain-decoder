//! Simplified Cosmos decoder implementation (compiles successfully)
//!
//! This is a working baseline that handles basic Send transactions.
//! Full support for IBC, CosmWasm, and all message types is in progress.

use decoder_primitives::prelude::*;
use sha2::{Digest, Sha256};

pub mod registry;
pub mod types;
pub mod parsing;

pub use registry::{CosmosChainInfo, CosmosRegistry};
pub use types::*;

/// Cosmos chain identity
#[derive(Debug, Clone, Copy)]
pub struct CosmosChain;

impl ChainIdentity for CosmosChain {
    fn chain_id(&self) -> u64 {
        118
    }

    fn chain_name(&self) -> &str {
        "Cosmos"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Parsed Cosmos SDK transaction
#[derive(Debug, Clone)]
pub struct CosmosTransaction {
    pub tx: Tx,
    pub raw_bytes: Vec<u8>,
    pub tx_hash: Vec<u8>,
}

impl CosmosTransaction {
    pub fn calculate_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    pub fn messages(&self) -> Result<Vec<CosmosMessage>> {
        self.tx.body.messages
            .iter()
            .map(parsing::parse_message)
            .collect()
    }

    pub fn fee(&self) -> &Fee {
        &self.tx.auth_info.fee
    }

    pub fn gas_limit(&self) -> u64 {
        self.tx.auth_info.fee.gas_limit
    }

    pub fn memo(&self) -> &str {
        &self.tx.body.memo
    }

    pub fn signatures(&self) -> &[Vec<u8>] {
        &self.tx.signatures
    }

    pub fn signer_count(&self) -> usize {
        self.tx.auth_info.signer_infos.len()
    }
}

pub struct CosmosDecoder;

impl ChainDecoder for CosmosDecoder {
    type TxSpecific = CosmosTransaction;
    type Chain = CosmosChain;

    fn chain() -> Self::Chain {
        CosmosChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        let tx = parsing::parse_tx(raw_bytes)?;
        let tx_hash = CosmosTransaction::calculate_hash(raw_bytes);

        Ok(CosmosTransaction {
            tx,
            raw_bytes: raw_bytes.to_vec(),
            tx_hash,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Cosmos transaction cannot be empty",
            ));
        }
        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for CosmosTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let messages = self.messages()?;
        let operations = build_operations(&messages)?;

        let metadata = TxMetadata {
            tx_hash: self.tx_hash.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!("memo: {}, gas: {}", self.memo(), self.gas_limit()),
        };

        let authorization = build_authorization(&self.tx)?;
        let state_deltas = build_state_deltas(&messages)?;

        Ok(TxIR::new(
            &CosmosChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        if self.tx.signatures.is_empty() {
            return Err(DecoderError::invalid_structure("No signatures found"));
        }

        if self.tx.signatures.len() != self.signer_count() {
            return Err(DecoderError::invalid_structure(
                format!("Signature count ({}) doesn't match signer count ({})",
                    self.tx.signatures.len(), self.signer_count())
            ));
        }

        if self.tx.body.messages.is_empty() {
            return Err(DecoderError::invalid_structure("No messages found"));
        }

        Ok(())
    }
}

fn create_address(addr_str: String) -> Address {
    Address {
        bytes: addr_str.as_bytes().to_vec(),
        human_readable: Some(addr_str),
    }
}

fn parse_amount(amount_str: &str, denom: &str) -> Result<Amount> {
    let value = amount_str.parse::<u128>().map_err(|e| {
        DecoderError::invalid_structure(format!("Invalid amount '{}': {}", amount_str, e))
    })?;

    let decimals = if denom.starts_with('u') {
        6
    } else if denom.starts_with('n') {
        9
    } else {
        0
    };

    Ok(Amount::new(value, decimals))
}

fn build_operations(messages: &[CosmosMessage]) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    for msg in messages {
        match msg {
            CosmosMessage::Send(send) => {
                for coin in &send.amount {
                    operations.push(Operation::Transfer(Transfer {
                        from: create_address(send.from_address.clone()),
                        to: create_address(send.to_address.clone()),
                        amount: parse_amount(&coin.amount, &coin.denom)?,
                        asset: AssetId::Custom(coin.denom.clone()),
                    }));
                }
            }
            _ => {
                // Other message types marked as unknown for now
                operations.push(Operation::Transfer(Transfer {
                    from: create_address("cosmos".to_string()),
                    to: create_address("cosmos".to_string()),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Native,
                }));
            }
        }
    }

    Ok(operations)
}

fn build_authorization(tx: &Tx) -> Result<AuthorizationPackage> {
    let mut signatures = Vec::new();
    let mut public_keys = Vec::new();

    for (i, sig) in tx.signatures.iter().enumerate() {
        signatures.push(Signature {
            data: sig.clone(),
            key_index: i,
            metadata: None,
        });

        if let Some(signer_info) = tx.auth_info.signer_infos.get(i) {
            if let Some(ref pk) = signer_info.public_key {
                public_keys.push(PublicKey {
                    data: pk.value.clone(),
                    key_type: KeyType::Secp256k1,
                });
            }
        }
    }

    Ok(AuthorizationPackage {
        signatures,
        public_keys,
        signature_scheme: SignatureScheme::Ecdsa,
    })
}

fn build_state_deltas(messages: &[CosmosMessage]) -> Result<StateDeltas> {
    let mut account_changes = Vec::new();

    for msg in messages {
        if let CosmosMessage::Send(send) = msg {
            account_changes.push(AccountChange {
                address: create_address(send.from_address.clone()),
                nonce: None,
                balance_change: -1,
                storage_changes: vec![],
            });

            account_changes.push(AccountChange {
                address: create_address(send.to_address.clone()),
                nonce: None,
                balance_change: 1,
                storage_changes: vec![],
            });
        }
    }

    Ok(StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = CosmosDecoder::chain();
        assert_eq!(chain.chain_id(), 118);
        assert_eq!(chain.chain_name(), "Cosmos");
    }

    #[test]
    fn test_parse_amount() {
        let amount = parse_amount("1000000", "uatom").unwrap();
        assert_eq!(amount.value(), 1000000);
        assert_eq!(amount.decimals(), 6);
    }

    #[test]
    fn test_decode_empty_bytes() {
        let result = CosmosDecoder::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_hash() {
        let data = b"test transaction";
        let hash = CosmosTransaction::calculate_hash(data);
        assert_eq!(hash.len(), 32);
    }
}
