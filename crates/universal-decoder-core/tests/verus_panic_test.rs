//! Verus Panic Detection Test
//!
//! This test demonstrates how Verus catches potential panics.
//! We'll create intentionally unsafe code and show that Verus verification fails.

// Verus annotations are conditionally compiled
// These imports are only available when Verus is installed and the feature is enabled
// #[cfg(feature = "formal-verification")]
// use builtin::*;
// #[cfg(feature = "formal-verification")]
// use builtin_macros::*;

/// **BAD EXAMPLE**: This function CAN panic but claims it won't
///
/// This demonstrates what Verus is designed to catch.
///
/// # Verus Specification (INCORRECT - will fail verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn parse_first_byte_unsafe(bytes: &[u8]) -> (result: u8)
///         ensures
///             // CLAIM: This function never panics
///             true  // This will FAIL verification!
///     {
///         // UNSAFE: unwrap() can panic if bytes is empty!
///         bytes.first().unwrap()
///     }
/// }
/// ```
///
/// **Why Verus will reject this:**
/// - `bytes.first()` returns `Option<&u8>`
/// - `unwrap()` panics if `None`
/// - No precondition requires `bytes.len() > 0`
/// - Therefore: function CAN panic with empty input
/// - Verus verification: ❌ FAILS
#[allow(dead_code)]
fn parse_first_byte_unsafe(bytes: &[u8]) -> u8 {
    // This will panic if bytes is empty!
    *bytes.first().unwrap()
}

/// **BAD EXAMPLE 2**: Array indexing without bounds check
///
/// # Verus Specification (INCORRECT - will fail verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn get_nth_byte_unsafe(bytes: &[u8], index: usize) -> (result: u8)
///         ensures
///             // CLAIM: This function never panics
///             true  // This will FAIL verification!
///     {
///         bytes[index]  // Can panic if index >= bytes.len()
///     }
/// }
/// ```
///
/// **Why Verus will reject this:**
/// - Array indexing `bytes[index]` panics if `index >= bytes.len()`
/// - No precondition `requires index < bytes.len()`
/// - Verus verification: ❌ FAILS
#[allow(dead_code)]
fn get_nth_byte_unsafe(bytes: &[u8], index: usize) -> u8 {
    bytes[index] // Panic if out of bounds!
}

/// **BAD EXAMPLE 3**: Division by zero
///
/// # Verus Specification (INCORRECT - will fail verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn divide_unsafe(a: u64, b: u64) -> (result: u64)
///         ensures
///             // CLAIM: This function never panics
///             true  // This will FAIL verification!
///     {
///         a / b  // Panics if b == 0
///     }
/// }
/// ```
///
/// **Why Verus will reject this:**
/// - Division by zero causes panic
/// - No precondition `requires b != 0`
/// - Verus verification: ❌ FAILS
#[allow(dead_code)]
fn divide_unsafe(a: u64, b: u64) -> u64 {
    a / b // Panic if b == 0!
}

/// **GOOD EXAMPLE**: Safe version with proper preconditions
///
/// # Verus Specification (CORRECT - will pass verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn parse_first_byte_safe(bytes: &[u8]) -> (result: u8)
///         requires
///             bytes.len() > 0,  // Precondition: must not be empty
///         ensures
///             // This function never panics (given precondition)
///             result == bytes[0],
///             true
///     {
///         bytes[0]  // Safe: precondition ensures bytes.len() > 0
///     }
/// }
/// ```
///
/// **Why Verus will accept this:**
/// - Precondition `requires bytes.len() > 0` ensures safety
/// - Array access `bytes[0]` is proven safe by precondition
/// - Verus verification: ✅ PASSES
#[allow(dead_code)]
fn parse_first_byte_safe(bytes: &[u8]) -> u8 {
    // Caller must ensure bytes is non-empty
    // This is enforced by Verus precondition
    bytes[0]
}

/// **GOOD EXAMPLE**: Using Result instead of panic
///
/// # Verus Specification (CORRECT - will pass verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn parse_first_byte_result(bytes: &[u8]) -> (result: Result<u8, &'static str>)
///         ensures
///             // If empty, returns Err (no panic)
///             bytes.len() == 0 ==> result.is_err(),
///             // If non-empty, returns Ok with first byte
///             bytes.len() > 0 ==> {
///                 result.is_ok() &&
///                 result.unwrap() == bytes[0]
///             },
///             // Never panics
///             true
///     {
///         bytes.first().copied().ok_or("empty input")
///     }
/// }
/// ```
///
/// **Why Verus will accept this:**
/// - Returns `Result` instead of panicking
/// - Handles empty case explicitly with `Err`
/// - No unwrap() or panic paths
/// - Verus verification: ✅ PASSES
#[allow(dead_code)]
fn parse_first_byte_result(bytes: &[u8]) -> Result<u8, &'static str> {
    bytes.first().copied().ok_or("empty input")
}

/// **GOOD EXAMPLE**: Safe division with precondition
///
/// # Verus Specification (CORRECT - will pass verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn divide_safe(a: u64, b: u64) -> (result: u64)
///         requires
///             b != 0,  // Precondition: divisor must not be zero
///         ensures
///             result == a / b,
///             // Never panics (given precondition)
///             true
///     {
///         a / b  // Safe: precondition ensures b != 0
///     }
/// }
/// ```
///
/// **Why Verus will accept this:**
/// - Precondition `requires b != 0` prevents division by zero
/// - Caller must prove divisor is non-zero
/// - Verus verification: ✅ PASSES
#[allow(dead_code)]
fn divide_safe(a: u64, b: u64) -> u64 {
    // Caller must ensure b != 0
    // This is enforced by Verus precondition
    a / b
}

