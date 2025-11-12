//! Primitive readers for different byte orderings

pub mod big_endian;
pub mod little_endian;

// Re-export commonly used readers
pub use big_endian::{
    read_address, read_u128_be, read_u16_be, read_u256_be, read_u32_be, read_u64_be,
};
pub use little_endian::{
    read_i32_le, read_u128_le, read_u16_le, read_u32_le, read_u64_le, read_u8,
};
