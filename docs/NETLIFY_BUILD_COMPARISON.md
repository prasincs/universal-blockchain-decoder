# Netlify Build Strategies: GitHub Actions vs Native Build

This document compares two approaches for deploying the WASM demo to Netlify.

---

## TL;DR Recommendation

**For most cases**: Use **Option A (GitHub Actions Build)** ✅

**Why**: Faster builds, full control, multi-platform support, easier debugging.

**When to use Option B**: If you want to minimize GitHub Actions usage or prefer Netlify's build dashboard.

---

## Option A: Build in GitHub Actions, Deploy to Netlify

### How It Works

```
GitHub Push → GitHub Actions → Build WASM → Deploy to Netlify
```

**Workflow**: `.github/workflows/deploy-wasm-preview-netlify.yml` (already created)

### Setup

1. **Add GitHub Secrets** (required):
   ```
   NETLIFY_AUTH_TOKEN=your-token
   NETLIFY_SITE_ID=your-site-id (optional)
   ```

2. **That's it!** No Netlify configuration needed.

### ✅ Advantages

| Benefit | Details |
|---------|---------|
| **Fast builds** | GitHub Actions cache for Rust is excellent (~2 min builds) |
| **Full control** | Choose exact Rust version, wasm-pack version, optimization flags |
| **Multi-platform** | Same build deploys to Netlify, Vercel, GitHub Pages |
| **Consistent** | Identical builds across all platforms |
| **Easy debugging** | GitHub Actions logs are comprehensive |
| **No Netlify config** | No `netlify.toml` needed |
| **Works offline** | Can run `./scripts/deploy-wasm.sh netlify` locally |

### ❌ Disadvantages

| Drawback | Impact |
|----------|--------|
| **Two steps** | Build → Deploy (vs single step) |
| **More complex workflow** | ~90 lines of YAML |
| **GitHub Actions minutes** | Uses ~4-5 minutes per build (free tier: 2,000 min/month) |

### Build Performance

```
First build:  ~4 minutes (cold cache)
Cached build: ~2 minutes (warm cache)
Deploy:       ~30 seconds
Total:        ~2.5 minutes average
```

---

## Option B: Let Netlify Build from Source

### How It Works

```
GitHub Push → Netlify Webhook → Netlify Builds WASM → Deploy
```

**Configuration**: `crates/universal-decoder-wasm/netlify.toml` (created for you)

### Setup

#### Step 1: Create Netlify Site Manually

