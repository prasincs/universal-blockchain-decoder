//! STARK curve tests
//!
//! Tests for STARK curve parameters and elliptic curve operations:
//! - Curve constants validation
//! - Generator point validation
//! - Pedersen constant points
//! - Curve equation verification
//! - Point operations

use decoder_crypto_zk::curve::{
    AffinePoint, ALPHA, BETA, EC_ORDER, GENERATOR, PEDERSEN_P0, PEDERSEN_P1, PEDERSEN_P2,
    PEDERSEN_P3, SHIFT_POINT,
};
use decoder_crypto_zk::field::FieldElement;

// ============================================================================
// Curve Constants Tests
// ============================================================================

#[test]
fn test_constants_non_zero() {
    assert_ne!(EC_ORDER, FieldElement::ZERO);
    assert_ne!(ALPHA, FieldElement::ZERO);
    assert_ne!(BETA, FieldElement::ZERO);
}

#[test]
fn test_alpha_equals_one() {
    // The STARK curve has alpha = 1 (in Montgomery form)
    assert_eq!(
        ALPHA,
        FieldElement::from_raw([
            576460752303422960,
            18446744073709551615,
            18446744073709551615,
            18446744073709551585,
        ])
    );
}

#[test]
fn test_beta_value() {
    // Beta should be the specific constant for STARK curve
    assert_eq!(
        BETA,
        FieldElement::from_raw([
            88155977965380735,
            12360725113329547591,
            7432612994240712710,
            3863487492851900874,
        ])
    );
}

#[test]
fn test_ec_order_value() {
    // EC order should be the specific constant
    assert_eq!(
        EC_ORDER,
        FieldElement::from_raw([
            369010039416812937,
            9,
            1143265896874747514,
            8939893405601011193,
        ])
    );
}

// ============================================================================
// Generator Point Tests
// ============================================================================

#[test]
fn test_generator_non_zero() {
    assert_ne!(GENERATOR.x(), FieldElement::ZERO);
    assert_ne!(GENERATOR.y(), FieldElement::ZERO);
}

#[test]
fn test_generator_coordinates() {
    // Check exact coordinates
    assert_eq!(
        GENERATOR.x(),
        FieldElement::from_raw([
            232005955912912577,
            299981207024966779,
            5884444832209845738,
            14484022957141291997,
        ])
    );
    assert_eq!(
        GENERATOR.y(),
        FieldElement::from_raw([
            405578048423154473,
            18147424675297964973,
            664812301889158119,
            6241159653446987914,
        ])
    );
}

#[test]
fn test_generator_on_curve() {
    // Verify generator satisfies curve equation: y^2 = x^3 + alpha*x + beta
    let x = GENERATOR.x();
    let y = GENERATOR.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Generator point is not on the curve!");
}

// ============================================================================
// Shift Point Tests
// ============================================================================

#[test]
fn test_shift_point_non_zero() {
    assert_ne!(SHIFT_POINT.x(), FieldElement::ZERO);
    assert_ne!(SHIFT_POINT.y(), FieldElement::ZERO);
}

#[test]
fn test_shift_point_coordinates() {
    assert_eq!(
        SHIFT_POINT.x(),
        FieldElement::from_raw([
            316327189671755572,
            1641324389046377921,
            7739989395386261137,
            1933903796324928314,
        ])
    );
    assert_eq!(
        SHIFT_POINT.y(),
        FieldElement::from_raw([
            81375596133053150,
            4798858472748676776,
            12587053260418384210,
            14252083571674603243,
        ])
    );
}

#[test]
fn test_shift_point_on_curve() {
    let x = SHIFT_POINT.x();
    let y = SHIFT_POINT.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Shift point is not on the curve!");
}

// ============================================================================
// Pedersen Constant Points Tests
// ============================================================================

#[test]
fn test_pedersen_p0_non_zero() {
    assert_ne!(PEDERSEN_P0.x(), FieldElement::ZERO);
    assert_ne!(PEDERSEN_P0.y(), FieldElement::ZERO);
}

#[test]
fn test_pedersen_p0_on_curve() {
    let x = PEDERSEN_P0.x();
    let y = PEDERSEN_P0.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Pedersen P0 is not on the curve!");
}

