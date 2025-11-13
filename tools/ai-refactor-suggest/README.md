# AI Refactoring Suggestion Tool

An AI-powered code analysis tool that uses Claude to review blockchain decoder implementations and suggest improvements based on latest protocol releases, best practices, and project-specific design principles.

## Overview

This tool analyzes decoder crates and generates actionable refactoring suggestions across five categories:

1. **Dependency**: Dependency management, versioning, and vendoring opportunities
2. **Security**: Unsafe code, input validation, overflow protection, canonical encoding
3. **Performance**: Allocation patterns, zero-cost abstractions, optimization opportunities
4. **Testing**: Test coverage, property-based tests, integration tests
5. **Architecture**: Trait implementation, separation of concerns, code organization

## Features

- **Chain Family-Specific Analysis**: Tailored prompts for UTXO, Account (EVM), Instruction-based, and Other chains
- **Automated Discovery**: Automatically discovers all decoder crates in the repository
- **Priority Classification**: Suggestions categorized as High, Medium, or Low priority
- **GitHub Integration**: Generates issue templates for high-priority suggestions
- **CI/CD Ready**: Runs weekly in GitHub Actions with Claude API

## Installation

The tool is built as part of the workspace:

```bash
cargo build --release -p ai-refactor-suggest
```

## Configuration

Configuration is stored in `scripts/refactor-config.json`:

```json
{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 4096,
  "temperature": 0.3,
  "enabled_categories": [
    "dependency",
    "security",
    "performance",
    "testing",
    "architecture"
  ],
  "min_priority": "low",
  "excluded_decoders": []
}
```

### Configuration Options

- `model`: Claude model to use (default: claude-sonnet-4-5-20250929)
- `max_tokens`: Maximum tokens for Claude response (default: 4096)
- `temperature`: Sampling temperature (default: 0.3 for more focused suggestions)
- `enabled_categories`: Which suggestion categories to include
- `min_priority`: Minimum priority level to include (low, medium, high)
- `excluded_decoders`: List of decoder names to skip

## Usage

### Prerequisites

Set your Anthropic API key:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

### Basic Usage

Analyze all decoders:

```bash
cargo run -p ai-refactor-suggest
```

### Advanced Options

```bash
# Analyze specific decoder
cargo run -p ai-refactor-suggest -- --decoder bitcoin

# Analyze specific chain family
cargo run -p ai-refactor-suggest -- --family utxo

# Custom output paths
cargo run -p ai-refactor-suggest -- \
  --output reports/refactor-suggestions.md \
  --issues-dir reports/issues

# Don't generate GitHub issues
cargo run -p ai-refactor-suggest -- --no-issues

# Use custom config
cargo run -p ai-refactor-suggest -- --config my-config.json

# Verbose output
cargo run -p ai-refactor-suggest -- --verbose
```

### Command-Line Options

- `--decoder <NAME>`: Analyze specific decoder only (e.g., 'bitcoin', 'ethereum')
- `--family <FAMILY>`: Analyze specific chain family (utxo, account, instruction, other)
- `--output <PATH>`: Output report path (default: refactor-suggestions.md)
- `--issues-dir <DIR>`: Directory for GitHub issue templates (default: github-issues)
- `--no-issues`: Don't generate GitHub issue templates
- `--config <PATH>`: Configuration file path (default: scripts/refactor-config.json)
- `--repo-root <PATH>`: Repository root directory (default: .)
- `--api-key <KEY>`: Anthropic API key (or set ANTHROPIC_API_KEY env var)
- `--verbose`: Enable verbose output

## Chain Family-Specific Prompts

The tool uses specialized prompts for each chain family:

### UTXO Family (Bitcoin, Litecoin, Dogecoin, Cardano)
- `scripts/refactor-prompts/prompt-utxo.md`
- Focuses on: Transaction structure, script parsing, SegWit/Taproot support

### Account Family (Ethereum, EVM chains)
- `scripts/refactor-prompts/prompt-account.md`
- Focuses on: RLP encoding, transaction types (EIP-2718), EVM compatibility

### Instruction Family (Solana, Aptos, Sui)
- `scripts/refactor-prompts/prompt-instruction.md`
- Focuses on: BCS encoding, instruction parsing, Move VM considerations

