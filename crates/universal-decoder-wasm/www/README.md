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
- **Safari/iOS compatibility**: WebAssembly polyfill for older Safari versions
- **Better error handling**: Detailed error messages with browser-specific hints
- **Loading indicator**: Visual feedback during WASM initialization

## Browser Compatibility

| Browser | Version | Status | Notes |
|---------|---------|--------|-------|
| Chrome  | 89+     | ✅ Full support | Recommended |
| Firefox | 89+     | ✅ Full support | Recommended |
| Safari  | 15+     | ✅ Full support | Includes iOS Safari |
| Edge    | 89+     | ✅ Full support | Chromium-based |
| Mobile  | iOS 15+, Android 89+ | ✅ Responsive | Touch-optimized |

### Safari/iOS Compatibility Notes

The WASM demo now includes specific fixes for Safari and iOS:

1. **WebAssembly.instantiateStreaming polyfill**: Older iOS versions (< 15) don't support this API natively
2. **Proper MIME types**: WASM files are served with `application/wasm` content type
3. **Enhanced error messages**: Safari-specific troubleshooting hints
4. **Loading indicator**: Visual feedback prevents confusion on slower connections

If you encounter issues on Safari/iOS:
- Update to the latest Safari/iOS version
- Check the browser console for detailed error messages
- Try Chrome or Firefox as an alternative
- Report issues at https://github.com/prasincs/universal-blockchain-decoder/issues

### Server Configuration

The following files ensure proper WASM MIME types on different hosting platforms:

- `_headers` - For GitHub Pages
- `netlify.toml` - For Netlify deployment (already configured at repository root)

Note: Netlify uses `netlify.toml` for configuration, GitHub Pages uses `_headers`.
Both platforms serve WASM files with the correct `application/wasm` MIME type.

## Documentation

- Main docs: `/docs/WASM_DEMO.md`
- Deployment guide: `DEPLOY.md`
- Build script: `../build.sh`
