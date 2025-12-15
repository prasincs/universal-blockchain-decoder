//! Arbitrum Orbit Transaction Types
//!
//! Arbitrum supports standard Ethereum transactions plus 6 Arbitrum-specific types
//! for L1↔L2 communication, retryable tickets, and system operations.

use borsh::{BorshDeserialize, BorshSerialize};
use decoder_ethereum::types::EthereumTransaction;
use serde::{Deserialize, Serialize};
use universal_decoder_core::prelude::*;

/// Arbitrum transaction type (standard EVM or Arbitrum-specific)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ArbitrumTransaction {
    /// Standard Ethereum-compatible transaction (types 0x00, 0x01, 0x02)
    Standard(EthereumTransaction),

    /// L1→L2 deposit transaction (type 0x64 / 100)
    Deposit(DepositTransaction),

    /// Unsigned transaction from L1 via bridge (type 0x65 / 101)
    Unsigned(UnsignedTransaction),

    /// Contract-initiated transaction from L1 (type 0x66 / 102)
    Contract(ContractTransaction),

    /// Retry a failed retryable ticket (type 0x68 / 104)
    Retry(RetryTransaction),

    /// Submit a new retryable ticket for L1→L2 messaging (type 0x69 / 105)
    SubmitRetryable(SubmitRetryableTransaction),

    /// ArbOS internal system transaction (type 0x6A / 106)
    Internal(InternalTransaction),
}

/// L1→L2 Deposit Transaction (Type 0x64 / 100)
///
/// Represents a deposit from L1 to L2, similar to Optimism deposits but with
/// Arbitrum-specific features.
///
/// ## Key Properties
///
/// - **No Signature**: Authorization via L1 transaction derivation
/// - **ETH Transfer**: Moves ETH from L1 to L2
/// - **Guaranteed Execution**: Ensured by Arbitrum protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct DepositTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// L1 block number when deposit was made
    pub l1_block_number: u64,

    /// Sender address on L1
    pub from: [u8; 20],

    /// Recipient address on L2 (or None for contract creation)
    pub to: Option<[u8; 20]>,

    /// Amount of ETH deposited
    pub value: u128,

    /// Gas limit for L2 execution
    pub gas_limit: u64,

    /// Transaction calldata
    pub data: Vec<u8>,
}

