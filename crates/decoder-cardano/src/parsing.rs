//! CBOR parsing functions for Cardano transactions using minicbor
//!
//! This module provides Cardano transaction parsing using the battle-tested
//! minicbor library for all CBOR operations.

use crate::types::*;
use decoder_primitives::prelude::*;
use minicbor::Decoder;

/// Parse transaction body from CBOR
pub fn parse_transaction_body(decoder: &mut Decoder) -> Result<TransactionBody> {
    // Parse CBOR map
    let map_len = decoder
        .map()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse tx body map: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length map for transaction body")
        })?;

    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut fee = 0u64;
    let mut ttl = None;
    let mut certificates = vec![];
    let mut withdrawals = vec![];
    let mut script_data_hash = None;
    let mut required_signers = vec![];
    let mut network_id = None;
    let mut collateral = vec![];
    let mut mint = vec![];

    for _ in 0..map_len {
        let key = decoder.u64().map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse map key: {}", e))
        })?;

        match key {
            0 => inputs = parse_transaction_inputs(decoder)?,
            1 => outputs = parse_transaction_outputs(decoder)?,
            2 => {
                fee = decoder.u64().map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to parse fee: {}", e))
                })?
            }
            3 => {
                ttl = Some(decoder.u64().map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to parse TTL: {}", e))
                })?)
            }
            4 => certificates = parse_certificates(decoder)?,
            5 => withdrawals = parse_withdrawals(decoder)?,
            11 => {
                script_data_hash = Some(
                    decoder
                        .bytes()
                        .map_err(|e| {
                            DecoderError::invalid_structure(format!(
                                "Failed to parse script data hash: {}",
                                e
                            ))
                        })?
                        .to_vec(),
                )
            }
            14 => required_signers = parse_required_signers(decoder)?,
            15 => {
                network_id = Some(decoder.u8().map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to parse network ID: {}", e))
                })?)
            }
            13 => collateral = parse_transaction_inputs(decoder)?,
            9 => mint = parse_multi_assets(decoder)?,
            _ => {
                // Skip unknown fields
                decoder.skip().map_err(|e| {
                    DecoderError::invalid_structure(format!(
                        "Failed to skip unknown field {}: {}",
                        key, e
                    ))
                })?;
            }
        }
    }

    Ok(TransactionBody {
        inputs,
        outputs,
        fee,
        ttl,
        certificates,
        withdrawals,
        script_data_hash,
        required_signers,
        network_id,
        collateral,
        mint,
    })
}

/// Parse transaction inputs
fn parse_transaction_inputs(decoder: &mut Decoder) -> Result<Vec<TransactionInput>> {
    let array_len = decoder
        .array()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse inputs array: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length array for inputs")
        })?;

    let mut inputs = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        let input_len = decoder
            .array()
            .map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to parse input array: {}", e))
            })?
            .ok_or_else(|| {
                DecoderError::invalid_structure("Expected definite-length array for input")
            })?;

        if input_len != 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected input array with 2 elements, got {}",
                input_len
            )));
        }

        let transaction_id = decoder
            .bytes()
            .map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to parse transaction ID: {}", e))
            })?
            .to_vec();

        let index = decoder.u64().map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse input index: {}", e))
        })?;

        inputs.push(TransactionInput {
            transaction_id,
            index,
        });
    }

    Ok(inputs)
}

/// Parse transaction outputs
fn parse_transaction_outputs(decoder: &mut Decoder) -> Result<Vec<TransactionOutput>> {
    let array_len = decoder
        .array()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse outputs array: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length array for outputs")
        })?;

    let mut outputs = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        let output_len = decoder
            .array()
            .map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to parse output array: {}", e))
            })?
            .ok_or_else(|| {
                DecoderError::invalid_structure("Expected definite-length array for output")
            })?;

        if output_len < 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected output array with at least 2 elements, got {}",
                output_len
            )));
        }

        let address = decoder
            .bytes()
            .map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to parse output address: {}", e))
            })?
            .to_vec();

        let amount = decoder.u64().map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse output amount: {}", e))
        })?;

        // Skip additional fields (datum hash, etc.) if present
        for _ in 2..output_len {
            decoder.skip().map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to skip output field: {}", e))
            })?;
        }

        outputs.push(TransactionOutput {
            address,
            amount,
            assets: vec![],
            datum_hash: None,
            inline_datum: None,
        });
    }

    Ok(outputs)
}

/// Parse certificates (simplified - just skip for now)
fn parse_certificates(decoder: &mut Decoder) -> Result<Vec<Certificate>> {
    decoder.skip().map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to skip certificates: {}", e))
    })?;
    Ok(vec![])
}

