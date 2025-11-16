# Netlify Setup for trustless-txir.netlify.app

## Quick Fix for Billing Issue

The error you saw:
```
The job was not started because recent account payments have failed or 
your spending limit needs to be increased.
```

**This happens even on the free tier**. Here's how to fix it:

### Option 1: Add Payment Method (Free Tier Still Free!)

1. Go to [app.netlify.com/teams/YOUR_TEAM/billing](https://app.netlify.com/teams)
2. Click on your team → **Billing**
3. Click **"Add payment method"**
4. Add a credit/debit card
5. **Important**: You'll stay on the free tier (0 builds used)
6. Netlify just requires payment info on file "just in case"

**Why this is safe**:
- ✅ Free tier is 300 build minutes/month (you'll use ~10/month)
- ✅ They don't charge unless you exceed limits
- ✅ You can set spending limits to $0
- ✅ You'll get email alerts before any charges

### Option 2: Get Site ID from Existing Site

Since you already have `trustless-txir.netlify.app`, let's use it!

1. Go to [app.netlify.com](https://app.netlify.com)
2. Click on **trustless-txir** site
3. Go to **Site Settings** → **General**
4. Find **Site ID** (looks like: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
5. Copy it

### Add GitHub Secrets

Go to your repository: **Settings → Secrets and variables → Actions**

Add these two secrets:

**NETLIFY_AUTH_TOKEN**
```bash
# Get from: https://app.netlify.com/user/applications
# Click "New access token"
# Name: "GitHub Actions - universal-blockchain-decoder"
# Copy the token
```

**NETLIFY_SITE_ID**
```bash
# Paste the site ID from trustless-txir site
# Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

## Testing Your Setup

Once secrets are added:

```bash
# Create a test branch
git checkout -b test/netlify-fix

# Make a small change
echo "// Test Netlify deployment" >> crates/universal-decoder-wasm/src/lib.rs

# Commit and push
git commit -am "test: Fix Netlify deployment"
git push -u origin test/netlify-fix
```

Watch the workflow at: https://github.com/prasincs/universal-blockchain-decoder/actions

## Expected Preview URL

Your preview URLs will be:
```
https://deploy-preview-{PR-NUMBER}--trustless-txir.netlify.app
https://{BRANCH-NAME}--trustless-txir.netlify.app
```

For example:
- PR #123: `https://deploy-preview-123--trustless-txir.netlify.app`
- Branch `test/netlify-fix`: `https://test-netlify-fix--trustless-txir.netlify.app`

## Troubleshooting

### Still getting billing error?

**Immediate fix**:
1. Go to [app.netlify.com/teams](https://app.netlify.com/teams)
2. Select your team
3. Go to **Billing** tab
4. Add payment method (even if free tier)
5. This unblocks deployments immediately

### Can't find Site ID?

Run this command to get it from Netlify CLI:

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login
netlify login

# List your sites
netlify sites:list

# Look for "trustless-txir" and copy the Site ID
```

### Token doesn't have permissions?

Regenerate with full access:
1. Go to [app.netlify.com/user/applications](https://app.netlify.com/user/applications)
2. Delete old token
3. Create new token
4. Copy immediately (you won't see it again)
5. Update `NETLIFY_AUTH_TOKEN` secret in GitHub

## Production Deployment

Your current workflow setup:
- **Branches/PRs**: Deploy to Netlify preview URLs
- **Main branch**: Deploy to GitHub Pages (production)

If you want main branch to also deploy to Netlify production:
- The workflow already handles this via `production-deploy: false` for previews
- Main branch deployments would need `production-deploy: true`

Current setup is good! Main → GitHub Pages, Previews → Netlify.

## Need Help?

Check workflow logs at:
https://github.com/prasincs/universal-blockchain-decoder/actions

Look for:
- ✅ "Build WASM module" - Should complete in ~2 min
- ✅ "Deploy to Netlify" - Should show preview URL
- ❌ Any red X's - Check the error message

Common fixes:
- Billing error → Add payment method
- Auth error → Regenerate NETLIFY_AUTH_TOKEN
- Site not found → Add NETLIFY_SITE_ID secret
