//! Standard validation functions for transaction format checking.
//!
//! This module provides common validation patterns used across all decoders
//! to ensure consistent error handling and reduce code duplication.

use universal_decoder_core::prelude::{DecoderError, Result};

/// Validates that the input is not empty.
///
/// # Arguments
///
/// * `data` - The byte slice to validate
/// * `chain_name` - Name of the blockchain (for error messages)
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(DecoderError::invalid_structure)` if data is empty
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::validation;
///
/// let data = vec![0x01, 0x02, 0x03];
/// assert!(validation::validate_not_empty(&data, "Bitcoin").is_ok());
///
/// let empty: &[u8] = &[];
/// assert!(validation::validate_not_empty(empty, "Bitcoin").is_err());
/// ```
pub fn validate_not_empty(data: &[u8], chain_name: &str) -> Result<()> {
    if data.is_empty() {
        return Err(DecoderError::invalid_structure(format!(
            "{} transaction cannot be empty",
            chain_name
        )));
    }
    Ok(())
}

/// Validates that the input size is within bounds.
///
/// # Arguments
///
/// * `data` - The byte slice to validate
/// * `min_size` - Minimum allowed size in bytes
/// * `max_size` - Maximum allowed size in bytes
/// * `chain_name` - Name of the blockchain (for error messages)
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(DecoderError::invalid_structure)` if size is out of bounds
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::validation;
///
/// let data = vec![0u8; 50];
/// assert!(validation::validate_size_bounds(&data, 10, 100, "Bitcoin").is_ok());
///
/// let too_small = vec![0u8; 5];
/// assert!(validation::validate_size_bounds(&too_small, 10, 100, "Bitcoin").is_err());
///
/// let too_large = vec![0u8; 150];
/// assert!(validation::validate_size_bounds(&too_large, 10, 100, "Bitcoin").is_err());
/// ```
pub fn validate_size_bounds(
    data: &[u8],
    min_size: usize,
    max_size: usize,
    chain_name: &str,
) -> Result<()> {
    let size = data.len();

    if size < min_size {
        return Err(DecoderError::invalid_structure(format!(
            "{} transaction too small: {} bytes (minimum {} bytes)",
            chain_name, size, min_size
        )));
    }

    if size > max_size {
        return Err(DecoderError::invalid_structure(format!(
            "{} transaction too large: {} bytes (maximum {} bytes)",
            chain_name, size, max_size
        )));
    }

    Ok(())
}

/// Validates minimum size requirement.
///
/// # Arguments
///
/// * `data` - The byte slice to validate
/// * `min_size` - Minimum allowed size in bytes
/// * `chain_name` - Name of the blockchain (for error messages)
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(DecoderError::invalid_structure)` if size is too small
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::validation;
///
/// let data = vec![0u8; 50];
/// assert!(validation::validate_min_size(&data, 10, "Bitcoin").is_ok());
///
/// let too_small = vec![0u8; 5];
/// assert!(validation::validate_min_size(&too_small, 10, "Bitcoin").is_err());
/// ```
pub fn validate_min_size(data: &[u8], min_size: usize, chain_name: &str) -> Result<()> {
    let size = data.len();

    if size < min_size {
        return Err(DecoderError::invalid_structure(format!(
            "{} transaction too small: {} bytes (minimum {} bytes)",
            chain_name, size, min_size
        )));
    }

    Ok(())
}

/// Validates maximum size requirement.
///
/// # Arguments
///
/// * `data` - The byte slice to validate
/// * `max_size` - Maximum allowed size in bytes
/// * `chain_name` - Name of the blockchain (for error messages)
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(DecoderError::invalid_structure)` if size is too large
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::validation;
///
/// let data = vec![0u8; 50];
/// assert!(validation::validate_max_size(&data, 100, "Bitcoin").is_ok());
///
/// let too_large = vec![0u8; 150];
/// assert!(validation::validate_max_size(&too_large, 100, "Bitcoin").is_err());
/// ```
pub fn validate_max_size(data: &[u8], max_size: usize, chain_name: &str) -> Result<()> {
    let size = data.len();

    if size > max_size {
        return Err(DecoderError::invalid_structure(format!(
            "{} transaction too large: {} bytes (maximum {} bytes)",
            chain_name, size, max_size
        )));
    }

    Ok(())
}

/// Performs standard format validation (not empty + size bounds).
///
/// This is a convenience function that combines `validate_not_empty` and
/// `validate_size_bounds` into a single call.
///
/// # Arguments
///
/// * `data` - The byte slice to validate
/// * `min_size` - Minimum allowed size in bytes
/// * `max_size` - Maximum allowed size in bytes
/// * `chain_name` - Name of the blockchain (for error messages)
///
/// # Returns
///
/// * `Ok(())` if all validations pass
/// * `Err(DecoderError::invalid_structure)` if any validation fails
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::validation;
///
/// let data = vec![0u8; 50];
/// assert!(validation::validate_format(&data, 10, 100, "Bitcoin").is_ok());
/// ```
pub fn validate_format(
    data: &[u8],
    min_size: usize,
    max_size: usize,
    chain_name: &str,
) -> Result<()> {
    validate_not_empty(data, chain_name)?;
    validate_size_bounds(data, min_size, max_size, chain_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty() {
        let data = vec![0x01, 0x02];
        assert!(validate_not_empty(&data, "Test").is_ok());

        let empty: &[u8] = &[];
        assert!(validate_not_empty(empty, "Test").is_err());
    }

    #[test]
    fn test_validate_size_bounds() {
        let data = vec![0u8; 50];
        assert!(validate_size_bounds(&data, 10, 100, "Test").is_ok());

        let too_small = vec![0u8; 5];
        assert!(validate_size_bounds(&too_small, 10, 100, "Test").is_err());

        let too_large = vec![0u8; 150];
        assert!(validate_size_bounds(&too_large, 10, 100, "Test").is_err());

        let exact_min = vec![0u8; 10];
        assert!(validate_size_bounds(&exact_min, 10, 100, "Test").is_ok());

        let exact_max = vec![0u8; 100];
        assert!(validate_size_bounds(&exact_max, 10, 100, "Test").is_ok());
    }

    #[test]
    fn test_validate_min_size() {
        let data = vec![0u8; 50];
        assert!(validate_min_size(&data, 10, "Test").is_ok());

        let too_small = vec![0u8; 5];
        assert!(validate_min_size(&too_small, 10, "Test").is_err());

        let exact = vec![0u8; 10];
        assert!(validate_min_size(&exact, 10, "Test").is_ok());
    }

    #[test]
    fn test_validate_max_size() {
        let data = vec![0u8; 50];
        assert!(validate_max_size(&data, 100, "Test").is_ok());

        let too_large = vec![0u8; 150];
        assert!(validate_max_size(&too_large, 100, "Test").is_err());

        let exact = vec![0u8; 100];
        assert!(validate_max_size(&exact, 100, "Test").is_ok());
    }

    #[test]
    fn test_validate_format() {
        let data = vec![0u8; 50];
        assert!(validate_format(&data, 10, 100, "Test").is_ok());

        let empty: &[u8] = &[];
        assert!(validate_format(empty, 10, 100, "Test").is_err());

        let too_small = vec![0u8; 5];
        assert!(validate_format(&too_small, 10, 100, "Test").is_err());

        let too_large = vec![0u8; 150];
        assert!(validate_format(&too_large, 10, 100, "Test").is_err());
    }
}
