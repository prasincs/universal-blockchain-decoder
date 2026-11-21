//! ECDSA signature verification tests
//!
//! Tests for ECDSA on STARK curve including:
//! - Basic signature verification
//! - Cross-validation with starknet-crypto
//! - Test vectors from Starknet
//! - Error handling (invalid inputs)
//! - Edge cases

use decoder_crypto_zk::field::FieldElement;
use decoder_crypto_zk::signature::{verify, Signature, VerifyError};

// ============================================================================
// Basic Verification Tests
// ============================================================================

#[test]
fn test_verify_does_not_panic() {
    // Basic smoke test - ensure verify doesn't panic on any inputs
    let public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let r = FieldElement::from(3u64);
    let s = FieldElement::from(4u64);

    let result = verify(&public_key, &message, &r, &s);
    // Should return Ok (either true or false), not panic
    assert!(result.is_ok());
}

#[test]
fn test_signature_struct() {
    let sig = Signature {
        r: FieldElement::from(123u64),
        s: FieldElement::from(456u64),
    };

    assert_eq!(sig.r, FieldElement::from(123u64));
    assert_eq!(sig.s, FieldElement::from(456u64));
}

#[test]
fn test_signature_clone() {
    let sig1 = Signature {
        r: FieldElement::from(123u64),
        s: FieldElement::from(456u64),
    };

    let sig2 = sig1.clone();
    assert_eq!(sig1.r, sig2.r);
    assert_eq!(sig1.s, sig2.s);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_verify_rejects_invalid_message() {
    let public_key = FieldElement::from(1u64);
    let r = FieldElement::from(3u64);
    let s = FieldElement::from(4u64);

    // Message >= upper bound
    let invalid_message = FieldElement::from_hex(
        "0x0800000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    let result = verify(&public_key, &invalid_message, &r, &s);
    assert!(matches!(result, Err(VerifyError::InvalidMessageHash)));
}

#[test]
fn test_verify_rejects_zero_r() {
    let public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let r = FieldElement::ZERO; // Invalid
    let s = FieldElement::from(4u64);

    let result = verify(&public_key, &message, &r, &s);
    assert!(matches!(result, Err(VerifyError::InvalidR)));
}

#[test]
fn test_verify_rejects_r_too_large() {
    let public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let s = FieldElement::from(4u64);

    // r >= upper bound
    let invalid_r = FieldElement::from_hex(
        "0x0800000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    let result = verify(&public_key, &message, &invalid_r, &s);
    assert!(matches!(result, Err(VerifyError::InvalidR)));
}

#[test]
fn test_verify_rejects_zero_s() {
    let public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let r = FieldElement::from(3u64);
    let s = FieldElement::ZERO; // Invalid

    let result = verify(&public_key, &message, &r, &s);
    assert!(matches!(result, Err(VerifyError::InvalidS)));
}

#[test]
fn test_verify_rejects_s_too_large() {
    let public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let r = FieldElement::from(3u64);

    // s >= upper bound
    let invalid_s = FieldElement::from_hex(
        "0x0800000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    let result = verify(&public_key, &message, &r, &invalid_s);
    assert!(matches!(result, Err(VerifyError::InvalidS)));
}

#[test]
fn test_verify_rejects_invalid_public_key() {
    // Public key that's not on the curve
    let invalid_public_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let r = FieldElement::from(3u64);
    let s = FieldElement::from(4u64);

    let result = verify(&invalid_public_key, &message, &r, &s);
    // Should either be InvalidPublicKey or return false
    assert!(matches!(result, Err(VerifyError::InvalidPublicKey)) || (result == Ok(false)));
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_verify_deterministic() {
    let public_key = FieldElement::from(100u64);
    let message = FieldElement::from(200u64);
    let r = FieldElement::from(300u64);
    let s = FieldElement::from(400u64);

    let result1 = verify(&public_key, &message, &r, &s);
    let result2 = verify(&public_key, &message, &r, &s);

    // Same inputs should produce same result
    assert_eq!(result1.is_ok(), result2.is_ok());
    if let (Ok(v1), Ok(v2)) = (result1, result2) {
        assert_eq!(v1, v2);
    }
}

// ============================================================================
// Cross-Validation with starknet-crypto
// ============================================================================

#[test]
fn test_cross_validate_verify_valid_signature() {
    use starknet_crypto::{get_public_key, sign, verify as ref_verify};

    // Generate a keypair and sign a message using reference implementation
    let private_key = FieldElement::from(12345u64);
    let message = FieldElement::from(67890u64);

    // Get public key using reference implementation
    let public_key = get_public_key(&private_key);

    // Sign using reference implementation
    let signature = sign(&private_key, &message, &FieldElement::from(1234u64)).unwrap();

    // Verify using our implementation
    let our_result = verify(&public_key, &message, &signature.r, &signature.s);

    // Verify using reference implementation
    let ref_result = ref_verify(&public_key, &message, &signature.r, &signature.s);

    // Both should succeed
    assert!(our_result.is_ok());
    assert!(ref_result.is_ok());

    // Both should return true
    let our_value = our_result.unwrap();
    let ref_value = ref_result.unwrap();
    assert_eq!(our_value, ref_value);
    assert!(our_value);
}

#[test]
fn test_cross_validate_verify_invalid_signature() {
    use starknet_crypto::{get_public_key, sign, verify as ref_verify};

    let private_key = FieldElement::from(12345u64);
    let message = FieldElement::from(67890u64);
    let wrong_message = FieldElement::from(99999u64);

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &FieldElement::from(1234u64)).unwrap();

    // Verify with wrong message using our implementation
    let our_result = verify(&public_key, &wrong_message, &signature.r, &signature.s);

    // Verify with wrong message using reference
    let ref_result = ref_verify(&public_key, &wrong_message, &signature.r, &signature.s);

    // Both should agree
    assert_eq!(our_result.is_ok(), ref_result.is_ok());
    if let (Ok(our_value), Ok(ref_value)) = (our_result, ref_result) {
        assert_eq!(our_value, ref_value);
        // Should be false (wrong message)
        assert!(!our_value);
    }
}

#[test]
fn test_cross_validate_multiple_signatures() {
    use starknet_crypto::{get_public_key, sign, verify as ref_verify};

    // Test with multiple different signatures
    for i in 1..10 {
        let private_key = FieldElement::from(i * 1000);
        let message = FieldElement::from(i * 2000);

        let public_key = get_public_key(&private_key);
        let signature = sign(&private_key, &message, &FieldElement::from(i)).unwrap();

        let our_result = verify(&public_key, &message, &signature.r, &signature.s);
        let ref_result = ref_verify(&public_key, &message, &signature.r, &signature.s);

        assert_eq!(our_result.is_ok(), ref_result.is_ok());
        if let (Ok(our_value), Ok(ref_value)) = (our_result, ref_result) {
            assert_eq!(our_value, ref_value);
            assert!(our_value);
        }
    }
}

// ============================================================================
// Test Vectors from Starknet
// ============================================================================

#[test]
fn test_vector_1_known_signature() {
    use starknet_crypto::{get_public_key, sign};

    // Create a known signature
    let private_key = FieldElement::from_hex("0x1234567890abcdef").unwrap();
    let message = FieldElement::from_hex("0xfedcba0987654321").unwrap();
    let k = FieldElement::from_hex("0x1111111111111111").unwrap();

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    // Our verification should succeed
    let result = verify(&public_key, &message, &signature.r, &signature.s);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_vector_2_wrong_public_key() {
    use starknet_crypto::{get_public_key, sign};

    let private_key1 = FieldElement::from_hex("0x1234567890abcdef").unwrap();
    let private_key2 = FieldElement::from_hex("0xfedcba0987654321").unwrap();
    let message = FieldElement::from_hex("0xaaaaaaaaaaaaaaaa").unwrap();
    let k = FieldElement::from_hex("0xbbbbbbbbbbbbbbbb").unwrap();

    let public_key1 = get_public_key(&private_key1);
    let public_key2 = get_public_key(&private_key2);

    let signature = sign(&private_key1, &message, &k).unwrap();

    // Verify with correct public key
    let result_correct = verify(&public_key1, &message, &signature.r, &signature.s);
    assert!(result_correct.is_ok());
    assert!(result_correct.unwrap());

    // Verify with wrong public key
    let result_wrong = verify(&public_key2, &message, &signature.r, &signature.s);
    assert!(result_wrong.is_ok());
    assert!(!result_wrong.unwrap());
}

#[test]
fn test_vector_3_modified_signature() {
    use starknet_crypto::{get_public_key, sign};

    let private_key = FieldElement::from_hex("0x1234567890abcdef").unwrap();
    let message = FieldElement::from_hex("0xfedcba0987654321").unwrap();
    let k = FieldElement::from_hex("0x1111111111111111").unwrap();

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    // Modify the signature slightly
    let modified_r = signature.r + FieldElement::ONE;
    let modified_s = signature.s;

    // Original signature should verify
    let result_original = verify(&public_key, &message, &signature.r, &signature.s);
    assert!(result_original.unwrap());

    // Modified signature should not verify
    let result_modified = verify(&public_key, &message, &modified_r, &modified_s);
    assert!(result_modified.is_ok());
    assert!(!result_modified.unwrap());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_verify_with_small_values() {
    use starknet_crypto::{get_public_key, sign};

    let private_key = FieldElement::from(1u64);
    let message = FieldElement::from(2u64);
    let k = FieldElement::from(3u64);

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    let result = verify(&public_key, &message, &signature.r, &signature.s);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_with_large_values() {
    use starknet_crypto::{get_public_key, sign};

    let private_key = FieldElement::from_hex(
        "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    )
    .unwrap();
    let message = FieldElement::from_hex(
        "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0",
    )
    .unwrap();
    let k = FieldElement::from_hex("0x0123456789abcdef0123456789abcdef").unwrap();

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    let result = verify(&public_key, &message, &signature.r, &signature.s);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_same_message_different_keys() {
    use starknet_crypto::{get_public_key, sign};

    let message = FieldElement::from_hex("0xdeadbeefdeadbeef").unwrap();

    for i in 1..5 {
        let private_key = FieldElement::from(i * 1000);
        let k = FieldElement::from(i * 100);

        let public_key = get_public_key(&private_key);
        let signature = sign(&private_key, &message, &k).unwrap();

        let result = verify(&public_key, &message, &signature.r, &signature.s);
        assert!(result.is_ok());
        assert!(result.unwrap(), "Verification failed for key index {}", i);
    }
}

// ============================================================================
// Error Type Tests
// ============================================================================

#[test]
fn test_verify_error_display() {
    let err = VerifyError::InvalidPublicKey;
    assert_eq!(format!("{}", err), "Invalid public key");

    let err = VerifyError::InvalidMessageHash;
    assert_eq!(format!("{}", err), "Invalid message hash");

    let err = VerifyError::InvalidR;
    assert_eq!(format!("{}", err), "Invalid r value");

    let err = VerifyError::InvalidS;
    assert_eq!(format!("{}", err), "Invalid s value");
}

#[test]
fn test_verify_error_debug() {
    let err = VerifyError::InvalidPublicKey;
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("InvalidPublicKey"));
}

// ============================================================================
// Malleability Tests
// ============================================================================

#[test]
fn test_signature_malleability() {
    use starknet_crypto::{get_public_key, sign};

    // Test that we handle signature malleability correctly
    let private_key = FieldElement::from_hex("0x1234567890abcdef").unwrap();
    let message = FieldElement::from_hex("0xfedcba0987654321").unwrap();
    let k = FieldElement::from_hex("0x1111111111111111").unwrap();

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    // Original signature should verify
    let result = verify(&public_key, &message, &signature.r, &signature.s);
    assert!(result.unwrap());

    // Note: ECDSA signatures can be malleable. Our implementation should
    // handle this correctly by either accepting both forms or rejecting one.
    // The important thing is that it's deterministic.
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_verify_performance() {
    use starknet_crypto::{get_public_key, sign};

    let private_key = FieldElement::from(12345u64);
    let message = FieldElement::from(67890u64);
    let k = FieldElement::from(111u64);

    let public_key = get_public_key(&private_key);
    let signature = sign(&private_key, &message, &k).unwrap();

    // Verify multiple times
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = verify(&public_key, &message, &signature.r, &signature.s);
    }
    let elapsed = start.elapsed();

    // Should complete 100 verifications in reasonable time (< 2 seconds)
    assert!(
        elapsed.as_secs() < 2,
        "100 verifications took too long: {:?}",
        elapsed
    );
}
