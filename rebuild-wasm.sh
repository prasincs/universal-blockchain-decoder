#!/bin/bash
# Rebuild and update WASM demo deployment
set -e

echo "🔨 Rebuilding Universal Blockchain Decoder WASM module..."

# Change to WASM crate directory
cd "$(dirname "$0")/crates/universal-decoder-wasm"

# Build WASM module
echo "📦 Building WASM with wasm-pack..."
wasm-pack build --target web --out-dir www/pkg

# Copy to root /wasm directory
echo "📂 Copying files to /wasm directory..."
cd "$(dirname "$0")"
rm -rf wasm/*
cp -r crates/universal-decoder-wasm/www/* wasm/

# Copy deployment guide
echo "📄 Copying deployment guide..."
cp wasm/DEPLOY.md wasm/DEPLOY.md.bak 2>/dev/null || true

# Show bundle size
echo ""
echo "📊 Bundle size:"
ls -lh wasm/pkg/universal_decoder_wasm_bg.wasm | awk '{print "   WASM: " $5}'
du -sh wasm/pkg/ | awk '{print "   Total: " $1}'

echo ""
echo "✅ Build complete! Files are in /wasm directory"
echo ""
echo "Next steps:"
echo "1. Test locally: cd wasm && python3 -m http.server 8080"
echo "2. Deploy to GitHub Pages: See wasm/DEPLOY.md"
echo ""
