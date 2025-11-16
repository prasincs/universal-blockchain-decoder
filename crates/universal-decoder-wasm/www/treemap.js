// Chain Ecosystem Treemap Visualization
// Shows 2500+ mainnet chains hierarchically with filters and drill-down
// (4900+ total including testnets)

// Chain data structure
// In production, this would be fetched from live APIs (DeFiLlama, chain-specific endpoints)
// EVM count from embedded registry: 2397 chains (see crates/decoder-evm/data/chains.metadata.txt)
const CHAIN_DATA = {
    name: "All Chains",
    children: [
        {
            name: "UTXO Family",
            family: "utxo",
            description: "Unspent Transaction Output model - like physical cash",
            children: [
                { name: "Bitcoin", family: "utxo", chainId: 0, network: "mainnet", tvl: 500000000000, txVolume: 50000000, unique: true },
                { name: "Litecoin", family: "utxo", chainId: 2, network: "mainnet", tvl: 5000000000, txVolume: 2000000, unique: true },
                { name: "Dogecoin", family: "utxo", chainId: 3, network: "mainnet", tvl: 8000000000, txVolume: 5000000, unique: true },
                { name: "Dash", family: "utxo", chainId: 5, network: "mainnet", tvl: 1000000000, txVolume: 500000, unique: true },
                { name: "Bitcoin Cash", family: "utxo", chainId: 145, network: "mainnet", tvl: 3000000000, txVolume: 1000000, unique: true },
                { name: "Bitcoin SV", family: "utxo", chainId: 236, network: "mainnet", tvl: 500000000, txVolume: 200000, unique: true },
                { name: "Cardano", family: "utxo", chainId: 1010, network: "mainnet", tvl: 10000000000, txVolume: 8000000, unique: true },
                { name: "Zcash", family: "utxo", chainId: 133, network: "mainnet", tvl: 2000000000, txVolume: 800000, unique: true, privacy: true },
                { name: "Avalanche X-Chain", family: "utxo", chainId: "43114-X", network: "mainnet", tvl: 1500000000, txVolume: 600000, unique: true }
            ]
        },
        {
            name: "Account Family",
            family: "account",
            description: "Account-based model - like bank accounts with balances",
            children: [
                {
                    name: "EVM Ecosystem (2397 chains)",
                    family: "account",
                    isGroup: true,
                    evmChains: true,
                    description: "Ethereum Virtual Machine compatible chains",
                    children: [
                        {
                            name: "OP Stack (~50 chains)",
                            family: "account",
                            isGroup: true,
                            evmSubtype: "opstack",
                            description: "Optimism's modular L2 framework - shared sequencer & fault proofs",
                            children: [
                                { name: "Optimism", family: "account", chainId: 10, network: "mainnet", tvl: 7000000000, txVolume: 8000000, evm: true, opstack: true, unique: false },
                                { name: "Base", family: "account", chainId: 8453, network: "mainnet", tvl: 5000000000, txVolume: 10000000, evm: true, opstack: true, unique: false },
                                { name: "Zora", family: "account", chainId: 7777777, network: "mainnet", tvl: 50000000, txVolume: 500000, evm: true, opstack: true, unique: false },
                                { name: "Mode", family: "account", chainId: 34443, network: "mainnet", tvl: 100000000, txVolume: 800000, evm: true, opstack: true, unique: false },
                                { name: "Fraxtal", family: "account", chainId: 252, network: "mainnet", tvl: 80000000, txVolume: 400000, evm: true, opstack: true, unique: false },
                                { name: "Blast", family: "account", chainId: 81457, network: "mainnet", tvl: 600000000, txVolume: 2000000, evm: true, opstack: true, unique: false },
                                { name: "Mantle", family: "account", chainId: 5000, network: "mainnet", tvl: 1200000000, txVolume: 1800000, evm: true, opstack: true, unique: false },
                                { name: "Kroma", family: "account", chainId: 255, network: "mainnet", tvl: 30000000, txVolume: 200000, evm: true, opstack: true, unique: false },
                                { name: "Public Goods Network", family: "account", chainId: 424, network: "mainnet", tvl: 20000000, txVolume: 150000, evm: true, opstack: true, unique: false },
                                { name: "Other OP Stack (~41)", family: "account", chainId: "opstack-others", network: "mixed", tvl: 500000000, txVolume: 1500000, evm: true, opstack: true, unique: false, isAggregate: true, aggregateCount: 41 }
                            ]
                        },
                        {
                            name: "Arbitrum Ecosystem (~30 chains)",
                            family: "account",
                            isGroup: true,
                            evmSubtype: "arbitrum",
                            description: "Arbitrum's Orbit framework - custom L2/L3 deployments",
                            children: [
                                { name: "Arbitrum One", family: "account", chainId: 42161, network: "mainnet", tvl: 12000000000, txVolume: 15000000, evm: true, arbitrum: true, unique: false },
                                { name: "Arbitrum Nova", family: "account", chainId: 42170, network: "mainnet", tvl: 500000000, txVolume: 2000000, evm: true, arbitrum: true, unique: false },
                                { name: "Xai", family: "account", chainId: 660279, network: "mainnet", tvl: 50000000, txVolume: 800000, evm: true, arbitrum: true, unique: false },
                                { name: "Other Arbitrum Orbit (~27)", family: "account", chainId: "arbitrum-others", network: "mixed", tvl: 200000000, txVolume: 1000000, evm: true, arbitrum: true, unique: false, isAggregate: true, aggregateCount: 27 }
                            ]
                        },
                        {
                            name: "Polygon Ecosystem (~20 chains)",
                            family: "account",
                            isGroup: true,
                            evmSubtype: "polygon",
                            description: "Polygon's CDK - customizable zkEVM L2 framework",
                            children: [
                                { name: "Polygon PoS", family: "account", chainId: 137, network: "mainnet", tvl: 8000000000, txVolume: 20000000, evm: true, polygon: true, unique: false },
                                { name: "Polygon zkEVM", family: "account", chainId: 1101, network: "mainnet", tvl: 500000000, txVolume: 1500000, evm: true, polygon: true, zkevm: true, unique: false },
                                { name: "Immutable zkEVM", family: "account", chainId: 13371, network: "mainnet", tvl: 100000000, txVolume: 800000, evm: true, polygon: true, zkevm: true, unique: false },
                                { name: "Other Polygon CDK (~17)", family: "account", chainId: "polygon-others", network: "mixed", tvl: 300000000, txVolume: 1000000, evm: true, polygon: true, unique: false, isAggregate: true, aggregateCount: 17 }
                            ]
                        },
                        {
                            name: "zkEVM L2s (~12 chains)",
                            family: "account",
                            isGroup: true,
                            evmSubtype: "zkevm",
                            description: "Zero-knowledge proof rollups - validity proofs for scalability",
                            children: [
                                { name: "zkSync Era", family: "account", chainId: 324, network: "mainnet", tvl: 3000000000, txVolume: 4000000, evm: true, zkevm: true, unique: false },
                                { name: "Scroll", family: "account", chainId: 534352, network: "mainnet", tvl: 800000000, txVolume: 1500000, evm: true, zkevm: true, unique: false },
                                { name: "Linea", family: "account", chainId: 59144, network: "mainnet", tvl: 1000000000, txVolume: 2000000, evm: true, zkevm: true, unique: false },
                                { name: "Taiko", family: "account", chainId: 167000, network: "mainnet", tvl: 200000000, txVolume: 600000, evm: true, zkevm: true, unique: false },
                                { name: "Starknet", family: "account", chainId: "starknet", network: "mainnet", tvl: 1500000000, txVolume: 2500000, evm: false, zkevm: true, unique: true },
                                { name: "Other zkEVMs (~7)", family: "account", chainId: "zkevm-others", network: "mixed", tvl: 400000000, txVolume: 1000000, evm: true, zkevm: true, unique: false, isAggregate: true, aggregateCount: 7 }
                            ]
                        },
                        {
                            name: "L1s & Other L2s (~2285 chains)",
                            family: "account",
                            isGroup: true,
                            evmSubtype: "standard",
                            description: "Layer 1 EVM chains and independent L2s",
                            children: [
                                { name: "Ethereum Mainnet", family: "account", chainId: 1, network: "mainnet", tvl: 45000000000, txVolume: 100000000, evm: true, unique: true },
                                { name: "BNB Chain", family: "account", chainId: 56, network: "mainnet", tvl: 50000000000, txVolume: 30000000, evm: true, unique: false },
                                { name: "Avalanche C-Chain", family: "account", chainId: 43114, network: "mainnet", tvl: 6000000000, txVolume: 5000000, evm: true, unique: false },
                                { name: "Fantom", family: "account", chainId: 250, network: "mainnet", tvl: 500000000, txVolume: 800000, evm: true, unique: false },
                                { name: "Celo", family: "account", chainId: 42220, network: "mainnet", tvl: 300000000, txVolume: 500000, evm: true, unique: false },
                                { name: "Gnosis", family: "account", chainId: 100, network: "mainnet", tvl: 400000000, txVolume: 600000, evm: true, unique: false },
                                { name: "Aurora", family: "account", chainId: 1313161554, network: "mainnet", tvl: 200000000, txVolume: 300000, evm: true, unique: false },
                                { name: "Moonbeam", family: "account", chainId: 1284, network: "mainnet", tvl: 150000000, txVolume: 400000, evm: true, unique: false },
                                { name: "Evmos", family: "account", chainId: 9001, network: "mainnet", tvl: 50000000, txVolume: 200000, evm: true, cosmos: true, unique: false },
                                { name: "Other EVM chains (~2276)", family: "account", chainId: "evm-others", network: "mixed", tvl: 2000000000, txVolume: 8000000, evm: true, unique: false, isAggregate: true, aggregateCount: 2276 }
                            ]
                        }
                    ]
                },
                {
                    name: "Cosmos SDK (~100 chains)",
                    family: "account",
                    isGroup: true,
                    cosmosChains: true,
                    description: "Cosmos SDK chains - IBC interoperability, Tendermint consensus",
                    children: [
                        { name: "Cosmos Hub", family: "account", chainId: 118, network: "mainnet", tvl: 2000000000, txVolume: 3000000, cosmos: true, unique: true },
                        { name: "Osmosis", family: "account", chainId: "osmosis-1", network: "mainnet", tvl: 800000000, txVolume: 1200000, cosmos: true, unique: true },
                        { name: "Celestia", family: "account", chainId: "celestia", network: "mainnet", tvl: 500000000, txVolume: 800000, cosmos: true, unique: true },
                        { name: "Sei", family: "account", chainId: "sei-1", network: "mainnet", tvl: 300000000, txVolume: 600000, cosmos: true, unique: true },
                        { name: "Juno", family: "account", chainId: "juno-1", network: "mainnet", tvl: 100000000, txVolume: 400000, cosmos: true, unique: true },
                        { name: "Akash", family: "account", chainId: "akashnet-2", network: "mainnet", tvl: 50000000, txVolume: 200000, cosmos: true, unique: true },
                        { name: "dYdX Chain", family: "account", chainId: "dydx-mainnet-1", network: "mainnet", tvl: 400000000, txVolume: 1500000, cosmos: true, unique: true },
                        { name: "Injective", family: "account", chainId: "injective-1", network: "mainnet", tvl: 200000000, txVolume: 700000, cosmos: true, unique: true },
                        { name: "Kava", family: "account", chainId: "kava_2222-10", network: "mainnet", tvl: 150000000, txVolume: 500000, cosmos: true, evm: true, unique: true },
                        { name: "Other Cosmos Chains (~91)", family: "account", chainId: "cosmos-others", network: "mixed", tvl: 500000000, txVolume: 2000000, cosmos: true, unique: false, isAggregate: true, aggregateCount: 91 }
                    ]
                },
                {
                    name: "SVM (1 chain)",
                    family: "account",
                    isGroup: true,
                    svmChains: true,
                    description: "Solana Virtual Machine - parallel transaction processing",
                    children: [
                        { name: "Solana", family: "account", chainId: 101, network: "mainnet-beta", tvl: 5000000000, txVolume: 50000000, svm: true, unique: true }
                    ]
                },
                {
                    name: "Other Account-Based (~10 chains)",
                    family: "account",
                    isGroup: true,
                    description: "Other account model chains with unique architectures",
                    children: [
                        { name: "Aptos", family: "account", chainId: 1001, network: "mainnet", tvl: 1000000000, txVolume: 2000000, movevm: true, unique: true },
                        { name: "Sui", family: "account", chainId: 1002, network: "mainnet", tvl: 800000000, txVolume: 1500000, movevm: true, unique: true },
                        { name: "NEAR", family: "account", chainId: 1003, network: "mainnet", tvl: 500000000, txVolume: 1000000, unique: true },
                        { name: "Stellar", family: "account", chainId: 1004, network: "mainnet", tvl: 2000000000, txVolume: 3000000, unique: true },
                        { name: "XRP Ledger", family: "account", chainId: 1005, network: "mainnet", tvl: 3000000000, txVolume: 5000000, unique: true },
                        { name: "Algorand", family: "account", chainId: 1006, network: "mainnet", tvl: 1500000000, txVolume: 2000000, unique: true },
                        { name: "Tron", family: "account", chainId: 1007, network: "mainnet", tvl: 5000000000, txVolume: 8000000, unique: true },
                        { name: "Polkadot", family: "account", chainId: 1009, network: "mainnet", tvl: 4000000000, txVolume: 6000000, unique: true },
                        { name: "TON", family: "account", chainId: "ton", network: "mainnet", tvl: 300000000, txVolume: 1000000, unique: true },
                        { name: "Hedera", family: "account", chainId: "hedera", network: "mainnet", tvl: 200000000, txVolume: 800000, unique: true }
                    ]
                }
            ]
        },
        {
            name: "Instruction Family",
            family: "instruction",
            description: "Program-based model - transactions as bundles of operations",
            children: [
                { name: "Note: See SVM under Account Family", family: "instruction", chainId: "note", network: "info", tvl: 0, txVolume: 0, unique: false, isNote: true }
            ]
        },
        {
            name: "Privacy Family",
            family: "privacy",
            description: "Shielded transactions with zero-knowledge proofs",
            children: [
                { name: "Zcash (shielded)", family: "privacy", chainId: 133, network: "mainnet", tvl: 1000000000, txVolume: 500000, unique: true, privacy: true },
                { name: "Monero", family: "privacy", chainId: "monero", network: "mainnet", tvl: 2000000000, txVolume: 1000000, unique: true, privacy: true }
            ]
        }
    ]
};

