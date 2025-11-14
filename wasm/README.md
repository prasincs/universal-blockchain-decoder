# Universal Blockchain Decoder - WASM Demo

A fully functional, browser-based blockchain transaction decoder built with WebAssembly.

## 🚀 Quick Start

### Local Testing

```bash
cd /home/user/universal-blockchain-decoder/wasm
python3 -m http.server 8080
```

Open browser to: **http://localhost:8080**

### Supported Chains

- ✅ **Bitcoin** - Legacy, SegWit
- ✅ **Ethereum** - Legacy, EIP-1559
- ✅ **Solana** - Token transfers
- ✅ **Cosmos** - Bank sends, IBC

## 📦 What's Inside

```
wasm/
├── index.html       # Main web interface
├── main.js          # Application logic
├── examples.js      # Pre-loaded examples
├── style.css        # Responsive dark theme
├── pkg/             # Compiled WASM module
│   ├── universal_decoder_wasm_bg.wasm  (358KB)
│   └── universal_decoder_wasm.js
├── DEPLOY.md        # Deployment instructions
└── README.md        # This file
```

## 🌐 Deployment

This directory is **ready to deploy** to any static hosting service:

- **GitHub Pages**: See [DEPLOY.md](./DEPLOY.md) for step-by-step guide
- **Netlify**: Drag & drop this folder to https://app.netlify.com/drop
- **Vercel**: Run `vercel` in this directory
- **Cloudflare Pages**: Connect repo and set build directory to `wasm`
- **Self-hosted**: Upload to any web server

Detailed instructions in [DEPLOY.md](./DEPLOY.md)

## 🔒 Security & Privacy

- ✅ **Zero-Trust**: All decoding happens in your browser
- ✅ **No Server**: Transaction data never leaves your machine
- ✅ **Offline Capable**: Works after initial load
- ✅ **Auditable**: View source directly in DevTools

## 📊 Performance

- **Bundle Size**: 358KB WASM + 16KB JS (compresses to ~100KB)
- **Load Time**: < 5 seconds on 3G
- **Decode Time**: < 100ms per transaction
- **Memory**: ~10MB

## 🔧 Rebuilding

To rebuild after making changes to the Rust code:

```bash
# From repository root
./rebuild-wasm.sh
```

This will:
1. Compile Rust to WASM
2. Update this `/wasm` directory
3. Show bundle size

## 📖 Documentation

- **Deployment Guide**: [DEPLOY.md](./DEPLOY.md) - How to deploy manually
- **WASM Architecture**: `../docs/WASM_DEMO.md` - Technical design details
- **Core Principles**: `../CLAUDE.md` - Design philosophy

## 🎯 Use Cases

- **Block Explorers**: Decode transactions client-side
- **Research**: Analyze transaction patterns without server
- **Education**: Learn blockchain transaction formats
- **Conferences**: Offline demo of multi-chain decoding
- **Privacy**: Decode sensitive transactions locally

## 🧪 Testing Checklist

Before deploying, verify:

- [ ] Page loads without errors
- [ ] All 4 chains appear in dropdown
- [ ] Example transactions load correctly
- [ ] "Decode Transaction" button works
- [ ] "Auto-detect Chain" identifies correctly
- [ ] JSON output displays properly
- [ ] Browser console shows no errors

## 🐛 Troubleshooting

**Issue**: "Failed to fetch WASM"
- **Fix**: Must use HTTP server, not file:// URLs
- **Solution**: `python3 -m http.server 8080`

**Issue**: "Module is not defined"
- **Fix**: Browser too old
- **Solution**: Use Chrome 61+, Firefox 60+, Safari 11+, Edge 79+

**Issue**: "CORS policy error"
- **Fix**: Web server not configured
- **Solution**: Add CORS headers or use `python3 -m http.server`

More in [DEPLOY.md](./DEPLOY.md)

## 📜 License

MIT OR Apache-2.0

## 🔗 Links

- **Repository**: https://github.com/prasincs/universal-blockchain-decoder
- **Issues**: https://github.com/prasincs/universal-blockchain-decoder/issues
- **WASM Docs**: https://docs.claude.com/en/docs/claude-code/

---

**Built with**: Rust 🦀 + WebAssembly 🕸️ + Zero Dependencies ✨
