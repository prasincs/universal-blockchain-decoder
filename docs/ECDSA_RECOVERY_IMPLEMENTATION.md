# ECDSA Recovery Implementation Guide

## Question: Why shouldn't we run ECDSA verify to recover address if it has signature?

**Answer**: We **SHOULD**! It's currently missing, not intentionally omitted.

## Current State

```rust
// crates/decoder-ethereum/src/types.rs:348-352
pub fn get_from(&self) -> [u8; 20] {
    // TODO: Implement ECDSA recovery from (v, r, s) signature
    // For now, return zero address as placeholder
    [0u8; 20]
}
```

## Why It's Not Implemented Yet

1. **Dependencies**: Requires `k256` or `secp256k1` crate (~500KB binary size)
2. **Phase prioritization**: Currently in Phase 1.5 (testing infrastructure)
3. **Complexity**: Need to reconstruct the signing message first

**But you're right - we should add it!** All the signature data is there (v, r, s).

## Implementation Plan

### Option 1: Using `k256` crate (Recommended)

**Pros**:
- Pure Rust, maintained by RustCrypto
- Already used in Ethereum ecosystem
- Good performance

**Cons**:
- Adds ~500KB to binary

```toml
# crates/decoder-ethereum/Cargo.toml
[dependencies]
k256 = { version = "0.13", features = ["ecdsa", "std"] }
```

### Option 2: Using `secp256k1` crate

**Pros**:
- C library (libsecp256k1) - battle-tested
- Used by Bitcoin Core
- Very fast

**Cons**:
- Requires C compiler
- Not pure Rust

```toml
[dependencies]
secp256k1 = { version = "0.29", features = ["recovery"] }
```

## Implementation

### Step 1: Add dependency

```toml
# crates/decoder-ethereum/Cargo.toml
[dependencies]
k256 = { version = "0.13", features = ["ecdsa", "std"] }
```

### Step 2: Implement recovery