// State
let currentView = CHAIN_DATA;
let currentMetric = 'chain_count';
let activeFilters = {
    utxo: true,
    account: true,
    instruction: true,
    privacy: true
};

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    setupControls();
    renderTreemap();
});

function setupControls() {
    // Metric selector
    document.getElementById('metric-select').addEventListener('change', (e) => {
        currentMetric = e.target.value;
        renderTreemap();
    });

    // Family filters
    ['utxo', 'account', 'instruction', 'privacy'].forEach(family => {
        document.getElementById(`filter-${family}`).addEventListener('change', (e) => {
            activeFilters[family] = e.target.checked;
            renderTreemap();
        });
    });

    // Breadcrumb root
    document.getElementById('breadcrumb-root').addEventListener('click', () => {
        currentView = CHAIN_DATA;
        renderTreemap();
    });
}

function getMetricValue(node, metric) {
    // Skip note entries (informational only)
    if (node.isNote) {
        return 0;
    }

    if (node.children) {
        // Aggregate children
        return node.children.reduce((sum, child) => sum + getMetricValue(child, metric), 0);
    }

    switch (metric) {
        case 'chain_count':
            return node.isAggregate ? (node.aggregateCount || 1) : 1;
        case 'tvl':
            return node.tvl || 1000000000; // Default 1B
        case 'tx_volume':
            return node.txVolume || 1000000; // Default 1M
        case 'equal':
            return node.isAggregate ? (node.aggregateCount || 1) : 1;
        default:
            return 1;
    }
}

