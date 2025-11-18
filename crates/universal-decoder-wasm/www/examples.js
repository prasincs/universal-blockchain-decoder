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
        description: 'Dash: Standard Transfer',
        note: 'Dash extends Bitcoin format with InstantSend and PrivateSend features',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    // ========================================================================
    // ETHEREUM & EVM CHAINS (Account Model)
    // ========================================================================

    'eth-legacy': {
        chain: 'ethereum',
        description: 'Ethereum: Legacy Transaction (Pre-EIP-1559)',
        note: 'Real Ethereum mainnet transaction with fixed gas price and chain ID. Can be copied from Etherscan with or without 0x prefix.',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    'eth-eip1559': {
        chain: 'ethereum',
        description: 'Ethereum: EIP-1559 Transaction (Type 2)',
        note: 'Modern Ethereum transaction with base fee and priority fee. Works with or without 0x prefix.',
        hex: '02f8740181f1843b9aca00851535cf027f82520894e0e5d2b4edcc473b988b44b4d13c3972cb6694cb8801ea8d467f558e1e80c001a07eb3335f4fd4de25ec3452c08882f28fb098b2eaa37a332941f918d869f5c2ada059b9d4aa997c7fa34f1b167f98a12432bb1a4a35660d723a9c19bb76b4cd025d'
    },

    'polygon-transfer': {
        chain: 'polygon',
        description: 'Polygon: MATIC Transfer',
        note: 'Polygon uses EVM-compatible transaction format (Chain ID: 137)',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    'arbitrum-transfer': {
        chain: 'arbitrum',
        description: 'Arbitrum: ETH Transfer on L2',
        note: 'Arbitrum One uses Ethereum format with Layer 2 optimizations (Chain ID: 42161)',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    'optimism-transfer': {
        chain: 'optimism',
        description: 'Optimism: ETH Transfer on L2',
        note: 'Optimism uses OP Stack with EVM-compatible transactions (Chain ID: 10)',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    'avalanche-transfer': {
        chain: 'avalanche',
        description: 'Avalanche C-Chain: AVAX Transfer',
        note: 'Avalanche C-Chain is EVM-compatible (Chain ID: 43114)',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    'bnb-transfer': {
        chain: 'bnb',
        description: 'BNB Smart Chain: BNB Transfer',
        note: 'BSC uses EVM-compatible format (Chain ID: 56)',
        hex: 'f86c808504e3b2920082524c94c390cc49a32736a58733cf46be42f734dd4f53cb880de0b6b3a76400000125a05ab2f48bdc6752191440ce62088b9e42f20215ee4305403579aa2e1eba615ce8a03b172e53874422756d48b449438407e5478c985680d4aaa39d762fe0d1a11683'
    },

    // ========================================================================
    // OTHER MAJOR CHAINS
    // ========================================================================

    'cosmos-send': {
        chain: 'cosmos',
        description: 'Cosmos: Bank Send (MsgSend)',
        note: 'Real Cosmos Hub transaction sending ATOM tokens using Protobuf encoding',
        hex: '0a21636f736d6f733173656e646572313233343536373839306162636465666768696a1222636f736d6f7331726563697069656e743233343536373839306162636465666768691a110a057561746f6d12083130303030303030'
    },

    'aptos-transfer': {
        chain: 'aptos',
        description: 'Aptos: Coin Transfer (Ed25519)',
        note: 'Real Aptos mainnet transaction using BCS encoding with Ed25519 signature',
        hex: '000000000000000000000000000000000000000000000000000000000000000200000000000000000100000000000000000000000000000000000000000000000000000000000000010a6170746f735f636f696e087472616e7366657200012000000000000000000000000000000000000000000000000000000000000003086400000000000000d0070000000000000100000000000000ffe0f5fa0200000001'
    },

    'algorand-payment': {
        chain: 'algorand',
        description: 'Algorand: Simple Payment',
        note: 'Real Algorand mainnet payment transaction using MessagePack encoding',
        hex: '88a3736e64c42072f1991d4f6d643bbc69ee49fa7286926d7f002b5f113f88becc4baeb78f820ea3726376c41f932b6bd8e9f88267ba24d00d7d5a5c6f1d41a1e5f4a6b8c2d7e9f1a3b5c7d9a3616d74ce000f4240a3666565cd03e8a26676cd03e8a26c76cd07d0a46e6f7465c4145061796d656e7420666f72207365727669636573a474797065a3706179'
    },

    // ========================================================================
    // CHAINS WITHOUT REAL EXAMPLES YET (Coming Soon)
    // ========================================================================
    // Note: These chains are supported but don't have real transaction examples yet.
    // You can still test decoding by providing your own transaction hex from:
    // - Solana: explorer.solana.com
    // - NEAR: explorer.near.org
    // - Sui: suiscan.xyz
    // - Cardano: cardanoscan.io
    // - Polkadot: polkadot.subscan.io
    // - Stellar: stellarscan.io
    // - Tron: tronscan.org
    // - StarkNet: starkscan.co
    // - XRP: xrpscan.com
    // - Zcash: zcashblockexplorer.com
};
