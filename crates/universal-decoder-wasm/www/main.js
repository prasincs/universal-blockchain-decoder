// Universal Blockchain Decoder - WASM Demo
// Main JavaScript module for UI interactions and WASM integration

// Example transactions for quick testing
import { EXAMPLES } from './examples.js';
import { initVerificationDashboard, addVerificationStyles } from './verification.js';

// Global state
let wasmModule = null;
let decode_transaction = null;
let supported_chains = null;
let get_chains_metadata = null;
let auto_detect_chain = null;

// Initialize WASM module on page load
async function initWasm() {
    try {
        // Store and temporarily remove window.ethereum to avoid conflicts with MetaMask
        const originalEthereum = window.ethereum;
        const ethereumDescriptor = Object.getOwnPropertyDescriptor(window, 'ethereum');

        // Temporarily make ethereum configurable if it exists
        if (ethereumDescriptor && !ethereumDescriptor.configurable) {
            try {
                delete window.ethereum;
            } catch (e) {
                console.warn('Could not delete window.ethereum, trying alternative approach');
            }
        }

        // Dynamic import to avoid conflicts
        wasmModule = await import('./pkg/universal_decoder_wasm.js');

        // Restore window.ethereum
        if (originalEthereum) {
            try {
                Object.defineProperty(window, 'ethereum', {
                    value: originalEthereum,
                    writable: ethereumDescriptor?.writable ?? true,
                    configurable: ethereumDescriptor?.configurable ?? true,
                    enumerable: ethereumDescriptor?.enumerable ?? true
                });
            } catch (e) {
                window.ethereum = originalEthereum;
            }
        }

        // Initialize WASM
        await wasmModule.default();

        // Store functions globally
        decode_transaction = wasmModule.decode_transaction;
        supported_chains = wasmModule.supported_chains;
        get_chains_metadata = wasmModule.get_chains_metadata;
        auto_detect_chain = wasmModule.auto_detect_chain;

        console.log('✅ WASM module loaded successfully');

        // Dynamically populate chain dropdown
        populateChainDropdown();

        // Dynamically populate example dropdown (scoped to first chain)
        const firstChain = chainSelect.value;
        populateExampleDropdown(firstChain);

        // Initialize verification dashboard
        addVerificationStyles();
        initVerificationDashboard(wasmModule);

        // Restore saved input for the initially selected chain
        if (firstChain) {
            const savedInput = localStorage.getItem(`decoder-input-${firstChain}`);
            if (savedInput) {
                console.log(`Restoring saved input for initial chain ${firstChain}`);
                inputEditor.value = savedInput;
            }
        }
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        showError('Failed to initialize decoder. Please refresh the page.');
    }
}

// Populate chain dropdown dynamically from WASM with family grouping
function populateChainDropdown() {
    try {
        const chainsMetadata = get_chains_metadata();
        const chainSelect = document.getElementById('chain-select');

        // Clear existing options
        chainSelect.innerHTML = '';

        // Group chains by family
        const chainsByFamily = {};
        chainsMetadata.forEach(chain => {
            if (!chainsByFamily[chain.family]) {
                chainsByFamily[chain.family] = [];
            }
            chainsByFamily[chain.family].push(chain);
        });

        // Add chains grouped by family
        const familyOrder = ['UTXO', 'Account', 'Instruction', 'Object', 'Privacy'];
        familyOrder.forEach(family => {
            if (chainsByFamily[family]) {
                const optgroup = document.createElement('optgroup');
                optgroup.label = `${family} Chains`;

                chainsByFamily[family].forEach(chain => {
                    const option = document.createElement('option');
                    option.value = chain.id;
                    option.textContent = chain.name;
                    optgroup.appendChild(option);
                });

                chainSelect.appendChild(optgroup);
            }
        });

        console.log(`Populated ${chainsMetadata.length} chains in dropdown`);
    } catch (error) {
        console.error('Failed to populate chain dropdown:', error);
    }
}

