# WASM Deployment Setup Guide

## Quick Feedback Deployment with Cloudflare Pages

This guide shows how to set up **preview deployments** for the Universal Blockchain Decoder WASM demo using Cloudflare Pages and environment variables. This enables:

- ✅ **No build artifacts in git** (WASM blobs stay out of version control)
- ✅ **Preview URLs for every PR/branch** (test before merging)
- ✅ **Fast feedback loop** (deploy in < 2 minutes)
- ✅ **Auto PR comments** with preview links
- ✅ **No payment info required** (truly free tier)
- ✅ **Global CDN** (Cloudflare's edge network)

> **Why Cloudflare Pages?** Netlify requires billing information even for the free tier. Cloudflare Pages has a genuinely free tier with no payment info required, unlimited sites, and unlimited requests.

---

## Table of Contents

1. [Cloudflare Pages Setup (Preview Deployments)](#cloudflare-pages-setup-preview-deployments)
2. [GitHub Pages Setup (Production Only)](#github-pages-setup-production-only)
3. [Local Development](#local-development)
4. [Manual Deployment Script](#manual-deployment-script)
5. [Troubleshooting](#troubleshooting)
6. [Alternative Platforms (Netlify, Vercel, etc.)](#alternative-platforms)

---

## Cloudflare Pages Setup (Preview Deployments)

Cloudflare Pages provides **completely free** preview deployments with automatic PR comments and **no payment information required**.

### Step 1: Create Cloudflare Account

1. **Sign up** at [cloudflare.com](https://dash.cloudflare.com/sign-up/pages) (free account, no credit card)
2. **Verify your email**
3. **Login** to the Cloudflare dashboard

### Step 2: Get Cloudflare API Token

1. Go to [dash.cloudflare.com/profile/api-tokens](https://dash.cloudflare.com/profile/api-tokens)
2. Click **"Create Token"**
3. Click **"Use template"** next to **"Edit Cloudflare Workers"**
   - Or create custom token with these permissions:
     - Account Settings: Read
     - Cloudflare Pages: Edit
4. Click **"Continue to summary"**
5. Click **"Create Token"**
6. **Copy the token** (you won't see it again!)

### Step 3: Get Cloudflare Account ID

1. Go to [dash.cloudflare.com](https://dash.cloudflare.com)
2. Select any website (or click "Workers & Pages" in sidebar)
3. Scroll down on the right side → **Account ID**
4. Click to copy (format: `32-character hex string`)

### Step 4: Add GitHub Secrets

1. Go to your repository: **Settings → Secrets and variables → Actions**

2. Click **"New repository secret"** and add **two secrets**:

   **CLOUDFLARE_API_TOKEN**
   ```
   <paste your Cloudflare API token here>
   ```

   **CLOUDFLARE_ACCOUNT_ID**
   ```
   <paste your Cloudflare account ID here>
   ```

### Step 5: Enable Workflow

The workflow is already created at `.github/workflows/deploy-wasm-preview-cloudflare.yml`.

**It will automatically**:
- ✅ Build WASM on every PR or branch push
- ✅ Deploy to Cloudflare Pages
- ✅ Create unique preview URL per branch
- ✅ Comment on PRs with the preview link
- ✅ Show bundle size in workflow logs
- ✅ Use Cloudflare's global CDN (fast worldwide)

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
   - PR comment (auto-posted by Cloudflare Pages action)

**Preview URL Format**:
```
https://universal-blockchain-decoder.pages.dev
https://branch-name.universal-blockchain-decoder.pages.dev
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

### Cloudflare Pages

| Variable | Required | Description | Where to Find |
|----------|----------|-------------|---------------|
| `CLOUDFLARE_API_TOKEN` | Yes | API token with Pages edit permissions | [dash.cloudflare.com/profile/api-tokens](https://dash.cloudflare.com/profile/api-tokens) |
| `CLOUDFLARE_ACCOUNT_ID` | Yes | Your Cloudflare account ID (32-char hex) | Dashboard → Account ID (right sidebar) |

### GitHub Pages

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | Automatically provided by GitHub Actions |
| `GITHUB_PAGES_BRANCH` | No | Branch to deploy to (default: `gh-pages`) |

---

## Troubleshooting

### "Cloudflare API authentication failed"

**Cause**: Invalid API token or missing permissions

**Fix**:
1. Verify secret exists: **Settings → Secrets and variables → Actions**
2. Check secret names match exactly: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` (case-sensitive)
3. Regenerate API token with correct permissions:
   - Account Settings: Read
   - Cloudflare Pages: Edit
4. Re-save the secret and re-run workflow

### "Cloudflare account ID not found"

**Cause**: Wrong account ID or account doesn't exist

**Fix**:
1. Go to [dash.cloudflare.com](https://dash.cloudflare.com)
2. Copy Account ID from right sidebar (32-character hex string)
3. Update `CLOUDFLARE_ACCOUNT_ID` secret in GitHub

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

**Netlify**: Automatic (keeps last 10 previews per PR)

**Vercel**: Configure in project settings:
- Go to: Project Settings → Git → Preview Deployments
- Enable: "Auto-delete preview deployments after 30 days"

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

### Netlify (Requires Billing Info)

**Status**: Disabled (workflow commented out)

**Why not used**:
- ❌ Requires billing/payment information even for free tier
- ❌ Blocks deployments if payment info not on file
- ✅ Cloudflare Pages provides same features without billing requirement

**Workflow**: `.github/workflows/deploy-wasm-preview-netlify.yml` (disabled via `workflow_dispatch` only)

If you have Netlify billing configured, you can re-enable it by changing the workflow trigger.

### Vercel (Not Yet Implemented)

Vercel can be added as an alternative deployment platform if needed:

**What you'd need**:
- Vercel account and auth token
- GitHub secrets: `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID`
- Workflow file (see `docs/NETLIFY_BUILD_COMPARISON.md` for reference)

**Why it's not enabled now**:
- Cloudflare Pages already provides everything we need
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

**Last Updated**: 2025-11-16
**Version**: 2.0.0
**Workflows**: Cloudflare Pages (preview), GitHub Pages (production)
