#!/bin/bash
set -e

echo "🔨 Building Universal Blockchain Decoder WASM module..."
echo ""

# Build for web target
echo "Building WASM module with wasm-pack..."
wasm-pack build --target web --out-dir www/pkg --release

# Measure size
echo ""
echo "📦 Bundle size:"
ls -lh www/pkg/*.wasm | awk '{print $5 " " $9}'

# Optional: Optimize further with wasm-opt (from binaryen)
if command -v wasm-opt &> /dev/null; then
    echo ""
    echo "🚀 Optimizing with wasm-opt..."
    for wasm_file in www/pkg/*_bg.wasm; do
        wasm-opt -Oz "$wasm_file" -o "$wasm_file.optimized"
        mv "$wasm_file.optimized" "$wasm_file"
    done
    echo "Optimized size:"
    ls -lh www/pkg/*.wasm | awk '{print $5 " " $9}'
fi

echo ""
echo "✅ Build complete!"
echo ""
echo "To test locally:"
echo "  cd www"
echo "  python3 -m http.server 8080"
echo "  # Open http://localhost:8080 in browser"
