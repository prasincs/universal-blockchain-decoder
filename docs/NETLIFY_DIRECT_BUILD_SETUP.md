# Netlify Direct Build Setup (No GitHub Actions)

## Your Site: trustless-txir.netlify.app

Since you can't run GitHub Actions, we'll configure Netlify to build directly from your GitHub repository.

---

## Step 1: Fix Billing Issue (Required!)

**The billing error blocks ALL builds**, even free tier. You must add payment info:

1. Go to: [app.netlify.com/teams](https://app.netlify.com/teams)
2. Click on your team
3. Click **"Billing"** tab
4. Click **"Add payment method"**
5. Add credit/debit card

**Don't worry**:
- ✅ You'll stay on FREE tier (300 min/month)
- ✅ You use ~2-3 min per build
- ✅ Won't charge unless you exceed free tier
- ✅ Can set spending limit to $0 after adding card

**This is the #1 blocker** - Netlify won't build without payment info on file.

---

## Step 2: Connect GitHub Repository

1. Go to [app.netlify.com/sites/trustless-txir](https://app.netlify.com)
2. Click **"Site settings"**
3. Click **"Build & deploy"** in sidebar
4. Under **"Continuous deployment"**:
   - Click **"Link repository"** (if not linked)
   - Choose **GitHub**
   - Select: `prasincs/universal-blockchain-decoder`
   - Authorize Netlify to access the repo

---

## Step 3: Configure Build Settings

### Option A: Use Netlify Dashboard (Quick)

1. In **Site settings → Build & deploy → Build settings**:
   
   **Base directory**:
   ```
   crates/universal-decoder-wasm
   ```
   
   **Build command**:
   ```bash
   chmod +x build.sh && ./build.sh
   ```
   
   **Publish directory**:
   ```
   www
   ```

2. Click **"Save"**

### Option B: Use netlify.toml (Better)

The `netlify.toml` file is already in your repo at:
```
crates/universal-decoder-wasm/netlify.toml
```

But Netlify needs to find it. Move it to the repository root:

```bash
# From your repo root
mv crates/universal-decoder-wasm/netlify.toml netlify.toml
git add netlify.toml
git commit -m "config: Move netlify.toml to root for Netlify builds"
git push
```

The file contains:
```toml
[build]
  base = "crates/universal-decoder-wasm"
  command = "./build.sh"
  publish = "www"

[build.environment]
  RUST_VERSION = "stable"
```

---

## Step 4: Install Build Dependencies in Netlify

Netlify's build image doesn't have Rust/wasm-pack by default. Update `build.sh` to install them:

**Edit**: `crates/universal-decoder-wasm/build.sh`

Add this at the top:

```bash
#!/bin/bash
set -e

echo "📦 Installing build dependencies..."

# Install Rust (if not present)
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (if not present)
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Install wasm-opt (optional but recommended)
if ! command -v wasm-opt &> /dev/null; then
    echo "Installing binaryen (wasm-opt)..."
    npm install -g wasm-opt || echo "⚠️ wasm-opt install failed, continuing..."
fi

echo "✅ Dependencies installed"
echo ""
echo "🔨 Building WASM module..."

# ... rest of your build script
wasm-pack build --target web --out-dir www/pkg

echo ""
echo "📦 WASM Bundle Size:"
ls -lh www/pkg/*.wasm
echo ""
echo "✅ Build complete!"
```

---

## Step 5: Configure Branch Deploys

1. In **Site settings → Build & deploy → Continuous deployment**
2. Under **"Deploy contexts"**:
   - **Production branch**: `main`
   - **Branch deploys**: `All` (or specific branches)
   - **Deploy previews**: Enable for pull requests

This gives you:
- Main branch → `https://trustless-txir.netlify.app` (production)
- Other branches → `https://BRANCH-NAME--trustless-txir.netlify.app` (preview)
- PRs → `https://deploy-preview-123--trustless-txir.netlify.app` (preview)

---

## Step 6: Trigger First Build

### Option 1: Manual Trigger

1. Go to **Deploys** tab in Netlify
2. Click **"Trigger deploy"** → **"Clear cache and deploy site"**

### Option 2: Push to GitHub

```bash
git checkout -b test/netlify-direct-build
echo "// Test Netlify direct build" >> crates/universal-decoder-wasm/src/lib.rs
git commit -am "test: Trigger Netlify direct build"
git push -u origin test/netlify-direct-build
```

Netlify will automatically:
1. Detect the push
2. Run the build
3. Deploy to `https://test-netlify-direct-build--trustless-txir.netlify.app`

---

## Expected Build Time

First build: ~8-10 minutes (installing Rust, wasm-pack, etc.)
Subsequent builds: ~5-7 minutes (Netlify caches dependencies)

**This is slower than GitHub Actions** (~2 min) because Netlify's cache isn't optimized for Rust.

---

## Troubleshooting

### Build fails: "command not found: rustc"

**Fix**: Ensure `build.sh` installs Rust (see Step 4 above)

### Build fails: "permission denied: build.sh"

**Fix**: Make build.sh executable:
```bash
chmod +x crates/universal-decoder-wasm/build.sh
git add crates/universal-decoder-wasm/build.sh
git commit -m "fix: Make build.sh executable"
git push
```

### Build timeout after 15 minutes

**Fix**: Netlify free tier has 15-minute timeout. Optimize:
```bash
# In netlify.toml
[build]
  command = "cargo install wasm-pack --locked && ./build.sh"
```

Or upgrade to Netlify Pro (longer timeouts).

### Still getting "payment failed" error?

**Fix**: You MUST add payment method. Even with $0 balance, Netlify requires card on file.

1. Go to billing: [app.netlify.com/teams/YOUR_TEAM/billing](https://app.netlify.com/teams)
2. Add card
3. Set spending limit to $0 (prevents accidental charges)
4. Retry build

---

## Verification

Once build succeeds:

1. **Check deploy log**:
   - Go to **Deploysmanship tab → Latest deploy
   - Look for "✅ Build complete!"
   - Should show WASM bundle size

2. **Test the site**:
   - Visit: `https://trustless-txir.netlify.app`
   - Open browser console (F12)
   - Should see: "WASM module initialized successfully"
   - Try loading an example transaction

3. **Test preview deploys**:
   - Create a PR
   - Netlify auto-comments with preview URL
   - Test preview before merging

---

## Production vs Preview URLs

**Production** (main branch):
```
https://trustless-txir.netlify.app
```

**Branch previews**:
```
https://BRANCH-NAME--trustless-txir.netlify.app
```

**PR previews**:
```
https://deploy-preview-PR-NUMBER--trustless-txir.netlify.app
```

---

## Alternative: Use Netlify CLI for Local Testing

Test builds locally before pushing:

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login
netlify login

# Link to your site
netlify link

# Test build locally
netlify build

# Deploy to Netlify
netlify deploy --prod
```

---

## Need More Help?

1. **Netlify Community**: https://answers.netlify.com
2. **Netlify Status**: https://status.netlify.com
3. **Build logs**: Check deploy log for exact error
4. **Netlify Support**: support@netlify.com (respond within 24h)

---

## Summary Checklist

- [ ] Add payment method to Netlify account (CRITICAL!)
- [ ] Connect GitHub repo to Netlify
- [ ] Configure build settings (base dir, build command, publish dir)
- [ ] Update build.sh to install Rust + wasm-pack
- [ ] Move netlify.toml to repo root (optional but recommended)
- [ ] Enable branch deploys and PR previews
- [ ] Trigger first build (manual or git push)
- [ ] Verify build succeeds (check deploy log)
- [ ] Test deployed site at trustless-txir.netlify.app

Once complete, every push to GitHub automatically triggers a Netlify build!
