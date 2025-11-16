// Chain Relationship Graph - Inspired by Echonest music graphs
// Shows blockchain chains as nodes connected by relationships

// Chain data with relationships
const NODES = [
    // UTXO Family
    { id: 'bitcoin', name: 'Bitcoin', family: 'utxo', size: 30, mainnet: true, unique: true },
    { id: 'bitcoin-cash', name: 'Bitcoin Cash', family: 'utxo', size: 18, mainnet: true, unique: true },
    { id: 'bitcoin-sv', name: 'Bitcoin SV', family: 'utxo', size: 12, mainnet: true, unique: true },
    { id: 'litecoin', name: 'Litecoin', family: 'utxo', size: 20, mainnet: true, unique: true },
    { id: 'dogecoin', name: 'Dogecoin', family: 'utxo', size: 22, mainnet: true, unique: true },
    { id: 'zcash', name: 'Zcash', family: 'privacy', size: 16, mainnet: true, unique: true },
    { id: 'monero', name: 'Monero', family: 'privacy', size: 18, mainnet: true, unique: true },
    { id: 'dash', name: 'Dash', family: 'utxo', size: 14, mainnet: true, unique: true },
    { id: 'bitcoin-testnet', name: 'Bitcoin Testnet', family: 'utxo', size: 8, mainnet: false, unique: false },

    // Account Family - Ethereum and major L1s
    { id: 'ethereum', name: 'Ethereum', family: 'account', size: 35, mainnet: true, unique: true, evm: true },
    { id: 'ethereum-classic', name: 'Ethereum Classic', family: 'account', size: 12, mainnet: true, unique: true, evm: true },
    { id: 'bsc', name: 'BNB Smart Chain', family: 'account', size: 28, mainnet: true, unique: false, evm: true },
    { id: 'polygon', name: 'Polygon', family: 'account', size: 26, mainnet: true, unique: false, evm: true },
    { id: 'avalanche', name: 'Avalanche C-Chain', family: 'account', size: 24, mainnet: true, unique: false, evm: true },
    { id: 'fantom', name: 'Fantom', family: 'account', size: 18, mainnet: true, unique: false, evm: true },
    { id: 'arbitrum', name: 'Arbitrum One', family: 'account', size: 25, mainnet: true, unique: false, evm: true, opstack: false },
    { id: 'optimism', name: 'Optimism', family: 'account', size: 24, mainnet: true, unique: false, evm: true, opstack: true },
    { id: 'base', name: 'Base', family: 'account', size: 22, mainnet: true, unique: false, evm: true, opstack: true },
    { id: 'zora', name: 'Zora', family: 'account', size: 15, mainnet: true, unique: false, evm: true, opstack: true },
    { id: 'mode', name: 'Mode', family: 'account', size: 12, mainnet: true, unique: false, evm: true, opstack: true },

    // Non-EVM Account chains
    { id: 'aptos', name: 'Aptos', family: 'account', size: 20, mainnet: true, unique: true, evm: false },
    { id: 'sui', name: 'Sui', family: 'account', size: 19, mainnet: true, unique: true, evm: false },
    { id: 'near', name: 'NEAR', family: 'account', size: 18, mainnet: true, unique: true, evm: false },
    { id: 'tron', name: 'Tron', family: 'account', size: 20, mainnet: true, unique: true, evm: false },

    // Instruction Family
    { id: 'solana', name: 'Solana', family: 'instruction', size: 32, mainnet: true, unique: true },
    { id: 'solana-testnet', name: 'Solana Testnet', family: 'instruction', size: 10, mainnet: false, unique: false },

    // Cosmos Ecosystem
    { id: 'cosmos', name: 'Cosmos Hub', family: 'account', size: 22, mainnet: true, unique: true, cosmos: true },
    { id: 'osmosis', name: 'Osmosis', family: 'account', size: 18, mainnet: true, unique: false, cosmos: true },
    { id: 'juno', name: 'Juno', family: 'account', size: 14, mainnet: true, unique: false, cosmos: true },
    { id: 'akash', name: 'Akash', family: 'account', size: 12, mainnet: true, unique: false, cosmos: true },
    { id: 'celestia', name: 'Celestia', family: 'account', size: 16, mainnet: true, unique: false, cosmos: true },
    { id: 'injective', name: 'Injective', family: 'account', size: 15, mainnet: true, unique: false, cosmos: true },

    // Additional EVM chains (showing diversity)
    { id: 'cronos', name: 'Cronos', family: 'account', size: 16, mainnet: true, unique: false, evm: true },
    { id: 'moonbeam', name: 'Moonbeam', family: 'account', size: 14, mainnet: true, unique: false, evm: true },
    { id: 'celo', name: 'Celo', family: 'account', size: 15, mainnet: true, unique: false, evm: true },
    { id: 'aurora', name: 'Aurora', family: 'account', size: 13, mainnet: true, unique: false, evm: true },
    { id: 'gnosis', name: 'Gnosis Chain', family: 'account', size: 14, mainnet: true, unique: false, evm: true },
    { id: 'evmos', name: 'Evmos', family: 'account', size: 13, mainnet: true, unique: false, evm: true, cosmos: true },

    // Testnets
    { id: 'sepolia', name: 'Sepolia', family: 'account', size: 10, mainnet: false, unique: false, evm: true },
    { id: 'goerli', name: 'Goerli', family: 'account', size: 9, mainnet: false, unique: false, evm: true },
    { id: 'mumbai', name: 'Mumbai', family: 'account', size: 8, mainnet: false, unique: false, evm: true },

    // Other notable chains
    { id: 'polkadot', name: 'Polkadot', family: 'account', size: 20, mainnet: true, unique: true },
    { id: 'kusama', name: 'Kusama', family: 'account', size: 15, mainnet: true, unique: true },
    { id: 'cardano', name: 'Cardano', family: 'utxo', size: 24, mainnet: true, unique: true },
    { id: 'algorand', name: 'Algorand', family: 'account', size: 16, mainnet: true, unique: true },
    { id: 'hedera', name: 'Hedera', family: 'account', size: 14, mainnet: true, unique: true },
    { id: 'icp', name: 'Internet Computer', family: 'account', size: 17, mainnet: true, unique: true },
    { id: 'stellar', name: 'Stellar', family: 'account', size: 15, mainnet: true, unique: true },
    { id: 'xrp', name: 'XRP Ledger', family: 'account', size: 22, mainnet: true, unique: true },
    { id: 'ton', name: 'TON', family: 'account', size: 18, mainnet: true, unique: true },
];

