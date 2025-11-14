# Docker WASM Build - Quick Reference Card

**One-page guide for building WASM using Docker**

---

## Prerequisites

```bash
# Install Docker Desktop from: https://docs.docker.com/get-docker/
# Verify installation:
docker --version
```

---

## Common Commands

### 🚀 Quick Start (Build and Test)

```bash
./docker-build-wasm.sh build-and-serve
# Opens: http://localhost:8080
```

### 📦 Build Only

```bash
./docker-build-wasm.sh
# Output: crates/universal-decoder-wasm/www/pkg/
```

### 🔧 Development

```bash
./docker-build-wasm.sh shell
# Opens interactive shell with all tools
```

### 🌐 Test Locally

```bash
./docker-build-wasm.sh serve
# Port 8080: crates/universal-decoder-wasm/www/
# Port 8081: /wasm/
```

### 🧹 Clean Up

```bash
./docker-build-wasm.sh clean
# Removes Docker images and volumes
```

---

## Direct Docker Commands

```bash
# Build WASM
docker-compose run --rm wasm-builder

# Dev shell
docker-compose run --rm wasm-dev

# Start server
docker-compose up wasm-server

# Stop server
docker-compose down
```

---

## Directory Structure

```
project-root/
├── Dockerfile                              # Docker image definition
├── docker-compose.yml                      # Service orchestration
├── .dockerignore                           # Build efficiency
├── docker-build-wasm.sh                    # Main build script
├── DOCKER_WASM.md                          # Full documentation
└── crates/universal-decoder-wasm/
    ├── build.sh                            # WASM build script
    └── www/
        └── pkg/                            # ← Output directory
            ├── universal_decoder_wasm.js
            ├── universal_decoder_wasm_bg.wasm
            └── ...
```

---

## Workflow Examples

### Example 1: First-time Build

```bash
# 1. Clone repository
git clone https://github.com/prasincs/universal-blockchain-decoder.git
cd universal-blockchain-decoder

# 2. Build WASM
./docker-build-wasm.sh

# 3. Test
./docker-build-wasm.sh serve
# Open http://localhost:8080

# 4. Deploy (if satisfied)
# See wasm/DEPLOY.md
```

### Example 2: Development Loop

```bash
# Terminal 1: Auto-rebuild on changes
./docker-build-wasm.sh shell
cd crates/universal-decoder-wasm
cargo watch -s "wasm-pack build --target web --dev"

# Terminal 2: Web server
./docker-build-wasm.sh serve

# Edit code → Auto-rebuild → Refresh browser
```

### Example 3: Production Build

```bash
# Full build with optimizations
./docker-build-wasm.sh full

# Check size
ls -lh wasm/pkg/*.wasm

# Deploy
cd wasm
# Follow DEPLOY.md instructions
```

---

## Troubleshooting Quick Fixes

| Problem | Solution |
|---------|----------|
| Docker not found | Install Docker Desktop |
| Permission denied (Linux) | `sudo usermod -aG docker $USER` + logout |
| Build is slow | First build: 10-20 min (one-time caching) |
| Changes not reflected | Hard refresh browser (Ctrl+Shift+R) |
| No space left | `docker system prune -a` |
| Cannot connect to daemon | Start Docker Desktop application |
| CORS errors in browser | Use HTTP server, not `file://` |

---

## File Sizes (Typical)

| Component | Size (uncompressed) | Size (gzipped) |
|-----------|---------------------|----------------|
| Minimal build (Bitcoin + Ethereum) | ~800KB | ~300KB |
| Full build (all decoders) | ~3MB | ~1.2MB |
| Docker image | ~2GB | - |

---

## Environment Variables

```bash
# Custom optimization
docker-compose run --rm \
    -e CARGO_PROFILE_RELEASE_OPT_LEVEL=s \
    wasm-builder

# Debug logging
docker-compose run --rm \
    -e RUST_LOG=debug \
    wasm-builder
```

---

## CI/CD Integration

```yaml
# .github/workflows/wasm.yml
jobs:
  build-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build WASM
        run: ./docker-build-wasm.sh
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: wasm
          path: crates/universal-decoder-wasm/www/pkg/
```

---

## Resources

- **Full Documentation**: `DOCKER_WASM.md`
- **WASM Demo Guide**: `docs/WASM_DEMO.md`
- **Deployment**: `wasm/DEPLOY.md`
- **Docker Docs**: https://docs.docker.com/
- **wasm-pack**: https://rustwasm.github.io/wasm-pack/

---

## Help

```bash
# Show all available commands
./docker-build-wasm.sh help

# Or just run without arguments to see usage
./docker-build-wasm.sh
```

---

**Last Updated**: 2025-11-14
**Quick Start Guide** | For detailed docs see `DOCKER_WASM.md`
