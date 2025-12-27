//! Secure input handling for privacy-sensitive data.
//!
//! This module ensures that sensitive data like viewing keys are:
//! 1. Never exposed in command-line history (read from files/env vars)
//! 2. Protected in memory (using `secrecy` and `zeroize`)
//! 3. Validated for proper file permissions
//! 4. Cleared from memory after use

use anyhow::{anyhow, Context, Result};
use secrecy::{ExposeSecret, Secret};
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// A securely stored viewing key that is zeroized on drop
#[allow(dead_code)]
pub struct SecureViewingKey {
    /// The actual key data, protected by secrecy
    key_data: Secret<Vec<u8>>,
    /// Type of viewing key (for validation)
    key_type: ViewingKeyType,
}

impl SecureViewingKey {
    /// Load viewing key from a file (preferred method)
    ///
    /// # Security
    ///
    /// - Validates file permissions (must be 0600 or 0400 on Unix)
    /// - Reads into protected memory
    /// - Returns error if file is world-readable
    pub fn from_file(path: &Path, key_type: ViewingKeyType) -> Result<Self> {
        // Validate file permissions on Unix systems
        #[cfg(unix)]
        Self::validate_file_permissions(path)?;

        // Read file contents into protected memory
        let key_bytes = fs::read(path)
            .with_context(|| format!("Failed to read viewing key from {}", path.display()))?;

        Self::from_bytes(key_bytes, key_type)
    }

    /// Load viewing key from environment variable (fallback)
    ///
    /// # Security
    ///
    /// Environment variables are safer than CLI args but less secure than files.
    /// Prefer `from_file()` when possible.
    pub fn from_env(var_name: &str, key_type: ViewingKeyType) -> Result<Self> {
        let key_hex = std::env::var(var_name)
            .with_context(|| format!("Environment variable {} not set", var_name))?;

        // Decode hex string
        let key_bytes = universal_decoder_core::hex::decode(&key_hex)
            .with_context(|| format!("Failed to decode hex from {}", var_name))?;

        Self::from_bytes(key_bytes, key_type)
    }

    /// Create from raw bytes (internal use)
    fn from_bytes(key_bytes: Vec<u8>, key_type: ViewingKeyType) -> Result<Self> {
        // Validate key length based on type
        key_type.validate_length(key_bytes.len())?;

        Ok(Self {
            key_data: Secret::new(key_bytes),
            key_type,
        })
    }

    /// Get the key type
    #[allow(dead_code)]
    pub fn key_type(&self) -> ViewingKeyType {
        self.key_type
    }

    /// Access the key data (use sparingly!)
    ///
    /// # Security Warning
    ///
    /// This exposes the secret key data. Use only when absolutely necessary
    /// and ensure the data is not logged or stored insecurely.
    #[allow(dead_code)]
    pub fn expose(&self) -> &[u8] {
        self.key_data.expose_secret()
    }

    /// Validate file permissions (Unix only)
    #[cfg(unix)]
    fn validate_file_permissions(path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Extract permission bits (last 9 bits)
        let perms = mode & 0o777;

        // Allow only 0600 (rw-------) or 0400 (r--------)
        // Reject any group or world permissions
        if perms & 0o077 != 0 {
            return Err(anyhow!(
                "Insecure file permissions for {}: {:o}. Must be 0600 or 0400 (no group/world access)",
                path.display(),
                perms
            ));
        }

        Ok(())
    }

    /// Validate file permissions (non-Unix: always succeeds)
    #[cfg(not(unix))]
    fn validate_file_permissions(_path: &Path) -> Result<()> {
        // On non-Unix systems, we can't easily check permissions
        // Log a warning instead
        eprintln!("Warning: Cannot validate file permissions on this platform");
        Ok(())
    }
}

/// Type of viewing key (determines validation rules)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewingKeyType {
    /// Zcash incoming viewing key (32 bytes)
    ZcashIncoming,
    /// Zcash full viewing key (96 bytes for Sapling)
    ZcashFull,
    /// Monero view key (32 bytes)
    Monero,
    /// Custom/unknown key type (no validation)
    Custom,
}

