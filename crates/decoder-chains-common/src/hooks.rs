//! Helper functions for hook execution in decoders.
//!
//! This module provides standardized hook execution patterns to ensure
//! consistent behavior across all decoder implementations.

use universal_decoder_core::prelude::{
    DecoderError, HookContext, HookRegistry, HookResult, HookStage, Result,
};

/// Executes pre-decode hooks for a transaction.
///
/// # Arguments
///
/// * `registry` - The hook registry to execute
/// * `raw_bytes` - The raw transaction bytes
///
/// # Returns
///
/// * `Ok(())` if hooks allow decoding to continue
/// * `Err(DecoderError::hook_execution)` if a hook aborts decoding
///
/// # Example
///
/// ```rust,ignore
/// use decoder_chains_common::hooks;
/// use universal_decoder_core::HookRegistry;
///
/// fn decode(raw_bytes: &[u8], registry: &HookRegistry) -> Result<Transaction> {
///     hooks::execute_pre_decode_hooks(registry, raw_bytes)?;
///     // ... continue with decoding ...
///     Ok(transaction)
/// }
/// ```
pub fn execute_pre_decode_hooks(registry: &HookRegistry, raw_bytes: &[u8]) -> Result<()> {
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => Err(DecoderError::hook_execution(msg)),
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => Ok(()),
    }
}

/// Executes post-decode hooks for a transaction.
///
/// # Arguments
///
/// * `registry` - The hook registry to execute
/// * `raw_bytes` - The raw transaction bytes
/// * `chain_specific` - Optional chain-specific data to include in hook context
///
/// # Returns
///
/// * `Ok(())` if hooks complete successfully
/// * `Err(DecoderError::hook_execution)` if a hook fails
///
/// # Example
///
/// ```rust,ignore
/// use decoder_chains_common::hooks;
/// use universal_decoder_core::HookRegistry;
///
/// fn decode(raw_bytes: &[u8], registry: &HookRegistry) -> Result<Transaction> {
///     let tx = parse_transaction(raw_bytes)?;
///     hooks::execute_post_decode_hooks(registry, raw_bytes, Some(&tx))?;
///     Ok(tx)
/// }
/// ```
pub fn execute_post_decode_hooks(
    registry: &HookRegistry,
    raw_bytes: &[u8],
    chain_specific: Option<&dyn std::any::Any>,
) -> Result<()> {
    let mut context = HookContext::new(HookStage::PostDecode, raw_bytes);

    if let Some(data) = chain_specific {
        context = context.with_chain_specific(data);
    }

    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => Err(DecoderError::hook_execution(msg)),
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => Ok(()),
    }
}

/// Executes post-canonicalization hooks.
///
/// # Arguments
///
/// * `registry` - The hook registry to execute
/// * `raw_bytes` - The raw transaction bytes
/// * `tx_ir` - The canonicalized transaction IR (as Any trait object)
///
/// # Returns
///
/// * `Ok(())` if hooks complete successfully
/// * `Err(DecoderError::hook_execution)` if a hook fails
///
/// # Example
///
/// ```rust,ignore
/// use decoder_chains_common::hooks;
/// use universal_decoder_core::HookRegistry;
///
/// fn canonicalize(raw_bytes: &[u8], registry: &HookRegistry) -> Result<TxIR> {
///     let tx_ir = build_tx_ir(raw_bytes)?;
///     hooks::execute_post_canonicalize_hooks(registry, raw_bytes, &tx_ir)?;
///     Ok(tx_ir)
/// }
/// ```
pub fn execute_post_canonicalize_hooks(
    registry: &HookRegistry,
    raw_bytes: &[u8],
    tx_ir: &dyn std::any::Any,
) -> Result<()> {
    let context = HookContext::new(HookStage::PostCanonicalize, raw_bytes).with_tx_ir(tx_ir);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => Err(DecoderError::hook_execution(msg)),
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => Ok(()),
    }
}

/// Generic function to execute decode with hooks.
///
/// This provides a standard pattern for decoders that support hooks:
/// 1. Execute pre-decode hooks
/// 2. Perform decoding
/// 3. Execute post-decode hooks
///
/// # Type Parameters
///
/// * `T` - The transaction type returned by the decoder
/// * `F` - The decode function
///
/// # Arguments
///
/// * `raw_bytes` - The raw transaction bytes
/// * `registry` - The hook registry to execute
/// * `decode_fn` - The actual decode function to call
///
/// # Returns
///
/// The decoded transaction or an error
///
/// # Example
///
/// ```rust,ignore
/// use decoder_chains_common::hooks;
/// use universal_decoder_core::HookRegistry;
///
/// pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<BitcoinTransaction> {
///     hooks::decode_with_hooks(raw_bytes, registry, BitcoinDecoder::decode)
/// }
/// ```
pub fn decode_with_hooks<T, F>(raw_bytes: &[u8], registry: &HookRegistry, decode_fn: F) -> Result<T>
where
    T: 'static,
    F: FnOnce(&[u8]) -> Result<T>,
{
    // Execute pre-decode hooks
    execute_pre_decode_hooks(registry, raw_bytes)?;

    // Perform decoding
    let tx = decode_fn(raw_bytes)?;

    // Execute post-decode hooks
    execute_post_decode_hooks(registry, raw_bytes, Some(&tx as &dyn std::any::Any))?;

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestTransaction {
        version: u32,
    }

    #[test]
    fn test_pre_decode_hooks_continue() {
        let registry = HookRegistry::new();
        let raw_bytes = b"test transaction";

        let result = execute_pre_decode_hooks(&registry, raw_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_post_decode_hooks_continue() {
        let registry = HookRegistry::new();
        let raw_bytes = b"test transaction";
        let tx = TestTransaction { version: 1 };

        let result =
            execute_post_decode_hooks(&registry, raw_bytes, Some(&tx as &dyn std::any::Any));
        assert!(result.is_ok());
    }

    #[test]
    fn test_post_decode_hooks_no_chain_specific() {
        let registry = HookRegistry::new();
        let raw_bytes = b"test transaction";

        let result: Result<()> = execute_post_decode_hooks(&registry, raw_bytes, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_with_hooks_success() {
        let registry = HookRegistry::new();
        let raw_bytes = b"test transaction";

        let decode_fn = |_bytes: &[u8]| Ok(TestTransaction { version: 1 });

        let result = decode_with_hooks(raw_bytes, &registry, decode_fn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, 1);
    }

    #[test]
    fn test_decode_with_hooks_decode_failure() {
        let registry = HookRegistry::new();
        let raw_bytes = b"test transaction";

        let decode_fn = |_bytes: &[u8]| Err(DecoderError::invalid_structure("test error"));

        let result: Result<TestTransaction> = decode_with_hooks(raw_bytes, &registry, decode_fn);
        assert!(result.is_err());
    }
}
