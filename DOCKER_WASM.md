# Docker WASM Build Environment

This document explains how to build WASM artifacts for the Universal Blockchain Decoder using Docker, enabling consistent builds across all platforms.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Build Commands](#build-commands)
4. [Development Workflow](#development-workflow)
5. [Troubleshooting](#troubleshooting)
6. [Advanced Usage](#advanced-usage)

---

## Prerequisites

**Required:**
- Docker Desktop (version 20.10 or later)
  - **macOS**: [Install Docker Desktop for Mac](https://docs.docker.com/desktop/install/mac-install/)
  - **Windows**: [Install Docker Desktop for Windows](https://docs.docker.com/desktop/install/windows-install/)
  - **Linux**: [Install Docker Engine](https://docs.docker.com/engine/install/)

**Optional:**
- docker-compose (usually included with Docker Desktop)

**Verify Installation:**
```bash
docker --version
docker-compose --version  # or: docker compose version
```

---

## Quick Start

### 1. Build WASM Module

```bash
# One-liner: Build WASM in release mode
./docker-build-wasm.sh

# Or using docker-compose directly
docker-compose run --rm wasm-builder
```

**Output:** `crates/universal-decoder-wasm/www/pkg/` directory with WASM artifacts

### 2. Test Locally

```bash
# Build and start web server in one command
./docker-build-wasm.sh build-and-serve

# Access the demo at: http://localhost:8080
```

### 3. Deploy

The built WASM files in `crates/universal-decoder-wasm/www/pkg/` are ready to deploy:
- Copy to your web server
- Deploy to GitHub Pages
- Upload to CDN

See `wasm/DEPLOY.md` for detailed deployment instructions.

---

## Build Commands

The `docker-build-wasm.sh` script provides several build modes:

### Release Build (Default)

```bash
./docker-build-wasm.sh release
# or simply:
./docker-build-wasm.sh
```

**What it does:**
- Builds WASM module in release mode (optimized for size)
- Applies wasm-opt optimizations (if available)
- Outputs to `crates/universal-decoder-wasm/www/pkg/`

**Use case:** Production builds for deployment

### Full Build

```bash
./docker-build-wasm.sh full
```

**What it does:**
- Runs `release` build
- Executes `rebuild-wasm.sh` to copy files to `/wasm` directory
- Creates deployment-ready artifacts in both locations

**Use case:** Preparing for GitHub Pages deployment

### Development Mode

```bash
./docker-build-wasm.sh dev
```

**What it does:**
- Starts a development container with all tools installed
- Mounts the project directory for live editing
- Provides interactive shell for manual commands

**Use case:** Active development, debugging, experimentation

### Interactive Shell

```bash
./docker-build-wasm.sh shell
```

**What it does:**
- Opens bash shell inside the development container
- Full access to Rust, wasm-pack, and all build tools
- Project mounted at `/app`

**Example session:**
```bash
# Inside container:
cd /app/crates/universal-decoder-wasm
wasm-pack build --target web --dev  # Debug build
wasm-pack test --headless --firefox  # Run tests
cargo clippy                         # Lint code
```

**Use case:** Manual builds, debugging, running specific commands

### Serve (Test Locally)

```bash
./docker-build-wasm.sh serve
```

**What it does:**
- Starts Python HTTP servers for WASM demo
- **Port 8080**: Serves `crates/universal-decoder-wasm/www/`
- **Port 8081**: Serves `/wasm/` directory

**Use case:** Testing WASM demo in browser before deployment

### Build and Serve

```bash
./docker-build-wasm.sh build-and-serve
```

**What it does:**
- Builds WASM in release mode
- Starts web server on port 8080
- One command for full build-test cycle

**Use case:** Quick iteration: build → test → repeat

### Clean

```bash
./docker-build-wasm.sh clean
```

**What it does:**
- Removes all Docker images and volumes
- Cleans up disk space
- **Does NOT** delete WASM build artifacts

**Use case:** Fresh start, freeing disk space

---

## Development Workflow

### Workflow 1: Quick Iteration

For rapid development with frequent rebuilds:

```bash
# Terminal 1: Start development container
./docker-build-wasm.sh shell

# Inside container:
cd crates/universal-decoder-wasm
cargo watch -s "wasm-pack build --target web --dev"

# Terminal 2: Start web server
./docker-build-wasm.sh serve
```

Now:
1. Edit Rust code in your IDE
2. cargo-watch auto-rebuilds on file changes
3. Refresh browser to see changes

### Workflow 2: Production Build

For creating deployment-ready artifacts:

```bash
# Build production WASM
./docker-build-wasm.sh full

# Test locally
./docker-build-wasm.sh serve
# Open http://localhost:8080

# If satisfied, deploy
cd wasm
# Follow deployment instructions in DEPLOY.md
```

### Workflow 3: CI/CD Integration

For automated builds in CI pipelines:

```bash
# In CI script (e.g., GitHub Actions, GitLab CI)
docker-compose run --rm wasm-builder

# Or using the script
./docker-build-wasm.sh release

# Collect artifacts
cp -r crates/universal-decoder-wasm/www/pkg ./artifacts/
```

See `.github/workflows/deploy-wasm-demo.yml` for a complete example.

---

## Troubleshooting

### Issue: "Docker command not found"

**Solution:**
```bash
# macOS/Windows: Install Docker Desktop
# https://docs.docker.com/get-docker/

# Linux: Install Docker Engine
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group (Linux)
sudo usermod -aG docker $USER
# Log out and back in for changes to take effect
```

### Issue: "Permission denied" on Linux

**Solution:**
```bash
# Add your user to the docker group
sudo usermod -aG docker $USER

# Log out and back in, then test:
docker run hello-world
```

### Issue: Build is very slow on first run

**Explanation:** Docker is downloading the Rust base image (~1GB) and building all dependencies.

**Solution:**
- **First build**: 10-20 minutes (one-time)
- **Subsequent builds**: 1-2 minutes (cached)

To speed up:
```bash
# Pre-pull base image (uses latest stable Rust)
docker pull rust:bookworm
```

### Issue: "No space left on device"

**Solution:**
```bash
# Clean up Docker artifacts
docker system prune -a

# Remove build caches
docker builder prune -a

# Clean project-specific images
./docker-build-wasm.sh clean
```

### Issue: WASM build succeeds but browser errors

**Check:**
1. **CORS issues**: Serve from HTTP server, not `file://`
   ```bash
   # ❌ Wrong: file:///path/to/index.html
   # ✅ Correct: http://localhost:8080
   ./docker-build-wasm.sh serve
   ```

2. **Cache issues**: Hard refresh in browser
   - Chrome/Firefox: Ctrl+Shift+R (Cmd+Shift+R on Mac)
   - Safari: Cmd+Option+R

3. **Module loading**: Check browser console for errors
   ```javascript
   // Ensure correct path in main.js
   import init from './pkg/universal_decoder_wasm.js';
   ```

### Issue: Changes not reflected after rebuild

**Solution:**
```bash
# Clean output directory
rm -rf crates/universal-decoder-wasm/www/pkg/*

# Rebuild
./docker-build-wasm.sh

# Hard refresh browser (Ctrl+Shift+R)
```

### Issue: "Cannot connect to the Docker daemon"

**Solution:**
```bash
# Ensure Docker is running
# macOS/Windows: Start Docker Desktop application

# Linux: Start Docker service
sudo systemctl start docker

# Verify
docker ps
```

---

## Advanced Usage

### Custom Rust Version

Edit `Dockerfile` to use a specific Rust version:

```dockerfile
# Current (uses latest stable):
FROM rust:bookworm as builder

# To pin to a specific version for reproducibility:
FROM rust:1.91-bookworm as builder
```

Then rebuild:
```bash
docker-compose build --no-cache
```

### Size Optimization

To minimize WASM bundle size:

1. **Enable wasm-opt** (automatic if installed):
   ```bash
   # Install binaryen in container (already included)
   # Optimizes WASM by ~20-30%
   ```

2. **Strip debug info**:
   ```toml
   # crates/universal-decoder-wasm/Cargo.toml
   [profile.release]
   opt-level = "z"     # Optimize for size
   lto = true          # Link-time optimization
   codegen-units = 1   # Better optimization
   strip = true        # Strip symbols
   ```

3. **Selective decoder inclusion**:
   ```toml
   # Only include needed decoders
   [dependencies]
   decoder-bitcoin = { path = "../decoder-bitcoin" }
   # decoder-ethereum = { path = "../decoder-ethereum" }  # Commented out
   ```

### Profiling Build Times

```bash
# Time the build
time ./docker-build-wasm.sh

# Detailed layer timing
docker-compose build --progress=plain 2>&1 | tee build.log

# Analyze build cache
docker-compose build --no-cache  # Force rebuild everything
```

### Multi-Platform Builds

To build for multiple architectures:

```bash
# Build for ARM64 (Apple Silicon)
docker buildx build --platform linux/arm64 -t ubd-wasm:arm64 .

# Build for AMD64 (Intel/AMD)
docker buildx build --platform linux/amd64 -t ubd-wasm:amd64 .

# Build both
docker buildx build --platform linux/amd64,linux/arm64 \
    -t ubd-wasm:multi .
```

### Mounting Custom Directories

```yaml
# docker-compose.override.yml (create this file)
version: '3.8'
services:
  wasm-dev:
    volumes:
      - ./custom-decoders:/app/crates/custom-decoders
      - ~/.cargo/registry:/usr/local/cargo/registry  # Share host cache
```

### Running Tests in Docker

```bash
# Unit tests
docker-compose run --rm wasm-dev \
    cargo test --package universal-decoder-wasm

# WASM-specific tests
docker-compose run --rm wasm-dev bash -c \
    "cd crates/universal-decoder-wasm && wasm-pack test --headless --chrome"

# All tests
docker-compose run --rm wasm-dev cargo test --all
```

### Debugging WASM in Container

```bash
# Start dev container with shell
./docker-build-wasm.sh shell

# Inside container: build debug version
cd /app/crates/universal-decoder-wasm
wasm-pack build --target web --dev  # Debug symbols included

# Inspect WASM
wasm-objdump -x www/pkg/*.wasm | less

# Disassemble
wasm-objdump -d www/pkg/*.wasm | less
```

---

## Comparison: Docker vs Native

| Aspect | Docker | Native |
|--------|--------|--------|
| **Setup Time** | 5 minutes | 30+ minutes |
| **Dependencies** | Docker only | Rust + wasm-pack + binaryen |
| **Consistency** | ✅ Identical on all platforms | ⚠️ Platform-dependent |
| **Build Speed** | First: slow, Then: fast | Fast (always) |
| **Disk Usage** | ~2GB (image + cache) | ~5GB (toolchains + cache) |
| **CI/CD** | ✅ Easy integration | ⚠️ Manual setup |
| **Offline** | ✅ After first pull | ✅ Yes |
| **Best For** | Teams, CI, reproducibility | Individual developers |

**Recommendation:**
- **Use Docker** if: Multiple developers, CI/CD, or want guaranteed reproducibility
- **Use Native** if: Solo developer, frequent builds, Docker not available

---

## Environment Variables

You can customize builds using environment variables:

```bash
# Example: Custom optimization level
docker-compose run --rm -e CARGO_PROFILE_RELEASE_OPT_LEVEL=s wasm-builder

# Example: Enable verbose logging
docker-compose run --rm -e RUST_LOG=debug wasm-builder

# Example: Custom target directory
docker-compose run --rm -e CARGO_TARGET_DIR=/tmp/target wasm-builder
```

---

## Docker Compose Services

The `docker-compose.yml` defines several services:

| Service | Purpose | Ports | Command |
|---------|---------|-------|---------|
| `wasm-builder` | Production builds | - | `build.sh` |
| `wasm-dev` | Development environment | - | `/bin/bash` |
| `wasm-server` | Test server (www) | 8080 | HTTP server |
| `wasm-server-root` | Test server (/wasm) | 8081 | HTTP server |

**Run specific service:**
```bash
docker-compose run --rm <service-name>
```

---

## Integration with Main Project

### Building WASM as Part of CI

Add to `.github/workflows/ci.yml`:

```yaml
wasm-build:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - name: Build WASM
      run: ./docker-build-wasm.sh release
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: wasm-build
        path: crates/universal-decoder-wasm/www/pkg/
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Build WASM before committing changes to WASM crate

if git diff --cached --name-only | grep -q "crates/universal-decoder-wasm"; then
    echo "🔨 Building WASM..."
    ./docker-build-wasm.sh || exit 1
fi
```

---

## Resources

- **Docker Documentation**: https://docs.docker.com/
- **wasm-pack Guide**: https://rustwasm.github.io/wasm-pack/
- **Rust WASM Book**: https://rustwasm.github.io/book/
- **Project WASM Demo**: `docs/WASM_DEMO.md`
- **Deployment Guide**: `wasm/DEPLOY.md`

---

## Support

**Issues:**
- Docker-specific issues: [Docker GitHub Issues](https://github.com/docker/docker-ce/issues)
- Project issues: [Universal Decoder Issues](https://github.com/prasincs/universal-blockchain-decoder/issues)

**Questions:**
- Docker: [Docker Community Forums](https://forums.docker.com/)
- WASM: [Rust WASM Discord](https://discord.com/invite/rust-lang)

---

**Last Updated**: 2025-11-14
**Tested With**:
- Docker version: 24.0.7
- docker-compose version: 2.23.3
- Platforms: Linux (Ubuntu 22.04), macOS (13.6), Windows (11)