// Edges represent relationships
const EDGES = [
    // Fork relationships (red, strong)
    { source: 'bitcoin', target: 'bitcoin-cash', type: 'fork', strength: 0.3 },
    { source: 'bitcoin-cash', target: 'bitcoin-sv', type: 'fork', strength: 0.3 },
    { source: 'bitcoin', target: 'litecoin', type: 'fork', strength: 0.25 },
    { source: 'bitcoin', target: 'dogecoin', type: 'fork', strength: 0.2 },
    { source: 'bitcoin', target: 'dash', type: 'fork', strength: 0.2 },
    { source: 'bitcoin', target: 'bitcoin-testnet', type: 'fork', strength: 0.15 },
    { source: 'ethereum', target: 'ethereum-classic', type: 'fork', strength: 0.3 },
    { source: 'ethereum', target: 'sepolia', type: 'fork', strength: 0.15 },
    { source: 'ethereum', target: 'goerli', type: 'fork', strength: 0.15 },
    { source: 'polygon', target: 'mumbai', type: 'fork', strength: 0.15 },
    { source: 'polkadot', target: 'kusama', type: 'fork', strength: 0.25 },
    { source: 'solana', target: 'solana-testnet', type: 'fork', strength: 0.15 },

    // Shared decoder relationships (blue, medium) - EVM chains share decoder
    { source: 'ethereum', target: 'bsc', type: 'decoder', strength: 0.2 },
    { source: 'ethereum', target: 'polygon', type: 'decoder', strength: 0.2 },
    { source: 'ethereum', target: 'avalanche', type: 'decoder', strength: 0.2 },
    { source: 'ethereum', target: 'fantom', type: 'decoder', strength: 0.18 },
    { source: 'ethereum', target: 'arbitrum', type: 'decoder', strength: 0.2 },
    { source: 'ethereum', target: 'optimism', type: 'decoder', strength: 0.2 },
    { source: 'ethereum', target: 'base', type: 'decoder', strength: 0.19 },
    { source: 'ethereum', target: 'cronos', type: 'decoder', strength: 0.17 },
    { source: 'ethereum', target: 'moonbeam', type: 'decoder', strength: 0.17 },
    { source: 'ethereum', target: 'celo', type: 'decoder', strength: 0.17 },
    { source: 'ethereum', target: 'aurora', type: 'decoder', strength: 0.16 },
    { source: 'ethereum', target: 'gnosis', type: 'decoder', strength: 0.17 },
    { source: 'ethereum', target: 'zora', type: 'decoder', strength: 0.16 },
    { source: 'ethereum', target: 'mode', type: 'decoder', strength: 0.15 },

    // OP Stack technology sharing (purple dashed, weak)
    { source: 'optimism', target: 'base', type: 'tech-sharing', strength: 0.25 },
    { source: 'optimism', target: 'zora', type: 'tech-sharing', strength: 0.2 },
    { source: 'optimism', target: 'mode', type: 'tech-sharing', strength: 0.2 },
    { source: 'base', target: 'zora', type: 'tech-sharing', strength: 0.15 },
    { source: 'base', target: 'mode', type: 'tech-sharing', strength: 0.15 },

    // Cosmos SDK technology sharing
    { source: 'cosmos', target: 'osmosis', type: 'tech-sharing', strength: 0.25 },
    { source: 'cosmos', target: 'juno', type: 'tech-sharing', strength: 0.2 },
    { source: 'cosmos', target: 'akash', type: 'tech-sharing', strength: 0.2 },
    { source: 'cosmos', target: 'celestia', type: 'tech-sharing', strength: 0.22 },
    { source: 'cosmos', target: 'injective', type: 'tech-sharing', strength: 0.21 },
    { source: 'cosmos', target: 'evmos', type: 'tech-sharing', strength: 0.23 },
    { source: 'osmosis', target: 'juno', type: 'tech-sharing', strength: 0.15 },
    { source: 'osmosis', target: 'celestia', type: 'tech-sharing', strength: 0.15 },

    // Cross-ecosystem bridges/connections
    { source: 'near', target: 'aurora', type: 'tech-sharing', strength: 0.2 },
    { source: 'polkadot', target: 'moonbeam', type: 'tech-sharing', strength: 0.2 },
];

