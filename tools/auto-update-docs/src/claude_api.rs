use anyhow::{Context, Result};
use api_rate_limiter::{RateLimitConfig, RateLimiter};
use llm_client::{GenerationParams, LlmClient};
use log::{debug, info};

use crate::analyzer::CodebaseAnalysis;
use crate::doc_updater::DocUpdate;

/// Create a default rate limiter for LLM API calls
pub fn create_rate_limiter() -> RateLimiter {
    // Conservative defaults: 10 RPM, $10/month
    RateLimiter::new(RateLimitConfig::conservative())
        .expect("conservative preset is a valid configuration")
}

/// Generate documentation update using an LLM
pub fn generate_doc_update(
    client: &LlmClient,
    model: &str,
    params: &GenerationParams,
    update: &DocUpdate,
    analysis: &CodebaseAnalysis,
) -> Result<String> {
    info!("Generating documentation update for {:?}", update.doc_path);

    let prompt = build_doc_update_prompt(update, analysis);

    debug!("Prompt length: {} chars", prompt.len());

    call_llm(client, model, params, &prompt)
}

/// Generate architecture diagram using an LLM
pub fn generate_architecture_diagram(
    client: &LlmClient,
    model: &str,
    params: &GenerationParams,
    analysis: &CodebaseAnalysis,
    diagram_type: &str,
) -> Result<String> {
    info!("Generating {} architecture diagram", diagram_type);

    let prompt = build_diagram_prompt(analysis, diagram_type);

    let response = call_llm(client, model, params, &prompt)?;

    // Extract Mermaid code from response
    extract_mermaid_diagram(&response)
}

/// Call the LLM with a prompt through the rate-limited client
fn call_llm(
    client: &LlmClient,
    model: &str,
    params: &GenerationParams,
    prompt: &str,
) -> Result<String> {
    debug!("Sending request (model: {})", model);

    let completion = client
        .complete(model, None, prompt, params)
        .context("LLM request failed")?;

    info!(
        "Actual usage: {} input + {} output tokens",
        completion.usage.input_tokens, completion.usage.output_tokens
    );
    if let Some(stats) = client.usage_stats() {
        info!(
            "Budget: {} daily tokens used, ${:.4} monthly cost",
            stats.daily_tokens,
            stats.monthly_cost_dollars()
        );
    }

    Ok(completion.text)
}

/// Build prompt for documentation update
pub fn build_doc_update_prompt(update: &DocUpdate, analysis: &CodebaseAnalysis) -> String {
    format!(
        r#"You are a technical documentation expert helping maintain the Universal Blockchain Decoder project.

# Project Context

The Universal Blockchain Decoder is a formally verified, minimal trusted computing base (TCB) library for decoding blockchain transactions from multiple chains into a unified intermediate representation (TxIR).

## Core Principles
1. Minimal TCB: Core library < 3000 LOC
2. Formally verifiable with Verus
3. Trait-based extensibility (no enum-based chains)
4. Canonical serialization with Borsh
5. Pure Rust decoders (blockchain libs in dev-dependencies only)
6. Supply chain security with vendored dependencies

# Current Codebase State

{}

Total modules: {}
Total LOC: {}

Recent changes:
{}

# Documentation to Update

File: {:?}
Reason: {}

Current content:
{}

# Task

Update the documentation file to reflect the current state of the codebase. Ensure:

1. **Accuracy**: All information matches the current codebase structure
2. **Completeness**: Cover all relevant modules and their relationships
3. **Clarity**: Use clear, concise language
4. **Consistency**: Follow existing documentation style and format
5. **Up-to-date**: Include recent changes and new features
6. **Diagrams**: If the document includes Mermaid diagrams, update them to reflect current architecture

**IMPORTANT**:
- Preserve the existing document structure and headings
- Keep the same tone and style
- Update facts, figures, and code examples
- Add new sections if needed for new features
- Mark deprecated features clearly

Return ONLY the updated documentation content in Markdown format. Do not include explanations or meta-commentary."#,
        analysis.architecture_summary,
        analysis.module_count,
        analysis.total_loc,
        format_recent_changes(&analysis.changes),
        update.doc_path,
        update.reason,
        update.current_content,
    )
}