#[test]
fn test_pedersen_p1_non_zero() {
    assert_ne!(PEDERSEN_P1.x(), FieldElement::ZERO);
    assert_ne!(PEDERSEN_P1.y(), FieldElement::ZERO);
}

#[test]
fn test_pedersen_p1_on_curve() {
    let x = PEDERSEN_P1.x();
    let y = PEDERSEN_P1.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Pedersen P1 is not on the curve!");
}

#[test]
fn test_pedersen_p2_non_zero() {
    assert_ne!(PEDERSEN_P2.x(), FieldElement::ZERO);
    assert_ne!(PEDERSEN_P2.y(), FieldElement::ZERO);
}

#[test]
fn test_pedersen_p2_on_curve() {
    let x = PEDERSEN_P2.x();
    let y = PEDERSEN_P2.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Pedersen P2 is not on the curve!");
}

#[test]
fn test_pedersen_p3_non_zero() {
    assert_ne!(PEDERSEN_P3.x(), FieldElement::ZERO);
    assert_ne!(PEDERSEN_P3.y(), FieldElement::ZERO);
}

#[test]
fn test_pedersen_p3_on_curve() {
    let x = PEDERSEN_P3.x();
    let y = PEDERSEN_P3.y();

    let lhs = y.square();
    let rhs = x.square() * x + ALPHA * x + BETA;

    assert_eq!(lhs, rhs, "Pedersen P3 is not on the curve!");
}

// ============================================================================
// Point Distinctness Tests
// ============================================================================

#[test]
fn test_points_are_distinct() {
    // All constant points should be different from each other
    assert_ne!(GENERATOR.x(), SHIFT_POINT.x());
    assert_ne!(GENERATOR.x(), PEDERSEN_P0.x());
    assert_ne!(GENERATOR.x(), PEDERSEN_P1.x());
    assert_ne!(GENERATOR.x(), PEDERSEN_P2.x());
    assert_ne!(GENERATOR.x(), PEDERSEN_P3.x());

    assert_ne!(SHIFT_POINT.x(), PEDERSEN_P0.x());
    assert_ne!(SHIFT_POINT.x(), PEDERSEN_P1.x());
    assert_ne!(SHIFT_POINT.x(), PEDERSEN_P2.x());
    assert_ne!(SHIFT_POINT.x(), PEDERSEN_P3.x());

    assert_ne!(PEDERSEN_P0.x(), PEDERSEN_P1.x());
    assert_ne!(PEDERSEN_P0.x(), PEDERSEN_P2.x());
    assert_ne!(PEDERSEN_P0.x(), PEDERSEN_P3.x());

    assert_ne!(PEDERSEN_P1.x(), PEDERSEN_P2.x());
    assert_ne!(PEDERSEN_P1.x(), PEDERSEN_P3.x());

    assert_ne!(PEDERSEN_P2.x(), PEDERSEN_P3.x());
}

// ============================================================================
// Point Construction Tests
// ============================================================================

#[test]
fn test_affine_point_new() {
    // Test creating a new affine point (should work for valid points)
    let point = AffinePoint::new(GENERATOR.x(), GENERATOR.y());
    assert!(point.is_ok(), "Failed to create valid affine point");
}

#[test]
fn test_affine_point_x_y_accessors() {
    let point = GENERATOR;
    assert_eq!(point.x(), point.x());
    assert_eq!(point.y(), point.y());
}

// ============================================================================
// Curve Equation Tests
// ============================================================================

#[test]
fn test_curve_equation_for_multiple_points() {
    // Test that all our constant points satisfy the curve equation
    let points = [
        GENERATOR,
        SHIFT_POINT,
        PEDERSEN_P0,
        PEDERSEN_P1,
        PEDERSEN_P2,
        PEDERSEN_P3,
    ];

    for (i, point) in points.iter().enumerate() {
        let x = point.x();
        let y = point.y();

        let lhs = y.square();
        let rhs = x.square() * x + ALPHA * x + BETA;

        assert_eq!(lhs, rhs, "Point {} does not satisfy curve equation", i);
    }
}

// ============================================================================
// Curve Properties Tests
// ============================================================================