impl ViewingKeyType {
    /// Validate key length based on type
    fn validate_length(&self, len: usize) -> Result<()> {
        match self {
            ViewingKeyType::ZcashIncoming => {
                if len != 32 {
                    return Err(anyhow!(
                        "Invalid Zcash incoming viewing key length: {} (expected 32 bytes)",
                        len
                    ));
                }
            }
            ViewingKeyType::ZcashFull => {
                if len != 96 {
                    return Err(anyhow!(
                        "Invalid Zcash full viewing key length: {} (expected 96 bytes)",
                        len
                    ));
                }
            }
            ViewingKeyType::Monero => {
                if len != 32 {
                    return Err(anyhow!(
                        "Invalid Monero view key length: {} (expected 32 bytes)",
                        len
                    ));
                }
            }
            ViewingKeyType::Custom => {
                // No validation for custom keys
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for ViewingKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewingKeyType::ZcashIncoming => write!(f, "Zcash Incoming Viewing Key"),
            ViewingKeyType::ZcashFull => write!(f, "Zcash Full Viewing Key"),
            ViewingKeyType::Monero => write!(f, "Monero View Key"),
            ViewingKeyType::Custom => write!(f, "Custom Viewing Key"),
        }
    }
}

/// Secure hex input that doesn't leave traces in shell history
pub struct SecureHexInput {
    data: Vec<u8>,
}

impl SecureHexInput {
    /// Read hex from file (avoids shell history)
    pub fn from_file(path: &Path) -> Result<Self> {
        let hex_string = fs::read_to_string(path)
            .with_context(|| format!("Failed to read hex from {}", path.display()))?;

        Self::from_hex_string(&hex_string)
    }

    /// Read hex from stdin (avoids shell history)
    pub fn from_stdin() -> Result<Self> {
        use std::io::Read;

        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;

        Self::from_hex_string(&buffer)
    }

    /// Create from hex string (public for CLI use)
    pub fn from_hex_string(hex: &str) -> Result<Self> {
        let hex_trimmed = hex.trim();

        // Validate hex format
        if hex_trimmed.is_empty() {
            return Err(anyhow!("Empty hex string"));
        }

        if !hex_trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!("Invalid hex string (contains non-hex characters)"));
        }

        if hex_trimmed.len() % 2 != 0 {
            return Err(anyhow!("Invalid hex string (odd length)"));
        }

        // Decode hex
        let data = universal_decoder_core::hex::decode(hex_trimmed)
            .context("Failed to decode hex string")?;

        Ok(Self { data })
    }

    /// Get the decoded bytes
    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Consume into bytes (clones to avoid Drop conflict)
    pub fn into_bytes(self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Drop for SecureHexInput {
    fn drop(&mut self) {
        // Zeroize on drop
        self.data.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_viewing_key_type_validation() {
        // Valid lengths
        assert!(ViewingKeyType::ZcashIncoming.validate_length(32).is_ok());
        assert!(ViewingKeyType::ZcashFull.validate_length(96).is_ok());
        assert!(ViewingKeyType::Monero.validate_length(32).is_ok());
        assert!(ViewingKeyType::Custom.validate_length(1000).is_ok());

        // Invalid lengths
        assert!(ViewingKeyType::ZcashIncoming.validate_length(31).is_err());
        assert!(ViewingKeyType::ZcashFull.validate_length(95).is_err());
        assert!(ViewingKeyType::Monero.validate_length(33).is_err());
    }

    #[test]
    fn test_secure_hex_input() {
        let hex = "deadbeef";
        let input = SecureHexInput::from_hex_string(hex).unwrap();
        assert_eq!(input.as_bytes(), &[0xde, 0xad, 0xbe, 0xef]);

        // Invalid hex
        assert!(SecureHexInput::from_hex_string("not hex").is_err());
        assert!(SecureHexInput::from_hex_string("abc").is_err()); // Odd length
        assert!(SecureHexInput::from_hex_string("").is_err()); // Empty
    }

    #[test]
    #[cfg(unix)]
    fn test_file_permissions_validation() {
        use std::os::unix::fs::PermissionsExt;

        // Create temp file with secure permissions
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_key");

        // Write test data
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"a".repeat(32).as_slice()).unwrap();

        // Set secure permissions (0600)
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&file_path, perms).unwrap();

        // Should succeed
        assert!(SecureViewingKey::validate_file_permissions(&file_path).is_ok());

        // Set insecure permissions (0644 - world readable)
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();

        // Should fail
        assert!(SecureViewingKey::validate_file_permissions(&file_path).is_err());
    }

    #[test]
    fn test_viewing_key_from_bytes() {
        let key_bytes = vec![0x42; 32];
        let key =
            SecureViewingKey::from_bytes(key_bytes.clone(), ViewingKeyType::ZcashIncoming).unwrap();

        assert_eq!(key.expose(), &key_bytes);
        assert_eq!(key.key_type(), ViewingKeyType::ZcashIncoming);
    }
}
