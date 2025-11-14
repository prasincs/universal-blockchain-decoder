#!/usr/bin/env node
// Test script for WASM decoder functionality
// Usage: node test_wasm.mjs

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Example transaction (Bitcoin simple P2PKH)
const BTC_TX_HEX = '0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000';

// Ethereum legacy transaction
const ETH_TX_HEX = 'f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83';

async function testWASM() {
    console.log('🧪 Testing WASM Decoder Functionality\n');

    try {
        // Import the JS bindings first
        console.log('📦 Loading WASM module...');
        const { default: init, decode_transaction, supported_chains, auto_detect_chain } = await import('./pkg/universal_decoder_wasm.js');

        // Check WASM file size
        const wasmPath = join(__dirname, 'pkg', 'universal_decoder_wasm_bg.wasm');
        const wasmBuffer = readFileSync(wasmPath);
        console.log('✅ WASM module found');
        console.log(`   Size: ${(wasmBuffer.length / 1024).toFixed(2)} KB\n`);

        // Initialize WASM by passing the buffer directly (for Node.js)
        console.log('🔧 Initializing WASM...');
        await init(wasmBuffer);
        console.log('✅ WASM initialized\n');

        // Test 1: Supported chains
        console.log('Test 1: Supported Chains');
        console.log('------------------------');
        const chains = supported_chains();
        console.log(`Supported chains: ${JSON.stringify(chains)}`);
        console.log(`✅ Found ${chains.length} chains\n`);

        // Test 2: Auto-detect Bitcoin
        console.log('Test 2: Auto-Detect Chain (Bitcoin)');
        console.log('------------------------------------');
        const detectedBtc = auto_detect_chain(BTC_TX_HEX);
        console.log(`Input: ${BTC_TX_HEX.substring(0, 40)}...`);
        console.log(`Detected: ${detectedBtc}`);
        if (detectedBtc === 'bitcoin') {
            console.log('✅ Correctly detected Bitcoin\n');
        } else {
            console.log('❌ Failed to detect Bitcoin\n');
            return false;
        }

        // Test 3: Decode Bitcoin transaction
        console.log('Test 3: Decode Bitcoin Transaction');
        console.log('-----------------------------------');
        const btcResult = decode_transaction('bitcoin', BTC_TX_HEX);
        console.log(`Result structure:`, JSON.stringify(btcResult, null, 2));
        console.log(`Chain: ${btcResult.chain_name} (ID: ${btcResult.chain_id})`);
        console.log(`Canonical hash: ${btcResult.canonical_hash.substring(0, 32)}...`);
        console.log(`Canonical size: ${btcResult.canonical_size} bytes`);

        if (btcResult.chain_name && btcResult.canonical_hash) {
            console.log('✅ Bitcoin decode successful\n');
        } else {
            console.log('❌ Bitcoin decode failed\n');
            return false;
        }

        // Test 4: Auto-detect Ethereum
        console.log('Test 4: Auto-Detect Chain (Ethereum)');
        console.log('-------------------------------------');
        const detectedEth = auto_detect_chain(ETH_TX_HEX);
        console.log(`Input: ${ETH_TX_HEX.substring(0, 40)}...`);
        console.log(`Detected: ${detectedEth}`);
        if (detectedEth === 'ethereum') {
            console.log('✅ Correctly detected Ethereum\n');
        } else {
            console.log('❌ Failed to detect Ethereum\n');
            return false;
        }

        // Test 5: Decode Ethereum transaction
        console.log('Test 5: Decode Ethereum Transaction');
        console.log('------------------------------------');
        const ethResult = decode_transaction('ethereum', ETH_TX_HEX);
        console.log(`Chain: ${ethResult.chain_name} (ID: ${ethResult.chain_id})`);
        console.log(`Canonical hash: ${ethResult.canonical_hash.substring(0, 32)}...`);
        console.log(`Canonical size: ${ethResult.canonical_size} bytes`);

        if (ethResult.chain_name && ethResult.canonical_hash) {
            console.log('✅ Ethereum decode successful\n');
        } else {
            console.log('❌ Ethereum decode failed\n');
            return false;
        }

        // All tests passed
        console.log('═══════════════════════════════════════');
        console.log('✅ All WASM tests passed!');
        console.log('═══════════════════════════════════════');
        return true;

    } catch (error) {
        console.error('❌ WASM test failed:', error.message);
        console.error(error.stack);
        return false;
    }
}

// Run tests
testWASM().then(success => {
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});
