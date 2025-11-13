use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::analyzer::CodebaseAnalysis;
use crate::doc_updater::DocUpdate;

/// AWS Bedrock API client for Claude models
///
/// Bedrock often has lower costs than direct Anthropic API, especially with
/// committed throughput or provisioned capacity.
///
/// Cost comparison (as of 2025):
/// - Anthropic API: ~$3 per 1M input tokens, ~$15 per 1M output tokens
/// - Bedrock on-demand: ~$3 per 1M input tokens, ~$15 per 1M output tokens
/// - Bedrock Provisioned: Up to 50% cheaper with committed throughput
///
/// Additional benefits:
/// - No separate API key (uses AWS credentials)
/// - Integration with AWS services
/// - VPC endpoints for private access
/// - CloudWatch metrics
/// - AWS support

#[derive(Debug, Serialize)]
struct BedrockRequest {
    anthropic_version: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct BedrockResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(rename = "type")]
    _content_type: String,
    text: String,
}

/// Generate documentation update using AWS Bedrock
pub fn generate_doc_update(
    region: &str,
    model_id: &str,
    max_tokens: u32,
    temperature: f32,
    update: &DocUpdate,
    analysis: &CodebaseAnalysis,
) -> Result<String> {
    info!(
        "Generating documentation update using Bedrock for {:?}",
        update.doc_path
    );

    let prompt = crate::claude_api::build_doc_update_prompt(update, analysis);

    debug!("Prompt length: {} chars", prompt.len());

    let response = call_bedrock_api(region, model_id, max_tokens, temperature, &prompt)?;

    Ok(response)
}

/// Generate architecture diagram using AWS Bedrock
pub fn generate_architecture_diagram(
    region: &str,
    model_id: &str,
    max_tokens: u32,
    temperature: f32,
    analysis: &CodebaseAnalysis,
    diagram_type: &str,
) -> Result<String> {
    info!(
        "Generating {} architecture diagram using Bedrock",
        diagram_type
    );

    let prompt = crate::claude_api::build_diagram_prompt(analysis, diagram_type);

    let response = call_bedrock_api(region, model_id, max_tokens, temperature, &prompt)?;

    // Extract Mermaid code from response
    extract_mermaid_diagram(&response)
}

/// Call AWS Bedrock API with a prompt
///
/// Uses the AWS SDK for Rust to invoke Bedrock runtime.
/// Requires AWS credentials to be configured (IAM role, environment variables, or AWS CLI).
fn call_bedrock_api(
    region: &str,
    model_id: &str,
    max_tokens: u32,
    temperature: f32,
    prompt: &str,
) -> Result<String> {
    // Build the request body
    let request = BedrockRequest {
        anthropic_version: "bedrock-2023-05-31".to_string(),
        max_tokens,
        temperature,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let request_body =
        serde_json::to_string(&request).context("Failed to serialize Bedrock request")?;

    debug!(
        "Sending request to Bedrock (region: {}, model: {})",
        region, model_id
    );

    // Use AWS CLI to invoke Bedrock (simpler than full SDK for now)
    // In production, you'd use the aws-sdk-bedrockruntime crate
    let output = std::process::Command::new("aws")
        .args([
            "bedrock-runtime",
            "invoke-model",
            "--region",
            region,
            "--model-id",
            model_id,
            "--body",
            &request_body,
            "--output",
            "json",
            "/dev/stdout",
        ])
        .output()
        .context("Failed to invoke AWS Bedrock. Is AWS CLI installed and configured?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bedrock API error: {}", stderr);
    }

    let response_text = String::from_utf8_lossy(&output.stdout);

    // Parse the response
    let bedrock_response: BedrockResponse =
        serde_json::from_str(&response_text).context("Failed to parse Bedrock response")?;

    // Extract text from first content block
    let text = bedrock_response
        .content
        .first()
        .map(|c| c.text.clone())
        .unwrap_or_default();

    debug!("Received response: {} chars", text.len());

    Ok(text)
}

/// Extract Mermaid diagram from response
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

/// Check if AWS Bedrock is available and configured
pub fn check_bedrock_available() -> Result<()> {
    // Check if AWS CLI is installed
    let output = std::process::Command::new("aws")
        .args(["--version"])
        .output();

    if output.is_err() {
        anyhow::bail!("AWS CLI not found. Install it from: https://aws.amazon.com/cli/");
    }

    // Check if credentials are configured
    let output = std::process::Command::new("aws")
        .args(["sts", "get-caller-identity"])
        .output()
        .context("Failed to check AWS credentials")?;

    if !output.status.success() {
        anyhow::bail!(
            "AWS credentials not configured. Run 'aws configure' or set environment variables."
        );
    }

    Ok(())
}

/// Get available Claude models on Bedrock
#[allow(dead_code)]
pub fn list_available_models(_region: &str) -> Vec<String> {
    vec![
        format!("anthropic.claude-3-5-sonnet-20241022-v2:0"), // Claude 3.5 Sonnet v2
        format!("anthropic.claude-3-5-sonnet-20240620-v1:0"), // Claude 3.5 Sonnet v1
        format!("anthropic.claude-3-opus-20240229-v1:0"),     // Claude 3 Opus
        format!("anthropic.claude-3-sonnet-20240229-v1:0"),   // Claude 3 Sonnet
        format!("anthropic.claude-3-haiku-20240307-v1:0"),    // Claude 3 Haiku
    ]
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
    fn test_list_available_models() {
        let models = list_available_models("us-east-1");
        assert!(!models.is_empty());
        assert!(models[0].contains("anthropic.claude"));
    }
}