/// Parse withdrawals (simplified - just skip for now)
fn parse_withdrawals(decoder: &mut Decoder) -> Result<Vec<Withdrawal>> {
    decoder.skip().map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to skip withdrawals: {}", e))
    })?;
    Ok(vec![])
}

/// Parse required signers
fn parse_required_signers(decoder: &mut Decoder) -> Result<Vec<Vec<u8>>> {
    let array_len = decoder
        .array()
        .map_err(|e| {
            DecoderError::invalid_structure(format!(
                "Failed to parse required signers array: {}",
                e
            ))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length array for required signers")
        })?;

    let mut signers = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        signers.push(
            decoder
                .bytes()
                .map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to parse signer: {}", e))
                })?
                .to_vec(),
        );
    }

    Ok(signers)
}

/// Parse multi-assets (simplified - just skip for now)
fn parse_multi_assets(decoder: &mut Decoder) -> Result<Vec<MultiAsset>> {
    decoder.skip().map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to skip multi-assets: {}", e))
    })?;
    Ok(vec![])
}

/// Parse witness set
pub fn parse_witness_set(decoder: &mut Decoder) -> Result<WitnessSet> {
    let map_len = decoder
        .map()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse witness set map: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length map for witness set")
        })?;

    let mut vkey_witnesses = vec![];
    let mut native_scripts = vec![];
    let mut plutus_v1_scripts = vec![];
    let mut plutus_v2_scripts = vec![];
    let mut redeemers = vec![];
    let mut plutus_data = vec![];

    for _ in 0..map_len {
        let key = decoder.u64().map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse witness set key: {}", e))
        })?;

        match key {
            0 => vkey_witnesses = parse_vkey_witnesses(decoder)?,
            1 => native_scripts = parse_scripts(decoder)?,
            3 => plutus_v1_scripts = parse_scripts(decoder)?,
            6 => plutus_v2_scripts = parse_scripts(decoder)?,
            5 => redeemers = parse_redeemers(decoder)?,
            4 => plutus_data = parse_plutus_data(decoder)?,
            _ => {
                decoder.skip().map_err(|e| {
                    DecoderError::invalid_structure(format!(
                        "Failed to skip witness field {}: {}",
                        key, e
                    ))
                })?;
            }
        }
    }

    Ok(WitnessSet {
        vkey_witnesses,
        native_scripts,
        plutus_v1_scripts,
        plutus_v2_scripts,
        redeemers,
        plutus_data,
    })
}

/// Parse verification key witnesses
fn parse_vkey_witnesses(decoder: &mut Decoder) -> Result<Vec<VKeyWitness>> {
    let array_len = decoder
        .array()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse vkey witnesses array: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length array for vkey witnesses")
        })?;

    let mut witnesses = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        let witness_len = decoder
            .array()
            .map_err(|e| {
                DecoderError::invalid_structure(format!(
                    "Failed to parse vkey witness array: {}",
                    e
                ))
            })?
            .ok_or_else(|| {
                DecoderError::invalid_structure("Expected definite-length array for vkey witness")
            })?;

        if witness_len != 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected witness array with 2 elements, got {}",
                witness_len
            )));
        }

        let vkey = decoder
            .bytes()
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to parse vkey: {}", e)))?
            .to_vec();

        let signature = decoder
            .bytes()
            .map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to parse signature: {}", e))
            })?
            .to_vec();

        witnesses.push(VKeyWitness { vkey, signature });
    }

    Ok(witnesses)
}

/// Parse scripts (generic)
fn parse_scripts(decoder: &mut Decoder) -> Result<Vec<Vec<u8>>> {
    let array_len = decoder
        .array()
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse scripts array: {}", e))
        })?
        .ok_or_else(|| {
            DecoderError::invalid_structure("Expected definite-length array for scripts")
        })?;

    let mut scripts = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        scripts.push(
            decoder
                .bytes()
                .map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to parse script: {}", e))
                })?
                .to_vec(),
        );
    }

    Ok(scripts)
}

/// Parse redeemers (simplified - just skip for now)
fn parse_redeemers(decoder: &mut Decoder) -> Result<Vec<Redeemer>> {
    decoder
        .skip()
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to skip redeemers: {}", e)))?;
    Ok(vec![])
}

/// Parse Plutus data (simplified - just skip for now)
fn parse_plutus_data(decoder: &mut Decoder) -> Result<Vec<Vec<u8>>> {
    decoder.skip().map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to skip plutus data: {}", e))
    })?;
    Ok(vec![])
}

/// Parse auxiliary data (metadata)
pub fn parse_auxiliary_data(decoder: &mut Decoder) -> Result<Option<AuxiliaryData>> {
    decoder.skip().map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to skip auxiliary data: {}", e))
    })?;
    Ok(None)
}