impl DepositTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x64;

    /// Validates deposit transaction invariants
    pub fn validate(&self) -> Result<()> {
        // Gas limit must be non-zero
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure(
                "Deposit gas_limit cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Unsigned Transaction (Type 0x65 / 101)
///
/// EOA-initiated transaction from L1 via the delayed inbox, without signature.
/// Address is remapped to distinguish from regular L2 transactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UnsignedTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// Sender address (remapped from L1 EOA)
    pub from: [u8; 20],

    /// Recipient address on L2
    pub to: [u8; 20],

    /// Amount of ETH to transfer
    pub value: u128,

    /// Gas limit
    pub gas_limit: u64,

    /// Gas price (or max fee per gas)
    pub gas_price: u128,

    /// Nonce
    pub nonce: u64,

    /// Transaction calldata
    pub data: Vec<u8>,
}

impl UnsignedTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x65;

    /// Validates unsigned transaction invariants
    pub fn validate(&self) -> Result<()> {
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure(
                "Unsigned tx gas_limit cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Contract Transaction (Type 0x66 / 102)
///
/// Contract-initiated transaction from L1 to L2 via the delayed inbox.
/// Uses sequential nonce instead of account nonce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContractTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// L1 contract address that initiated this transaction
    pub from: [u8; 20],

    /// Recipient address on L2
    pub to: [u8; 20],

    /// Amount of ETH to transfer
    pub value: u128,

    /// Gas limit
    pub gas_limit: u64,

    /// Gas price
    pub gas_price: u128,

    /// Sequential nonce (not account nonce)
    pub nonce: u64,

    /// Transaction calldata
    pub data: Vec<u8>,
}

impl ContractTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x66;

    /// Validates contract transaction invariants
    pub fn validate(&self) -> Result<()> {
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure(
                "Contract tx gas_limit cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Retry Transaction (Type 0x68 / 104)
///
/// Redeem a previously submitted retryable ticket with fresh gas allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RetryTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// Ticket ID being retried (keccak256 of original retryable)
    pub ticket_id: [u8; 32],

    /// Address attempting the retry
    pub from: [u8; 20],

    /// Gas limit for retry attempt
    pub gas_limit: u64,

    /// Gas price for retry
    pub gas_price: u128,

    /// Nonce
    pub nonce: u64,
}

impl RetryTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x68;

    /// Validates retry transaction invariants
    pub fn validate(&self) -> Result<()> {
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure(
                "Retry tx gas_limit cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Submit Retryable Transaction (Type 0x69 / 105)
///
/// Submit a new retryable ticket for guaranteed L2 execution.
/// Most important Arbitrum-specific transaction type for L1→L2 messaging.
///
/// ## Key Properties
///
/// - **Guaranteed Execution**: Can be retried for ~24 hours
/// - **Censorship Resistant**: Force-include via delayed inbox
/// - **Ticket ID**: Computed as keccak256(RLP(all fields))
///
/// ## Specification
///
/// See: <https://docs.arbitrum.io/arbos/l1-to-l2-messaging>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubmitRetryableTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// Unique request ID for this retryable
    pub request_id: [u8; 32],

    /// L1 base fee at submission time
    pub l1_base_fee: u128,

    /// Total ETH deposited for this retryable
    pub deposit: u128,

    /// ETH to send with the L2 call
    pub callvalue: u128,

    /// Maximum gas price (wei per gas)
    pub gas_fee_cap: u128,

    /// Gas limit for execution
    pub gas_limit: u64,

    /// Maximum submission fee paid to sequencer
    pub max_submission_fee: u128,

    /// Address receiving excess submission fees
    pub fee_refund_address: [u8; 20],

    /// Address that can redeem/cancel this ticket
    pub beneficiary: [u8; 20],

    /// Target contract address on L2
    pub retry_to: [u8; 20],

    /// Calldata for the L2 call
    pub retry_data: Vec<u8>,
}

impl SubmitRetryableTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x69;

    /// Validates submit retryable transaction invariants
    pub fn validate(&self) -> Result<()> {
        // Gas limit must be non-zero
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure(
                "SubmitRetryable gas_limit cannot be zero".to_string(),
            ));
        }

        // Callvalue cannot exceed deposit
        if self.callvalue > self.deposit {
            return Err(DecoderError::invalid_structure(format!(
                "SubmitRetryable callvalue ({}) exceeds deposit ({})",
                self.callvalue, self.deposit
            )));
        }

        // Gas fee cap must be non-zero
        if self.gas_fee_cap == 0 {
            return Err(DecoderError::invalid_structure(
                "SubmitRetryable gas_fee_cap cannot be zero".to_string(),
            ));
        }

        Ok(())
    }

    /// Computes the ticket ID for this retryable
    ///
    /// Ticket ID = keccak256(RLP(all fields))
    pub fn compute_ticket_id(&self) -> [u8; 32] {
        // TODO: Implement RLP encoding and keccak256 hash
        // For now, return placeholder (will implement in parsing module)
        self.request_id
    }
}

/// Internal Transaction (Type 0x6A / 106)
///
/// ArbOS-generated system transaction for internal state updates.
/// Always the first transaction in each L2 block.
///
/// ## Key Properties
///
/// - **No Sender/Signature**: Generated by ArbOS itself
/// - **Block Metadata**: Updates L1 block number and base fee
/// - **First Transaction**: Always appears before user transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct InternalTransaction {
    /// Chain ID for replay protection
    pub chain_id: u64,

    /// Type of internal transaction (currently only L1 block update)
    pub internal_type: InternalTxType,

    /// L1 block number being recorded
    pub l1_block_number: u64,

    /// L1 base fee at this block
    pub l1_base_fee: u128,

    /// Timestamp of L1 block
    pub l1_timestamp: u64,
}

