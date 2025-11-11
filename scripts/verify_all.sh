#!/usr/bin/env bash
#
# verify_all.sh - Run Verus formal verification on all annotated files
#
# Usage: ./scripts/verify_all.sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "  Verus Formal Verification Suite"
echo "========================================="
echo ""

# Check if Verus is installed
if ! command -v verus &> /dev/null; then
    echo -e "${RED}Error: Verus not found in PATH${NC}"
    echo "Please install Verus:"
    echo "  1. Clone: git clone https://github.com/verus-lang/verus.git"
    echo "  2. Install Z3: ./tools/get-z3.sh && source tools/activate"
    echo "  3. Build: cargo build --release"
    echo "  4. Add to PATH: export PATH=\"/path/to/verus/target/release:\$PATH\""
    echo ""
    echo "See docs/VERUS_SETUP.md for detailed instructions"
    exit 1
fi

# Check if Z3 is available
if ! command -v z3 &> /dev/null; then
    echo -e "${YELLOW}Warning: Z3 not found in PATH${NC}"
    echo "Verus requires Z3. Install it with:"
    echo "  Ubuntu/Debian: sudo apt-get install z3"
    echo "  macOS: brew install z3"
    echo "  Or use Verus's script: ./tools/get-z3.sh"
    echo ""
fi

echo "Verus version: $(verus --version 2>&1 | head -1)"
echo "Z3 version: $(z3 --version 2>&1 | head -1 || echo 'Not found')"
echo ""

# Files to verify (add more as we annotate them)
VERIFY_FILES=(
    "crates/universal-decoder-core/src/ir.rs"
    # "crates/universal-decoder-core/src/canonical.rs"  # Phase 2
    # "crates/universal-decoder-core/src/traits.rs"     # Phase 2
)

# Counters
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

echo "Files to verify:"
for file in "${VERIFY_FILES[@]}"; do
    echo "  - $file"
    TOTAL=$((TOTAL + 1))
done
echo ""

# Verify each file
for file in "${VERIFY_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo -e "${YELLOW}⊘ SKIP${NC} $file (file not found)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    echo -e "Verifying ${YELLOW}$file${NC}..."

    # Run Verus with timeout (10 minutes per file)
    if timeout 600 verus "$file" > /tmp/verus_output_$$.log 2>&1; then
        echo -e "${GREEN}✓ PASS${NC} $file"
        PASSED=$((PASSED + 1))
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            echo -e "${RED}✗ TIMEOUT${NC} $file (> 10 minutes)"
        else
            echo -e "${RED}✗ FAIL${NC} $file"
            echo "--- Error output ---"
            cat /tmp/verus_output_$$.log
            echo "--------------------"
        fi
        FAILED=$((FAILED + 1))
        rm -f /tmp/verus_output_$$.log
    fi
    echo ""
done

# Summary
echo "========================================="
echo "  Verification Summary"
echo "========================================="
echo "Total files: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All verifications passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some verifications failed.${NC}"
    echo "Review the output above for details."
    exit 1
fi
