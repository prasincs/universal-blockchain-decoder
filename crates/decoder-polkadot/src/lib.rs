//! Polkadot transaction decoder
//!
//! Decodes SCALE-encoded extrinsics from Polkadot and all Substrate-based chains.
//! Supports both signed and unsigned extrinsics.

use blake2::{Blake2b512, Digest};
use decoder_primitives::prelude::*;

pub mod parsing;
pub mod registry;
pub mod types;

pub use parsing::*;
pub use registry::*;
pub use types::*;

/// Polkadot chain identity
#[derive(Debug, Clone, Copy)]
pub struct PolkadotChain;

impl ChainIdentity for PolkadotChain {
    fn chain_id(&self) -> u64 {
        0 // Polkadot relay chain
    }

    fn chain_name(&self) -> &str {
        "Polkadot"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Parsed Polkadot transaction
#[derive(Debug, Clone)]
pub struct PolkadotTransaction {
    pub extrinsic: Extrinsic,
    pub raw_bytes: Vec<u8>,
    pub tx_hash: Vec<u8>,
}

impl PolkadotTransaction {
    /// Calculate Blake2b-512 hash (standard for Polkadot)
    pub fn calculate_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Blake2b512::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Get the call data
    pub fn call_data(&self) -> &[u8] {
        self.extrinsic.call_data()
    }

    /// Parse the call
    pub fn call(&self) -> Result<Call> {
        parsing::parse_call(self.call_data())
    }
}

pub struct PolkadotDecoder;

impl ChainDecoder for PolkadotDecoder {
    type TxSpecific = PolkadotTransaction;
    type Chain = PolkadotChain;

    fn chain() -> Self::Chain {
        PolkadotChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        let extrinsic = parsing::parse_extrinsic(raw_bytes)?;
        let tx_hash = PolkadotTransaction::calculate_hash(raw_bytes);

        Ok(PolkadotTransaction {
            extrinsic,
            raw_bytes: raw_bytes.to_vec(),
            tx_hash,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Polkadot transaction cannot be empty",
            ));
        }

        // Minimum: length (1) + version (1) + call (2) = 4 bytes
        if raw_bytes.len() < 4 {
            return Err(DecoderError::invalid_structure(
                "Polkadot transaction too short",
            ));
        }

        Ok(())
    }
}

impl ChainEncoder for PolkadotTransaction {
    /// Re-encode the transaction back to its original byte format
    ///
    /// Since we store the original raw bytes during decoding, this simply
    /// returns a clone of those bytes, guaranteeing exact reconstruction.
    ///
    /// # Formal Properties
    ///
    /// This implementation trivially satisfies the injective property:
    /// ```text
    /// ∀ tx_bytes: PolkadotDecoder::decode(tx_bytes)?.to_bytes()? == tx_bytes
    /// ```
    ///
    /// Because we store `raw_bytes` during decode, the roundtrip is guaranteed.
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for PolkadotTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let call = self.call()?;
        let operations = build_operations(&self.extrinsic, &call)?;
        let state_deltas = build_state_deltas(&self.extrinsic, &call)?;

        let metadata = TxMetadata {
            tx_hash: self.tx_hash.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!("pallet: {}, call: {}", call.pallet_name(), call.call_name()),
        };

        let authorization = build_authorization(&self.extrinsic)?;

        Ok(TxIR::new(
            &PolkadotChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Validate that call data is parseable
        let _call = self.call()?;

        // For signed extrinsics, validate signature presence
        if let Extrinsic::Signed(ref signed) = self.extrinsic {
            if matches!(signed.signature, PolkadotSignature::Sr25519(ref s) if s.is_empty()) {
                return Err(DecoderError::invalid_structure("Empty signature"));
            }
        }

        Ok(())
    }
}

/// Build authorization package from extrinsic
fn build_authorization(extrinsic: &Extrinsic) -> Result<AuthorizationPackage> {
    match extrinsic {
        Extrinsic::Signed(signed) => {
            let (signature_data, key_type, scheme) = match &signed.signature {
                PolkadotSignature::Sr25519(s) => {
                    (s.clone(), KeyType::Ed25519, SignatureScheme::Schnorr)
                }
                PolkadotSignature::Ed25519(s) => {
                    (s.clone(), KeyType::Ed25519, SignatureScheme::EdDsa)
                }
                PolkadotSignature::Ecdsa(s) => {
                    (s.clone(), KeyType::Secp256k1, SignatureScheme::Ecdsa)
                }
            };

            let public_key_data = match &signed.from {
                PolkadotAddress::Id(id) => id.clone(),
                PolkadotAddress::Address32(addr) => addr.clone(),
                PolkadotAddress::Index(idx) => idx.to_le_bytes().to_vec(),
                PolkadotAddress::Raw(raw) => raw.clone(),
                PolkadotAddress::Address20(addr) => addr.clone(),
            };

            let public_key = PublicKey {
                data: public_key_data,
                key_type,
            };

            let signature = Signature {
                data: signature_data,
                key_index: 0, // First (and only) key
                metadata: Some(format!(
                    "era: {:?}, nonce: {}, tip: {}",
                    signed.extension.era, signed.extension.nonce, signed.extension.tip
                )),
            };

            Ok(AuthorizationPackage {
                signatures: vec![signature],
                public_keys: vec![public_key],
                signature_scheme: scheme,
            })
        }
        Extrinsic::Unsigned(_) => Ok(AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Custom(0), // No signature
        }),
    }
}

/// Build operations from extrinsic call
fn build_operations(extrinsic: &Extrinsic, call: &Call) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    // For Balances pallet transfers
    if call.pallet_index == 4 && call.call_index == 0 {
        // Balances::transfer
        if let Some(transfer_op) = parse_balances_transfer(extrinsic, call)? {
            operations.push(Operation::Transfer(transfer_op));
        }
    } else {
        // Generic contract call for other operations
        let contract_call = Operation::ContractCall(ContractCall {
            contract: create_pallet_address(call.pallet_index),
            method: format!("{}::{}", call.pallet_name(), call.call_name())
                .as_bytes()
                .to_vec(),
            data: call.parameters.clone(),
            value: None, // No value transfer for generic calls
            resource_limits: ResourceLimits {
                max_units: 0,
                unit_price: 0,
                resource_type: ResourceType::Gas,
            },
        });
        operations.push(contract_call);
    }