/// Type of internal transaction
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum InternalTxType {
    /// Update L1 block metadata (most common)
    UpdateL1BlockNumber,

    /// Future internal transaction types (reserved)
    Unknown(u8),
}

impl InternalTransaction {
    /// Transaction type identifier
    pub const TYPE_ID: u8 = 0x6A;

    /// Validates internal transaction invariants
    pub fn validate(&self) -> Result<()> {
        // L1 block number must be non-zero (genesis is block 0, but we start from 1)
        if self.l1_block_number == 0 {
            return Err(DecoderError::invalid_structure(
                "Internal tx l1_block_number cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

impl ChainEncoder for ArbitrumTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            ArbitrumTransaction::Standard(eth_tx) => {
                // Delegate to Ethereum re-encoding for standard transactions
                eth_tx.to_bytes()
            }
            ArbitrumTransaction::Deposit(_)
            | ArbitrumTransaction::Unsigned(_)
            | ArbitrumTransaction::Contract(_)
            | ArbitrumTransaction::Retry(_)
            | ArbitrumTransaction::SubmitRetryable(_)
            | ArbitrumTransaction::Internal(_) => {
                // TODO: Custom Arbitrum transaction types need raw_bytes field added for re-encoding
                // For now, return error indicating this is not yet supported
                Err(DecoderError::invalid_structure(
                    "Re-encoding custom Arbitrum transaction types not yet supported. \
                     Custom types (Deposit, Unsigned, Contract, Retry, SubmitRetryable, Internal) \
                     need to store raw_bytes for exact reconstruction.",
                ))
            }
        }
    }
}

impl<'a> universal_decoder_core::traits::Canonicalizer<'a> for ArbitrumTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        match self {
            ArbitrumTransaction::Standard(eth_tx) => {
                // Delegate to Ethereum canonicalization for standard transactions
                eth_tx.canonicalize()
            }
            ArbitrumTransaction::Deposit(deposit) => canonicalize_deposit(deposit),
            ArbitrumTransaction::Unsigned(unsigned) => canonicalize_unsigned(unsigned),
            ArbitrumTransaction::Contract(contract) => canonicalize_contract(contract),
            ArbitrumTransaction::Retry(retry) => canonicalize_retry(retry),
            ArbitrumTransaction::SubmitRetryable(retryable) => {
                canonicalize_submit_retryable(retryable)
            }
            ArbitrumTransaction::Internal(internal) => canonicalize_internal(internal),
        }
    }
}