### Other Chains (XRP, Tron, Polkadot, NEAR, etc.)
- `scripts/refactor-prompts/prompt-other.md`
- Focuses on: Chain-specific encoding formats, unique transaction models

### Base Template
- `scripts/refactor-prompts/prompt-base.md`
- Fallback template with general analysis guidelines

## Output

The tool generates two types of output:

### 1. Markdown Report (`refactor-suggestions.md`)

A comprehensive report containing:
- Summary statistics (total suggestions, priority breakdown)
- Detailed suggestions by decoder
- Code locations and suggested changes

Example structure:
```markdown
# AI Refactoring Suggestions

**Generated**: 2025-11-13 10:30:00
**Total Suggestions**: 15
**Decoders Analyzed**: 5

## Summary by Priority
- **High Priority**: 3
- **Medium Priority**: 8
- **Low Priority**: 4

## Detailed Suggestions

### bitcoin

#### High Priority

**Move bitcoin crate to dev-dependencies** (dependency)

The bitcoin crate is currently in production dependencies...

*Location*: `crates/decoder-bitcoin/Cargo.toml:10`
...
```

### 2. GitHub Issue Templates (`github-issues/`)

For each high-priority suggestion, generates a ready-to-use issue template:

```markdown
---
title: "[bitcoin] Move bitcoin crate to dev-dependencies"
labels: refactoring, dependency, ai-suggested
---

## Description
...

## Checklist
- [ ] Review suggestion validity
- [ ] Implement changes
- [ ] Add tests
- [ ] Run cargo fmt --all
- [ ] Run cargo clippy --all --all-targets --all-features -- -D warnings
```

## CI/CD Integration

The tool runs automatically every Monday at 9:00 AM UTC via GitHub Actions (`.github/workflows/ai-refactor-suggest.yml`).

### Workflow Features

- **Scheduled Runs**: Weekly automated analysis
- **Manual Trigger**: Can be triggered manually with optional filters
- **Artifact Upload**: Reports saved for 90 days
- **Auto-Issue Creation**: Creates tracking issue for high-priority suggestions
- **Optional PR**: Can create PR with report for easy review

### Triggering Manually

Go to Actions → AI Refactoring Suggestions → Run workflow

Options:
- Specific decoder to analyze
- Chain family filter (utxo, account, instruction, other)

### Required Secrets

Add to repository secrets:
- `ANTHROPIC_API_KEY`: Your Anthropic Claude API key

## Analysis Methodology

### 1. Discovery Phase
- Scans `crates/decoder-*` directories
- Parses `Cargo.toml` for dependencies
- Counts lines of code
- Checks for test existence

### 2. Context Building
- Reads up to 10 source files per decoder (to stay within token limits)
- Extracts dependency information
- Identifies blockchain-specific dependencies in production
- Determines chain family and loads appropriate prompt template

### 3. Claude Analysis
- Sends contextual prompt to Claude API
- Includes:
  - Decoder metadata (LOC, dependencies, tests)
  - Source code samples
  - Latest protocol/ecosystem updates
  - Chain family-specific analysis criteria
  - Project design principles

### 4. Response Parsing
- Extracts JSON array from Claude's response
- Validates suggestion structure
- Filters by enabled categories and priority threshold

### 5. Report Generation
- Groups suggestions by decoder and priority
- Generates markdown report
- Creates GitHub issue templates for high-priority items

## Design Principles Checked

The tool evaluates decoders against project design principles:

1. **Minimal TCB**: Core library < 3000 LOC
2. **Formally Verifiable**: No unsafe code, explicit contracts
3. **Canonical Serialization**: Borsh for TxIR, not JSON for hashing
4. **Trait-Based Extensibility**: Open-closed principle
5. **Zero-Cost Abstractions**: Static dispatch, compile-time optimization
6. **Pure Rust Decoders**: Blockchain libs in dev-dependencies only
7. **Supply Chain Security**: Minimal dependencies, vendoring capability

## Customization

### Adding New Prompt Templates

Create a new markdown file in `scripts/refactor-prompts/`:

