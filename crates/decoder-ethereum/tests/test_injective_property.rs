use decoder_ethereum::types::EthereumTransaction;
use universal_decoder_core::prelude::*;

#[test]
fn test_injective_property_arbitrum_tx1() {
    // Your Arbitrum transaction
    let original_hex = "f8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a2783014985a05837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012aa05e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab";

    let original_bytes = universal_decoder_core::hex::decode(original_hex).unwrap();

    // Step 1: Decode
    let tx = EthereumTransaction::from_raw_bytes(&original_bytes).unwrap();

    // Step 2: Re-encode
    let roundtrip_bytes = tx.to_bytes().unwrap();

    // Step 3: Verify injective property
    assert_eq!(
        original_bytes, roundtrip_bytes,
        "Injective property violated: encode(decode(x)) != x"
    );

    println!("✅ Injective property holds!");
    println!("Original:  {}", original_hex);
    println!(
        "Roundtrip: {}",
        universal_decoder_core::hex::encode(&roundtrip_bytes)
    );
}

#[test]
fn test_injective_property_arbitrum_tx2() {
    // Your EIP-1559 transaction
    let original_hex = "02f9013082a4b182192d808398968083092e0294802b65b5d9016621e66003aed0b16615093f328b80b8c5a00597a00000000000000000000000000000000000000000000000000000000001c412c10000000000000000000000000000000000058ff0955d44f32ab0099e950abfbf000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000000000100000000000000000000000005477c22a5349cee601500da0489dad137fd6bfa00000000000000000000000000000000000000000000000000000000691ce4c20cc001a0dcc1b67fd15f72e5ce782ca5c88c3e401079220648bd548a9aa7cdb14023b5e9a05c121e1bf217a0c4d1f5dc43d8f28358815709e3e1796ee0b35ba64dea0499c1";

    let original_bytes = universal_decoder_core::hex::decode(original_hex).unwrap();

    let tx = EthereumTransaction::from_raw_bytes(&original_bytes).unwrap();
    let roundtrip_bytes = tx.to_bytes().unwrap();

    assert_eq!(
        original_bytes, roundtrip_bytes,
        "Injective property violated: encode(decode(x)) != x"
    );

    println!("✅ Injective property holds!");
    println!("Original:  {}", original_hex);
    println!(
        "Roundtrip: {}",
        universal_decoder_core::hex::encode(&roundtrip_bytes)
    );
}

#[test]
fn test_hardcoded_values_not_in_encoding_path() {
    let tx_hex = "f8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a2783014985a05837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012aa05e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab";

    let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    // The "hardcoded" values are only in TxIR, not in chain-specific encoding
    let tx_ir = tx.canonicalize().unwrap();

    // These values appear in TxIR (canonical representation)
    match &tx_ir.operations[0] {
        Operation::ContractCall(call) => {
            // decimals: 18 is here (TxIR)
            assert_eq!(call.value.as_ref().unwrap().decimals, 18);

            // ResourceType::Gas is here (TxIR)
            assert_eq!(call.resource_limits.resource_type, ResourceType::Gas);
        }
        _ => panic!("Expected ContractCall"),
    }

    assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::Ecdsa);

    // But the chain-specific encoding path doesn't use these
    let re_encoded = tx.to_bytes().unwrap();

    // The re-encoded bytes are identical to original (injective)
    assert_eq!(tx_bytes, re_encoded);

    println!("✅ Hardcoded values are only in TxIR, not in chain-specific encoding");
    println!("✅ Chain-specific encoding is still injective");
}
