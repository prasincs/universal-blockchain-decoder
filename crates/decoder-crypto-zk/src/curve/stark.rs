//! STARK curve parameters
//!
//! This module provides all constants needed for STARK curve operations,
//! including ECDSA signature verification and Pedersen hash computation.
//!
//! # Curve Definition
//!
//! The STARK curve is defined by the Weierstrass equation:
//! ```text
//! y^2 = x^3 + alpha * x + beta (mod p)
//! ```
//!
//! Where:
//! - `p = 2^251 + 17 * 2^192 + 1` (STARK field prime)
//! - `alpha = 1`
//! - `beta = 0x06f21413efbe40de150e596d72f7a8c5609ad26c15c915c1f4cdfcb99cee9e89`
//!
//! # Usage
//!
//! ```
//! use decoder_crypto_zk::curve::{GENERATOR, EC_ORDER};
//! use decoder_crypto_zk::field::FieldElement;
//!
//! // Generator point is on the curve
//! let g = GENERATOR;
//! assert_ne!(g.x(), FieldElement::ZERO);
//! ```

use crate::field::FieldElement;
pub use starknet_types_core::curve::AffinePoint;

/// EC order of the STARK curve for ECDSA.
///
/// Value: `0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f`
///
/// This is the order of the generator point, meaning:
/// ```text
/// EC_ORDER * GENERATOR = point_at_infinity
/// ```
pub const EC_ORDER: FieldElement = FieldElement::from_raw([
    369010039416812937,
    9,
    1143265896874747514,
    8939893405601011193,
]);

/// The alpha parameter of the STARK curve.
///
/// Value: `1`
///
/// Used in the curve equation: `y^2 = x^3 + alpha * x + beta`
pub const ALPHA: FieldElement = FieldElement::from_raw([
    576460752303422960,
    18446744073709551615,
    18446744073709551615,
    18446744073709551585,
]);

/// The beta parameter of the STARK curve.
///
/// Value: `0x06f21413efbe40de150e596d72f7a8c5609ad26c15c915c1f4cdfcb99cee9e89`
///
/// Used in the curve equation: `y^2 = x^3 + alpha * x + beta`
pub const BETA: FieldElement = FieldElement::from_raw([
    88155977965380735,
    12360725113329547591,
    7432612994240712710,
    3863487492851900874,
]);

/// Generator point of the STARK curve.
///
/// Coordinates:
/// - x: `0x01ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca`
/// - y: `0x005668060aa49730b7be4801df46ec62de53ecd11abe43a32873000c36e8dc1f`
///
/// This is the base point used for all ECDSA operations on the STARK curve.
pub const GENERATOR: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        232005955912912577,
        299981207024966779,
        5884444832209845738,
        14484022957141291997,
    ]),
    FieldElement::from_raw([
        405578048423154473,
        18147424675297964973,
        664812301889158119,
        6241159653446987914,
    ]),
);

/// Shift point of the STARK curve.
///
/// Coordinates:
/// - x: `0x049ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804`
/// - y: `0x03ca0cfe4b3bc6ddf346d49d06ea0ed34e621062c0e056c1d0405d266e10268a`
///
/// Used in various cryptographic protocols on Starknet.
pub const SHIFT_POINT: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        316327189671755572,
        1641324389046377921,
        7739989395386261137,
        1933903796324928314,
    ]),
    FieldElement::from_raw([
        81375596133053150,
        4798858472748676776,
        12587053260418384210,
        14252083571674603243,
    ]),
);

/// The P0 constant point for Pedersen hash.
///
/// Coordinates:
/// - x: `0x0234287dcbaffe7f969c748655fca9e58fa8120b6d56eb0c1080d17957ebe47b`
/// - y: `0x03b056f100f96fb21e889527d41f4e39940135dd7a6c94cc6ed0268ee89e5615`
///
/// Used in the Pedersen hash function for bit decomposition.
pub const PEDERSEN_P0: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        241691544791834578,
        518715844721862878,
        13758484295849329960,
        3602345268353203007,
    ]),
    FieldElement::from_raw([
        368891789801938570,
        433857700841878496,
        13001553326386915570,
        13441546676070136227,
    ]),
);

