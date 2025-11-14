# Docker WASM Build - Testing & Validation Checklist

Use this checklist to validate the Docker WASM build setup on your machine.

---

## Pre-flight Checks

- [ ] **Docker installed**: Run `docker --version`
  - Expected: Docker version 20.10 or later
  - If not: Install from https://docs.docker.com/get-docker/

- [ ] **Docker running**: Run `docker ps`
  - Expected: Table showing running containers (may be empty)
  - If error: Start Docker Desktop application

- [ ] **docker-compose available**: Run `docker-compose --version` or `docker compose version`
  - Expected: Version 2.x or later
  - If not: Usually included with Docker Desktop

- [ ] **Sufficient disk space**: Run `df -h .` (Linux/Mac) or check disk space (Windows)
  - Required: At least 5GB free
  - Docker images: ~2GB
  - Build cache: ~1GB
  - WASM output: ~10MB

---

## Build Tests

### Test 1: Basic Build

```bash
# Run the build
./docker-build-wasm.sh

# Expected output:
# 🐳 Universal Blockchain Decoder - Docker WASM Build
# 📦 Building WASM in release mode...
# [build logs]
# ✅ Build complete!
```

**Validation:**
- [ ] Build completes without errors
- [ ] Output directory exists: `crates/universal-decoder-wasm/www/pkg/`
- [ ] WASM file present: `crates/universal-decoder-wasm/www/pkg/universal_decoder_wasm_bg.wasm`
- [ ] JavaScript glue code present: `crates/universal-decoder-wasm/www/pkg/universal_decoder_wasm.js`
- [ ] TypeScript definitions present: `crates/universal-decoder-wasm/www/pkg/universal_decoder_wasm.d.ts`

**Check file sizes:**
```bash
ls -lh crates/universal-decoder-wasm/www/pkg/*.wasm
# Expected: 500KB - 3MB depending on included decoders
```

### Test 2: Full Build

```bash
./docker-build-wasm.sh full
```

**Validation:**
- [ ] Build completes
- [ ] Files copied to `/wasm/pkg/` directory
- [ ] `wasm/index.html` exists
- [ ] `wasm/main.js` exists

### Test 3: Development Shell

```bash
./docker-build-wasm.sh shell
```

**Inside container:**
```bash
# Check Rust version
rustc --version
# Expected: rustc 1.91.x or later (latest stable)

# Check wasm-pack
wasm-pack --version
# Expected: wasm-pack 0.12.x or later

# Check wasm-opt
wasm-opt --version
# Expected: Binaryen version_116

# Check working directory
pwd
# Expected: /app

# Check files
ls crates/universal-decoder-wasm/
# Expected: Cargo.toml, src/, www/, build.sh

# Exit container
exit
```

**Validation:**
- [ ] All tools available
- [ ] Project files accessible
- [ ] Can build manually

### Test 4: Local Server

```bash
./docker-build-wasm.sh build-and-serve
```

**Browser tests:**
- [ ] Open http://localhost:8080
- [ ] Page loads without errors
- [ ] Check browser console for errors (F12)
- [ ] Try decoding a sample transaction
- [ ] Verify output appears

**Sample transaction to test (Bitcoin):**
```
0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000
```

**Expected:**
- Transaction decodes successfully
- JSON output shows transaction details
- Canonical hash displayed
- No JavaScript errors

**Stop server:**
```bash
# Press Ctrl+C in terminal
# Or in another terminal:
docker-compose down
```

**Validation:**
- [ ] Server starts successfully
- [ ] Page loads in browser
- [ ] WASM module loads
- [ ] Can decode transactions
- [ ] No console errors

### Test 5: Clean Up

```bash
./docker-build-wasm.sh clean
```

**Validation:**
- [ ] Command completes
- [ ] Docker images removed: `docker images | grep universal-decoder`
  - Should show no images (or old images if not removed)
- [ ] Volumes removed: `docker volume ls`
- [ ] WASM files still exist (not deleted by clean command)

---

## Performance Tests

### Test 6: Build Time

```bash
# First build (cold cache)
time ./docker-build-wasm.sh

# Second build (warm cache)
docker-compose run --rm wasm-builder
time docker-compose run --rm wasm-builder
```