function filterData(node) {
    if (!node.children) {
        // Leaf node - check if family is active
        return activeFilters[node.family] ? node : null;
    }

    // Parent node - filter children
    const filteredChildren = node.children
        .map(filterData)
        .filter(child => child !== null);

    if (filteredChildren.length === 0) return null;

    return {
        ...node,
        children: filteredChildren
    };
}

function renderTreemap() {
    const container = document.getElementById('treemap');
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Filter data
    const filteredData = filterData(currentView);
    if (!filteredData) {
        container.innerHTML = '<div style="color: #ecf0f1; padding: 2rem; text-align: center;">No chains match the current filters</div>';
        return;
    }

    // Calculate layout
    const layout = squarify(filteredData, width, height, currentMetric);

    // Render
    container.innerHTML = '';
    layout.forEach(rect => {
        const cell = createCell(rect);
        container.appendChild(cell);
    });

    // Update breadcrumb
    updateBreadcrumb();

    // Update stats
    updateStats(filteredData);
}

function squarify(node, width, height, metric) {
    const result = [];

    function layoutChildren(children, x, y, w, h) {
        if (children.length === 0) return;

        const totalValue = children.reduce((sum, child) => sum + getMetricValue(child, metric), 0);
        const isVertical = w > h;

        let offset = 0;
        children.forEach(child => {
            const value = getMetricValue(child, metric);
            const ratio = value / totalValue;

            let cellX, cellY, cellW, cellH;
            if (isVertical) {
                cellW = w * ratio;
                cellH = h;
                cellX = x + offset;
                cellY = y;
                offset += cellW;
            } else {
                cellW = w;
                cellH = h * ratio;
                cellX = x;
                cellY = y + offset;
                offset += cellH;
            }

            result.push({
                node: child,
                x: cellX,
                y: cellY,
                width: cellW,
                height: cellH,
                value: value
            });
        });
    }

    if (node.children) {
        layoutChildren(node.children, 0, 0, width, height);
    }

    return result;
}

