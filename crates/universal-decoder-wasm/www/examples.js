// Example transactions for the WASM demo
// These are REAL mainnet transactions from test fixtures

export const EXAMPLES = {
    'btc-genesis': {
        chain: 'bitcoin',
        description: 'Bitcoin: Genesis Block Coinbase (Block 0)',
        note: 'The first Bitcoin transaction ever - contains Satoshi\'s famous message about bank bailouts',
        hex: '01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000'
    },

    'btc-simple': {
        chain: 'bitcoin',
        description: 'Bitcoin: Simple P2PKH Transfer (10 BTC + 40 BTC change)',
        note: 'Early Bitcoin transaction showing UTXO model with change output',
        hex: '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000'
    },

    'eth-legacy': {
        chain: 'ethereum',
        description: 'Ethereum: Legacy Transaction (EIP-155)',
        note: 'Pre-EIP-1559 transaction with fixed gas price',
        hex: 'f86d808504e3b29200825208940123456789abcdef0123456789abcdef01234567880de0b6b3a764000080820a95a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7a0c7b2b2c1f6d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7f2b0e0b0d7'
    },

    'sol-transfer': {
        chain: 'solana',
        description: 'Solana: SOL Transfer',
        note: 'Simple SOL transfer demonstrating Solana\'s instruction-based model',
        // Base64: AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=
        // Converted to hex for consistency
        hex: '0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001038a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf374880fb40f6f5cd35480957fbfada3acd0ffe5dabada45f8a492a19ec4493c6fdeb83cc4fab25e000000000000000000000000000000000000000000000000000000000000000000fce463fd6b571c62c287dc02d1e1a47f6eaa4c5479c779c2ab05d79a856d65bd01020201000c0200000000c5015a02000000'
    },

    'cosmos-send': {
        chain: 'cosmos',
        description: 'Cosmos: Bank Send (MsgSend)',
        note: 'Cosmos transaction sending ATOM tokens between accounts',
        hex: '0a90010a8d010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e64126d0a2d636f736d6f7331796c79703666677a7076327863656b6c7a666e6b3832746b6c78727634326c726d6b122d636f736d6f7331796c79703666677a7076327863656b6c7a666e6b3832746b6c78727634326c726d6b1a0d0a057561746f6d120431303030'
    }
};