// Graph state
let nodes = [];
let edges = [];
let canvas, ctx;
let width, height;
let transform = { x: 0, y: 0, scale: 1 };
let selectedNode = null;
let hoveredNode = null;
let isDragging = false;
let dragStart = { x: 0, y: 0 };
let isSimulationRunning = true;
let layoutMode = 'force';

// Filter state
let filters = {
    families: { utxo: true, account: true, instruction: true, privacy: true },
    edges: { fork: true, decoder: true, 'tech-sharing': true }
};

// Force simulation parameters
const FORCE_PARAMS = {
    repulsion: 3000,
    attraction: 0.05,
    centerGravity: 0.001,
    damping: 0.85,
    minDistance: 50,
    maxForce: 10
};

// Initialize
function init() {
    canvas = document.getElementById('graph-canvas');
    ctx = canvas.getContext('2d');

    // Set canvas size
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    // Initialize nodes with random positions
    initializeNodes();

    // Set up event listeners
    setupEventListeners();

    // Start animation loop
    animate();
}

function resizeCanvas() {
    const container = canvas.parentElement;
    const rect = container.getBoundingClientRect();

    canvas.width = rect.width - 4; // Account for padding
    canvas.height = 700;

    width = canvas.width;
    height = canvas.height;

    // Center transform on resize
    if (nodes.length > 0 && transform.x === 0 && transform.y === 0) {
        transform.x = width / 2;
        transform.y = height / 2;
    }
}

function initializeNodes() {
    // Deep copy nodes and add physics properties
    nodes = NODES.map(n => ({
        ...n,
        x: width / 2 + (Math.random() - 0.5) * 200,
        y: height / 2 + (Math.random() - 0.5) * 200,
        vx: 0,
        vy: 0,
        fx: null, // Fixed position
        fy: null
    }));

    // Deep copy edges and resolve node references
    edges = EDGES.map(e => ({
        ...e,
        sourceNode: nodes.find(n => n.id === e.source),
        targetNode: nodes.find(n => n.id === e.target)
    })).filter(e => e.sourceNode && e.targetNode);

    updateStats();
}