function createCell(rect) {
    const { node, x, y, width, height, value } = rect;

    const cell = document.createElement('div');
    cell.className = `treemap-cell ${node.family}`;
    cell.style.left = `${x}px`;
    cell.style.top = `${y}px`;
    cell.style.width = `${width}px`;
    cell.style.height = `${height}px`;

    // Add ecosystem-specific data attribute for styling
    if (node.evmSubtype) {
        cell.setAttribute('data-ecosystem', node.evmSubtype);
    } else if (node.evmChains) {
        cell.setAttribute('data-ecosystem', 'evm');
    } else if (node.cosmosChains) {
        cell.setAttribute('data-ecosystem', 'cosmos');
    } else if (node.svmChains) {
        cell.setAttribute('data-ecosystem', 'svm');
    } else if (node.opstack) {
        cell.setAttribute('data-ecosystem', 'opstack');
    } else if (node.arbitrum) {
        cell.setAttribute('data-ecosystem', 'arbitrum');
    } else if (node.polygon) {
        cell.setAttribute('data-ecosystem', 'polygon');
    } else if (node.zkevm && !node.polygon) {
        cell.setAttribute('data-ecosystem', 'zkevm');
    }

    // Add size classes for responsive text
    if (width < 80 || height < 60) {
        cell.classList.add('tiny');
    } else if (width < 120 || height < 80) {
        cell.classList.add('small');
    }

    // Name
    const name = document.createElement('div');
    name.className = 'treemap-cell-name';
    name.textContent = node.name;
    cell.appendChild(name);

    // Value
    const valueDiv = document.createElement('div');
    valueDiv.className = 'treemap-cell-value';
    valueDiv.textContent = formatValue(value, currentMetric);
    cell.appendChild(valueDiv);

    // Click handler
    if (node.children && node.children.length > 0) {
        cell.style.cursor = 'pointer';
        cell.addEventListener('click', (e) => {
            e.stopPropagation();
            currentView = node;
            renderTreemap();
        });
    } else {
        // Leaf node - show details
        cell.addEventListener('click', (e) => {
            e.stopPropagation();
            showDetails(node);
        });
    }

    // Tooltip
    const tooltip = getTooltip(node);
    cell.title = tooltip;

    return cell;
}

