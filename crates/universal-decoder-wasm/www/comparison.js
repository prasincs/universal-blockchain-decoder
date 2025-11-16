// Type System Visualization - Interactive Comparison

// Chain data for comparison
const CHAIN_DATA = {
    bitcoin: {
        name: 'Bitcoin',
        family: 'UTXO',
        familyClass: 'utxo',
        size: '226 bytes',
        signature: 'ECDSA (secp256k1)',
        hash: 'Double SHA-256',
        encoding: 'Custom binary + VarInt',
        fee: 'Implicit (inputs - outputs)',
        replay: 'UTXO uniqueness',
        txir: `operations: [Transfer]
state_deltas:
  inputs: [InputReference]
  outputs: [OutputValue]
  account_changes: []
privacy: None`
    },
    ethereum: {
        name: 'Ethereum',
        family: 'Account',
        familyClass: 'account',
        size: '109 bytes',
        signature: 'ECDSA (secp256k1)',
        hash: 'Keccak-256',
        encoding: 'RLP (Recursive Length Prefix)',
        fee: 'Gas (gas_used × gas_price)',
        replay: 'Nonce (sequential)',
        txir: `operations: [Transfer]
state_deltas:
  inputs: []
  outputs: []
  account_changes: [AccountChange]
privacy: None`
    },
    solana: {
        name: 'Solana',
        family: 'Instruction',
        familyClass: 'instruction',
        size: '335 bytes',
        signature: 'EdDSA (ed25519)',
        hash: 'SHA-256',
        encoding: 'Borsh',
        fee: 'Per-signature (5000 lamports)',
        replay: 'Recent blockhash',
        txir: `operations: [Generic]
state_deltas:
  inputs: []
  outputs: []
  account_changes: [AccountChange]
privacy: None`
    },
    polygon: {
        name: 'Polygon',
        family: 'Account',
        familyClass: 'account',
        size: '109 bytes',
        signature: 'ECDSA (secp256k1)',
        hash: 'Keccak-256',
        encoding: 'RLP (same as Ethereum)',
        fee: 'Gas (same as Ethereum)',
        replay: 'Nonce (sequential)',
        txir: `operations: [Transfer]
state_deltas:
  inputs: []
  outputs: []
  account_changes: [AccountChange]
privacy: None`
    }
};

// Initialize comparison
function initComparison() {
    const chainASelect = document.getElementById('chain-a-select');
    const chainBSelect = document.getElementById('chain-b-select');

    chainASelect.addEventListener('change', () => updateComparison());
    chainBSelect.addEventListener('change', () => updateComparison());

    // Initial update
    updateComparison();
}

// Update comparison display
function updateComparison() {
    const chainAKey = document.getElementById('chain-a-select').value;
    const chainBKey = document.getElementById('chain-b-select').value;

    const chainA = CHAIN_DATA[chainAKey];
    const chainB = CHAIN_DATA[chainBKey];

    // Update Chain A
    document.getElementById('chain-a-name').textContent = chainA.name;
    document.getElementById('chain-a-family').textContent = `${chainA.family} Family`;
    document.getElementById('chain-a-family').className = `family-tag ${chainA.familyClass}`;
    document.getElementById('chain-a-size').textContent = chainA.size;
    document.getElementById('chain-a-sig').textContent = chainA.signature;
    document.getElementById('chain-a-hash').textContent = chainA.hash;
    document.getElementById('chain-a-encoding').textContent = chainA.encoding;
    document.getElementById('chain-a-fee').textContent = chainA.fee;
    document.getElementById('chain-a-replay').textContent = chainA.replay;
    document.getElementById('chain-a-txir').textContent = chainA.txir;

    // Update Chain B
    document.getElementById('chain-b-name').textContent = chainB.name;
    document.getElementById('chain-b-family').textContent = `${chainB.family} Family`;
    document.getElementById('chain-b-family').className = `family-tag ${chainB.familyClass}`;
    document.getElementById('chain-b-size').textContent = chainB.size;
    document.getElementById('chain-b-sig').textContent = chainB.signature;
    document.getElementById('chain-b-hash').textContent = chainB.hash;
    document.getElementById('chain-b-encoding').textContent = chainB.encoding;
    document.getElementById('chain-b-fee').textContent = chainB.fee;
    document.getElementById('chain-b-replay').textContent = chainB.replay;
    document.getElementById('chain-b-txir').textContent = chainB.txir;

    // Update differences list
    updateDifferencesList(chainA, chainB);
}

