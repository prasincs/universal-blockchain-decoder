# Universal Blockchain Decoder - WASM Demo Deployment Guide

> **🚀 NEW: Automated Preview Deployments**
>
> For **quick feedback** and **preview URLs** using environment variables (Netlify/Vercel), see:
> **[`docs/WASM_DEPLOYMENT_SETUP.md`](../docs/WASM_DEPLOYMENT_SETUP.md)**
>
> This guide covers **manual deployments** for production environments.

---

## Quick Start - Manual Deployment

This directory contains a fully functional WASM-based blockchain transaction decoder that runs entirely in the browser.

### What's Included

- `index.html` - Main web interface
- `main.js` - JavaScript application logic
- `examples.js` - Pre-loaded example transactions
- `style.css` - Responsive dark theme styling
- `pkg/` - Compiled WASM module (358KB)
  - `universal_decoder_wasm_bg.wasm` - The WebAssembly binary
  - `universal_decoder_wasm.js` - JavaScript bindings
  - `*.d.ts` - TypeScript type definitions

### Supported Chains

- ✅ Bitcoin (Legacy, SegWit)
- ✅ Ethereum (Legacy, EIP-1559)
- ✅ Solana
- ✅ Cosmos SDK

---

## Deployment Options

### Option 1: GitHub Pages (Recommended)

#### Using gh-pages Branch

```bash
# From the repository root
cd /home/user/universal-blockchain-decoder

# Create orphan gh-pages branch
git checkout --orphan gh-pages

# Remove all tracked files
git rm -rf .

# Copy WASM demo files
cp -r wasm/* .

# Add and commit
git add .
git commit -m "Deploy WASM demo to GitHub Pages"

# Push to gh-pages branch
git push -u origin gh-pages --force

# Return to main branch
git checkout main
```

**Enable GitHub Pages**:
1. Go to repository Settings
2. Navigate to "Pages" section
3. Under "Source", select branch: `gh-pages`, folder: `/ (root)`
4. Click "Save"
5. Your site will be published at: `https://prasincs.github.io/universal-blockchain-decoder/`

#### Using Subtree (Alternative)

```bash
# One-time setup
git subtree add --prefix wasm origin gh-pages

# To update deployment
git subtree push --prefix wasm origin gh-pages
```

---

### Option 2: Deploy to Netlify

