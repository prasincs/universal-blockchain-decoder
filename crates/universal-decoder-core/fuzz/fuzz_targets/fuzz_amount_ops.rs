#![no_main]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use universal_decoder_core::ir::Amount;

#[derive(Debug)]
struct FuzzInput {
    value1: u128,
    value2: u128,
    decimals: u8,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(FuzzInput {
            value1: u.arbitrary()?,
            value2: u.arbitrary()?,
            decimals: u.int_in_range(0..=38)?,  // Reasonable decimal range
        })
    }
}

fuzz_target!(|input: FuzzInput| {
    let amount1 = Amount {
        value: input.value1,
        decimals: input.decimals,
    };

    let amount2 = Amount {
        value: input.value2,
        decimals: input.decimals,
    };

    // Test that arithmetic operations never panic
    // checked_add should return None on overflow, not panic
    let _ = amount1.checked_add(amount2);

    // Test that checked_sub behaves correctly
    let _ = amount1.checked_sub(amount2);

    // Test serialization never panics
    let _ = borsh::to_vec(&amount1);
    let _ = borsh::to_vec(&amount2);

    // Test that Amount with same value and decimals are equal
    let amount1_copy = Amount {
        value: input.value1,
        decimals: input.decimals,
    };
    assert_eq!(amount1, amount1_copy);
});
