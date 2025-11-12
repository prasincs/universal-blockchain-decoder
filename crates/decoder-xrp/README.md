# decoder-xrp

XRP Ledger transaction decoder for the Universal Blockchain Decoder.

## Status

**Phase**: 1 (Scaffolding) ✅
**Production Ready**: ❌ (Phase 2 planned)

## Overview

XRP Ledger uses a custom binary serialization format (ripple-binary-codec) for transactions. This decoder will implement a pure Rust parser for this format.

## Chain Specification

- **Chain ID**: 144 (custom)
- **Consensus**: Ripple Protocol Consensus Algorithm (RPCA)
- **Chain Family**: Account
- **Transaction Format**: Binary serialization (custom)
- **Address Format**: Base58 with checksum (r-prefixed)

## Implementation Plan

### Phase 1: Scaffolding ✅

- [x] Chain identity implementation
- [x] Transaction type enumeration
- [x] Basic decoder structure
- [x] Stub implementation

### Phase 2: Pure Rust Implementation (Planned)

**Complexity**: High (custom binary format)

**Timeline**: Week 9-10

**Tasks**:
1. Implement binary codec parser
2. Handle canonical field ordering
3. Parse amount encoding (XRP drops + IOUs)
4. Support 16+ transaction types
5. Implement base58 address decoding

**Implementation Strategy**:

```rust
// Binary codec structure
pub struct BinaryCodec {
    cursor: Cursor<Vec<u8>>,
}

impl BinaryCodec {
    // Field types in XRP binary format
    fn read_field_header(&mut self) -> Result<(u16, u8)> {
        // Returns (field_id, field_type)
    }

    fn read_amount(&mut self) -> Result<Amount> {
        // XRP: 64-bit with special encoding
        // IOU: custom format with currency + issuer
    }

    fn read_account(&mut self) -> Result<AccountId> {
        // 20 bytes + base58 encoding
    }
}

// Transaction parser
impl XrpDecoder {
    fn parse_payment(codec: &mut BinaryCodec) -> Result<PaymentTx> {
        // Payment-specific fields
    }

    fn parse_offer_create(codec: &mut BinaryCodec) -> Result<OfferCreateTx> {
        // Offer-specific fields
    }

    // ... 16+ transaction type parsers
}
```

### Phase 3: Testing & Validation

**Test Plan**:
1. Unit tests for binary codec
2. Unit tests for each transaction type
3. Integration tests with real XRP transactions
4. Property tests (canonical field ordering)

**Test Fixtures**:
- Payment transactions
- DEX trades (OfferCreate, OfferCancel)
- Trust lines (TrustSet)
- Escrow transactions
- NFT transactions (mint, burn, offers)

### Phase 4: Documentation

- Binary codec specification
- Transaction type reference
- Migration guide from xrpl.js
- Example usage

## Transaction Types

XRP Ledger supports 20+ transaction types:

| Type | ID | Description |
|------|-----|-------------|
| Payment | 0 | Send XRP or tokens |
| EscrowCreate | 1 | Create held payment |
| EscrowFinish | 2 | Finish held payment |
| AccountSet | 3 | Modify account settings |
| EscrowCancel | 4 | Cancel held payment |
| SetRegularKey | 5 | Set regular key |
| OfferCreate | 7 | Create DEX order |
| OfferCancel | 8 | Cancel DEX order |
| TicketCreate | 10 | Create ticket |
| SignerListSet | 12 | Multi-sig setup |
| PaymentChannelCreate | 13 | Create payment channel |
| PaymentChannelFund | 14 | Fund payment channel |
| PaymentChannelClaim | 15 | Claim from channel |
| CheckCreate | 16 | Create check |
| CheckCash | 17 | Cash check |
| CheckCancel | 18 | Cancel check |
| DepositPreauth | 19 | Preauthorize deposit |
| TrustSet | 20 | Create/modify trust line |
| AccountDelete | 21 | Delete account |
| NFTokenMint | 25 | Mint NFT |
| NFTokenBurn | 26 | Burn NFT |
| NFTokenCreateOffer | 27 | Create NFT offer |
| NFTokenCancelOffer | 28 | Cancel NFT offer |
| NFTokenAcceptOffer | 29 | Accept NFT offer |

## Binary Codec Details

### Field Ordering

Fields are sorted by field ID for canonical serialization.

### Amount Encoding

- **XRP**: 64-bit integer (drops), with special bit flags
  - Bit 63: 0 = XRP, 1 = IOU
  - Bit 62: sign (for IOUs)
  - Bits 0-61: amount

- **IOU**: 48 bytes
  - 8 bytes: amount (scientific notation)
  - 20 bytes: currency code
  - 20 bytes: issuer address

### Address Encoding

- Base58 encoding with checksum
- Prefix: `r` for accounts
- 20-byte account ID + 4-byte checksum

## Dependencies

### Production Dependencies

- `universal-decoder-core` - Core traits and types
- `decoder-primitives` - Shared primitives
- Custom binary codec parser (to be implemented)

### Dev Dependencies

- `serde_json` - For test fixtures
- `proptest` - Property-based testing

## Resources

- [XRP Ledger Docs](https://xrpl.org/)
- [Binary Format](https://xrpl.org/serialization.html)
- [Transaction Format](https://xrpl.org/transaction-formats.html)
- [ripple-binary-codec](https://github.com/XRPLF/xrpl.js/tree/main/packages/ripple-binary-codec)

## License

MIT OR Apache-2.0