/// The P1 constant point for Pedersen hash.
///
/// Coordinates:
/// - x: `0x04fa56f376c83db33f9dab2656558f3399099ec1de5e3018b7a6932dba8aa378`
/// - y: `0x03fa0984c931c9e38113e0c0e47e4401562761f92a7a23b45168f4e80ff5b54d`
///
/// Used in the Pedersen hash function for bit decomposition.
pub const PEDERSEN_P1: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        253000153565733272,
        10043949394709899044,
        12382025591154462459,
        16491878934996302286,
    ]),
    FieldElement::from_raw([
        285630633187035523,
        5191292837124484988,
        2545498000137298346,
        13950428914333633429,
    ]),
);

/// The P2 constant point for Pedersen hash.
///
/// Coordinates:
/// - x: `0x04ba4cc166be8dec764910f75b45f74b40c690c74709e90f3aa372f0bd2d6997`
/// - y: `0x0040301cf5c1751f4b971e46c4ede85fcac5c59a5ce5ae7c48151f27b24b219c`
///
/// Used in the Pedersen hash function for bit decomposition.
pub const PEDERSEN_P2: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        338510149841406402,
        12916675983929588442,
        18195981508842736832,
        1203723169299412240,
    ]),
    FieldElement::from_raw([
        161068411212710156,
        11088962269971685343,
        11743524503750604092,
        12352616181161700245,
    ]),
);

/// The P3 constant point for Pedersen hash.
///
/// Coordinates:
/// - x: `0x054302dcb0e6cc1c6e44cca8f61a63bb2ca65048d53fb325d36ff12c49a58202`
/// - y: `0x01b77b3e37d13504b348046268d8ae25ce98ad783c25561a879dcc77e99c2426`
///
/// Used in the Pedersen hash function for bit decomposition.
pub const PEDERSEN_P3: AffinePoint = AffinePoint::new_unchecked(
    FieldElement::from_raw([
        425493972656615276,
        299781701614706065,
        10664803185694787051,
        1145636535101238356,
    ]),
    FieldElement::from_raw([
        345457391846365716,
        6033691581221864148,
        4428713245976508844,
        8187986478389849302,
    ]),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_constants_non_zero() {
        // Ensure all curve constants are properly initialized
        assert_ne!(EC_ORDER, FieldElement::ZERO);
        assert_ne!(ALPHA, FieldElement::ZERO);
        assert_ne!(BETA, FieldElement::ZERO);

        assert_ne!(GENERATOR.x(), FieldElement::ZERO);
        assert_ne!(GENERATOR.y(), FieldElement::ZERO);

        assert_ne!(SHIFT_POINT.x(), FieldElement::ZERO);
        assert_ne!(SHIFT_POINT.y(), FieldElement::ZERO);
    }

    #[test]
    fn test_pedersen_points_non_zero() {
        // Ensure all Pedersen constant points are properly initialized
        assert_ne!(PEDERSEN_P0.x(), FieldElement::ZERO);
        assert_ne!(PEDERSEN_P0.y(), FieldElement::ZERO);

        assert_ne!(PEDERSEN_P1.x(), FieldElement::ZERO);
        assert_ne!(PEDERSEN_P1.y(), FieldElement::ZERO);

        assert_ne!(PEDERSEN_P2.x(), FieldElement::ZERO);
        assert_ne!(PEDERSEN_P2.y(), FieldElement::ZERO);

        assert_ne!(PEDERSEN_P3.x(), FieldElement::ZERO);
        assert_ne!(PEDERSEN_P3.y(), FieldElement::ZERO);
    }

    #[test]
    fn test_alpha_equals_one() {
        // The STARK curve has alpha = 1
        // Note: This is 1 in Montgomery form
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
}