1. **Via Drag & Drop**:
   - Go to [Netlify Drop](https://app.netlify.com/drop)
   - Drag the `/wasm` directory onto the page
   - Get instant URL like `https://random-name.netlify.app`

2. **Via Netlify CLI**:
   ```bash
   npm install -g netlify-cli
   cd /home/user/universal-blockchain-decoder/wasm
   netlify deploy --prod
   ```

---

### Option 3: Deploy to Vercel

```bash
npm install -g vercel
cd /home/user/universal-blockchain-decoder/wasm
vercel --prod
```

---

### Option 4: Deploy to Cloudflare Pages

1. Push code to GitHub/GitLab
2. Go to [Cloudflare Pages](https://pages.cloudflare.com/)
3. Create new project → Connect repository
4. Build settings:
   - Framework: None (static)
   - Build directory: `wasm`
   - Build command: (leave empty)
5. Deploy!

---

### Option 5: Self-Hosted (Any Web Server)

Simply upload the contents of this `/wasm` directory to any web server that can serve static files.

**Apache (.htaccess)**:
```apache
<IfModule mod_mime.c>
    AddType application/wasm .wasm
</IfModule>

# Enable CORS if needed
<IfModule mod_headers.c>
    Header set Access-Control-Allow-Origin "*"
</IfModule>
```

**Nginx**:
```nginx
server {
    listen 80;
    server_name your-domain.com;

    root /var/www/decoder;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }

    location ~* \.wasm$ {
        types {
            application/wasm wasm;
        }
        add_header 'Access-Control-Allow-Origin' '*';
    }
}
```

---

## Local Testing

Before deploying, test locally:

### Using Python (Built-in)

```bash
cd /home/user/universal-blockchain-decoder/wasm
python3 -m http.server 8080
```

Visit: http://localhost:8080

### Using Node.js (http-server)

```bash
npm install -g http-server
cd /home/user/universal-blockchain-decoder/wasm
http-server -p 8080
```

### Using PHP (Built-in)

```bash
cd /home/user/universal-blockchain-decoder/wasm
php -S localhost:8080
```

---

## Manual GitHub Pages Deployment (Without Actions)

If you can't use GitHub Actions runners, here's the manual process:

### Step 1: Prepare gh-pages Branch

```bash
# Create and switch to gh-pages branch
git checkout -b gh-pages

# Remove everything except wasm directory
find . -maxdepth 1 ! -name 'wasm' ! -name '.git' ! -name '.' -exec rm -rf {} +

# Move wasm contents to root
mv wasm/* .
mv wasm/.* . 2>/dev/null || true
rmdir wasm

# Create .nojekyll to bypass Jekyll processing
touch .nojekyll

# Add everything
git add -A
git commit -m "Initial WASM demo deployment"

# Push to GitHub
git push origin gh-pages --force
```

### Step 2: Configure GitHub Pages

1. Go to: `https://github.com/prasincs/universal-blockchain-decoder/settings/pages`
2. Under "Source":
   - Branch: `gh-pages`
   - Folder: `/ (root)`
3. Click "Save"
4. Wait 1-2 minutes for deployment
5. Visit: `https://prasincs.github.io/universal-blockchain-decoder/`

### Step 3: Update Deployment (Future Changes)

```bash
# Switch to gh-pages branch
git checkout gh-pages

# Pull latest changes
git pull origin gh-pages

# Copy updated WASM files from main branch
git checkout main -- wasm
mv wasm/* .
rmdir wasm

# Commit and push
git add -A
git commit -m "Update WASM demo"
git push origin gh-pages
```

---

## Verifying Deployment

Once deployed, test the following:

1. **Load Test**: Page loads without errors
2. **WASM Init**: Check browser console for initialization message
3. **Chain Selection**: All 4 chains appear in dropdown
4. **Example Loading**: Load pre-defined examples
5. **Decoding**: Click "Decode Transaction" successfully
6. **Auto-Detect**: "Auto-detect Chain" button works
7. **JSON Output**: Decoded output displays correctly

### Browser Console Checks

Open DevTools (F12) and check:

```javascript
// Should see:
// "WASM module initialized successfully"

// Verify WASM is loaded
console.log(typeof wasm !== 'undefined'); // true

// Test supported chains
console.log(supported_chains());
// ["bitcoin", "ethereum", "solana", "cosmos"]
```

---

## Troubleshooting

### Issue: "Failed to fetch WASM"

**Cause**: Web server not serving `.wasm` files with correct MIME type

**Fix**: Ensure server sends `Content-Type: application/wasm` for `.wasm` files

### Issue: "WASM module not found"

**Cause**: Incorrect path to WASM files

**Fix**: Check that `pkg/` directory is at the same level as `index.html`

### Issue: "CORS policy error"

**Cause**: Trying to load local file:// URLs

**Fix**: Must use HTTP server (see "Local Testing" section above)

### Issue: "Module is not defined"

**Cause**: Browser doesn't support ES6 modules

**Fix**: Use modern browser (Chrome 61+, Firefox 60+, Safari 11+, Edge 79+)

---

## Bundle Size

- **WASM Binary**: 358KB (uncompressed)
- **JavaScript**: 16KB
- **Total Download**: ~374KB (compresses to ~100KB with gzip)

**Performance**:
- Load time (3G): < 5 seconds
- Decode time: < 100ms per transaction
- Memory usage: ~10MB

---

## Security Notes

✅ **Zero-Trust Architecture**: All decoding happens client-side in browser
✅ **No Server Communication**: Transaction data never leaves your machine
✅ **Privacy-Preserving**: Perfect for sensitive transactions
✅ **Auditable**: View source code directly in browser DevTools
✅ **Offline Capable**: Works after initial load (PWA-ready)

---

## Next Steps

### Adding More Chains

To add support for additional blockchains:

1. Implement decoder in Rust (see `crates/decoder-*/`)
2. Add to `Cargo.toml` dependencies
3. Import in `src/lib.rs`
4. Add to `decode_transaction()` match statement
5. Rebuild: `wasm-pack build --target web --out-dir www/pkg`
6. Copy to `/wasm` and redeploy

### Customization

- **Styling**: Edit `style.css` (dark theme variables at top)
- **Examples**: Modify `examples.js` to add more sample transactions
- **UI**: Update `index.html` for layout changes
- **Logic**: Extend `main.js` for additional features

---

## Support

- **Documentation**: See `docs/WASM_DEMO.md` for architecture details
- **Issues**: Report at https://github.com/prasincs/universal-blockchain-decoder/issues
- **CLAUDE.md**: Core design philosophy and principles

---

## License

MIT OR Apache-2.0

---

**Last Updated**: 2025-11-14
**WASM Version**: 0.1.0
**Build**: Release (optimized for size)