function setupEventListeners() {
    // Mouse events
    canvas.addEventListener('mousemove', onMouseMove);
    canvas.addEventListener('mousedown', onMouseDown);
    canvas.addEventListener('mouseup', onMouseUp);
    canvas.addEventListener('mouseleave', onMouseLeave);
    canvas.addEventListener('wheel', onWheel);

    // Controls
    document.getElementById('reset-btn').addEventListener('click', resetView);
    document.getElementById('pause-btn').addEventListener('click', togglePause);
    document.getElementById('layout-select').addEventListener('change', onLayoutChange);

    // Edge filters
    document.getElementById('edge-forks').addEventListener('change', e => {
        filters.edges.fork = e.target.checked;
        updateStats();
    });
    document.getElementById('edge-decoder').addEventListener('change', e => {
        filters.edges.decoder = e.target.checked;
        updateStats();
    });
    document.getElementById('edge-tech').addEventListener('change', e => {
        filters.edges['tech-sharing'] = e.target.checked;
        updateStats();
    });

    // Family filters
    document.getElementById('filter-utxo').addEventListener('change', e => {
        filters.families.utxo = e.target.checked;
        updateStats();
    });
    document.getElementById('filter-account').addEventListener('change', e => {
        filters.families.account = e.target.checked;
        updateStats();
    });
    document.getElementById('filter-instruction').addEventListener('change', e => {
        filters.families.instruction = e.target.checked;
        updateStats();
    });
    document.getElementById('filter-privacy').addEventListener('change', e => {
        filters.families.privacy = e.target.checked;
        updateStats();
    });
}

function getVisibleNodes() {
    return nodes.filter(n => filters.families[n.family]);
}

function getVisibleEdges() {
    const visibleNodes = getVisibleNodes();
    const visibleIds = new Set(visibleNodes.map(n => n.id));

    return edges.filter(e =>
        filters.edges[e.type] &&
        visibleIds.has(e.source) &&
        visibleIds.has(e.target)
    );
}

function updateStats() {
    const visibleNodes = getVisibleNodes();
    const visibleEdges = getVisibleEdges();

    document.getElementById('stat-nodes').textContent = visibleNodes.length;
    document.getElementById('stat-edges').textContent = visibleEdges.length;

    // Count clusters (simplified: groups of connected nodes)
    const clusters = countClusters(visibleNodes, visibleEdges);
    document.getElementById('stat-clusters').textContent = clusters.numClusters;
    document.getElementById('stat-isolated').textContent = clusters.isolated;
}

function countClusters(nodes, edges) {
    const visited = new Set();
    let numClusters = 0;
    let isolated = 0;

    // Build adjacency list
    const adj = new Map();
    nodes.forEach(n => adj.set(n.id, new Set()));
    edges.forEach(e => {
        adj.get(e.source).add(e.target);
        adj.get(e.target).add(e.source);
    });

    // DFS to find connected components
    function dfs(nodeId) {
        if (visited.has(nodeId)) return 0;
        visited.add(nodeId);

        const neighbors = adj.get(nodeId);
        let size = 1;
        neighbors.forEach(neighbor => {
            size += dfs(neighbor);
        });
        return size;
    }

    nodes.forEach(n => {
        if (!visited.has(n.id)) {
            const clusterSize = dfs(n.id);
            numClusters++;
            if (clusterSize === 1) isolated++;
        }
    });

    return { numClusters, isolated };
}

