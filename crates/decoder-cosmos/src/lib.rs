//! Simplified Cosmos decoder implementation (compiles successfully)
//!
//! This is a working baseline that handles basic Send transactions.
//! Full support for IBC, CosmWasm, and all message types is in progress.

use decoder_primitives::prelude::*;
use sha2::{Digest, Sha256};

pub mod parsing;
pub mod registry;
pub mod types;

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
        self.tx
            .body
            .messages
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

impl ChainEncoder for CosmosTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
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
            return Err(DecoderError::invalid_structure(format!(
                "Signature count ({}) doesn't match signer count ({})",
                self.tx.signatures.len(),
                self.signer_count()
            )));
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
            // === Bank Messages ===
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
            CosmosMessage::MultiSend(multi) => {
                // Track multi-send as multiple transfers
                for output in &multi.outputs {
                    for coin in &output.coins {
                        operations.push(Operation::Transfer(Transfer {
                            from: create_address("multi".to_string()), // Aggregated source
                            to: create_address(output.address.clone()),
                            amount: parse_amount(&coin.amount, &coin.denom)?,
                            asset: AssetId::Custom(coin.denom.clone()),
                        }));
                    }
                }
            }

            // === Staking Messages ===
            CosmosMessage::Delegate(delegate) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(delegate.delegator_address.clone()),
                    to: create_address(delegate.validator_address.clone()),
                    amount: parse_amount(&delegate.amount.amount, &delegate.amount.denom)?,
                    asset: AssetId::Custom(format!("stake:{}", delegate.amount.denom)),
                }));
            }
            CosmosMessage::Undelegate(undelegate) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(undelegate.validator_address.clone()),
                    to: create_address(undelegate.delegator_address.clone()),
                    amount: parse_amount(&undelegate.amount.amount, &undelegate.amount.denom)?,
                    asset: AssetId::Custom(format!("unstake:{}", undelegate.amount.denom)),
                }));
            }
            CosmosMessage::BeginRedelegate(redelegate) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(redelegate.validator_src_address.clone()),
                    to: create_address(redelegate.validator_dst_address.clone()),
                    amount: parse_amount(&redelegate.amount.amount, &redelegate.amount.denom)?,
                    asset: AssetId::Custom(format!("redelegate:{}", redelegate.amount.denom)),
                }));
            }

            // === IBC Messages ===
            CosmosMessage::IbcTransfer(ibc) => {
                // IBC transfer: cross-chain token transfer
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(ibc.sender.clone()),
                    to: create_address(format!("{}@{}", ibc.receiver, ibc.source_channel)),
                    amount: parse_amount(&ibc.token.amount, &ibc.token.denom)?,
                    asset: AssetId::Custom(format!(
                        "ibc/{}/{}",
                        ibc.source_channel, ibc.token.denom
                    )),
                }));
            }
            CosmosMessage::IbcRecvPacket(recv) => {
                // Receiving an IBC packet (cross-chain incoming)
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(format!(
                        "ibc:{}/{}",
                        recv.packet.source_port, recv.packet.source_channel
                    )),
                    to: create_address(format!(
                        "ibc:{}/{}",
                        recv.packet.destination_port, recv.packet.destination_channel
                    )),
                    amount: Amount::new(recv.packet.sequence as u128, 0),
                    asset: AssetId::Custom(format!("ibc:packet:{}", recv.packet.sequence)),
                }));
            }
            CosmosMessage::IbcAcknowledgement(_ack) => {
                // IBC acknowledgement (confirming receipt)
                operations.push(Operation::Transfer(Transfer {
                    from: create_address("ibc:ack".to_string()),
                    to: create_address("ibc:ack".to_string()),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Custom("ibc:acknowledgement".to_string()),
                }));
            }
            CosmosMessage::IbcTimeout(_timeout) => {
                // IBC timeout (packet expired)
                operations.push(Operation::Transfer(Transfer {
                    from: create_address("ibc:timeout".to_string()),
                    to: create_address("ibc:timeout".to_string()),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Custom("ibc:timeout".to_string()),
                }));
            }
            CosmosMessage::IbcCreateClient(_create) => {
                // Creating IBC light client
                operations.push(Operation::Transfer(Transfer {
                    from: create_address("ibc:client".to_string()),
                    to: create_address("ibc:client".to_string()),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Custom("ibc:create_client".to_string()),
                }));
            }
            CosmosMessage::IbcUpdateClient(update) => {
                // Updating IBC light client
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(format!("ibc:client:{}", update.client_id)),
                    to: create_address(format!("ibc:client:{}", update.client_id)),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Custom("ibc:update_client".to_string()),
                }));
            }

            // === Governance Messages ===
            CosmosMessage::Vote(vote) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(vote.voter.clone()),
                    to: create_address(format!("gov:proposal:{}", vote.proposal_id)),
                    amount: Amount::new(vote.option as u128, 0),
                    asset: AssetId::Custom("vote".to_string()),
                }));
            }
            CosmosMessage::SubmitProposal(proposal) => {
                for coin in &proposal.initial_deposit {
                    operations.push(Operation::Transfer(Transfer {
                        from: create_address(proposal.proposer.clone()),
                        to: create_address("gov:proposal_deposit".to_string()),
                        amount: parse_amount(&coin.amount, &coin.denom)?,
                        asset: AssetId::Custom(coin.denom.clone()),
                    }));
                }
            }
            CosmosMessage::Deposit(deposit) => {
                for coin in &deposit.amount {
                    operations.push(Operation::Transfer(Transfer {
                        from: create_address(deposit.depositor.clone()),
                        to: create_address(format!("gov:proposal:{}", deposit.proposal_id)),
                        amount: parse_amount(&coin.amount, &coin.denom)?,
                        asset: AssetId::Custom(coin.denom.clone()),
                    }));
                }
            }

            // === Distribution Messages ===
            CosmosMessage::WithdrawDelegatorReward(withdraw) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(withdraw.validator_address.clone()),
                    to: create_address(withdraw.delegator_address.clone()),
                    amount: Amount::new(0, 0), // Amount unknown until execution
                    asset: AssetId::Custom("reward".to_string()),
                }));
            }

            // === CosmWasm Messages ===
            CosmosMessage::StoreCode(store) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(store.sender.clone()),
                    to: create_address("wasm:code_storage".to_string()),
                    amount: Amount::new(store.wasm_byte_code.len() as u128, 0),
                    asset: AssetId::Custom("wasm:bytecode".to_string()),
                }));
            }
            CosmosMessage::InstantiateContract(instantiate) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(instantiate.sender.clone()),
                    to: create_address(format!("wasm:code:{}", instantiate.code_id)),
                    amount: Amount::new(instantiate.code_id as u128, 0),
                    asset: AssetId::Custom(format!("wasm:instantiate:{}", instantiate.label)),
                }));
            }
            CosmosMessage::ExecuteContract(execute) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(execute.sender.clone()),
                    to: create_address(execute.contract.clone()),
                    amount: Amount::new(execute.msg.len() as u128, 0),
                    asset: AssetId::Custom("wasm:execute".to_string()),
                }));
            }
            CosmosMessage::MigrateContract(migrate) => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address(migrate.contract.clone()),
                    to: create_address(format!("wasm:code:{}", migrate.code_id)),
                    amount: Amount::new(migrate.code_id as u128, 0),
                    asset: AssetId::Custom("wasm:migrate".to_string()),
                }));
            }

            // === Unknown Messages ===
            CosmosMessage::Unknown { type_url, .. } => {
                operations.push(Operation::Transfer(Transfer {
                    from: create_address("unknown".to_string()),
                    to: create_address("unknown".to_string()),
                    amount: Amount::new(0, 0),
                    asset: AssetId::Custom(format!("unknown:{}", type_url)),
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
    let inputs = Vec::new();
    let mut outputs = Vec::new();

    for msg in messages {
        match msg {
            // === Bank Messages ===
            CosmosMessage::Send(send) => {
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
            CosmosMessage::MultiSend(multi) => {
                // Track inputs
                for input in &multi.inputs {
                    account_changes.push(AccountChange {
                        address: create_address(input.address.clone()),
                        nonce: None,
                        balance_change: -1,
                        storage_changes: vec![],
                    });
                }
                // Track outputs
                for output in &multi.outputs {
                    account_changes.push(AccountChange {
                        address: create_address(output.address.clone()),
                        nonce: None,
                        balance_change: 1,
                        storage_changes: vec![],
                    });
                }
            }

            // === Staking Messages ===
            CosmosMessage::Delegate(delegate) => {
                account_changes.push(AccountChange {
                    address: create_address(delegate.delegator_address.clone()),
                    nonce: None,
                    balance_change: -1,
                    storage_changes: vec![],
                });
            }
            CosmosMessage::Undelegate(undelegate) => {
                account_changes.push(AccountChange {
                    address: create_address(undelegate.delegator_address.clone()),
                    nonce: None,
                    balance_change: 1,
                    storage_changes: vec![],
                });
            }
            CosmosMessage::BeginRedelegate(_) => {
                // Redelegate doesn't change account balances directly
            }

            // === IBC Messages ===
            CosmosMessage::IbcTransfer(ibc) => {
                // IBC transfer: tokens leave this chain
                account_changes.push(AccountChange {
                    address: create_address(ibc.sender.clone()),
                    nonce: None,
                    balance_change: -1,
                    storage_changes: vec![],
                });

                // Track IBC packet as output (cross-chain state change)
                outputs.push(OutputValue {
                    index: outputs.len() as u32,
                    address: create_address(format!("ibc:{}:{}", ibc.source_channel, ibc.receiver)),
                    value: parse_amount(&ibc.token.amount, &ibc.token.denom)?,
                    script: vec![],
                });
            }
            CosmosMessage::IbcRecvPacket(recv) => {
                // IBC receive: tokens enter this chain
                outputs.push(OutputValue {
                    index: outputs.len() as u32,
                    address: create_address(format!(
                        "ibc:{}/{}",
                        recv.packet.destination_port, recv.packet.destination_channel
                    )),
                    value: Amount::new(recv.packet.sequence as u128, 0),
                    script: recv.packet.data.clone(),
                });
            }
            CosmosMessage::IbcAcknowledgement(_) | CosmosMessage::IbcTimeout(_) => {
                // State change already happened in original transfer
            }
            CosmosMessage::IbcCreateClient(_) | CosmosMessage::IbcUpdateClient(_) => {
                // Client state changes tracked at protocol level
            }

            // === Governance Messages ===
            CosmosMessage::Vote(_) => {
                // Vote doesn't change account balances
            }
            CosmosMessage::SubmitProposal(proposal) => {
                // Proposal submission locks deposit
                account_changes.push(AccountChange {
                    address: create_address(proposal.proposer.clone()),
                    nonce: None,
                    balance_change: -1,
                    storage_changes: vec![],
                });
            }
            CosmosMessage::Deposit(deposit) => {
                // Deposit locks tokens for proposal
                account_changes.push(AccountChange {
                    address: create_address(deposit.depositor.clone()),
                    nonce: None,
                    balance_change: -1,
                    storage_changes: vec![],
                });
            }

            // === Distribution Messages ===
            CosmosMessage::WithdrawDelegatorReward(withdraw) => {
                // Reward withdrawal increases delegator balance
                account_changes.push(AccountChange {
                    address: create_address(withdraw.delegator_address.clone()),
                    nonce: None,
                    balance_change: 1,
                    storage_changes: vec![],
                });
            }

            // === CosmWasm Messages ===
            CosmosMessage::StoreCode(store) => {
                // Code storage creates new state
                outputs.push(OutputValue {
                    index: outputs.len() as u32,
                    address: create_address(format!("wasm:code:{}", outputs.len())),
                    value: Amount::new(store.wasm_byte_code.len() as u128, 0),
                    script: vec![],
                });
            }
            CosmosMessage::InstantiateContract(instantiate) => {
                // Contract instantiation creates new contract state
                account_changes.push(AccountChange {
                    address: create_address(instantiate.sender.clone()),
                    nonce: None,
                    balance_change: -1,
                    storage_changes: vec![],
                });
                outputs.push(OutputValue {
                    index: outputs.len() as u32,
                    address: create_address(format!("wasm:contract:{}", instantiate.label)),
                    value: Amount::new(instantiate.code_id as u128, 0),
                    script: instantiate.msg.clone(),
                });
            }
            CosmosMessage::ExecuteContract(execute) => {
                // Contract execution may change state
                account_changes.push(AccountChange {
                    address: create_address(execute.contract.clone()),
                    nonce: None,
                    balance_change: 0,
                    storage_changes: vec![],
                });
            }
            CosmosMessage::MigrateContract(migrate) => {
                // Contract migration changes code pointer
                account_changes.push(AccountChange {
                    address: create_address(migrate.contract.clone()),
                    nonce: None,
                    balance_change: 0,
                    storage_changes: vec![],
                });
            }

            // === Unknown Messages ===
            CosmosMessage::Unknown { .. } => {
                // Unknown messages don't track state changes
            }
        }
    }

    Ok(StateDeltas {
        inputs,
        outputs,
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
        assert_eq!(amount.value, 1000000);
        assert_eq!(amount.decimals, 6);
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