**Expected timings:**
- **First build**: 10-20 minutes (downloading base images, building dependencies)
- **Subsequent builds**: 1-5 minutes (cached dependencies)

**Validation:**
- [ ] First build completes (may be slow)
- [ ] Second build is significantly faster
- [ ] Build times acceptable for your workflow

### Test 7: WASM Size

```bash
# Build with optimizations
./docker-build-wasm.sh

# Check size
ls -lh crates/universal-decoder-wasm/www/pkg/*.wasm

# Check gzipped size (what users download)
gzip -c crates/universal-decoder-wasm/www/pkg/*.wasm | wc -c
```

**Expected sizes:**
- **Uncompressed**: 500KB - 3MB
- **Gzipped**: 200KB - 1.2MB

**Validation:**
- [ ] WASM file size is reasonable
- [ ] Gzipped size < 2MB
- [ ] Size acceptable for web deployment

---

## Edge Case Tests

### Test 8: Rebuild Without Clean

```bash
# Build once
./docker-build-wasm.sh

# Make a change to Rust code
echo "// test comment" >> crates/universal-decoder-wasm/src/lib.rs

# Rebuild
./docker-build-wasm.sh

# Verify new build
stat crates/universal-decoder-wasm/www/pkg/*.wasm
```

**Validation:**
- [ ] Rebuild succeeds
- [ ] Changes reflected in WASM
- [ ] Build uses cache (faster than first build)

### Test 9: Multiple Decoders

Check that all decoder dependencies build correctly:

```bash
./docker-build-wasm.sh shell

# Inside container:
cd /app/crates/universal-decoder-wasm
cat Cargo.toml | grep "decoder-"
# Should show: decoder-bitcoin, decoder-ethereum, decoder-solana, decoder-cosmos

cargo tree | grep decoder
# Verify all decoders are included

exit
```

**Validation:**
- [ ] All decoders listed in Cargo.toml
- [ ] All decoders compile
- [ ] No dependency conflicts

### Test 10: Error Handling

Test that errors are reported clearly:

```bash
# Introduce a syntax error
echo "invalid rust code" >> crates/universal-decoder-wasm/src/lib.rs

# Try to build
./docker-build-wasm.sh

# Expected: Clear error message showing compilation failure

# Fix the error
git checkout crates/universal-decoder-wasm/src/lib.rs

# Rebuild
./docker-build-wasm.sh
```

**Validation:**
- [ ] Build errors are visible
- [ ] Error messages are clear
- [ ] Can recover from errors

---

## Platform-Specific Tests

### Linux

```bash
# Check SELinux issues (if applicable)
getenforce
# If "Enforcing", verify volumes mount correctly

# Check user permissions
id
# Verify user is in docker group

# Test build
./docker-build-wasm.sh
```

**Validation:**
- [ ] No permission errors
- [ ] Volumes mount correctly
- [ ] Build succeeds

### macOS

```bash
# Check Docker Desktop running
docker info | grep "Operating System"
# Should show Docker Desktop

# Test build
./docker-build-wasm.sh

# Check for M1/M2 ARM issues
uname -m
# If "arm64", verify build works
```

**Validation:**
- [ ] Docker Desktop running
- [ ] Build succeeds on ARM Macs
- [ ] No platform compatibility errors

### Windows

```powershell
# Check Docker Desktop (use PowerShell or Git Bash)
docker info

# Test build (use Git Bash)
./docker-build-wasm.sh

# Or PowerShell
docker-compose run --rm wasm-builder
```

**Validation:**
- [ ] Docker Desktop running (WSL 2 backend recommended)
- [ ] Build succeeds
- [ ] Line endings handled correctly (CRLF vs LF)

---

## Integration Tests

### Test 11: CI/CD Simulation

Simulate a CI environment:

```bash
# Clean everything
./docker-build-wasm.sh clean
rm -rf crates/universal-decoder-wasm/www/pkg

# Build from scratch (like CI would)
./docker-build-wasm.sh

# Verify output
test -f crates/universal-decoder-wasm/www/pkg/universal_decoder_wasm_bg.wasm && echo "✅ WASM build succeeded"
```

**Validation:**
- [ ] Build succeeds from clean state
- [ ] All artifacts generated
- [ ] No manual intervention needed

