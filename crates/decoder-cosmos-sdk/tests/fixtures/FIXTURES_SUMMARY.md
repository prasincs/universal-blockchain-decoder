# Cosmos SDK Transaction Fixtures

## Overview

This directory contains test fixtures for Cosmos SDK blockchain transactions. The fixtures are hand-crafted based on the official Cosmos SDK protobuf message definitions from the cloned repository at `/home/user/universal-blockchain-decoder/tmp/fixture_fetch/cosmos-sdk`.

## Fixture Format

Each fixture consists of two files:
- `{name}.proto.hex` - The protobuf-encoded message bytes in hexadecimal format
- `{name}.json` - Metadata about the fixture including the hex bytes and message structure

## Generated Fixtures

### 1. msg_send_simple

**Description**: Simple MsgSend - 10 uatom transfer

**Source**: `cosmos/bank/v1beta1/tx.proto` - MsgSend message

**Message Structure**:
```
Field 1 (from_address): cosmos1sender1234567890abcdefghij
Field 2 (to_address):   cosmos1recipient234567890abcdefghi
Field 3 (amount):       10000000 uatom
```

**Protobuf Bytes**: 90 bytes
**Hex String**: `0a21636f736d6f733173656e646572313233343536373839306162636465666768696a1222636f736d6f7331726563697069656e743233343536373839306162636465666768691a110a057561746f6d12083130303030303030`

### 2. msg_send_multi_denom

**Description**: MsgSend with multiple denominations (5M uatom, 2.5M uosmo)

**Source**: `cosmos/bank/v1beta1/tx.proto` - MsgSend message with repeated coins

**Message Structure**:
```
Field 1 (from_address): cosmos1sender1234567890abcdefghij
Field 2 (to_address):   cosmos1recipient234567890abcdefghi
Field 3 (amount):       [
  {denom: "uatom", amount: "5000000"},
  {denom: "uosmo", amount: "2500000"}
]
```

**Protobuf Bytes**: 107 bytes

### 3. msg_send_large_amount

**Description**: MsgSend with large amount (999999999999999999 uatom)

**Source**: `cosmos/bank/v1beta1/tx.proto` - MsgSend message

**Message Structure**:
```
Field 1 (from_address): cosmos1sender1234567890abcdefghij
Field 2 (to_address):   cosmos1recipient234567890abcdefghi
Field 3 (amount):       999999999999999999 uatom
```

**Protobuf Bytes**: 100 bytes

### 4. msg_delegate_simple

**Description**: MsgDelegate - 100 stake delegation

**Source**: `cosmos/staking/v1beta1/tx.proto` - MsgDelegate message

**Message Structure**:
```
Field 1 (delegator_address): cosmos1delegator1234567890abcdefg
Field 2 (validator_address): cosmosvaloper1delegated1234567890abcdefg
Field 3 (amount):           {denom: "stake", amount: "100000000"}
```

**Protobuf Bytes**: 97 bytes
**Hex String**: `0a21636f736d6f733164656c656761746f7231323334353637383930616263646566671228636f736d6f7376616c6f7065723164656c65676174656431323334353637383930616263646566671a120a057374616b651209313030303030303030`

### 5. msg_undelegate_simple

**Description**: MsgUndelegate - 50 stake undelegation

**Source**: `cosmos/staking/v1beta1/tx.proto` - MsgUndelegate message

**Message Structure**:
```
Field 1 (delegator_address): cosmos1delegator1234567890abcdefg
Field 2 (validator_address): cosmosvaloper1delegated1234567890abcdefg
Field 3 (amount):           {denom: "stake", amount: "50000000"}
```

**Protobuf Bytes**: 96 bytes

### 6. msg_redelegate_simple

**Description**: MsgBeginRedelegate - 75 stake redelegation between validators

**Source**: `cosmos/staking/v1beta1/tx.proto` - MsgBeginRedelegate message

**Message Structure**:
```
Field 1 (delegator_address):     cosmos1delegator1234567890abcdefg
Field 2 (validator_src_address): cosmosvaloper1src1234567890abcdefghijk
Field 3 (validator_dst_address): cosmosvaloper1dst1234567890abcdefghijk
Field 4 (amount):               {denom: "stake", amount: "75000000"}
```

