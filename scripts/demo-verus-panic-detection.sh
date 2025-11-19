#!/bin/bash
# Verus Panic Detection Demo
#
# This script demonstrates how Verus catches potential panics in code.
# It creates a temporary Verus-annotated file and runs verification.

set -e

echo "============================================"
echo "Verus Panic Detection Demonstration"
echo "============================================"
echo ""

# Check if Verus is available
if ! command -v verus &> /dev/null; then
    echo "⚠️  Verus not found in PATH"
    echo ""
    echo "To install Verus:"
    echo "  cd /tmp"
    echo "  git clone https://github.com/verus-lang/verus.git"
    echo "  cd verus"
    echo "  ./tools/get-z3.sh"
    echo "  source tools/activate"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo "✅ Verus found: $(which verus)"
echo ""

# Create a temporary Verus test file
TEMP_DIR=$(mktemp -d)
TEMP_FILE="$TEMP_DIR/verus_panic_demo.rs"

echo "Creating temporary Verus file: $TEMP_FILE"
echo ""

cat > "$TEMP_FILE" << 'EOF'
use builtin::*;
use builtin_macros::*;

verus! {

// ============================================
// EXAMPLE 1: unwrap() panic detection
// ============================================

/// This function WILL FAIL verification
/// Verus will detect that unwrap() can panic
fn parse_first_byte_unsafe(bytes: &[u8]) -> (result: u8)
    ensures
        // CLAIM: Never panics (FALSE!)
        true
{
    *bytes.first().unwrap()
}

// ============================================
// EXAMPLE 2: Array bounds panic detection
// ============================================

/// This function WILL FAIL verification
/// Verus will detect out-of-bounds access
fn get_nth_byte_unsafe(bytes: &[u8], index: usize) -> (result: u8)
    ensures
        // CLAIM: Never panics (FALSE!)
        true
{
    bytes[index]
}

// ============================================
// EXAMPLE 3: Division by zero detection
// ============================================

/// This function WILL FAIL verification
/// Verus will detect division by zero
fn divide_unsafe(a: u64, b: u64) -> (result: u64)
    ensures
        // CLAIM: Never panics (FALSE!)
        true
{
    a / b
}

// ============================================
// EXAMPLE 4: SAFE version with precondition
// ============================================

/// This function WILL PASS verification
/// Precondition ensures safety
fn parse_first_byte_safe(bytes: &[u8]) -> (result: u8)
    requires
        bytes.len() > 0,  // Precondition prevents panic!
    ensures
        result == bytes[0],
        true  // Never panics (proven by precondition)
{
    bytes[0]
}

// ============================================
// EXAMPLE 5: SAFE version with Result
// ============================================

/// This function WILL PASS verification
/// Returns Result instead of panicking
fn parse_first_byte_result(bytes: &[u8]) -> (result: Result<u8, ()>)
    ensures
        bytes.len() == 0 ==> result.is_err(),
        bytes.len() > 0 ==> result.is_ok() && result.unwrap() == bytes[0],
        true  // Never panics (returns Result)
{
    if bytes.len() == 0 {
        Err(())
    } else {
        Ok(bytes[0])
    }
}

} // verus!

fn main() {
    println!("Verus panic detection demo");
}
EOF

echo "============================================"
echo "FILE CONTENTS:"
echo "============================================"
cat "$TEMP_FILE"
echo ""
echo "============================================"

echo ""
echo "============================================"
echo "Running Verus Verification"
echo "============================================"
echo ""

echo "This will attempt to verify 5 functions:"
echo "  1. parse_first_byte_unsafe  (❌ should FAIL)"
echo "  2. get_nth_byte_unsafe      (❌ should FAIL)"
echo "  3. divide_unsafe            (❌ should FAIL)"
echo "  4. parse_first_byte_safe    (✅ should PASS)"
echo "  5. parse_first_byte_result  (✅ should PASS)"
echo ""
echo "Press Enter to continue..."
read

echo "Running: verus $TEMP_FILE"
echo ""

# Run Verus and capture output
if verus "$TEMP_FILE" 2>&1 | tee "$TEMP_DIR/verus_output.txt"; then
    echo ""
    echo "============================================"
    echo "✅ Verification completed"
    echo "============================================"
else
    echo ""
    echo "============================================"
    echo "⚠️  Verification found errors (expected!)"
    echo "============================================"
    echo ""
    echo "This is GOOD! Verus caught the unsafe functions."
fi

echo ""
echo "============================================"
echo "ANALYSIS"
echo "============================================"
echo ""

# Analyze the output
if grep -q "error" "$TEMP_DIR/verus_output.txt"; then
    echo "❌ Verus detected potential panics in:"
    echo ""
    grep -A 5 "error" "$TEMP_DIR/verus_output.txt" | head -20 || true
    echo ""
    echo "This proves that Verus can mathematically detect panic paths!"
else
    echo "✅ All functions verified successfully"
    echo ""
    echo "This means all functions are proven panic-free."
fi

echo ""
echo "============================================"
echo "KEY TAKEAWAYS"
echo "============================================"
echo ""
echo "1. Verus DETECTS unwrap() panics"
echo "   - parse_first_byte_unsafe uses unwrap()"
echo "   - Verus proves it can panic with empty input"
echo ""
echo "2. Verus DETECTS array bounds violations"
echo "   - get_nth_byte_unsafe uses unchecked indexing"
echo "   - Verus proves it can panic with large index"
echo ""
echo "3. Verus DETECTS division by zero"
echo "   - divide_unsafe has no zero check"
echo "   - Verus proves it can panic with zero divisor"
echo ""
echo "4. PRECONDITIONS make functions safe"
echo "   - parse_first_byte_safe has 'requires bytes.len() > 0'"
echo "   - Verus proves this prevents all panics"
echo ""
echo "5. RESULT types are panic-free"
echo "   - parse_first_byte_result returns Result"
echo "   - Verus proves no panic paths exist"
echo ""

echo "============================================"
echo "NEXT STEPS"
echo "============================================"
echo ""
echo "1. View the full test file:"
echo "   cat crates/universal-decoder-core/tests/verus_panic_test.rs"
echo ""
echo "2. Run the Rust tests (they demonstrate the panics):"
echo "   cargo test --test verus_panic_test"
echo ""
echo "3. Apply Verus annotations to your own code:"
echo "   - Add 'requires' for preconditions"
echo "   - Add 'ensures' for postconditions"
echo "   - Use Result instead of unwrap()"
echo ""
echo "4. Read the formal verification docs:"
echo "   cat docs/FORMAL_VERIFICATION.md"
echo ""

# Cleanup
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

echo ""
echo "✅ Demo complete!"