### Test 12: Multi-user Scenario

If multiple developers use this setup:

```bash
# User 1 builds
./docker-build-wasm.sh

# User 2 builds (simulate by cleaning cache)
docker-compose build --no-cache
./docker-build-wasm.sh

# Both should produce identical WASM
sha256sum crates/universal-decoder-wasm/www/pkg/*.wasm
```

**Validation:**
- [ ] Builds are reproducible
- [ ] No user-specific paths in output
- [ ] Consistent results across builds

---

## Documentation Tests

### Test 13: Documentation Accuracy

Verify all documentation is accurate:

- [ ] `DOCKER_WASM.md` commands work
- [ ] `DOCKER_QUICK_START.md` examples work
- [ ] `docker-build-wasm.sh --help` (or no args) shows usage
- [ ] README mentions Docker option
- [ ] All links in docs are valid

### Test 14: Example Commands

Test every command in the documentation:

```bash
# From DOCKER_QUICK_START.md
./docker-build-wasm.sh                    # ✓
./docker-build-wasm.sh build-and-serve    # ✓
./docker-build-wasm.sh shell              # ✓
./docker-build-wasm.sh serve              # ✓
./docker-build-wasm.sh clean              # ✓
./docker-build-wasm.sh full               # ✓
./docker-build-wasm.sh dev                # ✓

# Direct docker-compose
docker-compose run --rm wasm-builder      # ✓
docker-compose run --rm wasm-dev          # ✓
docker-compose up wasm-server             # ✓
docker-compose down                       # ✓
```

**Validation:**
- [ ] All documented commands work
- [ ] No outdated instructions
- [ ] Help text is clear

---

## Security Tests

### Test 15: Security Scan

```bash
# Scan Docker image for vulnerabilities
docker-compose build
docker scan universal-decoder-wasm-builder:latest

# Or use trivy
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    aquasec/trivy image universal-decoder-wasm-builder:latest
```

**Validation:**
- [ ] No critical vulnerabilities
- [ ] Base image is up to date
- [ ] Dependencies are recent versions

### Test 16: Network Isolation

Verify builds don't require network access after initial setup:

```bash
# Build once (download dependencies)
./docker-build-wasm.sh

# Disconnect network
# (Platform specific - e.g., turn off WiFi, or use Docker network flags)

# Rebuild (should use cache)
docker-compose run --rm --network none wasm-builder || echo "Network required"

# Reconnect network
```

**Validation:**
- [ ] First build requires network (expected)
- [ ] Cached builds may work offline
- [ ] No unexpected network calls

---

## Final Validation

### Checklist Summary

Before marking the Docker setup as "complete", ensure:

**Build System:**
- [ ] Docker builds succeed on your platform
- [ ] All scripts are executable (`chmod +x *.sh`)
- [ ] Build times are acceptable
- [ ] WASM output is correct

**Development:**
- [ ] Dev shell provides all needed tools
- [ ] Can build, test, and iterate
- [ ] Volume mounts work correctly

**Testing:**
- [ ] Local server works
- [ ] WASM demo loads in browser
- [ ] Can decode sample transactions

**Documentation:**
- [ ] All docs are accurate
- [ ] Examples work
- [ ] Troubleshooting helps

**Production:**
- [ ] Release builds are optimized
- [ ] Output is deployment-ready
- [ ] Size is acceptable

---

## Reporting Issues

If any test fails:

1. **Check logs**: `docker-compose logs`
2. **Verify Docker**: `docker info`
3. **Check disk space**: `df -h`
4. **Review error messages**
5. **Consult troubleshooting**: `DOCKER_WASM.md` section
6. **Report issue**: https://github.com/prasincs/universal-blockchain-decoder/issues

Include:
- OS and version
- Docker version
- Complete error message
- Steps to reproduce

---

## Next Steps

After all tests pass:

1. **Update README**: Add Docker build instructions to main README.md
2. **CI Integration**: Add Docker build to `.github/workflows/`
3. **Team Onboarding**: Share `DOCKER_QUICK_START.md` with team
4. **Deploy**: Use built WASM for production deployment

---

**Last Updated**: 2025-11-14
**Testing Checklist** | Report issues at GitHub
