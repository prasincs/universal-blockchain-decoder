//! Hook system for extensible transaction decoding.
//!
//! This module provides a flexible hook system that allows applications to inject
//! custom processing logic at various stages of the decoding pipeline.

use crate::error::{DecoderError, Result};
use crate::ir::TxIR;
use std::sync::Arc;

/// Represents different stages in the decoding pipeline where hooks can be executed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookStage {
    /// Before any decoding begins (raw bytes validation)
    PreDecode,
    /// After chain-specific decoding, before canonicalization
    PostDecode,
    /// After canonicalization into TxIR
    PostCanonicalize,
    /// Before verification
    PreVerify,
    /// After successful verification
    PostVerify,
}

/// Context passed to hooks during execution
pub struct HookContext<'a> {
    /// The current stage
    pub stage: HookStage,
    /// Raw transaction bytes (always available)
    pub raw_bytes: &'a [u8],
    /// Chain-specific data (opaque, as type is not known at this level)
    pub chain_specific: Option<&'a dyn std::any::Any>,
    /// The canonical IR (available in post-canonicalize and later stages)
    pub tx_ir: Option<&'a dyn std::any::Any>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl<'a> HookContext<'a> {
    /// Create a new hook context for a specific stage
    pub fn new(stage: HookStage, raw_bytes: &'a [u8]) -> Self {
        Self {
            stage,
            raw_bytes,
            chain_specific: None,
            tx_ir: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set the chain-specific data
    pub fn with_chain_specific(mut self, data: &'a dyn std::any::Any) -> Self {
        self.chain_specific = Some(data);
        self
    }

    /// Set the TxIR data
    pub fn with_tx_ir(mut self, ir: &'a dyn std::any::Any) -> Self {
        self.tx_ir = Some(ir);
        self
    }

    /// Try to downcast the TxIR to a specific type
    pub fn get_tx_ir<const V: u8>(&self) -> Option<&TxIR<V>> {
        self.tx_ir?.downcast_ref::<TxIR<V>>()
    }
}

/// Result of hook execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookResult {
    /// Continue processing normally
    Continue,
    /// Skip remaining hooks at this stage but continue pipeline
    Skip,
    /// Stop the entire pipeline with an error
    Abort(String),
    /// Continue with modified metadata
    ContinueWithMetadata(std::collections::HashMap<String, String>),
}

/// Trait for implementing custom hooks
pub trait Hook: Send + Sync {
    /// Get the hook name
    fn name(&self) -> &str;

    /// Get the stages this hook should be executed on
    fn stages(&self) -> Vec<HookStage>;

    /// Execute the hook
    fn execute(&self, context: &HookContext) -> Result<HookResult>;

    /// Priority for execution order (higher priority runs first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Hook registry for managing and executing hooks
pub struct HookRegistry {
    hooks: Vec<Arc<dyn Hook>>,
}

impl HookRegistry {
    /// Create a new empty hook registry
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a new hook
    pub fn register<H: Hook + 'static>(&mut self, hook: H) {
        self.hooks.push(Arc::new(hook));
        // Sort by priority (descending)
        self.hooks.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Execute all hooks for a given stage
    pub fn execute_stage(&self, context: &HookContext) -> Result<HookResult> {
        let mut accumulated_metadata = std::collections::HashMap::new();

        for hook in &self.hooks {
            // Check if this hook is registered for this stage
            if !hook.stages().contains(&context.stage) {
                continue;
            }

            // Execute the hook
            match hook.execute(context)? {
                HookResult::Continue => {
                    // Continue to next hook
                }
                HookResult::Skip => {
                    // Skip remaining hooks but continue pipeline
                    return Ok(HookResult::Skip);
                }
                HookResult::Abort(msg) => {
                    // Stop the entire pipeline
                    return Err(DecoderError::hook_execution(format!(
                        "Hook '{}' aborted pipeline: {}",
                        hook.name(),
                        msg
                    )));
                }
                HookResult::ContinueWithMetadata(metadata) => {
                    // Accumulate metadata
                    accumulated_metadata.extend(metadata);
                }
            }
        }

        if accumulated_metadata.is_empty() {
            Ok(HookResult::Continue)
        } else {
            Ok(HookResult::ContinueWithMetadata(accumulated_metadata))
        }
    }