    Ok(operations)
}

/// Parse Balances::transfer call parameters
fn parse_balances_transfer(extrinsic: &Extrinsic, call: &Call) -> Result<Option<Transfer>> {
    if call.parameters.len() < 33 {
        return Ok(None);
    }

    let mut offset = 0;

    // Parse destination address
    let dest_address_type = parsing::read_u8(&call.parameters, &mut offset)?;
    let dest = if dest_address_type == 0x00 {
        // Id: 32-byte account
        parsing::read_bytes(&call.parameters, &mut offset, 32)?
    } else {
        // Other address types - use raw bytes
        call.parameters[..32.min(call.parameters.len())].to_vec()
    };

    // Parse amount (compact-encoded)
    let amount = parsing::read_compact_u128(&call.parameters, &mut offset)?;

    // Get sender address
    let from = match extrinsic {
        Extrinsic::Signed(signed) => match &signed.from {
            PolkadotAddress::Id(id) => id.clone(),
            PolkadotAddress::Address32(addr) => addr.clone(),
            PolkadotAddress::Index(idx) => idx.to_le_bytes().to_vec(),
            PolkadotAddress::Raw(raw) => raw.clone(),
            PolkadotAddress::Address20(addr) => addr.clone(),
        },
        Extrinsic::Unsigned(_) => vec![0; 32], // System address
    };

    Ok(Some(Transfer {
        from: create_account_address(&from),
        to: create_account_address(&dest),
        amount: Amount::new(amount, 10), // DOT has 10 decimals
        asset: AssetId::Native,
    }))
}

/// Build state deltas from extrinsic
fn build_state_deltas(extrinsic: &Extrinsic, call: &Call) -> Result<StateDeltas> {
    let mut account_changes = Vec::new();

    // For signed transactions, record nonce change
    if let Extrinsic::Signed(signed) = extrinsic {
        if let PolkadotAddress::Id(id) = &signed.from {
            account_changes.push(AccountChange {
                address: create_account_address(id),
                nonce: Some(signed.extension.nonce),
                balance_change: 0,
                storage_changes: vec![],
            });
        }
    }

    // For balance transfers, record balance changes
    if call.pallet_index == 4 && call.call_index == 0 {
        if let Ok(Some(transfer)) = parse_balances_transfer(extrinsic, call) {
            // Deduct from sender
            account_changes.push(AccountChange {
                address: transfer.from.clone(),
                nonce: None,
                balance_change: -(transfer.amount.value as i128),
                storage_changes: vec![],
            });

            // Add to recipient
            account_changes.push(AccountChange {
                address: transfer.to.clone(),
                nonce: None,
                balance_change: transfer.amount.value as i128,
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

/// Create an account address from bytes
fn create_account_address(bytes: &[u8]) -> Address {
    Address {
        bytes: bytes.to_vec(),
        human_readable: None,
    }
}

/// Create a pallet (contract) address from pallet index
fn create_pallet_address(pallet_index: u8) -> Address {
    Address {
        bytes: vec![pallet_index],
        human_readable: Some(format!("Pallet #{}", pallet_index)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = PolkadotDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Polkadot");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_calculate_hash() {
        let data = b"test";
        let hash = PolkadotTransaction::calculate_hash(data);
        assert_eq!(hash.len(), 64); // Blake2b-512 produces 64 bytes
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction
        assert!(PolkadotDecoder::validate_format(&[]).is_err());

        // Too short
        assert!(PolkadotDecoder::validate_format(&[0x01]).is_err());

        // Minimum valid length (compact length + version + pallet + call)
        assert!(PolkadotDecoder::validate_format(&[0x04, 0x84, 0x00, 0x00]).is_ok());
    }

    #[test]
    fn test_extrinsic_version() {
        let version = ExtrinsicVersion::from_byte(0x84);
        assert_eq!(version.version, 4);
        assert!(version.is_signed);

        let version = ExtrinsicVersion::from_byte(0x04);
        assert_eq!(version.version, 4);
        assert!(!version.is_signed);
    }

    #[test]
    fn test_call_pallet_names() {
        let call = Call {
            pallet_index: 4,
            call_index: 0,
            parameters: vec![],
        };
        assert_eq!(call.pallet_name(), "Balances");
        assert_eq!(call.call_name(), "transfer");
    }
}
