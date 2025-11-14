// Universal Blockchain Decoder - WASM Demo
// Main JavaScript module for UI interactions and WASM integration

import init, { decode_transaction, supported_chains, auto_detect_chain } from './pkg/universal_decoder_wasm.js';

// Example transactions for quick testing
import { EXAMPLES } from './examples.js';

// Global state
let wasmModule = null;

// Initialize WASM module on page load
async function initWasm() {
    try {
        wasmModule = await init();
        console.log('✅ WASM module loaded successfully');
        const chains = supported_chains();
        console.log('Supported chains:', chains);
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        showError('Failed to initialize decoder. Please refresh the page.');
    }
}

// UI Elements
const chainSelect = document.getElementById('chain-select');
const exampleSelect = document.getElementById('example-select');
const decodeBtn = document.getElementById('decode-btn');
const autoDetectBtn = document.getElementById('auto-detect-btn');
const inputEditor = document.getElementById('input-editor');
const outputJson = document.getElementById('output-json');
const outputCanonical = document.getElementById('output-canonical');
const privacyContent = document.getElementById('privacy-content');
const outputMetadata = document.getElementById('output-metadata');
const errorToast = document.getElementById('error-toast');

// Tab switching
const tabBtns = document.querySelectorAll('.tab-btn');
const tabContents = document.querySelectorAll('.tab-content');

tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
        const targetTab = btn.dataset.tab;

        // Update buttons
        tabBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');

        // Update content
        tabContents.forEach(content => content.classList.remove('active'));
        document.getElementById(`${targetTab}-tab`).classList.add('active');
    });
});

// Load example transaction
exampleSelect.addEventListener('change', (e) => {
    const exampleKey = e.target.value;
    if (!exampleKey) return;

    const example = EXAMPLES[exampleKey];
    if (!example) {
        showError('Example not found');
        return;
    }

    inputEditor.value = example.hex;
    chainSelect.value = example.chain;

    console.log(`Loaded example: ${example.description}`);
});

// Decode transaction
decodeBtn.addEventListener('click', async () => {
    if (!wasmModule) {
        showError('WASM module not initialized');
        return;
    }

    const hex = inputEditor.value.trim();
    if (!hex) {
        showError('Please enter transaction hex');
        return;
    }

    const chain = chainSelect.value;

    // Show loading state
    decodeBtn.classList.add('loading');
    decodeBtn.disabled = true;

    try {
        const result = decode_transaction(chain, hex);
        displayResult(result);
        console.log('Decode result:', result);
    } catch (error) {
        showError(`Decode failed: ${error}`);
        console.error('Decode error:', error);
    } finally {
        decodeBtn.classList.remove('loading');
        decodeBtn.disabled = false;
    }
});

// Auto-detect chain
autoDetectBtn.addEventListener('click', async () => {
    if (!wasmModule) {
        showError('WASM module not initialized');
        return;
    }

    const hex = inputEditor.value.trim();
    if (!hex) {
        showError('Please enter transaction hex');
        return;
    }

    autoDetectBtn.classList.add('loading');
    autoDetectBtn.disabled = true;

    try {
        const detectedChain = auto_detect_chain(hex);
        chainSelect.value = detectedChain;
        console.log('Detected chain:', detectedChain);

        // Show success feedback
        const originalText = autoDetectBtn.textContent;
        autoDetectBtn.textContent = `✓ Detected: ${detectedChain}`;
        setTimeout(() => {
            autoDetectBtn.textContent = originalText;
        }, 2000);
    } catch (error) {
        showError(`Auto-detection failed: ${error}`);
        console.error('Auto-detect error:', error);
    } finally {
        autoDetectBtn.classList.remove('loading');
        autoDetectBtn.disabled = false;
    }
});

// Display decode result
function displayResult(result) {
    // JSON output (pretty-printed)
    outputJson.value = JSON.stringify(result.json, null, 2);

    // Canonical Borsh output
    outputCanonical.value = formatHexWithLineBreaks(result.canonical_hex);

    // Privacy analysis
    displayPrivacyAnalysis(result);

    // Metadata
    outputMetadata.style.display = 'block';
    document.getElementById('meta-chain').textContent = `${result.chain_name} (ID: ${result.chain_id})`;
    document.getElementById('meta-hash').textContent = result.canonical_hash.substring(0, 32) + '...';
    document.getElementById('meta-hash').title = result.canonical_hash;
    document.getElementById('meta-size').textContent = `${result.canonical_size} bytes`;

    // Privacy badge
    const privacyBadge = document.getElementById('meta-privacy');
    privacyBadge.textContent = getPrivacyBadge(result.privacy_score);
    privacyBadge.className = 'privacy-badge ' + getPrivacyClass(result.privacy_score);
}

// Display privacy analysis
function displayPrivacyAnalysis(result) {
    privacyContent.innerHTML = '';

    // Privacy score
    const scoreDiv = document.createElement('div');
    scoreDiv.className = 'privacy-feature';
    scoreDiv.innerHTML = `
        <h3>Privacy Score: ${result.privacy_score}/100</h3>
        <p>${getPrivacyDescription(result.privacy_score)}</p>
    `;
    privacyContent.appendChild(scoreDiv);

    // Privacy features
    if (result.privacy_features && result.privacy_features.length > 0) {
        const featuresDiv = document.createElement('div');
        featuresDiv.className = 'privacy-feature';
        featuresDiv.innerHTML = `
            <h3>Detected Features:</h3>
            <ul>
                ${result.privacy_features.map(f => `<li>${f}</li>`).join('')}
            </ul>
        `;
        privacyContent.appendChild(featuresDiv);
    }

    // Transaction type
    const typeDiv = document.createElement('div');
    typeDiv.className = 'privacy-feature';
    typeDiv.innerHTML = `
        <h3>Transaction Type:</h3>
        <p>${result.transaction_type}</p>
    `;
    privacyContent.appendChild(typeDiv);
}

// Helper functions
function getPrivacyBadge(score) {
    if (score >= 75) return '🟢 Fully Private';
    if (score >= 25) return '🟡 Partially Private';
    return '🔴 Fully Transparent';
}

function getPrivacyClass(score) {
    if (score >= 75) return 'private';
    if (score >= 25) return 'partial';
    return 'transparent';
}

function getPrivacyDescription(score) {
    if (score >= 75) {
        return 'This transaction uses strong privacy features to protect sender, receiver, or amount information.';
    } else if (score >= 25) {
        return 'This transaction has some privacy features, but not all information is protected.';
    } else {
        return 'This transaction is fully transparent. All sender, receiver, and amount information is publicly visible on the blockchain.';
    }
}

function formatHexWithLineBreaks(hex, bytesPerLine = 32) {
    // Add line breaks every N characters (2 chars = 1 byte)
    const charsPerLine = bytesPerLine * 2;
    let formatted = '';
    for (let i = 0; i < hex.length; i += charsPerLine) {
        formatted += hex.substring(i, i + charsPerLine) + '\n';
    }
    return formatted.trim();
}

function showError(message) {
    errorToast.textContent = message;
    errorToast.classList.add('show');

    setTimeout(() => {
        errorToast.classList.remove('show');
    }, 5000);
}

// Initialize on page load
window.addEventListener('DOMContentLoaded', () => {
    initWasm();
});
