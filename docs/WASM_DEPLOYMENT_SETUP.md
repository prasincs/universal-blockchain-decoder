# WASM Deployment Setup Guide

## Quick Feedback Deployment with Environment Variables

This guide shows how to set up **preview deployments** for the Universal Blockchain Decoder WASM demo using environment variables. This enables:

- ✅ **No build artifacts in git** (WASM blobs stay out of version control)
- ✅ **Preview URLs for every PR/branch** (test before merging)
- ✅ **Fast feedback loop** (deploy in < 2 minutes)
- ✅ **Multiple deployment targets** (Netlify, Vercel, GitHub Pages)

---

## Table of Contents

1. [Netlify Setup (Recommended)](#netlify-setup-recommended)
2. [Vercel Setup (Alternative)](#vercel-setup-alternative)
3. [GitHub Pages Setup (Production Only)](#github-pages-setup-production-only)
4. [Local Development](#local-development)
5. [Manual Deployment Script](#manual-deployment-script)
6. [Troubleshooting](#troubleshooting)

---

## Netlify Setup (Recommended)

Netlify provides the **fastest and easiest** preview deployments with automatic PR comments.

### Step 1: Create Netlify Site

1. **Sign up/Login** at [netlify.com](https://netlify.com)

2. **Create a new site**:
   - Option A: Manual deploy (drag & drop `crates/universal-decoder-wasm/www/` after building)
   - Option B: Skip this step and let the GitHub Action create the site automatically

3. **Get your Site ID** (if you created manually):
   ```bash
   # Navigate to: Site Settings → General → Site Details
   # Copy the "Site ID" (format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)
   ```

### Step 2: Get Netlify Auth Token

1. Go to [app.netlify.com/user/applications](https://app.netlify.com/user/applications)
2. Click **"New access token"**
3. Give it a descriptive name: `GitHub Actions - universal-blockchain-decoder`
4. Click **"Generate token"**
5. **Copy the token** (you won't see it again!)

### Step 3: Add GitHub Secrets

1. Go to your repository: **Settings → Secrets and variables → Actions**

2. Click **"New repository secret"** and add:

   **NETLIFY_AUTH_TOKEN**
   ```
   <paste your Netlify token here>
   ```

   **NETLIFY_SITE_ID** (optional, but recommended)
   ```
   <paste your Netlify site ID here>
   ```

   > **Note**: If you skip `NETLIFY_SITE_ID`, Netlify will create a new site on first deployment.

### Step 4: Enable Workflow

The workflow is already created at `.github/workflows/deploy-wasm-preview-netlify.yml`.

**It will automatically**:
- ✅ Build WASM on every PR or branch push
- ✅ Deploy to a unique preview URL
- ✅ Comment on PRs with the preview link
- ✅ Show bundle size in workflow logs

### Step 5: Test It!

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
https://deploy-preview-123--your-site-name.netlify.app
```

---

## Vercel Setup (Alternative)

Vercel is a great alternative if you prefer their platform or already use it.

### Step 1: Create Vercel Project

1. **Sign up/Login** at [vercel.com](https://vercel.com)

2. **Import your repository**:
   - Go to [vercel.com/new](https://vercel.com/new)
   - Select your GitHub repository
   - Configure:
     - **Framework Preset**: Other
     - **Root Directory**: `crates/universal-decoder-wasm/www`
     - **Build Command**: (leave empty - we build in GitHub Actions)
     - **Output Directory**: `.` (current directory)
   - Click **"Deploy"**

3. **Get Project IDs**:
   ```bash
   # After import, go to: Project Settings → General
   # Copy the "Project ID" and "Team/Org ID"
   ```

### Step 2: Get Vercel Auth Token

1. Go to [vercel.com/account/tokens](https://vercel.com/account/tokens)
2. Click **"Create Token"**
3. Give it a name: `GitHub Actions - universal-blockchain-decoder`
4. Set scope: **Full Account** (or specific team)
5. Click **"Create"**
6. **Copy the token**

### Step 3: Add GitHub Secrets

1. Go to: **Repository Settings → Secrets and variables → Actions**

2. Add three secrets:

   **VERCEL_TOKEN**
   ```
   <paste your Vercel token here>
   ```

   **VERCEL_ORG_ID**
   ```
   <paste your Vercel organization/team ID>
   ```

   **VERCEL_PROJECT_ID**
   ```
   <paste your Vercel project ID>
   ```

### Step 4: Enable Workflow

The workflow is at `.github/workflows/deploy-wasm-preview-vercel.yml`.

**It will automatically**:
- ✅ Build WASM on every PR or branch push
- ✅ Deploy to Vercel preview
- ✅ Comment on PRs with preview URL
- ✅ Show bundle size in logs

### Step 5: Test It!

Same process as Netlify (see above).

**Preview URL Format**:
```
https://universal-blockchain-decoder-git-branch-name-yourteam.vercel.app
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
| `NETLIFY_AUTH_TOKEN` | Yes | Personal access token | [app.netlify.com/user/applications](https://app.netlify.com/user/applications) |
| `NETLIFY_SITE_ID` | No* | Site ID (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx) | Site Settings → General → Site Details |
| `NETLIFY_PRODUCTION` | No | Set to `"true"` for production deploy | Default: `"false"` |

*If not provided, Netlify will create a new site automatically.

### Vercel

| Variable | Required | Description | Where to Find |
|----------|----------|-------------|---------------|
| `VERCEL_TOKEN` | Yes | Personal access token | [vercel.com/account/tokens](https://vercel.com/account/tokens) |
| `VERCEL_ORG_ID` | Yes | Team/Organization ID | Project Settings → General |
| `VERCEL_PROJECT_ID` | Yes | Project ID | Project Settings → General |
| `VERCEL_PRODUCTION` | No | Set to `"true"` for production deploy | Default: `"false"` |

### GitHub Pages

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | Automatically provided by GitHub Actions |
| `GITHUB_PAGES_BRANCH` | No | Branch to deploy to (default: `gh-pages`) |

---

## Troubleshooting

### "NETLIFY_AUTH_TOKEN environment variable is required"

**Cause**: GitHub secret not set or workflow doesn't have access

**Fix**:
1. Verify secret exists: **Settings → Secrets and variables → Actions**
2. Check secret name matches exactly: `NETLIFY_AUTH_TOKEN` (case-sensitive)
3. Re-save the secret and re-run workflow

### "Vercel authentication failed"

**Cause**: Invalid token or expired token

**Fix**:
1. Generate a new token at [vercel.com/account/tokens](https://vercel.com/account/tokens)
2. Update `VERCEL_TOKEN` secret in GitHub
3. Re-run workflow

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

## Support

- **Documentation**: See `docs/WASM_DEMO.md` for architecture details
- **Issues**: Report at [GitHub Issues](https://github.com/prasincs/universal-blockchain-decoder/issues)
- **Deployment Guide**: See `wasm/DEPLOY.md` for alternative deployment methods

---

**Last Updated**: 2025-11-16
**Version**: 1.0.0
**Workflows**: Netlify, Vercel, GitHub Pages
