# decoder-mina

Pure Rust decoder for Mina Protocol transactions.

## Overview

Mina is the world's lightest blockchain with a constant 22KB size, powered by recursive zkSNARKs.
This decoder supports:

- **zkApp Transactions**: Smart contract transactions using recursive SNARK proofs
- **Account Updates**: State changes and transitions in zkApps
- **Payment Transactions**: Standard value transfers
- **Delegation Transactions**: Stake delegation operations

## Architecture

Mina uses a unique account-based model with zkSNARK-powered smart contracts (zkApps).
Key features:

- **Constant-size blockchain**: 22KB regardless of history length
- **Recursive zkSNARKs**: Every transaction is a proof
- **Pallas/Vesta curves**: Pasta curves for efficient proof composition
- **Poseidon hash**: ZK-friendly hash function

## Transaction Types

### zkApp Transactions

zkApp transactions contain:
- **Account updates**: State changes for one or more accounts
- **Proofs**: Recursive SNARK proofs
- **Fee**: Transaction fee paid to block producer
- **Memo**: Optional 32-byte memo field

### Payment Transactions

Standard value transfers:
- **Source**: Sender public key
- **Receiver**: Recipient public key
- **Amount**: Transfer amount in nanomina (10^-9 MINA)
- **Fee**: Transaction fee
- **Nonce**: Account nonce for replay protection

## Usage

```rust
use decoder_mina::MinaDecoder;
use universal_decoder_core::Decoder;

let tx_bytes = /* Mina transaction bytes */;
let decoder = MinaDecoder::new();
let tx_ir = decoder.decode(&tx_bytes)?;

// Access decoded transaction
println!("From: {}", tx_ir.operations[0].from);
println!("To: {}", tx_ir.operations[0].to);
println!("Amount: {}", tx_ir.operations[0].amount);
```

## Cryptographic Primitives

This decoder uses the Pallas field and Poseidon hash from `decoder-crypto-zk`:

- **Pallas field**: 255-bit field (28948022309329048855892746252171976963363056481941560715954676764349967630337)
- **Poseidon hash**: ZK-friendly hash with x^7 S-box
- **Account addressing**: Base58check encoding

## References

- [Mina Protocol](https://minaprotocol.com/)
- [Mina Book](https://o1-labs.github.io/proof-systems/)
- [zkApps Documentation](https://docs.minaprotocol.com/zkapps)
- [Pasta Curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)

## Implementation Status

- [x] Pallas field arithmetic
- [x] Poseidon hash for Pallas
- [ ] Transaction type definitions
- [ ] zkApp transaction parsing
- [ ] Account update parsing
- [ ] TxIR conversion
- [ ] Signature verification
- [ ] 25+ tests

## License

See workspace LICENSE file.
