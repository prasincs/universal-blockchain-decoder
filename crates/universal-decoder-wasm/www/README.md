# Universal Blockchain Decoder - WASM Demo

**NOTE**: This directory is now a **symlink** to `crates/universal-decoder-wasm/www/`.

## Directory Structure

```
/wasm/                                      ← Symlink to www/
└─> crates/universal-decoder-wasm/www/     ← Source of truth
    ├── index.html                          ← Main demo page
    ├── main.js                             ← Demo logic
    ├── examples.js                         ← Example transactions
    ├── style.css                           ← Styles
    ├── comparison.html                     ← Type system comparison
    ├── treemap.html                        ← Chain ecosystem treemap
    ├── graph.html                          ← Chain relationship graph
    └── pkg/                                ← Built WASM module (generated)
        ├── universal_decoder_wasm.js
        ├── universal_decoder_wasm_bg.wasm
        └── ...
```

## Why the Symlink?

Previously, we had duplicate directories:
- `/wasm/` - Old location (now removed)
- `/crates/universal-decoder-wasm/www/` - Netlify deployment source

This caused sync issues where changes to one weren't reflected in the other.

**Solution**: `/wasm/` is now a symlink to `www/`, ensuring there's only **one source of truth**.

## Editing Files

All edits should be made in this directory (which resolves to `www/`):
- ✅ Edit `wasm/index.html` → Actually edits `crates/universal-decoder-wasm/www/index.html`
- ✅ Edit `wasm/main.js` → Actually edits `crates/universal-decoder-wasm/www/main.js`
- ✅ No more sync issues!

## Building WASM

From the repository root:

```bash
cd crates/universal-decoder-wasm
./build.sh
```

This builds the WASM module to `www/pkg/`.

## Testing Locally

```bash
cd wasm  # (or crates/universal-decoder-wasm/www)
python3 -m http.server 8080
# Open http://localhost:8080
```

## Netlify Deployment

Netlify automatically:
1. Runs `crates/universal-decoder-wasm/build.sh`
2. Deploys the `www/` directory
3. Everything is in sync because `/wasm/` is a symlink

## Recent Enhancements

- **23+ blockchain support**: Auto-deployed from decoder crates
- **Chain family grouping**: Dropdown organized by UTXO/Account/Instruction/Object/Privacy
- **Scoped examples**: Examples filtered by selected chain
- **Auto-clear input**: Prevents decode errors when switching chains
- **TX ID + Canonical hash**: Shows both original txid and Borsh hash
- **Enhanced Borsh fields**: Full input/output transaction details

## Documentation

- Main docs: `/docs/WASM_DEMO.md`
- Deployment guide: `DEPLOY.md`
- Build script: `../build.sh`