```rust
// crates/decoder-ethereum/src/types.rs

use k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

impl EthereumTransaction {
    /// Recover the sender address from the signature
    ///
    /// This performs ECDSA public key recovery using the (v, r, s) signature
    /// components and the transaction hash.
    ///
    /// # Returns
    ///
    /// The 20-byte Ethereum address of the sender, or an error if recovery fails.
    ///
    /// # Algorithm
    ///
    /// 1. Compute the signing hash (depends on transaction type)
    /// 2. Extract recovery ID from v
    /// 3. Reconstruct public key from (r, s, recovery_id, hash)
    /// 4. Derive address from public key (keccak256(pubkey)[12..32])
    pub fn recover_sender(&self) -> Result<[u8; 20]> {
        // Step 1: Compute signing hash
        let signing_hash = self.signing_hash()?;

        // Step 2: Extract recovery ID from v
        let recovery_id = self.get_recovery_id()?;

        // Step 3: Construct signature
        let mut sig_bytes = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&self.r);
        sig_bytes[32..64].copy_from_slice(&self.s);

        let signature = K256Signature::from_bytes(&sig_bytes.into())
            .map_err(|e| DecoderError::signature_verification(format!("Invalid signature: {}", e)))?;

        // Step 4: Recover public key
        let verifying_key = VerifyingKey::recover_from_prehash(&signing_hash, &signature, recovery_id)
            .map_err(|e| DecoderError::signature_verification(format!("Recovery failed: {}", e)))?;

        // Step 5: Derive address from public key
        let pubkey_bytes = verifying_key.to_encoded_point(false); // Uncompressed
        let pubkey = &pubkey_bytes.as_bytes()[1..]; // Remove 0x04 prefix

        // Address = keccak256(pubkey)[12..32]
        let hash = Keccak256::digest(pubkey);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..32]);

        Ok(address)
    }

    /// Compute the signing hash for this transaction
    ///
    /// The signing hash is what was actually signed by the sender.
    /// It differs based on transaction type.
    fn signing_hash(&self) -> Result<[u8; 32]> {
        match self.tx_type {
            TxType::Legacy => self.legacy_signing_hash(),
            TxType::Eip2930 => self.eip2930_signing_hash(),
            TxType::Eip1559 | TxType::Eip4844 => self.eip1559_signing_hash(),
        }
    }

    /// Legacy transaction signing hash
    ///
    /// For legacy transactions:
    /// - Without EIP-155: hash(rlp([nonce, gasPrice, gas, to, value, data]))
    /// - With EIP-155: hash(rlp([nonce, gasPrice, gas, to, value, data, chainId, 0, 0]))
    fn legacy_signing_hash(&self) -> Result<[u8; 32]> {
        use decoder_encodings::rlp_encoder::RlpEncoder;

        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // EIP-155: append chain_id, 0, 0
        if let Some(chain_id) = self.chain_id {
            list.append_u64(chain_id)?;
            list.append_u64(0)?;
            list.append_u64(0)?;
        }

        list.finalize()?;
        let rlp_bytes = encoder.finalize();

        // Hash the RLP encoding
        let hash = Keccak256::digest(&rlp_bytes);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// EIP-2930 transaction signing hash
    ///
    /// hash(0x01 || rlp([chainId, nonce, gasPrice, gas, to, value, data, accessList]))
    fn eip2930_signing_hash(&self) -> Result<[u8; 32]> {
        use decoder_encodings::rlp_encoder::RlpEncoder;

        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        list.append_list(|access_list| {
            for item in &self.access_list {
                access_list.append_list(|entry| {
                    entry.append_bytes(&item.address)?;
                    entry.append_list(|keys| {
                        for key in &item.storage_keys {
                            keys.append_bytes(key)?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;

        list.finalize()?;
        let rlp_bytes = encoder.finalize();

        // Prepend type byte and hash
        let mut payload = vec![0x01];
        payload.extend_from_slice(&rlp_bytes);

        let hash = Keccak256::digest(&payload);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// EIP-1559 transaction signing hash
    ///
    /// hash(0x02 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gas, to, value, data, accessList]))
    fn eip1559_signing_hash(&self) -> Result<[u8; 32]> {
        use decoder_encodings::rlp_encoder::RlpEncoder;

        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.max_priority_fee_per_gas)?;
        list.append_optional_u128(self.max_fee_per_gas)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        list.append_list(|access_list| {
            for item in &self.access_list {
                access_list.append_list(|entry| {
                    entry.append_bytes(&item.address)?;
                    entry.append_list(|keys| {
                        for key in &item.storage_keys {
                            keys.append_bytes(key)?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;

        list.finalize()?;
        let rlp_bytes = encoder.finalize();

        // Prepend type byte and hash
        let mut payload = vec![0x02];
        payload.extend_from_slice(&rlp_bytes);

        let hash = Keccak256::digest(&payload);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// Extract ECDSA recovery ID from v
    ///
    /// For legacy transactions:
    /// - Without EIP-155: v is 27 or 28, recovery_id = v - 27
    /// - With EIP-155: v = chain_id * 2 + 35 + recovery_id
    ///
    /// For typed transactions (EIP-2930/1559):
    /// - v is 0 or 1 (the recovery_id directly)
    fn get_recovery_id(&self) -> Result<RecoveryId> {
        let recovery_id = match self.tx_type {
            TxType::Legacy => {
                if let Some(chain_id) = self.chain_id {
                    // EIP-155: v = chain_id * 2 + 35 + recovery_id
                    ((self.v - 35) % 2) as u8
                } else {
                    // Pre-EIP-155: v = 27 + recovery_id
                    (self.v - 27) as u8
                }
            }
            _ => {
                // EIP-2930/EIP-1559: v is recovery_id directly (0 or 1)
                self.v as u8
            }
        };

        RecoveryId::try_from(recovery_id)
            .map_err(|e| DecoderError::signature_verification(format!("Invalid recovery ID: {}", e)))
    }

    /// Get the sender address (convenience method that calls recover_sender)
    pub fn get_from(&self) -> Result<[u8; 20]> {
        self.recover_sender()
    }
}
```

### Step 3: Update TxIR to use recovered address

```rust
// In canonicalize() method
let from_address = self.recover_sender().unwrap_or([0u8; 20]);

// Use from_address in operations and state deltas
```

### Step 4: Add tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_sender_legacy() {
        // Use a real signed transaction
        let tx_hex = "f86c..."; // Real signed tx
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

        let sender = tx.recover_sender().unwrap();
        assert_eq!(
            hex::encode(sender),
            "expected_sender_address"
        );
    }

    #[test]
    fn test_recover_sender_eip1559() {
        // Test with EIP-1559 transaction
        let tx_hex = "02f8...";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

        let sender = tx.recover_sender().unwrap();
        assert_eq!(
            hex::encode(sender),
            "expected_sender_address"
        );
    }
}
```

## Benefits of Implementation

1. **Complete transaction data**: From address is critical for analysis
2. **No external dependencies**: Can recover address from signature alone
3. **Verification**: Can verify signatures by re-computing
4. **Forensics**: Essential for tracing fund flows

## When to Implement

**Recommendation**: Add in **Phase 2** (Pure Rust Decoders) after testing infrastructure is complete.

**Timeline**: 1-2 days of work
**Difficulty**: Medium (need to handle different transaction types correctly)

## Alternatives

If you want to avoid the dependency:
1. Accept from address as external input (from block data)
2. Use simplified placeholder for now
3. Make ECDSA recovery optional feature flag

## Conclusion

**You're absolutely right** - we should implement ECDSA recovery! It's a standard feature of Ethereum transaction decoders and all the data is available in the signature. The only reason it's not implemented yet is dependency management and phase prioritization, not a technical limitation.

Would you like me to implement this now?