/// **GOOD EXAMPLE**: Safe division with checked arithmetic
///
/// # Verus Specification (CORRECT - will pass verification)
///
/// ```rust,ignore
/// #[cfg(verus)]
/// verus! {
///     fn divide_checked(a: u64, b: u64) -> (result: Option<u64>)
///         ensures
///             // If b == 0, returns None (no panic)
///             b == 0 ==> result.is_none(),
///             // If b != 0, returns Some(a / b)
///             b != 0 ==> {
///                 result.is_some() &&
///                 result.unwrap() == a / b
///             },
///             // Never panics
///             true
///     {
///         if b == 0 {
///             None
///         } else {
///             Some(a / b)
///         }
///     }
/// }
/// ```
///
/// **Why Verus will accept this:**
/// - Explicitly checks for zero before dividing
/// - Returns `Option` to handle error case
/// - No panic paths exist
/// - Verus verification: ✅ PASSES
#[allow(dead_code)]
fn divide_checked(a: u64, b: u64) -> Option<u64> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_unsafe_function_panics_on_empty_input() {
        // This test verifies that the unsafe function DOES panic
        // Verus would reject this function because of this panic path
        let empty: &[u8] = &[];
        parse_first_byte_unsafe(empty); // This will panic!
    }

    #[test]
    #[should_panic]
    fn test_unsafe_indexing_panics() {
        // This test verifies that unsafe indexing DOES panic
        let bytes = &[1, 2, 3];
        get_nth_byte_unsafe(bytes, 10); // Out of bounds!
    }

    #[test]
    #[should_panic]
    fn test_unsafe_division_panics() {
        // This test verifies that unsafe division DOES panic
        divide_unsafe(10, 0); // Division by zero!
    }

    #[test]
    fn test_safe_function_with_valid_input() {
        // Safe function works when precondition is met
        let bytes = &[42, 10, 20];
        assert_eq!(parse_first_byte_safe(bytes), 42);
    }

    #[test]
    fn test_result_function_with_empty_input() {
        // Result-based function handles empty input gracefully
        let empty: &[u8] = &[];
        assert!(parse_first_byte_result(empty).is_err());
    }

    #[test]
    fn test_result_function_with_valid_input() {
        // Result-based function works with valid input
        let bytes = &[42, 10, 20];
        assert_eq!(parse_first_byte_result(bytes), Ok(42));
    }

    #[test]
    fn test_safe_division() {
        // Safe division with non-zero divisor
        assert_eq!(divide_safe(10, 2), 5);
    }

    #[test]
    fn test_checked_division_with_zero() {
        // Checked division handles zero gracefully
        assert_eq!(divide_checked(10, 0), None);
    }

    #[test]
    fn test_checked_division_with_non_zero() {
        // Checked division works with non-zero divisor
        assert_eq!(divide_checked(10, 2), Some(5));
    }
}

// # Summary: How to Run This Test with Verus
//
// ## Step 1: Install Verus
// ```bash
// cd /tmp
// git clone https://github.com/verus-lang/verus.git
// cd verus
// ./tools/get-z3.sh
// source tools/activate
// ```
//
// ## Step 2: Create Verus-Annotated Version
// To actually verify with Verus, uncomment the annotations above and wrap in `verus!` blocks.
//
// ## Step 3: Run Verus Verification
// ```bash
// # Try to verify the UNSAFE functions (will FAIL)
// verus crates/universal-decoder-core/tests/verus_panic_test.rs
//
// # Expected output for unsafe functions:
// # error: postcondition not satisfied
// #   --> verus_panic_test.rs:XX:YY
// #    |
// #    |     ensures true  // Never panics
// #    |     ^^^^^^^ postcondition not satisfied
// #    |
// # note: unwrap() can panic when Option is None
// #
// # Verification: FAILED
// ```
//
// ## Step 4: Fix and Re-verify
// ```bash
// # Verify the SAFE functions (will PASS)
// # These have proper preconditions or use Result
// #
// # Expected output:
// # verification results:: 5 verified, 0 errors
// # Verification: SUCCESS
// ```
//
// ## What This Demonstrates
//
// 1. **Verus catches unwrap() panics**: Functions using `unwrap()` without preconditions fail verification
// 2. **Verus catches array bounds violations**: Indexing without bounds checks fails verification
// 3. **Verus catches division by zero**: Division without zero checks fails verification
// 4. **Preconditions make functions safe**: Adding `requires` clauses enables verification
// 5. **Result types are panic-free**: Using `Result` or `Option` always passes verification
//
// ## Key Insight
//
// Verus doesn't just warn about potential panics—it **mathematically proves** that they can
// occur by finding concrete counterexamples:
//
// - `parse_first_byte_unsafe(&[])` → counterexample: empty slice causes panic
// - `get_nth_byte_unsafe(&[1], 5)` → counterexample: index 5 out of bounds
// - `divide_unsafe(1, 0)` → counterexample: zero divisor causes panic
//
// This is much stronger than testing, which can only check specific inputs!