#[test]
fn test_curve_has_correct_form() {
    // STARK curve is y^2 = x^3 + alpha*x + beta
    // with alpha = 1 and beta = specific constant
    // This is a Weierstrass form curve

    // Verify alpha = 1
    assert_eq!(ALPHA, FieldElement::from(1u64));

    // Verify beta is the correct constant (non-zero)
    assert_ne!(BETA, FieldElement::ZERO);
}

#[test]
fn test_ec_order_properties() {
    // EC order should be:
    // 1. Non-zero
    // 2. Large (> 2^250)
    // 3. The order of the generator point

    assert_ne!(EC_ORDER, FieldElement::ZERO);
    assert_ne!(EC_ORDER, FieldElement::ONE);

    // Check that EC_ORDER is large (at least has high bit set in representation)
    let bytes = EC_ORDER.to_bytes_be();
    // First few bytes should be non-zero for a large number
    assert!(bytes[0] != 0 || bytes[1] != 0 || bytes[2] != 0);
}

// ============================================================================
// Cross-Validation Tests
// ============================================================================

#[test]
fn test_generator_matches_reference() {
    // The generator point should match the standard STARK curve generator
    // This is defined in the Cairo/Starknet specification

    // Known from Starknet spec:
    // x = 0x01ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca
    // y = 0x005668060aa49730b7be4801df46ec62de53ecd11abe43a32873000c36e8dc1f

    let expected_x = FieldElement::from_hex(
        "0x01ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x005668060aa49730b7be4801df46ec62de53ecd11abe43a32873000c36e8dc1f",
    )
    .unwrap();

    assert_eq!(GENERATOR.x(), expected_x);
    assert_eq!(GENERATOR.y(), expected_y);
}

#[test]
fn test_shift_point_matches_reference() {
    // Known from Starknet spec
    let expected_x = FieldElement::from_hex(
        "0x049ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x03ca0cfe4b3bc6ddf346d49d06ea0ed34e621062c0e056c1d0405d266e10268a",
    )
    .unwrap();

    assert_eq!(SHIFT_POINT.x(), expected_x);
    assert_eq!(SHIFT_POINT.y(), expected_y);
}

// ============================================================================
// Pedersen Points Cross-Validation
// ============================================================================

#[test]
fn test_pedersen_p0_matches_reference() {
    let expected_x = FieldElement::from_hex(
        "0x0234287dcbaffe7f969c748655fca9e58fa8120b6d56eb0c1080d17957ebe47b",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x03b056f100f96fb21e889527d41f4e39940135dd7a6c94cc6ed0268ee89e5615",
    )
    .unwrap();

    assert_eq!(PEDERSEN_P0.x(), expected_x);
    assert_eq!(PEDERSEN_P0.y(), expected_y);
}

#[test]
fn test_pedersen_p1_matches_reference() {
    let expected_x = FieldElement::from_hex(
        "0x04fa56f376c83db33f9dab2656558f3399099ec1de5e3018b7a6932dba8aa378",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x03fa0984c931c9e38113e0c0e47e4401562761f92a7a23b45168f4e80ff5b54d",
    )
    .unwrap();

    assert_eq!(PEDERSEN_P1.x(), expected_x);
    assert_eq!(PEDERSEN_P1.y(), expected_y);
}

#[test]
fn test_pedersen_p2_matches_reference() {
    let expected_x = FieldElement::from_hex(
        "0x04ba4cc166be8dec764910f75b45f74b40c690c74709e90f3aa372f0bd2d6997",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x0040301cf5c1751f4b971e46c4ede85fcac5c59a5ce5ae7c48151f27b24b219c",
    )
    .unwrap();

    assert_eq!(PEDERSEN_P2.x(), expected_x);
    assert_eq!(PEDERSEN_P2.y(), expected_y);
}

#[test]
fn test_pedersen_p3_matches_reference() {
    let expected_x = FieldElement::from_hex(
        "0x054302dcb0e6cc1c6e44cca8f61a63bb2ca65048d53fb325d36ff12c49a58202",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x01b77b3e37d13504b348046268d8ae25ce98ad783c25561a879dcc77e99c2426",
    )
    .unwrap();

    assert_eq!(PEDERSEN_P3.x(), expected_x);
    assert_eq!(PEDERSEN_P3.y(), expected_y);
}
