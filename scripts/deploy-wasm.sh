#!/bin/bash
set -e

# Universal Blockchain Decoder - WASM Deployment Script
#
# Supports deployment to multiple platforms using environment variables
# No build artifacts are committed to the repository
#
# Usage:
#   ./scripts/deploy-wasm.sh [platform]
#
# Platforms:
#   netlify   - Deploy to Netlify (preview or production)
#   vercel    - Deploy to Vercel (preview or production)
#   github    - Deploy to GitHub Pages (production only)
#   local     - Start local server for testing
#
# Required Environment Variables:
#
# For Netlify:
#   NETLIFY_AUTH_TOKEN - Your Netlify personal access token
#   NETLIFY_SITE_ID - Your Netlify site ID (optional for new sites)
#
# For Vercel:
#   VERCEL_TOKEN - Your Vercel personal access token
#   VERCEL_ORG_ID - Your Vercel organization/team ID
#   VERCEL_PROJECT_ID - Your Vercel project ID
#
# For GitHub Pages:
#   (Uses git credentials, no additional env vars needed)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print with color
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Get the platform from argument
PLATFORM=${1:-local}

# Determine repository root
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
WASM_CRATE="$REPO_ROOT/crates/universal-decoder-wasm"
WWW_DIR="$WASM_CRATE/www"

# Check if we're in the right directory
if [[ ! -f "$WASM_CRATE/Cargo.toml" ]]; then
    error "Cannot find WASM crate. Make sure you're in the repository root."
fi

# Build WASM module
build_wasm() {
    info "Building WASM module..."

    # Check if build.sh exists and is executable
    if [[ ! -x "$WASM_CRATE/build.sh" ]]; then
        chmod +x "$WASM_CRATE/build.sh"
    fi

    # Build
    cd "$WASM_CRATE"
    ./build.sh

    # Show bundle size
    echo ""
    info "📦 WASM Bundle Size:"
    ls -lh "$WWW_DIR/pkg/"*.wasm 2>/dev/null || warning "WASM files not found"
    echo ""
    info "📊 Total www/ directory size:"
    du -sh "$WWW_DIR"
    echo ""

    success "WASM build complete"
}

# Deploy to Netlify
deploy_netlify() {
    info "Deploying to Netlify..."

    # Check environment variables
    if [[ -z "$NETLIFY_AUTH_TOKEN" ]]; then
        error "NETLIFY_AUTH_TOKEN environment variable is required"
    fi

    # Check if netlify CLI is installed
    if ! command -v netlify &> /dev/null; then
        warning "Netlify CLI not found. Installing..."
        npm install -g netlify-cli
    fi

    # Determine if this is a production or preview deployment
    local PROD_FLAG=""
    if [[ -n "$NETLIFY_PRODUCTION" ]] && [[ "$NETLIFY_PRODUCTION" == "true" ]]; then
        PROD_FLAG="--prod"
        info "Deploying to production..."
    else
        info "Deploying preview..."
    fi

    # Deploy
    cd "$WWW_DIR"

    if [[ -n "$NETLIFY_SITE_ID" ]]; then
        netlify deploy $PROD_FLAG --dir=. --site="$NETLIFY_SITE_ID" --auth="$NETLIFY_AUTH_TOKEN"
    else
        netlify deploy $PROD_FLAG --dir=. --auth="$NETLIFY_AUTH_TOKEN"
    fi

    success "Deployed to Netlify"
}

# Deploy to Vercel
deploy_vercel() {
    info "Deploying to Vercel..."

    # Check environment variables
    if [[ -z "$VERCEL_TOKEN" ]]; then
        error "VERCEL_TOKEN environment variable is required"
    fi

    # Check if vercel CLI is installed
    if ! command -v vercel &> /dev/null; then
        warning "Vercel CLI not found. Installing..."
        npm install -g vercel
    fi

    # Determine if this is a production or preview deployment
    local PROD_FLAG=""
    if [[ -n "$VERCEL_PRODUCTION" ]] && [[ "$VERCEL_PRODUCTION" == "true" ]]; then
        PROD_FLAG="--prod"
        info "Deploying to production..."
    else
        info "Deploying preview..."
    fi

    # Deploy
    cd "$WWW_DIR"

    if [[ -n "$VERCEL_ORG_ID" ]] && [[ -n "$VERCEL_PROJECT_ID" ]]; then
        vercel $PROD_FLAG --token="$VERCEL_TOKEN"
    else
        vercel $PROD_FLAG --token="$VERCEL_TOKEN"
    fi

    success "Deployed to Vercel"
}

# Deploy to GitHub Pages
deploy_github() {
    info "Deploying to GitHub Pages..."

    cd "$REPO_ROOT"

    # Check if gh-pages branch exists
    if git show-ref --quiet refs/heads/gh-pages; then
        info "gh-pages branch exists, updating..."

        # Stash current changes
        git stash push -m "Temporary stash for GitHub Pages deployment"

        # Switch to gh-pages
        git checkout gh-pages

        # Copy new build
        cp -r "$WWW_DIR"/* .

        # Commit and push
        git add -A
        git commit -m "Deploy WASM demo - $(date '+%Y-%m-%d %H:%M:%S')" || warning "No changes to commit"
        git push origin gh-pages

        # Return to previous branch
        git checkout -

        # Restore stashed changes
        git stash pop || info "No stashed changes to restore"
    else
        error "gh-pages branch doesn't exist. Create it first (see wasm/DEPLOY.md)"
    fi

    success "Deployed to GitHub Pages"
}

# Start local server
deploy_local() {
    info "Starting local development server..."

    cd "$WWW_DIR"

    # Check for available server
    if command -v python3 &> /dev/null; then
        success "Starting Python HTTP server on http://localhost:8080"
        python3 -m http.server 8080
    elif command -v http-server &> /dev/null; then
        success "Starting Node.js HTTP server on http://localhost:8080"
        http-server -p 8080
    elif command -v php &> /dev/null; then
        success "Starting PHP built-in server on http://localhost:8080"
        php -S localhost:8080
    else
        error "No HTTP server found. Install Python 3, Node.js (http-server), or PHP."
    fi
}

# Main execution
info "Universal Blockchain Decoder - WASM Deployment"
echo ""

# Build WASM first (unless deploying to GitHub Pages, which uses existing build)
if [[ "$PLATFORM" != "github" ]]; then
    build_wasm
fi

# Deploy to selected platform
case $PLATFORM in
    netlify)
        deploy_netlify
        ;;
    vercel)
        deploy_vercel
        ;;
    github)
        deploy_github
        ;;
    local)
        deploy_local
        ;;
    *)
        error "Unknown platform: $PLATFORM. Use: netlify, vercel, github, or local"
        ;;
esac

echo ""
success "Deployment complete! 🎉"
