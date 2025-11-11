#!/bin/bash
# CI Monitor Script
# Automatically monitors CI state and resolves common issues

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
ISSUES_FOUND=0
ISSUES_FIXED=0
ISSUES_UNFIXED=0

echo "========================================="
echo "  CI State Monitor & Auto-Fixer"
echo "========================================="
echo ""

# Function to log messages
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run a check and report status
run_check() {
    local name="$1"
    local command="$2"

    echo ""
    echo "===== Checking: $name ====="

    if eval "$command" > /tmp/ci-check.log 2>&1; then
        log_info "$name: ✓ PASSED"
        return 0
    else
        log_error "$name: ✗ FAILED"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
        cat /tmp/ci-check.log
        return 1
    fi
}

# Function to attempt auto-fix for formatting
fix_formatting() {
    log_info "Attempting to fix formatting issues..."
    if cargo fmt --all; then
        log_info "Formatting fixes applied"
        ISSUES_FIXED=$((ISSUES_FIXED + 1))
        return 0
    else
        log_error "Failed to apply formatting fixes"
        ISSUES_UNFIXED=$((ISSUES_UNFIXED + 1))
        return 1
    fi
}

# Function to analyze clippy errors and suggest fixes
analyze_clippy_errors() {
    log_info "Analyzing clippy errors..."

    cargo clippy --all-targets --all-features -- -D warnings > /tmp/clippy-errors.log 2>&1 || true

    # Check for common patterns
    if grep -q "needless_borrows_for_generic_args" /tmp/clippy-errors.log; then
        log_warn "Found needless borrow errors - these typically require manual fixing"
        echo "  Tip: Remove & from arguments where clippy suggests"
    fi

    if grep -q "unused_imports" /tmp/clippy-errors.log; then
        log_warn "Found unused imports - these require manual removal"
    fi

    if grep -q "empty_line_after_doc_comments" /tmp/clippy-errors.log; then
        log_warn "Found empty line after doc comments"
        echo "  Tip: Remove empty lines between /// comments and code"
    fi

    if grep -q "useless_vec" /tmp/clippy-errors.log; then
        log_warn "Found useless vec! - use arrays instead"
        echo "  Tip: Replace vec![...] with [...]"
    fi

    # Show first 50 lines of errors
    echo ""
    echo "===== Clippy Error Details ====="
    head -50 /tmp/clippy-errors.log
}

# Function to check doc build
check_documentation() {
    log_info "Checking documentation build..."

    if cargo doc --no-deps --lib --all 2>&1 | grep -E "(error|warning:)" > /tmp/doc-errors.log; then
        log_error "Documentation has errors/warnings"
        cat /tmp/doc-errors.log
        return 1
    else
        log_info "Documentation builds cleanly"
        return 0
    fi
}

# Main monitoring loop
main() {
    local auto_fix=${1:-false}

    log_info "Starting CI checks..."
    log_info "Auto-fix mode: $auto_fix"
    echo ""

    # Check 1: Formatting
    if ! run_check "Formatting" "cargo fmt --all -- --check"; then
        if [ "$auto_fix" = "true" ] || [ "$auto_fix" = "--fix" ]; then
            fix_formatting
            # Re-check after fix
            if run_check "Formatting (after fix)" "cargo fmt --all -- --check"; then
                log_info "Formatting issue resolved!"
            fi
        else
            log_warn "Run with --fix to automatically apply formatting"
        fi
    fi

    # Check 2: Clippy
    if ! run_check "Clippy" "cargo clippy --all-targets --all-features -- -D warnings"; then
        analyze_clippy_errors
        ISSUES_UNFIXED=$((ISSUES_UNFIXED + 1))
    fi

    # Check 3: Documentation
    if ! check_documentation; then
        ISSUES_UNFIXED=$((ISSUES_UNFIXED + 1))
    fi

    # Check 4: Tests (if requested)
    if [ "$2" = "--test" ]; then
        run_check "Unit Tests" "cargo test --lib --all"
    fi

    # Summary
    echo ""
    echo "========================================="
    echo "  CI Monitor Summary"
    echo "========================================="
    echo "Issues found:     $ISSUES_FOUND"
    echo "Issues fixed:     $ISSUES_FIXED"
    echo "Issues remaining: $ISSUES_UNFIXED"
    echo ""

    if [ $ISSUES_UNFIXED -eq 0 ]; then
        log_info "✓ All CI checks passed!"
        exit 0
    else
        log_error "✗ Some issues remain - manual intervention needed"
        exit 1
    fi
}

# Parse arguments
case "${1:-}" in
    -h|--help)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --fix       Automatically fix issues where possible"
        echo "  --test      Also run unit tests"
        echo "  --watch     Run in watch mode (continuous monitoring)"
        echo "  --help      Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                    # Just check CI state"
        echo "  $0 --fix              # Check and auto-fix"
        echo "  $0 --fix --test       # Check, fix, and run tests"
        exit 0
        ;;
    --watch)
        log_info "Starting watch mode (Ctrl+C to stop)..."
        while true; do
            main "${@:2}"
            sleep 30
        done
        ;;
    *)
        main "$@"
        ;;
esac
