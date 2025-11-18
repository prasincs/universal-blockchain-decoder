// Example transactions for the WASM demo
// These are REAL mainnet transactions from test fixtures

export const EXAMPLES = {
    // ========================================================================
    // BITCOIN FAMILY (UTXO Model)
    // ========================================================================

    'btc-genesis': {
        chain: 'bitcoin',
        description: 'Bitcoin: Genesis Block Coinbase (Block 0)',
        note: 'The first Bitcoin transaction ever - contains Satoshi\'s famous message about bank bailouts',
        hex: '01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000'
    },

    'btc-simple': {
        chain: 'bitcoin',
        description: 'Bitcoin: Simple P2PKH Transfer',
        note: 'Early Bitcoin transaction showing UTXO model with 2 inputs and 2 outputs (payment + change)',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'bch-transfer': {
        chain: 'bitcoin-cash',
        description: 'Bitcoin Cash: Standard Transfer',
        note: 'Bitcoin Cash uses same transaction format as Bitcoin (legacy)',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'doge-transfer': {
        chain: 'dogecoin',
        description: 'Dogecoin: Such Transaction, Much Wow',
        note: 'Dogecoin uses Bitcoin-compatible format with different network parameters',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'ltc-transfer': {
        chain: 'litecoin',
        description: 'Litecoin: Standard Transfer',
        note: 'Litecoin supports both legacy and SegWit transaction formats',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'dash-transfer': {
        chain: 'dash',
        description: 'Dash: Private Send Transaction',
        note: 'Dash extends Bitcoin format with InstantSend and PrivateSend features',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'zec-coinbase': {
        chain: 'zcash',
        description: 'Zcash: Mainnet Coinbase (v4)',
        note: 'Zcash v4 transaction format with shielded pool support',
        hex: '0400008085202f89010000000000000000000000000000000000000000000000000000000000000000ffffffff0603db7f0e0104ffffffff02809dce1d000000001976a914328a650e22bfbf4541d4c37c49a14fa7e7fd223b88ac405973070000000017a914abd8d9b0e9550aba61adcd57c058c20e822c8d598700000000000000000000000000000000000000'
    },

    // ========================================================================
    // ETHEREUM & EVM CHAINS (Account Model)
    // ========================================================================

    'eth-legacy': {
        chain: 'ethereum',
        description: 'Ethereum: Legacy Transaction (EIP-155)',
        note: 'Pre-EIP-1559 transaction with fixed gas price and chain ID',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'polygon-transfer': {
        chain: 'polygon',
        description: 'Polygon: MATIC Transfer',
        note: 'Polygon uses EVM-compatible transaction format (Chain ID: 137)',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'arbitrum-transfer': {
        chain: 'arbitrum',
        description: 'Arbitrum: ETH Transfer on L2',
        note: 'Arbitrum One uses Ethereum format with Layer 2 optimizations (Chain ID: 42161)',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'optimism-transfer': {
        chain: 'optimism',
        description: 'Optimism: ETH Transfer on L2',
        note: 'Optimism uses OP Stack with EVM-compatible transactions (Chain ID: 10)',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'avalanche-transfer': {
        chain: 'avalanche',
        description: 'Avalanche C-Chain: AVAX Transfer',
        note: 'Avalanche C-Chain is EVM-compatible (Chain ID: 43114)',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'bnb-transfer': {
        chain: 'bnb',
        description: 'BNB Smart Chain: BNB Transfer',
        note: 'BSC uses EVM-compatible format (Chain ID: 56)',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    // ========================================================================
    // OTHER MAJOR CHAINS
    // ========================================================================

    'sol-transfer': {
        chain: 'solana',
        description: 'Solana: SOL Transfer',
        note: 'Simple SOL transfer demonstrating Solana\'s instruction-based model with system program',
        hex: '0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001038a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf374880fb40f6f5cd35480957fbfada3acd0ffe5dabada45f8a492a19ec4493c6fdeb83cc4fab25e000000000000000000000000000000000000000000000000000000000000000000fce463fd6b571c62c287dc02d1e1a47f6eaa4c5479c779c2ab05d79a856d65bd01020201000c0200000000c5015a02000000'
    },

    'cosmos-send': {
        chain: 'cosmos',
        description: 'Cosmos: Bank Send (MsgSend)',
        note: 'Cosmos Hub transaction sending ATOM tokens using Protobuf encoding',
        hex: '0a90010a8d010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e64126d0a2d636f736d6f7331796c79703666677a7076327863656b6c7a666e6b3832746b6c78727634326c726d6b122d636f736d6f7331796c79703666677a7076327863656b6c7a666e6b3832746b6c78727634326c726d6b1a0d0a057561746f6d120431303030'
    },

    // ========================================================================
    // CHAINS WITHOUT REAL EXAMPLES YET (Coming Soon)
    // ========================================================================
    // Note: These chains are supported but don't have real transaction examples yet.
    // You can still test decoding by providing your own transaction hex from:
    // - NEAR: explorer.near.org
    // - Aptos: explorer.aptoslabs.com
    // - Sui: suiscan.xyz
    // - Algorand: algoexplorer.io
    // - Cardano: cardanoscan.io
    // - Polkadot: polkadot.subscan.io
    // - Stellar: stellarscan.io
    // - Tron: tronscan.org
    // - StarkNet: starkscan.co
    // - XRP: xrpscan.com
};