**Protobuf Bytes**: 134 bytes

## Protobuf Message Structures

### MsgSend (cosmos.bank.v1beta1)

```protobuf
message MsgSend {
  string from_address = 1;
  string to_address = 2;
  repeated cosmos.base.v1beta1.Coin amount = 3;
}

message Coin {
  string denom = 1;
  string amount = 2;
}
```

### MsgDelegate (cosmos.staking.v1beta1)

```protobuf
message MsgDelegate {
  string delegator_address = 1;
  string validator_address = 2;
  cosmos.base.v1beta1.Coin amount = 3;
}
```

### MsgUndelegate (cosmos.staking.v1beta1)

```protobuf
message MsgUndelegate {
  string delegator_address = 1;
  string validator_address = 2;
  cosmos.base.v1beta1.Coin amount = 3;
}
```

### MsgBeginRedelegate (cosmos.staking.v1beta1)

```protobuf
message MsgBeginRedelegate {
  string delegator_address = 1;
  string validator_src_address = 2;
  string validator_dst_address = 3;
  cosmos.base.v1beta1.Coin amount = 4;
}
```

## Fixture Generation Method

Fixtures were generated using hand-crafted protobuf encoding based on the official Cosmos SDK protobuf definitions. The process:

1. **Sources**: 
   - Proto files: `/tmp/fixture_fetch/cosmos-sdk/proto/cosmos/bank/v1beta1/tx.proto`
   - Proto files: `/tmp/fixture_fetch/cosmos-sdk/proto/cosmos/staking/v1beta1/tx.proto`

2. **Generation**: Python script (`/tmp/create_fixtures.py`) that:
   - Encodes messages using protobuf wire format
   - Implements varint encoding for field tags and lengths
   - Creates length-delimited wire type (type 2) fields
   - Outputs both hex bytes and JSON metadata

3. **Verification**: Each fixture includes:
   - Complete hex representation of protobuf bytes
   - Size metrics (protobuf bytes and hex string length)
   - Description of message contents

## Message Encoding Format

Fixtures use Protocol Buffers (protobuf3) wire format:

- **Field Tags**: (field_number << 3) | wire_type
- **Wire Types**:
  - 0 = Varint
  - 1 = 64-bit
  - 2 = Length-delimited (strings, bytes, embedded messages)
  - 3-5 = Group/fixed types (deprecated)

- **Varint Encoding**: Variable-length integer with continuation bit (MSB)

Example for field 1 (from_address):
```
Tag:  (1 << 3) | 2 = 0x0a (length-delimited)
Length: varint(33) = 0x21
Data:   "cosmos1sender1234567890abcdefghij"
Result: 0a21 + utf8_bytes
```

## Testing

These fixtures are designed for:
1. **Decoder Testing**: Verify Cosmos SDK message parsing
2. **Roundtrip Testing**: Encode → Decode → Verify
3. **Edge Case Testing**: Large amounts, multiple denoms
4. **Integration Testing**: Cross-validate with Cosmos SDK libraries

## Usage in Tests

Example Rust test:

```rust
#[test]
fn test_decode_msg_send_simple() {
    let hex = include_str\!("fixtures/simple/msg_send_simple.proto.hex");
    let bytes = hex::decode(hex).unwrap();
    
    let msg = MsgSend::decode(&bytes).unwrap();
    assert_eq\!(msg.from_address, "cosmos1sender1234567890abcdefghij");
    assert_eq\!(msg.to_address, "cosmos1recipient234567890abcdefghi");
    assert_eq\!(msg.amount[0].denom, "uatom");
    assert_eq\!(msg.amount[0].amount, "10000000");
}
```

## References

- **Cosmos SDK Repository**: https://github.com/cosmos/cosmos-sdk
- **Protobuf Specification**: https://developers.google.com/protocol-buffers/docs/encoding
- **Cosmos Bank Module**: https://docs.cosmos.network/main/modules/bank
- **Cosmos Staking Module**: https://docs.cosmos.network/main/modules/staking

## Fixture Creation Date

Generated: November 18, 2025

All fixtures are based on the official Cosmos SDK protobuf definitions and represent standard transaction messages used on Cosmos blockchains.
