# Bitcoin Forks Test Fixtures

This directory contains real transaction data from various Bitcoin forks for testing compatibility.

## Supported Forks

### Bitcoin Cash (BCH)
- **Fork Date**: August 1, 2017
- **Key Differences**: Larger blocks (32MB), SIGHASH_FORKID for replay protection
- **Transaction Format**: Compatible with Bitcoin decoder (same structure)
- **Test**: `bch_simple_tx.hex` - Simple P2PKH transaction

### Litecoin (LTC)
- **Fork Date**: October 7, 2011
- **Key Differences**: Scrypt hashing, 2.5 min blocks
- **Transaction Format**: Identical to Bitcoin
- **Test**: `ltc_simple_tx.hex` - Simple P2PKH transaction

### Dogecoin (DOGE)
- **Fork Date**: December 6, 2013 (forked from Litecoin)
- **Key Differences**: Fast blocks (1 min), high supply
- **Transaction Format**: Identical to Bitcoin/Litecoin
- **Test**: `doge_simple_tx.hex` - Simple P2PKH transaction

### Bitcoin SV (BSV)
- **Fork Date**: November 15, 2018 (forked from BCH)
- **Key Differences**: Very large blocks, SIGHASH_FORKID
- **Transaction Format**: Compatible with Bitcoin decoder
- **Test**: `bsv_simple_tx.hex` - Simple P2PKH transaction

### Dash (DASH)
- **Fork Date**: January 18, 2014
- **Key Differences**: InstantSend, PrivateSend, special transactions (v3+)
- **Transaction Format**: v1-v2 identical to Bitcoin, v3+ has extra_payload
- **Test**: `dash_v1_tx.hex` - Version 1 transaction (compatible)
- **Test**: `dash_v3_special_tx.hex` - Version 3 special transaction (may need special handling)

### Bitcoin Gold (BTG)
- **Fork Date**: October 24, 2017
- **Key Differences**: Equihash mining algorithm, SIGHASH_FORK_BTG
- **Transaction Format**: Identical to Bitcoin
- **Test**: `btg_simple_tx.hex` - Simple P2PKH transaction

### Zcash (ZEC)
- **Fork Date**: October 28, 2016
- **Key Differences**: zk-SNARKs for privacy, shielded transactions
- **Transaction Format**:
  - v1-v2 transparent txs: Compatible with Bitcoin decoder
  - v4+ shielded txs: Different structure (not compatible)
- **Test**: `zec_transparent_tx.hex` - Transparent transaction (compatible)
- **Test**: `zec_shielded_tx.hex` - Shielded transaction (requires special decoder)

## Transaction Format Compatibility

| Fork | Version | Compatible | Notes |
|------|---------|------------|-------|
| BCH  | All     | ✅ Yes     | Same structure, different signature hashing |
| LTC  | All     | ✅ Yes     | Identical to Bitcoin |
| DOGE | All     | ✅ Yes     | Identical to Bitcoin/Litecoin |
| BSV  | All     | ✅ Yes     | Same structure as BCH |
| BTG  | All     | ✅ Yes     | Identical to Bitcoin |
| DASH | v1-v2   | ✅ Yes     | Standard Bitcoin format |
| DASH | v3+     | ⚠️ Partial | Has extra_payload field |
| ZEC  | v1-v2   | ✅ Yes     | Transparent transactions |
| ZEC  | v4+     | ❌ No      | Shielded transactions need special handling |

## Data Sources

All transaction data is from public blockchains:
- Bitcoin Cash: blockchain.com, explorer.bitcoin.com
- Litecoin: blockchair.com, litecoin.org
- Dogecoin: dogechain.info, blockchair.com
- Bitcoin SV: whatsonchain.com
- Dash: explorer.dash.org, blockchair.com
- Bitcoin Gold: btgexplorer.com
- Zcash: zcashblockexplorer.com, blockchair.com

## License

Transaction data is factual information from public blockchains (public domain).
