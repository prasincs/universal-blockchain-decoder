#!/bin/bash
set -e

echo "🔨 Building Universal Blockchain Decoder WASM module..."
echo ""

# Install dependencies for Netlify (or other CI environments)
if [ "$NETLIFY" = "true" ] || [ "$CI" = "true" ]; then
    echo "📦 Installing build dependencies for CI environment..."

    # Install Rust if not present
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        source "$HOME/.cargo/env"
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Add wasm32 target
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown

    # Install wasm-pack if not present
    if ! command -v wasm-pack &> /dev/null; then
        echo "Installing wasm-pack..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Try to install wasm-opt (optional, improves size)
    if ! command -v wasm-opt &> /dev/null; then
        echo "Attempting to install binaryen (wasm-opt)..."
        npm install -g wasm-opt || echo "⚠️  wasm-opt install failed (optional), continuing..."
    fi

    echo "✅ Dependencies installed"
    echo ""
fi

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
