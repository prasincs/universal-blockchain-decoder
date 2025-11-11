//! Tests for TxIR construction and versioning
//!
//! Verifies that the transaction intermediate representation (TxIR)
//! can be constructed correctly with different versions and chains.

use universal_decoder_core::chain::{ChainFamily, ChainIdentity};
use universal_decoder_core::prelude::*;

#[derive(Debug)]
struct TestChain {
    id: u64,
    name: String,
}

impl ChainIdentity for TestChain {
    fn chain_id(&self) -> u64 {
        self.id
    }

    fn chain_name(&self) -> &str {
        &self.name
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

fn create_empty_txir<const V: u8>(chain: &impl ChainIdentity) -> TxIR<V> {
    TxIR::new(
        chain,
        TxMetadata {
            tx_hash: vec![0; 32],
            block_height: None,
            timestamp: None,
            size: 0,
            extra: "{}".to_string(),
        },
        AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        },
        vec![],
        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    )
}

#[test]
fn test_txir_version_1() {
    let chain = TestChain {
        id: 0,
        name: "Bitcoin".to_string(),
    };
    let tx = create_empty_txir::<1>(&chain);
    assert_eq!(tx.version(), 1);
}

#[test]
fn test_txir_version_2() {
    let chain = TestChain {
        id: 1,
        name: "Ethereum".to_string(),
    };
    let tx = create_empty_txir::<2>(&chain);
    assert_eq!(tx.version(), 2);
}

#[test]
fn test_txir_with_transfer() {
    let chain = TestChain {
        id: 0,
        name: "Bitcoin".to_string(),
    };

    let transfer = Operation::Transfer(Transfer {
        from: Address {
            bytes: vec![1; 20],
            human_readable: Some("addr1".to_string()),
        },
        to: Address {
            bytes: vec![2; 20],
            human_readable: Some("addr2".to_string()),
        },
        amount: Amount {
            value: 1_000_000,
            decimals: 8,
        },
        asset: AssetId::Native,
    });

    let tx = TxIR::<1>::new(
        &chain,
        TxMetadata {
            tx_hash: vec![0xaa; 32],
            block_height: Some(100),
            timestamp: Some(1234567890),
            size: 250,
            extra: "{}".to_string(),
        },
        AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        },
        vec![transfer],
        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    );

    assert_eq!(tx.operations.len(), 1);
    match &tx.operations[0] {
        Operation::Transfer(t) => {
            assert_eq!(t.amount.value, 1_000_000);
            assert_eq!(t.amount.decimals, 8);
        }
        _ => panic!("Expected Transfer operation"),
    }
}

#[test]
fn test_txir_with_multiple_operations() {
    let chain = TestChain {
        id: 1,
        name: "Ethereum".to_string(),
    };

    let ops = vec![
        Operation::Transfer(Transfer {
            from: Address {
                bytes: vec![1; 20],
                human_readable: None,
            },
            to: Address {
                bytes: vec![2; 20],
                human_readable: None,
            },
            amount: Amount {
                value: 100,
                decimals: 18,
            },
            asset: AssetId::Native,
        }),
        Operation::ContractCall(ContractCall {
            contract: Address {
                bytes: vec![3; 20],
                human_readable: None,
            },
            method: b"transfer".to_vec(),
            data: vec![0xde, 0xad, 0xbe, 0xef],
            value: Some(Amount {
                value: 0,
                decimals: 18,
            }),
            resource_limits: ResourceLimits {
                max_units: 21000,
                unit_price: 20,
                resource_type: ResourceType::Gas,
            },
        }),
    ];

    let tx = TxIR::<1>::new(
        &chain,
        TxMetadata {
            tx_hash: vec![0; 32],
            block_height: None,
            timestamp: None,
            size: 0,
            extra: "{}".to_string(),
        },
        AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        },
        ops,
        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    );

    assert_eq!(tx.operations.len(), 2);
}

#[test]
fn test_txir_canonical_conversion() {
    let chain = TestChain {
        id: 0,
        name: "Test".to_string(),
    };
    let tx = create_empty_txir::<1>(&chain);
    let canonical = tx.to_canonical();

    assert_eq!(canonical.version, 1);
    assert_eq!(canonical.chain.id, 0);
    assert_eq!(canonical.chain.name, "Test");
}

#[test]
fn test_txir_with_signatures() {
    let chain = TestChain {
        id: 0,
        name: "Bitcoin".to_string(),
    };

    let sig = Signature {
        data: vec![0x30, 0x44, 0x02, 0x20], // DER format prefix
        key_index: 0,
        metadata: Some(r#"{"type":"ecdsa"}"#.to_string()),
    };

    let pubkey = PublicKey {
        data: vec![0x02; 33], // Compressed secp256k1 pubkey
        key_type: KeyType::Secp256k1,
    };

    let tx = TxIR::<1>::new(
        &chain,
        TxMetadata {
            tx_hash: vec![0; 32],
            block_height: None,
            timestamp: None,
            size: 0,
            extra: "{}".to_string(),
        },
        AuthorizationPackage {
            signatures: vec![sig],
            public_keys: vec![pubkey],
            signature_scheme: SignatureScheme::Ecdsa,
        },
        vec![],
        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    );

    assert_eq!(tx.authorization.signatures.len(), 1);
    assert_eq!(tx.authorization.public_keys.len(), 1);
    assert_eq!(tx.authorization.signature_scheme, SignatureScheme::Ecdsa);
}
