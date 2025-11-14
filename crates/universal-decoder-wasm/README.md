# Universal Blockchain Decoder - WASM Demo

<div align="center">

🚀 **Interactive browser-based transaction decoder**

[![Try it live!](https://img.shields.io/badge/Try-Live%20Demo-blue?style=for-the-badge)](https://prasincs.github.io/universal-blockchain-decoder/)

</div>

---

## Overview

This is a **thin WASM wrapper** around the Universal Blockchain Decoder that runs entirely in your browser. It reuses all the existing decoder implementations (Bitcoin, Ethereum, Solana, Cosmos) with zero modifications.

### Key Features

- ✅ **Zero-trust**: All decoding happens locally in your browser
- ✅ **Offline-capable**: Works without internet after initial load
- ✅ **Privacy-preserving**: No transaction data sent to any server
- ✅ **Multi-chain**: Bitcoin, Ethereum, Solana, Cosmos support
- ✅ **Reuses core**: Leverages existing decoder infrastructure

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Browser (Your Machine)                                      │
├─────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────┐  │
│  │  HTML + CSS + JavaScript (www/)                       │  │
│  │  └─ CodeMirror editors + UI logic                     │  │
│  └───────────────────────────────────────────────────────┘  │
│                             ↕                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  WASM Module (thin wrapper)                           │  │
│  │  ├─ wasm-bindgen (JS ↔ Rust bridge)                   │  │
│  │  └─ Reuses existing decoders:                         │  │
│  │     • decoder-bitcoin                                 │  │
│  │     • decoder-ethereum                                │  │
│  │     • decoder-solana                                  │  │
│  │     • decoder-cosmos                                  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
          No network calls • Zero server infrastructure
```

## Building Locally

### Prerequisites

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Optional: Install wasm-opt for smaller bundles
sudo apt-get install binaryen  # Ubuntu/Debian
brew install binaryen           # macOS
```

### Build

```bash
cd crates/universal-decoder-wasm

# Build WASM module (optimized for size)
./build.sh

# The output will be in www/pkg/
```

### Test Locally

```bash
cd www

# Start local server
python3 -m http.server 8080

# Open http://localhost:8080 in your browser
```

## Bundle Size

Current bundle sizes (with optimization):

| Build Type | Size (gzipped) | Chains |
|-----------|----------------|--------|
| Minimal   | ~400KB         | Bitcoin, Ethereum |
| Full      | ~800KB         | Bitcoin, Ethereum, Solana, Cosmos |

**Target**: < 500KB for minimal, < 2MB for full

## API

The WASM module exposes a simple JavaScript API:

```javascript
import init, {
    decode_transaction,
    supported_chains,
    auto_detect_chain
} from './pkg/universal_decoder_wasm.js';

// Initialize (call once)
await init();

// Get supported chains
const chains = supported_chains();
// => ["bitcoin", "ethereum", "solana", "cosmos"]

// Decode a transaction
const result = decode_transaction("bitcoin", "0100000001...");
console.log(result.canonical_hex);
console.log(result.json);
console.log(result.privacy_score);

// Auto-detect chain
const chain = auto_detect_chain("0100000001...");
// => "bitcoin"
```

## Embedding

You can embed the demo in your blog posts, papers, or documentation:

```html
<iframe
    src="https://prasincs.github.io/universal-blockchain-decoder/"
    width="100%"
    height="800px"
    frameborder="0"
    allow="clipboard-read; clipboard-write">
</iframe>
```

## Use Cases

### 📝 Blog Posts
Embed interactive examples directly in articles about blockchain transaction formats.

### 🎓 Education
Students can decode real transactions and understand different blockchain models (UTXO vs Account vs Instruction).

### 🔬 Research
Analyze privacy features across chains without sending data to third-party services.

### 🎤 Conferences
Live demos that work offline (no WiFi dependency after initial load).

### 📄 Papers
Interactive figures in HTML versions of academic papers.

## Development

### Project Structure

```
crates/universal-decoder-wasm/
├── Cargo.toml              # WASM dependencies
├── build.sh                # Build script
├── src/
│   └── lib.rs              # Thin WASM wrapper (reuses decoders!)
├── www/                    # Web UI
│   ├── index.html          # Main page
│   ├── style.css           # Styling
│   ├── main.js             # UI logic + WASM loader
│   ├── examples.js         # Pre-loaded transaction examples
│   └── pkg/                # Built WASM output (gitignored)
└── tests/
    └── web.rs              # Browser-based integration tests
```

### Adding a New Chain

To add support for a new chain:

1. **Implement decoder** in main project (e.g., `crates/decoder-dogecoin`)
2. **Add dependency** to `Cargo.toml`:
   ```toml
   decoder-dogecoin = { path = "../decoder-dogecoin" }
   ```
3. **Add case** in `src/lib.rs`:
   ```rust
   "dogecoin" => decode_dogecoin_transaction(&bytes),
   ```
4. **Add to supported_chains()** function
5. **Rebuild WASM**: `./build.sh`

The decoder will automatically be deployed via CI/CD!

## Testing

```bash
# Run WASM tests in headless browser
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Run in actual browser (for debugging)
wasm-pack test --chrome
```

## Deployment

The demo is automatically deployed to GitHub Pages on every push to `main` that affects:
- `crates/universal-decoder-wasm/**`
- `crates/universal-decoder-core/**`
- `crates/decoder-*/**`

See `.github/workflows/deploy-wasm-demo.yml` for details.

## Performance

### Load Time
- **First load**: ~2-3 seconds (download WASM + initialization)
- **Cached**: ~100ms (WASM loaded from browser cache)

### Decode Time
- **Bitcoin**: ~10-50ms
- **Ethereum**: ~5-30ms
- **Solana**: ~20-80ms
- **Cosmos**: ~30-100ms

All measurements on average hardware (2020 MacBook Pro).

## Browser Compatibility

| Browser | Version | Status |
|---------|---------|--------|
| Chrome  | 89+     | ✅ Full support |
| Firefox | 89+     | ✅ Full support |
| Safari  | 15+     | ✅ Full support |
| Edge    | 89+     | ✅ Full support |
| Mobile  | iOS 15+, Android 89+ | ✅ Responsive |

## Security

### What Happens to Your Transaction Data?

**Nothing.** All decoding happens in your browser using WebAssembly. Zero network calls are made during operation (except for the initial WASM module download).

### Can I Verify This?

Yes! Open browser DevTools → Network tab. After the initial page load, you'll see **zero network requests** when decoding transactions.

### Formal Verification

The WASM module uses the same core library that will be formally verified with Verus (Phase 4). All security properties of the core library apply to the WASM build.

## FAQ

**Q: Why WASM instead of a server-side API?**
A: Privacy, cost, reliability, and trust. WASM runs locally, costs $0 to host (GitHub Pages), has zero downtime, and is open-source verifiable.

**Q: Does it work offline?**
A: Yes! After the initial load, the WASM module is cached by your browser. You can disconnect from the internet and continue decoding transactions.

**Q: How big is the download?**
A: The full bundle (all 4 chains) is ~800KB gzipped, similar to a medium-sized image.

**Q: Can I use this in production?**
A: The WASM demo is primarily for education and demonstrations. For production use cases, use the native Rust library for better performance and more features.

**Q: How do I report bugs?**
A: Open an issue at https://github.com/prasincs/universal-blockchain-decoder/issues

## Contributing

We welcome contributions! To improve the WASM demo:

1. Improve UI/UX
2. Add more example transactions
3. Optimize bundle size
4. Add visualization features
5. Improve mobile responsiveness

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0

## References

- **Main Project**: https://github.com/prasincs/universal-blockchain-decoder
- **Documentation**: https://github.com/prasincs/universal-blockchain-decoder/tree/main/docs
- **wasm-bindgen**: https://rustwasm.github.io/wasm-bindgen/
- **wasm-pack**: https://rustwasm.github.io/wasm-pack/

---

**Built with ❤️ using Rust, WebAssembly, and the existing Universal Blockchain Decoder infrastructure.**