// Update the differences list based on selected chains
function updateDifferencesList(chainA, chainB) {
    const differencesList = document.getElementById('differences-list');
    const differences = [];

    // State Model
    if (chainA.family !== chainB.family) {
        differences.push(`<strong>Chain Family:</strong> ${chainA.family} vs ${chainB.family}`);
    }

    // Signature
    if (chainA.signature !== chainB.signature) {
        differences.push(`<strong>Signature Scheme:</strong> ${chainA.signature} vs ${chainB.signature}`);
    } else {
        differences.push(`<strong>Signature Scheme:</strong> Both use ${chainA.signature} ✓`);
    }

    // Hash
    if (chainA.hash !== chainB.hash) {
        differences.push(`<strong>Hash Algorithm:</strong> ${chainA.hash} vs ${chainB.hash}`);
    } else {
        differences.push(`<strong>Hash Algorithm:</strong> Both use ${chainA.hash} ✓`);
    }

    // Encoding
    if (chainA.encoding !== chainB.encoding) {
        differences.push(`<strong>Encoding:</strong> ${chainA.encoding} vs ${chainB.encoding}`);
    }

    // Fee
    if (chainA.fee !== chainB.fee) {
        differences.push(`<strong>Fee Mechanism:</strong> ${chainA.fee} vs ${chainB.fee}`);
    }

    // Replay
    if (chainA.replay !== chainB.replay) {
        differences.push(`<strong>Replay Protection:</strong> ${chainA.replay} vs ${chainB.replay}`);
    }

    // Size comparison
    const sizeA = parseInt(chainA.size);
    const sizeB = parseInt(chainB.size);
    if (sizeA !== sizeB) {
        const larger = sizeA > sizeB ? chainA.name : chainB.name;
        const pct = Math.abs(Math.round((sizeA - sizeB) / Math.min(sizeA, sizeB) * 100));
        differences.push(`<strong>Transaction Size:</strong> ${larger} is ${pct}% larger`);
    }

    differencesList.innerHTML = differences.map(d => `<li>${d}</li>`).join('');
}

// Toggle type node expansion
function toggleNode(header) {
    const node = header.parentElement;
    node.classList.toggle('collapsed');
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    initComparison();

    // Add smooth scroll for section links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Animate stats on scroll
    const observerOptions = {
        threshold: 0.5,
        rootMargin: '0px 0px -100px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.animation = 'fadeIn 0.6s ease-out';
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    document.querySelectorAll('.stat-card').forEach(card => {
        observer.observe(card);
    });
});

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    // Press 'C' to toggle all type nodes
    if (e.key === 'c' || e.key === 'C') {
        const rootNode = document.querySelector('.type-node.root');
        if (rootNode) {
            toggleNode(rootNode.querySelector('.node-header'));
        }
    }

    // Press numbers 1-4 to quick-select chain families
    const familyMap = {
        '1': 'bitcoin',    // UTXO
        '2': 'ethereum',   // Account
        '3': 'solana',     // Instruction
        '4': 'polygon'     // Account (alternative)
    };

    if (familyMap[e.key]) {
        document.getElementById('chain-a-select').value = familyMap[e.key];
        updateComparison();
    }
});

// Export for potential embedding/integration
window.ChainComparison = {
    data: CHAIN_DATA,
    update: updateComparison,
    toggleNode: toggleNode
};