```bash
# For a new chain family
cat > scripts/refactor-prompts/prompt-mychainfamily.md <<EOF
# My Chain Family Analysis

You are analyzing a ... blockchain decoder.

## Focus Areas
...

## Analysis Instructions
...
EOF
```

Then update `CHAIN_FAMILIES` in `src/decoder_info.rs` if needed.

### Modifying Analysis Criteria

Edit the appropriate prompt template in `scripts/refactor-prompts/`:
- `prompt-utxo.md` - UTXO chains
- `prompt-account.md` - EVM/Account chains
- `prompt-instruction.md` - Solana/Aptos/Sui
- `prompt-other.md` - Other chains
- `prompt-base.md` - Fallback template

### Adjusting Token Usage

If hitting token limits, adjust in `src/decoder_info.rs`:

```rust
// Reduce number of files read
let source_files = decoder.read_source_files(5)?; // default: 10
```

Or in `src/prompts.rs`:

```rust
// Reduce truncation threshold
let truncated = if content.len() > 2000 { // default: 4000
    ...
}
```

## Troubleshooting

### API Key Not Found

```
Error: ANTHROPIC_API_KEY must be set
```

Solution: Set environment variable or use `--api-key` flag

### JSON Parsing Errors

```
Error: Failed to parse JSON response
```

Possible causes:
- Claude returned explanation text along with JSON
- Response format changed
- Token limit exceeded (partial response)

Solution: Check response text in error output, adjust `max_tokens` in config

### Decoder Not Found

```
Error: Decoder 'xyz' not found
```

Solution: Check decoder name (without `decoder-` prefix), ensure Cargo.toml exists

### No High-Priority Suggestions

This is normal! It means the code is in good shape for that week.

## Development

### Project Structure

```
tools/ai-refactor-suggest/
├── Cargo.toml
├── README.md (this file)
└── src/
    ├── main.rs          # CLI entry point
    ├── analyzer.rs      # Claude API integration
    ├── decoder_info.rs  # Decoder discovery
    ├── prompts.rs       # Prompt management
    └── suggestions.rs   # Report generation
```

### Adding New Features

1. Update relevant module in `src/`
2. Add tests (if applicable)
3. Update this README
4. Run formatting and linting:
   ```bash
   cargo fmt --all
   cargo clippy --all --all-targets --all-features -- -D warnings
   ```

### Testing Locally

```bash
# Build
cargo build -p ai-refactor-suggest

# Run with verbose output
ANTHROPIC_API_KEY=your-key-here \
  cargo run -p ai-refactor-suggest -- --verbose --decoder bitcoin

# Check generated reports
cat refactor-suggestions.md
ls github-issues/
```

## Cost Considerations

Claude API usage costs depend on:
- Model: Sonnet 4.5 is balanced for quality/cost
- Input tokens: ~5-10k per decoder (depends on LOC)
- Output tokens: ~1-2k per decoder (suggestions)

Estimated cost per run:
- ~20 decoders * 7k average tokens = ~140k tokens input
- ~20 decoders * 1.5k tokens = ~30k tokens output
- Cost: ~$1-2 per full run (check current Anthropic pricing)

Weekly cost: ~$4-8/month

To reduce costs:
- Analyze specific decoders/families only
- Reduce `max_tokens` in config
- Increase analysis interval (biweekly instead of weekly)

## Future Enhancements

Potential improvements:

- [ ] Cache analysis results to avoid re-analyzing unchanged decoders
- [ ] Integrate with cargo-tarpaulin for actual test coverage metrics
- [ ] Add diff-based analysis (only analyze changed files)
- [ ] Support multiple AI models (GPT-4, Gemini) for comparison
- [ ] Generate PR with fixes for low-hanging fruit
- [ ] Integration with GitHub Copilot for inline suggestions
- [ ] Historical trending (track improvement over time)

## License

Same as parent project: MIT OR Apache-2.0

## References

- [Claude API Documentation](https://docs.anthropic.com/claude/reference/getting-started-with-the-api)
- [Project Design Principles](../../CLAUDE.md)
- [Testing Strategy](../../docs/TESTING_STRATEGY.md)
- [Roadmap](../../ROADMAP.md)