// Force-directed layout algorithm
function applyForces() {
    if (!isSimulationRunning || layoutMode !== 'force') return;

    const visibleNodes = getVisibleNodes();
    const visibleEdges = getVisibleEdges();

    // Reset forces
    visibleNodes.forEach(node => {
        if (node.fx === null) {
            node.vx *= FORCE_PARAMS.damping;
            node.vy *= FORCE_PARAMS.damping;
        }
    });

    // Repulsion between all nodes
    for (let i = 0; i < visibleNodes.length; i++) {
        for (let j = i + 1; j < visibleNodes.length; j++) {
            const a = visibleNodes[i];
            const b = visibleNodes[j];

            if (a.fx !== null && b.fx !== null) continue;

            const dx = b.x - a.x;
            const dy = b.y - a.y;
            const dist = Math.sqrt(dx * dx + dy * dy) || 1;

            if (dist < FORCE_PARAMS.minDistance * 3) {
                const force = FORCE_PARAMS.repulsion / (dist * dist);
                const fx = (dx / dist) * Math.min(force, FORCE_PARAMS.maxForce);
                const fy = (dy / dist) * Math.min(force, FORCE_PARAMS.maxForce);

                if (a.fx === null) {
                    a.vx -= fx;
                    a.vy -= fy;
                }
                if (b.fx === null) {
                    b.vx += fx;
                    b.vy += fy;
                }
            }
        }
    }

    // Attraction along edges
    visibleEdges.forEach(edge => {
        const source = edge.sourceNode;
        const target = edge.targetNode;

        if (!source || !target) return;
        if (source.fx !== null && target.fx !== null) return;

        const dx = target.x - source.x;
        const dy = target.y - source.y;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;

        const force = (dist - 100) * FORCE_PARAMS.attraction * edge.strength;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        if (source.fx === null) {
            source.vx += fx;
            source.vy += fy;
        }
        if (target.fx === null) {
            target.vx -= fx;
            target.vy -= fy;
        }
    });

    // Center gravity
    visibleNodes.forEach(node => {
        if (node.fx === null) {
            node.vx += (width / 2 - node.x) * FORCE_PARAMS.centerGravity;
            node.vy += (height / 2 - node.y) * FORCE_PARAMS.centerGravity;
        }
    });

    // Update positions
    visibleNodes.forEach(node => {
        if (node.fx === null) {
            node.x += node.vx;
            node.y += node.vy;
        } else {
            node.x = node.fx;
            node.y = node.fy;
        }
    });
}

// Rendering
function render() {
    ctx.clearRect(0, 0, width, height);

    ctx.save();
    ctx.translate(transform.x, transform.y);
    ctx.scale(transform.scale, transform.scale);
    ctx.translate(-width / 2, -height / 2);

    const visibleNodes = getVisibleNodes();
    const visibleEdges = getVisibleEdges();

    // Draw edges
    visibleEdges.forEach(edge => {
        drawEdge(edge);
    });

    // Draw nodes
    visibleNodes.forEach(node => {
        drawNode(node);
    });

    // Highlight selected node connections
    if (selectedNode) {
        highlightConnections(selectedNode);
    }

    ctx.restore();
}

function drawEdge(edge) {
    const source = edge.sourceNode;
    const target = edge.targetNode;

    if (!source || !target) return;

    ctx.beginPath();
    ctx.moveTo(source.x, source.y);
    ctx.lineTo(target.x, target.y);

    // Style based on type
    if (edge.type === 'fork') {
        ctx.strokeStyle = 'rgba(231, 76, 60, 0.6)';
        ctx.lineWidth = 2;
    } else if (edge.type === 'decoder') {
        ctx.strokeStyle = 'rgba(52, 152, 219, 0.4)';
        ctx.lineWidth = 1.5;
    } else if (edge.type === 'tech-sharing') {
        ctx.strokeStyle = 'rgba(155, 89, 182, 0.4)';
        ctx.lineWidth = 1;
        ctx.setLineDash([5, 5]);
    }

    ctx.stroke();
    ctx.setLineDash([]);
}