/// Canonicalize a deposit transaction
fn canonicalize_deposit<'a>(deposit: &'a DepositTransaction) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(
        r#"{{"tx_type":"deposit","l1_block":{},"value":{}}}"#,
        deposit.l1_block_number, deposit.value
    );

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x64];
            bytes.extend_from_slice(&borsh::to_vec(deposit).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: Some(deposit.l1_block_number),
        timestamp: None,
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let mut operations = Vec::new();
    if deposit.value > 0 {
        operations.push(Operation::Transfer(Transfer {
            from: Address {
                bytes: deposit.from.to_vec(),
                human_readable: Some(format!(
                    "0x{}",
                    universal_decoder_core::hex::encode(deposit.from)
                )),
            },
            to: Address {
                bytes: deposit.to.map(|a| a.to_vec()).unwrap_or_default(),
                human_readable: deposit
                    .to
                    .map(|a| format!("0x{}", universal_decoder_core::hex::encode(a))),
            },
            amount: Amount {
                value: deposit.value,
                decimals: 18,
            },
            asset: AssetId::Native,
        }));
    }

    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain =
        crate::ArbitrumChain::from_chain_id(deposit.chain_id).unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

/// Canonicalize an unsigned transaction
fn canonicalize_unsigned<'a>(unsigned: &'a UnsignedTransaction) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(r#"{{"tx_type":"unsigned","nonce":{}}}"#, unsigned.nonce);

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x65];
            bytes.extend_from_slice(&borsh::to_vec(unsigned).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: None,
        timestamp: None,
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let operations = vec![Operation::Transfer(Transfer {
        from: Address {
            bytes: unsigned.from.to_vec(),
            human_readable: Some(format!(
                "0x{}",
                universal_decoder_core::hex::encode(unsigned.from)
            )),
        },
        to: Address {
            bytes: unsigned.to.to_vec(),
            human_readable: Some(format!(
                "0x{}",
                universal_decoder_core::hex::encode(unsigned.to)
            )),
        },
        amount: Amount {
            value: unsigned.value,
            decimals: 18,
        },
        asset: AssetId::Native,
    })];

    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain =
        crate::ArbitrumChain::from_chain_id(unsigned.chain_id).unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

/// Canonicalize a contract transaction
fn canonicalize_contract<'a>(contract: &'a ContractTransaction) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(r#"{{"tx_type":"contract","nonce":{}}}"#, contract.nonce);

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x66];
            bytes.extend_from_slice(&borsh::to_vec(contract).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: None,
        timestamp: None,
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let operations = vec![Operation::ContractCall(ContractCall {
        contract: Address {
            bytes: contract.to.to_vec(),
            human_readable: Some(format!(
                "0x{}",
                universal_decoder_core::hex::encode(contract.to)
            )),
        },
        method: if contract.data.len() >= 4 {
            contract.data[0..4].to_vec()
        } else {
            vec![]
        },
        data: contract.data.clone(),
        value: Some(Amount {
            value: contract.value,
            decimals: 18,
        }),
        resource_limits: ResourceLimits {
            max_units: contract.gas_limit,
            unit_price: contract.gas_price.min(u64::MAX as u128) as u64,
            resource_type: ResourceType::Gas,
        },
    })];

    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain =
        crate::ArbitrumChain::from_chain_id(contract.chain_id).unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

/// Canonicalize a retry transaction
fn canonicalize_retry<'a>(retry: &'a RetryTransaction) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(
        r#"{{"tx_type":"retry","ticket_id":"{}","nonce":{}}}"#,
        universal_decoder_core::hex::encode(retry.ticket_id),
        retry.nonce
    );

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x68];
            bytes.extend_from_slice(&borsh::to_vec(retry).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: None,
        timestamp: None,
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let operations = vec![]; // Retry operations don't have direct state changes
    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain =
        crate::ArbitrumChain::from_chain_id(retry.chain_id).unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

/// Canonicalize a submit retryable transaction
fn canonicalize_submit_retryable<'a>(
    retryable: &'a SubmitRetryableTransaction,
) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(
        r#"{{"tx_type":"submit_retryable","request_id":"{}","deposit":{},"callvalue":{}}}"#,
        universal_decoder_core::hex::encode(retryable.request_id),
        retryable.deposit,
        retryable.callvalue
    );

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x69];
            bytes.extend_from_slice(&borsh::to_vec(retryable).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: None,
        timestamp: None,
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let operations = vec![Operation::ContractCall(ContractCall {
        contract: Address {
            bytes: retryable.retry_to.to_vec(),
            human_readable: Some(format!(
                "0x{}",
                universal_decoder_core::hex::encode(retryable.retry_to)
            )),
        },
        method: if retryable.retry_data.len() >= 4 {
            retryable.retry_data[0..4].to_vec()
        } else {
            vec![]
        },
        data: retryable.retry_data.clone(),
        value: Some(Amount {
            value: retryable.callvalue,
            decimals: 18,
        }),
        resource_limits: ResourceLimits {
            max_units: retryable.gas_limit,
            unit_price: retryable.gas_fee_cap.min(u64::MAX as u128) as u64,
            resource_type: ResourceType::Gas,
        },
    })];

    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain = crate::ArbitrumChain::from_chain_id(retryable.chain_id)
        .unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

/// Canonicalize an internal transaction
fn canonicalize_internal<'a>(internal: &'a InternalTransaction) -> Result<TxIR<'a, 1>> {
    use sha3::{Digest, Keccak256};
    use universal_decoder_core::prelude::*;

    let extra = format!(
        r#"{{"tx_type":"internal","l1_block":{},"l1_base_fee":{},"l1_timestamp":{}}}"#,
        internal.l1_block_number, internal.l1_base_fee, internal.l1_timestamp
    );

    let metadata = TxMetadata {
        tx_hash: {
            let mut bytes = vec![0x6A];
            bytes.extend_from_slice(&borsh::to_vec(internal).unwrap());
            Keccak256::digest(&bytes).to_vec()
        },
        block_height: Some(internal.l1_block_number),
        timestamp: Some(internal.l1_timestamp),
        size: 0,
        extra,
    };

    let authorization = AuthorizationPackage {
        signatures: vec![],
        public_keys: vec![],
        signature_scheme: SignatureScheme::Custom(0),
    };

    let operations = vec![]; // Internal transactions don't have user operations
    let state_deltas = StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![],
    };
    let chain =
        crate::ArbitrumChain::from_chain_id(internal.chain_id).unwrap_or(crate::ArbitrumChain::ONE);

    Ok(TxIR::new(
        &chain,
        metadata,
        authorization,
        operations,
        state_deltas,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_validate() {
        let deposit = DepositTransaction {
            chain_id: 42161,
            l1_block_number: 1000,
            from: [1u8; 20],
            to: Some([2u8; 20]),
            value: 1_000_000_000,
            gas_limit: 100_000,
            data: vec![],
        };
        assert!(deposit.validate().is_ok());

        // Zero gas limit should fail
        let invalid = DepositTransaction {
            gas_limit: 0,
            ..deposit
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_submit_retryable_validate() {
        let retryable = SubmitRetryableTransaction {
            chain_id: 42161,
            request_id: [1u8; 32],
            l1_base_fee: 1_000_000_000,
            deposit: 10_000_000_000,
            callvalue: 5_000_000_000,
            gas_fee_cap: 10_000_000_000,
            gas_limit: 1_000_000,
            max_submission_fee: 100_000_000,
            fee_refund_address: [3u8; 20],
            beneficiary: [4u8; 20],
            retry_to: [5u8; 20],
            retry_data: vec![1, 2, 3],
        };
        assert!(retryable.validate().is_ok());

        // Callvalue > deposit should fail
        let invalid = SubmitRetryableTransaction {
            callvalue: 15_000_000_000,
            ..retryable.clone()
        };
        assert!(invalid.validate().is_err());

        // Zero gas limit should fail
        let invalid = SubmitRetryableTransaction {
            gas_limit: 0,
            ..retryable
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_internal_validate() {
        let internal = InternalTransaction {
            chain_id: 42161,
            internal_type: InternalTxType::UpdateL1BlockNumber,
            l1_block_number: 1000,
            l1_base_fee: 1_000_000_000,
            l1_timestamp: 1234567890,
        };
        assert!(internal.validate().is_ok());

        // Zero L1 block number should fail
        let invalid = InternalTransaction {
            l1_block_number: 0,
            ..internal
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_transaction_type_ids() {
        assert_eq!(DepositTransaction::TYPE_ID, 0x64);
        assert_eq!(UnsignedTransaction::TYPE_ID, 0x65);
        assert_eq!(ContractTransaction::TYPE_ID, 0x66);
        assert_eq!(RetryTransaction::TYPE_ID, 0x68);
        assert_eq!(SubmitRetryableTransaction::TYPE_ID, 0x69);
        assert_eq!(InternalTransaction::TYPE_ID, 0x6A);
    }
}