/// Build prompt for architecture diagram generation
pub fn build_diagram_prompt(analysis: &CodebaseAnalysis, diagram_type: &str) -> String {
    let diagram_instructions = match diagram_type {
        "overview" => {
            "Create a high-level architecture overview showing:
- Core library (universal-decoder-core)
- Decoder modules (decoder-bitcoin, decoder-ethereum, etc.)
- Tool modules
- Key relationships and data flow
Use a flowchart or graph diagram."
        }
        "dependency" => {
            "Create a dependency graph showing:
- All modules (crates)
- Dependencies between modules
- External dependencies (grouped)
Use a graph diagram with clear hierarchy."
        }
        "data-flow" => {
            "Create a data flow diagram showing:
- How raw transaction bytes flow through the system
- Decoder processing
- TxIR generation
- Canonical serialization
Use a flowchart with clear steps."
        }
        "layer" => {
            "Create a layered architecture diagram showing:
- Layer 1: Core types and traits
- Layer 2: Decoder implementations
- Layer 3: Tools and utilities
- Layer 4: Applications
Use a layered block diagram."
        }
        _ => "Create an appropriate architecture diagram for the codebase.",
    };

    format!(
        r#"You are a technical architect creating Mermaid diagrams for the Universal Blockchain Decoder project.

# Project Context

The Universal Blockchain Decoder is a formally verified, minimal trusted computing base (TCB) library for decoding blockchain transactions.

# Current Codebase State

{}

Modules:
{}

Dependencies:
{}

# Task

{}

**Requirements**:
1. Use valid Mermaid syntax
2. Keep the diagram clear and readable
3. Use appropriate diagram type (flowchart, graph, etc.)
4. Include all major modules
5. Show key relationships
6. Use consistent naming
7. Add helpful labels and annotations

Return ONLY the Mermaid diagram code wrapped in ```mermaid``` code blocks. Do not include explanations."#,
        analysis.architecture_summary,
        format_modules(&analysis.modules),
        format_dependencies(&analysis.dependencies),
        diagram_instructions,
    )
}

/// Format recent changes for prompt
fn format_recent_changes(changes: &[crate::analyzer::CodeChange]) -> String {
    if changes.is_empty() {
        return "No recent changes detected.".to_string();
    }

    let mut output = String::new();
    for (i, change) in changes.iter().take(20).enumerate() {
        output.push_str(&format!(
            "{}. {:?}: {} ({})\n",
            i + 1,
            change.change_type,
            change.file_path.display(),
            change.summary
        ));
    }

    if changes.len() > 20 {
        output.push_str(&format!("... and {} more changes\n", changes.len() - 20));
    }

    output
}

/// Format modules for prompt
fn format_modules(modules: &[crate::analyzer::ModuleInfo]) -> String {
    let mut output = String::new();
    for module in modules {
        output.push_str(&format!(
            "- {} ({:?}, {} LOC): {}\n",
            module.name, module.module_type, module.loc, module.description
        ));
    }
    output
}

/// Format dependencies for prompt
fn format_dependencies(deps: &std::collections::HashMap<String, Vec<String>>) -> String {
    let mut output = String::new();
    for (module, module_deps) in deps {
        if !module_deps.is_empty() {
            output.push_str(&format!("- {}: {}\n", module, module_deps.join(", ")));
        }
    }
    output
}

/// Extract Mermaid diagram from Claude response
fn extract_mermaid_diagram(response: &str) -> Result<String> {
    // Look for ```mermaid ... ``` blocks
    let start_marker = "```mermaid";
    let end_marker = "```";

    if let Some(start) = response.find(start_marker) {
        let content_start = start + start_marker.len();
        if let Some(end) = response[content_start..].find(end_marker) {
            let diagram = response[content_start..content_start + end].trim();
            return Ok(diagram.to_string());
        }
    }

    // If no code block, return entire response
    Ok(response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mermaid_diagram() {
        let response = r#"Here's the diagram:

```mermaid
graph TD
    A --> B
    B --> C
```

This shows the flow."#;

        let diagram = extract_mermaid_diagram(response).unwrap();
        assert!(diagram.contains("graph TD"));
        assert!(diagram.contains("A --> B"));
    }

    #[test]
    fn test_extract_mermaid_diagram_no_code_block() {
        let response = "graph TD\n    A --> B";
        let diagram = extract_mermaid_diagram(response).unwrap();
        assert_eq!(diagram, "graph TD\n    A --> B");
    }
}
