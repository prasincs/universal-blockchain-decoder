# Cosmos SDK Test Fixtures

## Sources

1. **Cosmos SDK Official Test Data**
   - Repository: https://github.com/cosmos/cosmos-sdk
   - Version: v0.50.0
   - Location: `x/*/testdata/`, `tests/integration/`

2. **Real Chain Transactions**
   - Cosmos Hub, Osmosis, and other IBC chains
   - Explorer: https://www.mintscan.io

## Message Types Covered

- Bank: MsgSend, MsgMultiSend
- Staking: MsgDelegate, MsgUndelegate, MsgRedelegate
- Distribution: MsgWithdrawDelegatorReward
- Governance: MsgSubmitProposal, MsgVote
- IBC: MsgTransfer, MsgChannelOpenInit
- CosmWasm: MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

## Format

All fixtures are stored as:
- `.proto.bin` - Protobuf-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Cosmos SDK is licensed under Apache 2.0
