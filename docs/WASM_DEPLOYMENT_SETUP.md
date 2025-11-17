# WASM Deployment Setup Guide

## Quick Feedback Deployment with Netlify

This guide shows how to set up **preview deployments** for the Universal Blockchain Decoder WASM demo using Netlify and GitHub Pages. This enables:

- ✅ **No build artifacts in git** (WASM blobs stay out of version control)
- ✅ **Preview URLs for every PR/branch** (test before merging)
- ✅ **Fast feedback loop** (deploy in < 2 minutes)
- ✅ **Auto PR comments** with preview links
- ✅ **Global CDN** (edge network deployment)

> **Note**: This project previously used CloudFlare Pages but has migrated to Netlify for preview deployments.

---

## Table of Contents

1. [Netlify Setup (Preview Deployments)](#netlify-setup-preview-deployments)
2. [GitHub Pages Setup (Production Only)](#github-pages-setup-production-only)
3. [Local Development](#local-development)
4. [Manual Deployment Script](#manual-deployment-script)
5. [Troubleshooting](#troubleshooting)
6. [Alternative Platforms](#alternative-platforms)

---

## Netlify Setup (Preview Deployments)

Netlify provides preview deployments with automatic PR comments and global CDN.

### Step 1: Create Netlify Account

1. **Sign up** at [netlify.com](https://app.netlify.com/signup) (free account)
2. **Verify your email**
3. **Login** to the Netlify dashboard

### Step 2: Create a New Site

1. Go to [app.netlify.com/start](https://app.netlify.com/start)
2. Click **"Add new site"** → **"Import an existing project"**
3. Choose **GitHub** and authorize Netlify
4. Select your repository
5. Leave build settings empty (we handle builds in GitHub Actions)
6. Click **"Deploy site"**

### Step 3: Get Netlify Credentials

1. **Get Site ID**:
   - Go to: Site Settings → General → Site details
   - Copy **Site ID** (format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)

2. **Get Auth Token**:
   - Go to: User Settings → Applications → Personal access tokens
   - Click **"New access token"**
   - Name it (e.g., "GitHub Actions Deployment")
   - Copy the token (you won't see it again!)

### Step 4: Add GitHub Secrets

1. Go to your repository: **Settings → Secrets and variables → Actions**

2. Click **"New repository secret"** and add:

   **NETLIFY_AUTH_TOKEN**
   ```
   <paste your Netlify auth token here>
   ```

   **NETLIFY_SITE_ID**
   ```
   <paste your Netlify site ID here>
   ```

### Step 5: Workflows Are Ready

The workflows are already configured:
- `.github/workflows/deploy-wasm-preview-netlify.yml` (preview deployments)
- `.github/workflows/deploy-wasm-netlify-native.yml` (alternative)

**They will automatically**:
- ✅ Build WASM on every PR or branch push
- ✅ Deploy to Netlify
- ✅ Create unique preview URL per branch
- ✅ Comment on PRs with the preview link
- ✅ Show bundle size in workflow logs

### Step 6: Test It!

1. Create a new branch:
   ```bash
   git checkout -b test/wasm-deployment
   ```

2. Make a small change to the WASM demo:
   ```bash
   echo "// Test deployment" >> crates/universal-decoder-wasm/src/lib.rs
   git commit -am "test: Trigger WASM preview deployment"
   git push -u origin test/wasm-deployment
   ```

3. **Watch the workflow** run at: `https://github.com/YOUR_USERNAME/universal-blockchain-decoder/actions`

4. **Get your preview URL** from:
   - Workflow output (look for "🔗 Preview URL")
   - PR comment (auto-posted by Netlify action)

**Preview URL Format**:
```
https://your-site-name.netlify.app
https://deploy-preview-123--your-site-name.netlify.app
```

---

## GitHub Pages Setup (Production Only)

GitHub Pages is **free** and great for production deployments, but **doesn't support preview URLs**.

### Step 1: Enable GitHub Pages

1. Go to: **Repository Settings → Pages**

2. Under **"Source"**:
   - Branch: `gh-pages`
   - Folder: `/ (root)`
   - Click **"Save"**

3. Wait 1-2 minutes for initial deployment

### Step 2: Workflow Configuration

The existing workflow (`.github/workflows/deploy-wasm-demo.yml`) already:
- ✅ Builds WASM on `main` branch push
- ✅ Uploads to GitHub Pages artifact
- ✅ Deploys automatically

**No environment variables needed!**

### Step 3: Access Your Site

After deployment:
```
https://YOUR_USERNAME.github.io/universal-blockchain-decoder/
```

---

## Local Development

For quick local testing **without** deploying:

### Option 1: Use Deployment Script

```bash
# Build and serve locally
./scripts/deploy-wasm.sh local
```

This will:
1. Build WASM module
2. Start HTTP server on `http://localhost:8080`

### Option 2: Manual Steps

```bash
# Build WASM
cd crates/universal-decoder-wasm
./build.sh

# Serve locally
cd www
python3 -m http.server 8080
```

Visit: http://localhost:8080

---

## Manual Deployment Script

For manual deployments from your local machine:

### Deploy to Netlify (Manual)

```bash
# Set environment variables
export NETLIFY_AUTH_TOKEN="your-token-here"
export NETLIFY_SITE_ID="your-site-id-here"
export NETLIFY_PRODUCTION="false"  # or "true" for production

# Deploy
./scripts/deploy-wasm.sh netlify
```

### Deploy to Vercel (Manual)

```bash
# Set environment variables
export VERCEL_TOKEN="your-token-here"
export VERCEL_ORG_ID="your-org-id-here"
export VERCEL_PROJECT_ID="your-project-id-here"
export VERCEL_PRODUCTION="false"  # or "true" for production

# Deploy
./scripts/deploy-wasm.sh vercel
```

### Deploy to GitHub Pages (Manual)

```bash
# No environment variables needed
./scripts/deploy-wasm.sh github
```

---

## Environment Variables Reference

### Netlify

| Variable | Required | Description | Where to Find |
|----------|----------|-------------|---------------|
| `NETLIFY_AUTH_TOKEN` | Yes | Personal access token for deployments | User Settings → Applications → Personal access tokens |
| `NETLIFY_SITE_ID` | Yes | Your site ID (UUID format) | Site Settings → General → Site details |

### GitHub Pages

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | Automatically provided by GitHub Actions |
| `GITHUB_PAGES_BRANCH` | No | Branch to deploy to (default: `gh-pages`) |

---

## Troubleshooting

### "Netlify authentication failed"

**Cause**: Invalid auth token or missing site ID

**Fix**:
1. Verify secrets exist: **Settings → Secrets and variables → Actions**
2. Check secret names match exactly: `NETLIFY_AUTH_TOKEN` and `NETLIFY_SITE_ID` (case-sensitive)
3. Regenerate auth token:
   - Go to User Settings → Applications → Personal access tokens
   - Create new token
   - Update `NETLIFY_AUTH_TOKEN` secret in GitHub
4. Verify Site ID is correct (UUID format)

### "Netlify site not found"

**Cause**: Wrong site ID or site doesn't exist

**Fix**:
1. Go to [app.netlify.com](https://app.netlify.com)
2. Select your site → Site Settings → General
3. Copy Site ID (UUID format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
4. Update `NETLIFY_SITE_ID` secret in GitHub

### "GitHub Pages deployment failed: 404"

**Cause**: `gh-pages` branch doesn't exist

**Fix**:
```bash
# Create gh-pages branch manually
git checkout --orphan gh-pages
git rm -rf .
touch .nojekyll
git add .nojekyll
git commit -m "Initialize gh-pages"
git push -u origin gh-pages
git checkout main
```

### "WASM module not found in browser"

**Cause**: WASM files didn't copy correctly or wrong path

**Fix**:
1. Check workflow logs for build errors
2. Verify `pkg/` directory exists in deployment
3. Check browser DevTools console for exact error
4. Ensure web server serves `.wasm` files with correct MIME type

### "Bundle size too large"

**Cause**: WASM binary is larger than expected

**Fix**:
1. Verify `wasm-opt` is running (check workflow logs)
2. Check `Cargo.toml` has correct optimization settings:
   ```toml
   [profile.release]
   opt-level = "z"
   lto = true
   strip = true
   ```
3. Consider feature flags to exclude unused decoders

---

## Best Practices

### 1. **Never Commit Build Artifacts**

The `.gitignore` already excludes:
```gitignore
crates/universal-decoder-wasm/www/pkg/
*.wasm
```

**Keep it this way!** Let CI/CD handle builds.

### 2. **Use Preview Deployments for Testing**

- ✅ **DO**: Test changes on preview URLs before merging
- ❌ **DON'T**: Push directly to `main` without preview

### 3. **Monitor Bundle Size**

Check workflow logs for bundle size:
```
📦 WASM Bundle Size:
-rw-r--r-- 1 runner runner 358K universal_decoder_wasm_bg.wasm
```

**Target**: < 500KB uncompressed, < 150KB gzipped

### 4. **Secure Your Tokens**

- ✅ **DO**: Use GitHub Secrets (never commit tokens)
- ✅ **DO**: Rotate tokens every 6 months
- ✅ **DO**: Use minimal permissions (e.g., Netlify site-specific tokens)
- ❌ **DON'T**: Share tokens in Slack, Discord, or public forums
- ❌ **DON'T**: Use personal tokens for team projects (use org tokens)

### 5. **Test Locally First**

Before pushing for CI deployment:
```bash
# 1. Build locally
cd crates/universal-decoder-wasm
./build.sh

# 2. Test in browser
cd www
python3 -m http.server 8080
# Visit http://localhost:8080

# 3. Verify no errors in browser console
# 4. Test all example transactions
# 5. THEN push to GitHub
```

---

## Next Steps

### Enable Auto-Cleanup of Preview Deployments

**Netlify**: Automatic (keeps last 10 previews per PR, configurable in site settings)

### Add Custom Domain

**Netlify**:
1. Site Settings → Domain management → Add custom domain
2. Follow DNS setup instructions

**Vercel**:
1. Project Settings → Domains → Add
2. Configure DNS (automatic with Vercel nameservers)

**GitHub Pages**:
1. Create `CNAME` file in `wasm/` directory:
   ```
   decoder.yourdomain.com
   ```
2. Configure DNS:
   ```
   CNAME decoder.yourdomain.com → YOUR_USERNAME.github.io
   ```

### Set Up Performance Monitoring

Add analytics to track:
- Page load time
- WASM initialization time
- Decode operation latency
- User geographic distribution

**Recommended Tools**:
- [Plausible Analytics](https://plausible.io) (privacy-friendly)
- [Fathom Analytics](https://usefathom.com) (privacy-friendly)
- Google Analytics (if privacy isn't critical)

---

## Alternative Platforms

### CloudFlare Pages (Removed)

**Status**: No longer supported

**Reason**: Migrated to Netlify for preview deployments. The CloudFlare Pages workflow has been removed from CI.

**Previous workflow**: `.github/workflows/deploy-wasm-preview-cloudflare.yml` (removed)

### Vercel (Not Implemented)

Vercel can be added as an alternative deployment platform if needed:

**What you'd need**:
- Vercel account and auth token
- GitHub secrets: `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID`
- Workflow file (see `docs/NETLIFY_BUILD_COMPARISON.md` for reference)

**Why it's not enabled now**:
- Netlify already provides everything we need
- No need to maintain multiple similar platforms
- Can add later if required

### Other Platforms

The deployment script (`scripts/deploy-wasm.sh`) can be extended to support:
- **AWS S3 + CloudFront**: For enterprise deployments
- **Azure Static Web Apps**: For Microsoft-centric teams
- **Firebase Hosting**: Google Cloud integration
- **Self-hosted**: Any web server with static file support

All platforms receive the same pre-built WASM artifacts from GitHub Actions.

---

## Support

- **Documentation**: See `docs/WASM_DEMO.md` for architecture details
- **Issues**: Report at [GitHub Issues](https://github.com/prasincs/universal-blockchain-decoder/issues)
- **Deployment Guide**: See `wasm/DEPLOY.md` for alternative deployment methods
- **Build Comparison**: See `docs/NETLIFY_BUILD_COMPARISON.md` for platform comparisons

---

**Last Updated**: 2025-11-17
**Version**: 2.1.0
**Workflows**: Netlify (preview), GitHub Pages (production)
