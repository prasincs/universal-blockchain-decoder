# Netlify Deployment Setup Guide

This document describes how to configure Netlify deployments for the Universal Blockchain Decoder WASM demo.

## Overview

The project uses **GitHub Actions to build** the WASM module, then **deploys to Netlify** for preview and production hosting.

### Deployment Targets

- **Production**: GitHub Pages (via `deploy-wasm-demo.yml`)
- **Preview**: Netlify (via `deploy-wasm-preview-netlify.yml`)
- **Local**: `python3 -m http.server 8080` in `crates/universal-decoder-wasm/www/`

## Required GitHub Secrets

**CRITICAL**: The following secrets MUST be configured in GitHub for Netlify deployments to work.

### 1. NETLIFY_AUTH_TOKEN

**What**: Personal access token from Netlify

**How to get**:
1. Log in to [Netlify](https://app.netlify.com)
2. Go to **User Settings** → **Applications**
3. Click **New Access Token**
4. Name: "GitHub Actions Deployment"
5. Copy the token (shown only once!)

**Where to add**:
```
GitHub Repository → Settings → Secrets and variables → Actions → New repository secret
Name: NETLIFY_AUTH_TOKEN
Value: <paste token>
```

### 2. NETLIFY_SITE_ID

**What**: Unique identifier for your Netlify site (UUID format)

**How to get**:
1. Log in to [Netlify](https://app.netlify.com)
2. Select your site (or create a new one)
3. Go to **Site Settings** → **General** → **Site details**
4. Copy the **Site ID** (format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)

**Where to add**:
```
GitHub Repository → Settings → Secrets and variables → Actions → New repository secret
Name: NETLIFY_SITE_ID
Value: <paste site ID>
```

## Initial Netlify Site Setup

If you don't have a Netlify site yet:

### Option 1: Create via Netlify Dashboard (Recommended)

1. Go to [Netlify](https://app.netlify.com) and log in
2. Click **Add new site** → **Import an existing project**
3. Select **GitHub** as the git provider
4. Choose the `universal-blockchain-decoder` repository
5. Configure build settings:
   - **Base directory**: `crates/universal-decoder-wasm`
   - **Build command**: `./build.sh`
   - **Publish directory**: `www`
6. Click **Deploy site**
7. Get the **Site ID** from **Site Settings** → **General**
8. Get or create a **Personal Access Token** from **User Settings** → **Applications**
9. Add both as GitHub Secrets (see above)

### Option 2: Use Netlify CLI

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login to Netlify
netlify login

# Create new site (from repo root)
cd crates/universal-decoder-wasm
netlify init

# This will guide you through setup and create a site
# It will show the Site ID - copy it for GitHub Secrets
```

## How It Works

### Deployment Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Developer pushes to branch/PR                              │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  GitHub Actions (deploy-wasm-preview-netlify.yml)           │
│  1. Checkout code                                           │
│  2. Setup Rust + wasm32 target                              │
│  3. Install wasm-pack                                       │
│  4. Run build.sh → generates www/pkg/*.wasm                 │
│  5. Show bundle size                                        │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  Deploy to Netlify (nwtgck/actions-netlify@v2)              │
│  - Uses NETLIFY_AUTH_TOKEN and NETLIFY_SITE_ID              │
│  - Creates preview URL                                      │
│  - Comments on PR with link                                 │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  Preview URL ready!                                         │
│  https://deploy-preview-XXX--your-site.netlify.app          │
└─────────────────────────────────────────────────────────────┘
```

### Production Deployment (main branch)

```
Push to main
     │
     ├─── deploy-wasm-preview-netlify.yml → Netlify (preview)
     │
     └─── deploy-wasm-demo.yml → GitHub Pages (production)
```

## Build Configuration

### netlify.toml

The `netlify.toml` file in the repository root configures:

- **Base directory**: `crates/universal-decoder-wasm`
- **Build command**: `./build.sh`
- **Publish directory**: `www`
- **WASM MIME types**: `Content-Type: application/wasm`
- **CORS headers**: `Access-Control-Allow-Origin: *`
- **SPA routing**: Redirects to `index.html`

### build.sh

The build script:

1. Detects CI environment (`$NETLIFY` or `$CI`)
2. Installs Rust and wasm-pack if needed
3. Adds `wasm32-unknown-unknown` target
4. Builds with `wasm-pack build --target web --out-dir www/pkg --release`
5. Optimizes with `wasm-opt` (if available)

### Cargo.toml Optimization

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit (better optimization)
strip = true         # Strip debug symbols
```

## Troubleshooting

### Deployment Fails: "Error: Missing required secret"

**Problem**: GitHub Secrets not configured

**Fix**: Add `NETLIFY_AUTH_TOKEN` and `NETLIFY_SITE_ID` (see above)

### WASM Module Fails to Load in Browser

**Problem**: Incorrect MIME type or CORS headers

**Fix**: Already configured in `netlify.toml` - check browser DevTools network tab to verify:
- `Content-Type: application/wasm` ✅
- `Access-Control-Allow-Origin: *` ✅

### Build Succeeds but Nothing Deploys

**Problem**: Netlify action has incorrect configuration

**Fix**: Check workflow logs for errors. Ensure:
- `NETLIFY_AUTH_TOKEN` is valid (not expired)
- `NETLIFY_SITE_ID` matches your Netlify site

### Bundle Size Too Large

**Current optimizations**:
- Cargo release profile: `opt-level = "z"`, `lto = true`
- wasm-opt: `-Oz` optimization
- Typical bundle size: ~1-3 MB (compressed)

**Further optimization**:
```bash
# Install wasm-opt locally
sudo apt-get install binaryen

# Run build script
cd crates/universal-decoder-wasm
./build.sh

# Check size
ls -lh www/pkg/*.wasm
```

### Preview URL Not Posted to PR

**Problem**: Workflow lacks PR comment permissions

**Fix**: Check that `github-token: ${{ secrets.GITHUB_TOKEN }}` is set in workflow (already configured)

## Testing Locally

```bash
# Build WASM module
cd crates/universal-decoder-wasm
./build.sh

# Serve locally
cd www
python3 -m http.server 8080

# Open browser
open http://localhost:8080
```

## Monitoring Deployments

### GitHub Actions

View workflow runs:
```
GitHub Repository → Actions → deploy-wasm-preview-netlify
```

Look for:
- ✅ Build WASM module (should succeed)
- ✅ Deploy to Netlify (should show preview URL)
- 📦 Bundle size metrics

### Netlify Dashboard

View deployment status:
```
https://app.netlify.com → Your Site → Deploys
```

Each deploy shows:
- Build log
- Deploy time
- Preview URL
- Production URL (if main branch)

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| `netlify.toml` | ✅ Configured | WASM MIME types, CORS, SPA routing |
| `build.sh` | ✅ Working | Installs deps, builds WASM, optimizes |
| GitHub Actions Workflow | ⚠️ Needs Secrets | Add `NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID` |
| Preview Deployments | ⚠️ Blocked | Waiting for secrets configuration |
| Production (GitHub Pages) | ✅ Working | Independent of Netlify |

## Next Steps

1. **Configure secrets** (see "Required GitHub Secrets" above)
2. **Test deployment** (push to a test branch)
3. **Verify preview URL** (check PR comments or workflow logs)
4. **Set up custom domain** (optional, via Netlify dashboard)

## References

- [Netlify Build Configuration](https://docs.netlify.com/configure-builds/file-based-configuration/)
- [GitHub Actions + Netlify](https://github.com/nwtgck/actions-netlify)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [WASM Optimization](https://rustwasm.github.io/book/reference/code-size.html)

---

**Last Updated**: 2025-11-18
**Maintainer**: @prasincs