1. Go to [app.netlify.com](https://app.netlify.com)
2. Click **"Add new site" → "Import an existing project"**
3. Choose **GitHub** and select your repository
4. Configure build settings:

   ```
   Base directory:    crates/universal-decoder-wasm
   Build command:     ./build.sh
   Publish directory: crates/universal-decoder-wasm/www
   ```

5. **Advanced build settings**:
   ```
   Environment variables:
   - RUST_VERSION = stable
   - RUSTUP_TOOLCHAIN = stable
   ```

6. Click **"Deploy site"**

#### Step 2: Install Build Dependencies in Netlify

Netlify's build image **doesn't include** wasm-pack or binaryen by default.

**Option 2a**: Use Netlify build plugin (recommended)

Create `crates/universal-decoder-wasm/package.json`:
```json
{
  "name": "universal-decoder-wasm",
  "version": "0.1.0",
  "devDependencies": {},
  "scripts": {
    "build": "./build.sh"
  }
}
```

Modify `build.sh` to install dependencies:
```bash
#!/bin/bash
# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Install wasm-opt if not present
if ! command -v wasm-opt &> /dev/null; then
    npm install -g wasm-opt
fi

# Build
wasm-pack build --target web --out-dir www/pkg
```

**Option 2b**: Use Docker-based build

Update `netlify.toml`:
```toml
[build]
  command = "docker run --rm -v $(pwd):/workspace -w /workspace rust:latest ./build.sh"
```

#### Step 3: Connect to GitHub (Automatic Deploys)

1. In Netlify dashboard: **Site settings → Build & deploy → Continuous deployment**
2. Under **Build settings**:
   - **Branch**: `main` (production) or `*` (all branches)
   - **Deploy previews**: Enable for pull requests
3. Click **"Save"**

#### Step 4: Optional - Use GitHub Actions Trigger Only

Use the workflow: `.github/workflows/deploy-wasm-netlify-native.yml` (created for you)

This workflow **doesn't build**, it just triggers Netlify via webhook.

### ✅ Advantages

| Benefit | Details |
|---------|---------|
| **Simple workflow** | Just trigger, Netlify does the rest |
| **No GitHub Actions usage** | Saves GitHub Actions minutes |
| **Netlify-native** | Uses Netlify's build cache and infrastructure |
| **Centralized logs** | All build logs in Netlify dashboard |
| **Automatic deploys** | Push to main → auto-deploy (no workflow needed) |

### ❌ Disadvantages

| Drawback | Impact | Workaround |
|----------|--------|------------|
| **Slow builds** | ~5-8 minutes (Netlify cold start) | Use Netlify build plugins for caching |
| **Limited Rust support** | Netlify doesn't have Rust pre-installed | Install in build script |
| **No wasm-pack** | Must install in build command | Install via curl in `build.sh` |
| **Hard to debug** | Netlify logs less detailed than GitHub Actions | Add `set -x` to `build.sh` |
| **No multi-platform** | Only deploys to Netlify | Can't reuse build for Vercel/GitHub Pages |
| **Netlify build minutes** | Free tier: 300 min/month (vs GitHub: 2,000) | Upgrade or use Option A |

### Build Performance

```
First build:  ~8 minutes (install Rust + wasm-pack + build)
Cached build: ~5 minutes (Netlify cache isn't as good for Rust)
Deploy:       Instant (same machine)
Total:        ~5-8 minutes average
```

---

## Side-by-Side Comparison

| Feature | Option A (GitHub Actions) | Option B (Netlify Native) |
|---------|---------------------------|---------------------------|
| **Build time (cached)** | ~2 minutes ✅ | ~5 minutes ❌ |
| **Setup complexity** | Medium (90-line workflow) | Medium (netlify.toml + build deps) |
| **Debugging** | Easy (GitHub Actions logs) ✅ | Harder (Netlify logs) ❌ |
| **Multi-platform** | Yes (reuse build) ✅ | No (Netlify only) ❌ |
| **Build cache** | Excellent (Rust native) ✅ | Poor (generic cache) ❌ |
| **Free tier minutes** | 2,000/month ✅ | 300/month ❌ |
| **Local testing** | Yes (`./scripts/deploy-wasm.sh`) ✅ | No (Netlify only) ❌ |
| **Rust control** | Full (choose version) ✅ | Limited (install in build) ❌ |
| **Auto PR comments** | Yes ✅ | Yes ✅ |
| **Auto deploys** | Yes (via workflow) ✅ | Yes (native) ✅ |

---

## Hybrid Approach (Best of Both Worlds)

Use **Option A for development** + **Option B for production**:

1. **Development (PR/branches)**: GitHub Actions builds → Netlify preview
   - Fast builds
   - Full control
   - Auto PR comments

2. **Production (main branch)**: Netlify native build
   - Simple (no workflow)
   - Centralized logs
   - Netlify-optimized CDN

### Configuration

```yaml
# .github/workflows/deploy-wasm-preview-netlify.yml
on:
  pull_request:  # Only for PRs
  push:
    branches-ignore: [main]  # Exclude main branch
```

```toml
# netlify.toml
[context.production]
  command = "./build.sh"
  publish = "www"
  # Only builds main branch automatically
```

---

## Decision Matrix

**Choose Option A if**:
- ✅ You want **fast builds** (< 3 minutes)
- ✅ You need **multi-platform** support (Netlify + Vercel + GitHub Pages)
- ✅ You want **full control** over build environment
- ✅ You have **complex build requirements** (custom Rust version, flags)
- ✅ You're okay with **GitHub Actions workflows**

**Choose Option B if**:
- ✅ You want **zero GitHub Actions usage**
- ✅ You prefer **Netlify's dashboard** for build logs
- ✅ You want **automatic deploys** without workflows
- ✅ Your builds are **simple** (standard Rust stable + wasm-pack)
- ✅ You don't need to deploy to other platforms

**Choose Hybrid if**:
- ✅ You want **fast preview builds** (Option A)
- ✅ You want **simple production deploys** (Option B)
- ✅ You're comfortable managing both approaches

---

## My Recommendation

### For Your Use Case: **Option A (Current Setup)** ✅

**Why**:
1. **Faster builds**: 2 min vs 5-8 min (important for quick feedback!)
2. **Better cache**: GitHub Actions cache is excellent for Rust
3. **Multi-platform**: You can add Vercel/GitHub Pages later without duplicating builds
4. **Full control**: You control Rust version, wasm-opt flags, optimization
5. **Easier debugging**: GitHub Actions logs are more detailed
6. **More free tier minutes**: 2,000 vs 300 (Netlify is stingy)

**The current setup I created is production-ready** and gives you:
- ✅ Preview URLs for every PR/branch
- ✅ Auto PR comments with links
- ✅ Bundle size reporting
- ✅ Fast builds (< 3 min)
- ✅ Local deployment script

### When to Switch to Option B

Consider switching if:
- You hit GitHub Actions minute limits (unlikely at 2,000/month)
- You want to minimize workflow complexity (but it's already done!)
- You strongly prefer Netlify's dashboard

---

## What About GitHub Pages?

**GitHub Pages is best for production-only deployments**:

```yaml
# .github/workflows/deploy-wasm-demo.yml (existing)
on:
  push:
    branches: [main]  # Only main branch
```

**Use GitHub Pages for**:
- ✅ **Free production hosting** (unlimited bandwidth)
- ✅ **Official demo URL** (yourusername.github.io/repo)
- ✅ **No external dependencies** (no Netlify/Vercel account needed)

**Use Netlify/Vercel for**:
- ✅ **Preview URLs** (test before merging)
- ✅ **Branch deploys** (share work-in-progress)
- ✅ **Custom domains** (easier than GitHub Pages)

---

## Final Workflow Recommendation

### Production Setup (What I Recommend)

```
┌─────────────────────────────────────────────────────┐
│ PR/Branch Push                                       │
│  ├─ GitHub Actions builds WASM                      │
│  ├─ Deploys to Netlify preview                      │
│  └─ Comments on PR with URL                         │
│                                                      │
│ Main Branch Push                                    │
│  ├─ GitHub Actions builds WASM                      │
│  └─ Deploys to GitHub Pages (production)            │
└─────────────────────────────────────────────────────┘
```

**Workflows to keep**:
- ✅ `.github/workflows/deploy-wasm-preview-netlify.yml` (for PRs/branches)
- ✅ `.github/workflows/deploy-wasm-demo.yml` (for production/main)

**Workflows to remove**:
- ❌ `.github/workflows/deploy-wasm-netlify-native.yml` (only if you choose Option B)

**Files to keep**:
- ✅ `scripts/deploy-wasm.sh` (for local testing)
- ✅ `docs/WASM_DEPLOYMENT_SETUP.md` (setup guide)

**Files to remove**:
- ❌ `crates/universal-decoder-wasm/netlify.toml` (only if you stick with Option A)

---

## Summary

| Aspect | Current Setup (Option A) | Recommended Change |
|--------|--------------------------|-------------------|
| **Preview deploys** | GitHub Actions → Netlify ✅ | Keep as-is |
| **Production deploys** | GitHub Actions → GitHub Pages ✅ | Keep as-is |
| **Build time** | ~2 minutes ✅ | Already optimal |
| **Setup complexity** | Medium, already done ✅ | Already done |
| **Free tier usage** | Efficient ✅ | Already efficient |

**Action**: **No changes needed!** Your current setup is optimal.

**Optional**: Add Netlify production deploys (main branch) if you prefer Netlify over GitHub Pages for production.

---

## Questions?

**Q: Can I use both Netlify and GitHub Pages for production?**

Yes! Deploy to both:
- Netlify: `https://decoder.netlify.app` (custom domain-friendly)
- GitHub Pages: `https://yourusername.github.io/repo` (official/canonical)

**Q: Should I delete the netlify.toml file I just created?**

If you're sticking with **Option A** (recommended), yes - you don't need it.

**Q: What if I want to try Option B later?**

Keep the `netlify.toml` file, disable the GitHub Actions workflow, and follow the "Setup" steps in Option B above.

---

**Last Updated**: 2025-11-16
**Recommendation**: Option A (GitHub Actions Build)
