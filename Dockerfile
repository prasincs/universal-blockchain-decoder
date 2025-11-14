# Universal Blockchain Decoder - WASM Build Container
# This Dockerfile provides a complete environment for building WASM artifacts
# from any machine that has Docker installed.

# Use official Rust image as base (latest stable)
# Note: For reproducible builds, pin to specific version (e.g., rust:1.91-bookworm)
FROM rust:bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    pkg-config \
    libssl-dev \
    build-essential \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Install wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install binaryen (provides wasm-opt for optimization)
RUN curl -L https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz | tar xz -C /usr/local --strip-components=1

# Create app directory
WORKDIR /app

# Copy the entire project
# Note: Use .dockerignore to exclude unnecessary files
COPY . .

# Default command: build WASM
CMD ["bash", "crates/universal-decoder-wasm/build.sh"]

# For development: create a stage with all tools available
FROM builder as dev

# Install additional development tools
RUN cargo install cargo-watch cargo-audit

# Set up volume mounts for development
VOLUME ["/app"]

# Default to bash for interactive development
CMD ["/bin/bash"]