function formatValue(value, metric) {
    switch (metric) {
        case 'chain_count':
            return `${value.toLocaleString()} chain${value !== 1 ? 's' : ''}`;
        case 'tvl':
            if (value >= 1e9) return `$${(value / 1e9).toFixed(1)}B`;
            if (value >= 1e6) return `$${(value / 1e6).toFixed(0)}M`;
            return `$${value.toLocaleString()}`;
        case 'tx_volume':
            if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M txs`;
            if (value >= 1e3) return `${(value / 1e3).toFixed(0)}K txs`;
            return `${value} txs`;
        case 'equal':
            return '';
        default:
            return value.toLocaleString();
    }
}

function getTooltip(node) {
    let tooltip = node.name;

    if (node.chainId !== undefined) {
        tooltip += `\nChain ID: ${node.chainId}`;
    }

    if (node.network) {
        tooltip += `\nNetwork: ${node.network}`;
    }

    if (node.evm !== undefined) {
        tooltip += `\nEVM: ${node.evm ? 'Yes' : 'No'}`;
    }

    if (node.unique !== undefined) {
        tooltip += `\nUnique: ${node.unique ? 'Yes (custom decoder)' : 'No (generic decoder)'}`;
    }

    if (node.tvl) {
        tooltip += `\nTVL: ${formatValue(node.tvl, 'tvl')}`;
    }

    if (node.description) {
        tooltip += `\n\n${node.description}`;
    }

    return tooltip;
}

function showDetails(node) {
    const panel = document.getElementById('info-panel');
    const title = document.getElementById('info-title');
    const details = document.getElementById('info-details');

    title.textContent = node.name;

    details.innerHTML = `
        <div class="detail-item">
            <div class="detail-label">Chain ID</div>
            <div class="detail-value">${node.chainId || 'N/A'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Network</div>
            <div class="detail-value">${node.network || 'N/A'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Family</div>
            <div class="detail-value">${node.family.toUpperCase()}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">EVM Compatible</div>
            <div class="detail-value">${node.evm !== undefined ? (node.evm ? 'Yes' : 'No') : 'N/A'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Decoder Type</div>
            <div class="detail-value">${node.unique ? 'Custom (unique)' : 'Generic (shared)'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">TVL (Mock)</div>
            <div class="detail-value">${node.tvl ? formatValue(node.tvl, 'tvl') : 'N/A'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Tx Volume (Mock)</div>
            <div class="detail-value">${node.txVolume ? formatValue(node.txVolume, 'tx_volume') : 'N/A'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Privacy Features</div>
            <div class="detail-value">${node.privacy ? 'Yes' : 'No'}</div>
        </div>
    `;

    panel.classList.add('active');
}

function updateBreadcrumb() {
    const breadcrumb = document.getElementById('breadcrumb');
    const root = document.getElementById('breadcrumb-root');

    if (currentView === CHAIN_DATA) {
        breadcrumb.innerHTML = '<a id="breadcrumb-root">All Chains</a>';
        document.getElementById('breadcrumb-root').addEventListener('click', () => {
            currentView = CHAIN_DATA;
            renderTreemap();
        });
    } else {
        breadcrumb.innerHTML = `
            <a id="breadcrumb-root">All Chains</a>
            <span style="color: #95a5a6;"> / </span>
            <span style="color: #ecf0f1;">${currentView.name}</span>
        `;
        document.getElementById('breadcrumb-root').addEventListener('click', () => {
            currentView = CHAIN_DATA;
            renderTreemap();
        });
    }
}

function updateStats(data) {
    // Count chains
    function countChains(node) {
        if (node.isNote) return 0; // Skip informational notes
        if (!node.children) {
            return node.isAggregate ? (node.aggregateCount || 1) : 1;
        }
        return node.children.reduce((sum, child) => sum + countChains(child), 0);
    }

    // Count families
    function countFamilies(node) {
        const families = new Set();
        function traverse(n) {
            if (n.family) families.add(n.family);
            if (n.children) n.children.forEach(traverse);
        }
        traverse(node);
        return families.size;
    }

    // Count decoders
    function countDecoders(node) {
        const unique = new Set();
        function traverse(n) {
            if (n.unique && n.name) unique.add(n.name);
            if (n.children) n.children.forEach(traverse);
        }
        traverse(node);
        // Add family decoders
        return unique.size + 1; // +1 for generic EVM decoder
    }

    // Calculate total TVL
    function totalTVL(node) {
        if (!node.children) {
            return node.tvl || 0;
        }
        return node.children.reduce((sum, child) => sum + totalTVL(child), 0);
    }

    const totalChains = countChains(data);
    const totalFamilies = countFamilies(data);
    const totalDecoders = countDecoders(data);
    const tvl = totalTVL(data);

    document.getElementById('stat-total').textContent = totalChains.toLocaleString();
    document.getElementById('stat-families').textContent = totalFamilies;
    document.getElementById('stat-decoders').textContent = totalDecoders;
    document.getElementById('stat-tvl').textContent = formatValue(tvl, 'tvl');
}

// Responsive resize
window.addEventListener('resize', () => {
    renderTreemap();
});