function drawNode(node) {
    const isSelected = selectedNode && selectedNode.id === node.id;
    const isHovered = hoveredNode && hoveredNode.id === node.id;

    // Node color by family
    const colors = {
        utxo: '#3498db',
        account: '#2ecc71',
        instruction: '#f39c12',
        privacy: '#9b59b6'
    };

    const color = colors[node.family] || '#95a5a6';

    // Draw node circle
    ctx.beginPath();
    ctx.arc(node.x, node.y, node.size, 0, 2 * Math.PI);

    if (isSelected || isHovered) {
        ctx.fillStyle = color;
        ctx.shadowColor = color;
        ctx.shadowBlur = 20;
    } else {
        ctx.fillStyle = color;
    }

    ctx.fill();
    ctx.shadowBlur = 0;

    // Border
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.lineWidth = isSelected ? 3 : (isHovered ? 2 : 1);
    ctx.stroke();

    // Draw label for larger nodes or selected/hovered
    if (node.size > 20 || isSelected || isHovered) {
        ctx.fillStyle = '#ecf0f1';
        ctx.font = `${isSelected || isHovered ? 'bold' : 'normal'} ${Math.max(10, node.size / 2)}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';

        // Text shadow for readability
        ctx.shadowColor = 'rgba(0, 0, 0, 0.8)';
        ctx.shadowBlur = 4;

        const label = node.name.length > 15 ? node.name.substring(0, 12) + '...' : node.name;
        ctx.fillText(label, node.x, node.y + node.size + 12);

        ctx.shadowBlur = 0;
    }
}

function highlightConnections(node) {
    const connectedEdges = edges.filter(e =>
        (e.source === node.id || e.target === node.id) && filters.edges[e.type]
    );

    connectedEdges.forEach(edge => {
        const source = edge.sourceNode;
        const target = edge.targetNode;

        if (!source || !target) return;

        ctx.beginPath();
        ctx.moveTo(source.x, source.y);
        ctx.lineTo(target.x, target.y);
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
        ctx.lineWidth = 3;
        ctx.stroke();
    });
}

// Animation loop
function animate() {
    applyForces();
    render();
    requestAnimationFrame(animate);
}

// Mouse interaction
function getMousePos(e) {
    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Transform to graph coordinates
    const graphX = (mouseX - transform.x) / transform.scale + width / 2;
    const graphY = (mouseY - transform.y) / transform.scale + height / 2;

    return { x: graphX, y: graphY, screenX: mouseX, screenY: mouseY };
}

function findNodeAt(x, y) {
    const visibleNodes = getVisibleNodes();

    for (let i = visibleNodes.length - 1; i >= 0; i--) {
        const node = visibleNodes[i];
        const dx = x - node.x;
        const dy = y - node.y;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist < node.size) {
            return node;
        }
    }

    return null;
}

function onMouseMove(e) {
    const pos = getMousePos(e);
    const node = findNodeAt(pos.x, pos.y);

    if (isDragging && selectedNode) {
        selectedNode.fx = pos.x;
        selectedNode.fy = pos.y;
        canvas.style.cursor = 'grabbing';
    } else if (node) {
        hoveredNode = node;
        canvas.style.cursor = 'pointer';
        showTooltip(node, e.clientX, e.clientY);
    } else {
        hoveredNode = null;
        canvas.style.cursor = isDragging ? 'grabbing' : 'move';
        hideTooltip();
    }
}

function onMouseDown(e) {
    const pos = getMousePos(e);
    const node = findNodeAt(pos.x, pos.y);

    if (node) {
        isDragging = true;
        selectedNode = node;
        node.fx = node.x;
        node.fy = node.y;
        canvas.style.cursor = 'grabbing';
        showNodeInfo(node);
    } else {
        isDragging = true;
        dragStart = { x: e.clientX - transform.x, y: e.clientY - transform.y };
    }
}

function onMouseUp(e) {
    if (selectedNode && isDragging) {
        selectedNode.fx = null;
        selectedNode.fy = null;
    }
    isDragging = false;
    canvas.style.cursor = hoveredNode ? 'pointer' : 'move';
}

function onMouseLeave(e) {
    isDragging = false;
    hoveredNode = null;
    hideTooltip();
    canvas.style.cursor = 'move';
}

function onWheel(e) {
    e.preventDefault();

    const delta = -e.deltaY * 0.001;
    const oldScale = transform.scale;
    transform.scale = Math.max(0.5, Math.min(3, transform.scale * (1 + delta)));

    // Zoom toward mouse position
    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    transform.x = mouseX - (mouseX - transform.x) * (transform.scale / oldScale);
    transform.y = mouseY - (mouseY - transform.y) * (transform.scale / oldScale);
}

// Tooltip
function showTooltip(node, x, y) {
    const tooltip = document.getElementById('tooltip');
    tooltip.classList.add('active');

    const info = [];
    info.push(`<div class="chain-name">${node.name}</div>`);
    info.push(`<div class="chain-info">`);
    info.push(`Family: ${node.family.toUpperCase()}`);
    if (node.evm) info.push(`EVM Compatible`);
    if (node.opstack) info.push(`OP Stack`);
    if (node.cosmos) info.push(`Cosmos SDK`);
    info.push(`${node.mainnet ? 'Mainnet' : 'Testnet'}`);
    info.push(`${node.unique ? 'Unique Decoder' : 'Generic Decoder'}`);
    info.push(`</div>`);

    tooltip.innerHTML = info.join('<br>');
    tooltip.style.left = (x + 15) + 'px';
    tooltip.style.top = (y + 15) + 'px';
}

function hideTooltip() {
    const tooltip = document.getElementById('tooltip');
    tooltip.classList.remove('active');
}

// Info panel
function showNodeInfo(node) {
    const panel = document.getElementById('info-panel');
    const title = document.getElementById('info-title');
    const details = document.getElementById('info-details');

    title.textContent = node.name;

    const connections = edges.filter(e => e.source === node.id || e.target === node.id);
    const forks = connections.filter(e => e.type === 'fork').length;
    const decoderLinks = connections.filter(e => e.type === 'decoder').length;
    const techLinks = connections.filter(e => e.type === 'tech-sharing').length;

    details.innerHTML = `
        <div class="detail-item">
            <div class="detail-label">Chain Family</div>
            <div class="detail-value">${node.family.toUpperCase()}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Network Type</div>
            <div class="detail-value">${node.mainnet ? 'Mainnet' : 'Testnet'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Decoder Type</div>
            <div class="detail-value">${node.unique ? 'Unique' : 'Generic'}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Total Connections</div>
            <div class="detail-value">${connections.length}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Fork Relationships</div>
            <div class="detail-value">${forks}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Shared Decoder</div>
            <div class="detail-value">${decoderLinks}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">Tech Sharing</div>
            <div class="detail-value">${techLinks}</div>
        </div>
        <div class="detail-item">
            <div class="detail-label">EVM Compatible</div>
            <div class="detail-value">${node.evm ? 'Yes' : 'No'}</div>
        </div>
    `;

    panel.classList.add('active');
}

// Controls
function resetView() {
    transform = { x: width / 2, y: height / 2, scale: 1 };
    selectedNode = null;
    document.getElementById('info-panel').classList.remove('active');
}

function togglePause() {
    isSimulationRunning = !isSimulationRunning;
    const btn = document.getElementById('pause-btn');
    btn.textContent = isSimulationRunning ? 'Pause' : 'Resume';
}

function onLayoutChange(e) {
    layoutMode = e.target.value;

    if (layoutMode === 'circular') {
        applyCircularLayout();
    } else if (layoutMode === 'hierarchical') {
        applyHierarchicalLayout();
    }
    // 'force' will be handled by the animation loop
}

function applyCircularLayout() {
    const visibleNodes = getVisibleNodes();
    const radius = Math.min(width, height) / 3;
    const angleStep = (2 * Math.PI) / visibleNodes.length;

    visibleNodes.forEach((node, i) => {
        const angle = i * angleStep;
        node.x = width / 2 + radius * Math.cos(angle);
        node.y = height / 2 + radius * Math.sin(angle);
        node.vx = 0;
        node.vy = 0;
    });
}

function applyHierarchicalLayout() {
    const visibleNodes = getVisibleNodes();

    // Group by family
    const families = { utxo: [], account: [], instruction: [], privacy: [] };
    visibleNodes.forEach(node => families[node.family].push(node));

    const familyOrder = ['utxo', 'account', 'instruction', 'privacy'];
    const levels = familyOrder.filter(f => families[f].length > 0);
    const levelHeight = height / (levels.length + 1);

    levels.forEach((family, levelIndex) => {
        const nodesInLevel = families[family];
        const levelWidth = width / (nodesInLevel.length + 1);

        nodesInLevel.forEach((node, i) => {
            node.x = levelWidth * (i + 1);
            node.y = levelHeight * (levelIndex + 1);
            node.vx = 0;
            node.vy = 0;
        });
    });
}

// Start
init();