// Populate example dropdown dynamically from examples.js, scoped to selected chain
function populateExampleDropdown(selectedChain = null) {
    const exampleSelect = document.getElementById('example-select');

    // Clear existing options (except placeholder)
    exampleSelect.innerHTML = '<option value="">-- Select Example --</option>';

    // Filter examples by selected chain
    const filteredExamples = Object.entries(EXAMPLES).filter(([key, example]) => {
        return !selectedChain || example.chain === selectedChain;
    });

    // Add examples
    filteredExamples.forEach(([key, example]) => {
        const option = document.createElement('option');
        option.value = key;
        option.textContent = example.description;
        if (example.note) {
            option.title = example.note;
        }
        exampleSelect.appendChild(option);
    });

    console.log(`Populated ${filteredExamples.length} examples for chain: ${selectedChain || 'all'}`);
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

// Chain selection change - update examples and restore saved input
chainSelect.addEventListener('change', (e) => {
    const selectedChain = e.target.value;

    // Update examples dropdown to show only examples for this chain
    populateExampleDropdown(selectedChain);

    // Restore saved input for this chain from localStorage
    const savedInput = localStorage.getItem(`decoder-input-${selectedChain}`);
    if (savedInput) {
        console.log(`Restoring saved input for chain ${selectedChain}`);
        inputEditor.value = savedInput;
    } else {
        // Clear input if no saved value for this chain
        console.log(`No saved input for chain ${selectedChain}, clearing`);
        inputEditor.value = '';
    }

    // Clear output when changing chains
    outputJson.value = '';
    outputCanonical.value = '';
    outputMetadata.style.display = 'none';
});

// Save input to localStorage when it changes
inputEditor.addEventListener('input', () => {
    const currentChain = chainSelect.value;
    if (currentChain) {
        localStorage.setItem(`decoder-input-${currentChain}`, inputEditor.value);
        console.log(`Saved input to localStorage for chain ${currentChain}`);
    }
});

// Load example transaction - only populates input, user must select chain
exampleSelect.addEventListener('change', (e) => {
    const exampleKey = e.target.value;
    if (!exampleKey) return;

    const example = EXAMPLES[exampleKey];
    if (!example) {
        showError('Example not found');
        return;
    }

    // Only populate the input text box, don't auto-select chain
    inputEditor.value = example.hex;

    // Save to localStorage for current chain
    const currentChain = chainSelect.value;
    if (currentChain) {
        localStorage.setItem(`decoder-input-${currentChain}`, example.hex);
    }

    console.log(`Loaded example: ${example.description} (for ${example.chain})`);
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
    if (!chain) {
        showError('Please select a blockchain');
        return;
    }

    console.log(`Decoding ${hex.length} chars of hex for chain: ${chain}`);

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

// Helper function to convert Map to plain object recursively
function mapToObject(map) {
    if (map instanceof Map) {
        const obj = {};
        for (const [key, value] of map.entries()) {
            obj[key] = mapToObject(value);
        }
        return obj;
    } else if (Array.isArray(map)) {
        return map.map(item => mapToObject(item));
    } else if (map && typeof map === 'object') {
        // Already a plain object, but might have nested Maps
        const obj = {};
        for (const [key, value] of Object.entries(map)) {
            obj[key] = mapToObject(value);
        }
        return obj;
    } else {
        return map;
    }
}

// Display decode result
function displayResult(result) {
    console.log('displayResult called with:', result);
    console.log('Result keys:', Object.keys(result));
    console.log('Result type:', typeof result);

    // Try accessing as properties vs methods
    console.log('result.json (property):', result.json);
    console.log('result.borsh_fields (property):', result.borsh_fields);

    // Check if they're functions
    console.log('typeof result.json:', typeof result.json);
    console.log('typeof result.borsh_fields:', typeof result.borsh_fields);

    console.log('outputJson element:', outputJson);
    console.log('outputCanonical element:', outputCanonical);

    // JSON output (pretty-printed)
    try {
        // Access as getter property (wasm-bindgen should handle this)
        const jsonData = result.json;
        console.log('JSON data retrieved');
        console.log('JSON data type:', typeof jsonData);
        console.log('JSON data value:', jsonData);
        console.log('JSON data is Map?', jsonData instanceof Map);

        if (jsonData === null || jsonData === undefined) {
            console.error('JSON data is null or undefined!');
            outputJson.value = 'Error: JSON data is null or undefined';
        } else {
            // Convert Map to plain object
            const jsonObject = mapToObject(jsonData);
            console.log('Converted to plain object:', jsonObject);

            const jsonString = JSON.stringify(jsonObject, null, 2);
            console.log('JSON stringified successfully, length:', jsonString.length);
            console.log('First 100 chars:', jsonString.substring(0, 100));
            outputJson.value = jsonString;
            console.log('Set outputJson.value, new value length:', outputJson.value.length);
        }
    } catch (e) {
        console.error('JSON display exception:', e);
        outputJson.value = 'Error displaying JSON: ' + e.message + '\nStack: ' + e.stack;
    }

    // Canonical Borsh output - INVERTED: Show fields first, then raw payload
    try {
        const borshFields = result.borsh_fields;
        console.log('Borsh fields retrieved');
        console.log('Borsh fields type:', typeof borshFields);
        console.log('Borsh fields value:', borshFields);
        console.log('Borsh fields is Map?', borshFields instanceof Map);

        let borshOutput = '// Borsh Fields (Structured Representation)\n';

        if (borshFields === null || borshFields === undefined) {
            console.error('Borsh fields are null or undefined!');
            borshOutput += 'Error: Borsh fields are null or undefined\n';
        } else {
            // Convert Map to plain object
            const borshObject = mapToObject(borshFields);
            console.log('Converted to plain object:', borshObject);

            const borshString = JSON.stringify(borshObject, null, 2);
            console.log('Borsh fields stringified successfully, length:', borshString.length);
            borshOutput += borshString;
        }

        borshOutput += '\n\n';
        borshOutput += '// Raw Borsh Payload (Hex)\n';
        borshOutput += formatHexWithLineBreaks(result.canonical_hex);

        console.log('Setting outputCanonical.value, total length:', borshOutput.length);
        outputCanonical.value = borshOutput;
        console.log('Set outputCanonical.value, new value length:', outputCanonical.value.length);
    } catch (e) {
        console.error('Borsh display exception:', e);
        outputCanonical.value = 'Error displaying Borsh: ' + e.message + '\nStack: ' + e.stack;
    }

    // Privacy analysis
    displayPrivacyAnalysis(result);

    // Metadata
    outputMetadata.style.display = 'block';
    document.getElementById('meta-chain').textContent = `${result.chain_name} (ID: ${result.chain_id})`;

    // Original transaction hash
    document.getElementById('meta-txid').textContent = result.tx_hash.substring(0, 32) + '...';
    document.getElementById('meta-txid').title = result.tx_hash;

    // Canonical Borsh hash
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

// Helper function to clear saved inputs (available in console)
window.clearDecoderInputs = function(chain = null) {
    if (chain) {
        localStorage.removeItem(`decoder-input-${chain}`);
        console.log(`Cleared saved input for chain: ${chain}`);
    } else {
        // Clear all decoder inputs
        Object.keys(localStorage).forEach(key => {
            if (key.startsWith('decoder-input-')) {
                localStorage.removeItem(key);
            }
        });
        console.log('Cleared all saved decoder inputs');
    }
};

// Initialize on page load
window.addEventListener('DOMContentLoaded', () => {
    initWasm();
});