    /// Get the number of registered hooks
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Remove all hooks
    pub fn clear(&mut self) {
        self.hooks.clear();
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple logging hook for debugging
pub struct LoggingHook {
    name: String,
    stages: Vec<HookStage>,
}

impl LoggingHook {
    pub fn new(name: String, stages: Vec<HookStage>) -> Self {
        Self { name, stages }
    }

    pub fn all_stages(name: String) -> Self {
        Self {
            name,
            stages: vec![
                HookStage::PreDecode,
                HookStage::PostDecode,
                HookStage::PostCanonicalize,
                HookStage::PreVerify,
                HookStage::PostVerify,
            ],
        }
    }
}

impl Hook for LoggingHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn stages(&self) -> Vec<HookStage> {
        self.stages.clone()
    }

    fn execute(&self, context: &HookContext) -> Result<HookResult> {
        eprintln!(
            "[{}] Stage: {:?}, Raw bytes length: {}",
            self.name,
            context.stage,
            context.raw_bytes.len()
        );
        Ok(HookResult::Continue)
    }
}

/// A validation hook that checks transaction size limits
pub struct SizeLimitHook {
    max_size: usize,
}

impl SizeLimitHook {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Hook for SizeLimitHook {
    fn name(&self) -> &str {
        "size_limit"
    }

    fn stages(&self) -> Vec<HookStage> {
        vec![HookStage::PreDecode]
    }

    fn execute(&self, context: &HookContext) -> Result<HookResult> {
        if context.raw_bytes.len() > self.max_size {
            Ok(HookResult::Abort(format!(
                "Transaction size {} exceeds maximum {}",
                context.raw_bytes.len(),
                self.max_size
            )))
        } else {
            Ok(HookResult::Continue)
        }
    }

    fn priority(&self) -> i32 {
        100 // High priority - validate size early
    }
}

/// Builder for constructing hook registries
pub struct HookRegistryBuilder {
    registry: HookRegistry,
}

impl HookRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: HookRegistry::new(),
        }
    }

    pub fn with_hook<H: Hook + 'static>(mut self, hook: H) -> Self {
        self.registry.register(hook);
        self
    }

    pub fn with_logging(self, name: String, stages: Vec<HookStage>) -> Self {
        self.with_hook(LoggingHook::new(name, stages))
    }

    pub fn with_size_limit(self, max_size: usize) -> Self {
        self.with_hook(SizeLimitHook::new(max_size))
    }

    pub fn build(self) -> HookRegistry {
        self.registry
    }
}

impl Default for HookRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook {
        should_abort: bool,
    }

    impl Hook for TestHook {
        fn name(&self) -> &str {
            "test_hook"
        }

        fn stages(&self) -> Vec<HookStage> {
            vec![HookStage::PreDecode]
        }

        fn execute(&self, _context: &HookContext) -> Result<HookResult> {
            if self.should_abort {
                Ok(HookResult::Abort("test abort".to_string()))
            } else {
                Ok(HookResult::Continue)
            }
        }
    }

    #[test]
    fn test_hook_registry() {
        let mut registry = HookRegistry::new();
        assert!(registry.is_empty());

        registry.register(TestHook { should_abort: false });
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_hook_execution() {
        let mut registry = HookRegistry::new();
        registry.register(TestHook { should_abort: false });

        let context = HookContext::new(HookStage::PreDecode, b"test");
        let result = registry.execute_stage(&context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HookResult::Continue);
    }

    #[test]
    fn test_hook_abort() {
        let mut registry = HookRegistry::new();
        registry.register(TestHook { should_abort: true });

        let context = HookContext::new(HookStage::PreDecode, b"test");
        let result = registry.execute_stage(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_size_limit_hook() {
        let mut registry = HookRegistry::new();
        registry.register(SizeLimitHook::new(10));

        // Test within limit
        let context = HookContext::new(HookStage::PreDecode, b"small");
        let result = registry.execute_stage(&context);
        assert!(result.is_ok());

        // Test exceeding limit
        let context = HookContext::new(HookStage::PreDecode, b"this is a very long transaction");
        let result = registry.execute_stage(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_pattern() {
        let registry = HookRegistryBuilder::new()
            .with_size_limit(1024)
            .with_logging("test".to_string(), vec![HookStage::PreDecode])
            .build();

        assert_eq!(registry.len(), 2);
    }
}
