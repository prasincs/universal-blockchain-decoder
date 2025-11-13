# Auto-Update Documentation Tool

An AI-powered tool that automatically keeps documentation and architecture diagrams up-to-date based on code changes, running nightly in CI.

## Overview

This tool uses Claude AI to:
- 📊 Generate and update Mermaid architecture diagrams
- 📝 Update documentation to reflect code changes
- 🔍 Analyze codebase structure and recent commits
- 🤖 Automatically create pull requests with documentation updates
- ⏰ Run nightly to keep docs continuously fresh

## Features

### Intelligent Documentation Analysis

The tool automatically detects when documentation needs updating by checking:

1. **Module Count Changes**: Detects when docs mention outdated module/crate counts
2. **Architecture Changes**: Identifies when code structure changes require doc updates
3. **Recent Code Changes**: Tracks git history to find affected documentation
4. **Outdated Diagrams**: Finds Mermaid diagrams that need regeneration
5. **Status Documents**: Prioritizes roadmap, status, and planning docs for updates

### AI-Powered Generation

Uses Claude Sonnet 4.5 to:
- Understand codebase structure and relationships
- Generate accurate, up-to-date documentation
- Create clear Mermaid diagrams (flowcharts, graphs, layer diagrams)
- Maintain consistent tone and style with existing docs
- Preserve important context and technical details

### Automated Workflow

Runs automatically:
- **Nightly**: Every night at 2:00 AM UTC
- **On Code Changes**: When Rust code or Cargo.toml files change
- **Manual Trigger**: Via GitHub Actions UI

## Installation

The tool is built as part of the workspace:

```bash
cargo build --release -p auto-update-docs
```

## Usage

### Prerequisites

Set your Anthropic API key:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

### Basic Usage

Update all documentation:

```bash
cargo run -p auto-update-docs
```

### Advanced Options

```bash
# Dry run (don't write files)
cargo run -p auto-update-docs -- --dry-run

# Only generate diagrams
cargo run -p auto-update-docs -- --update-docs=false

# Only update docs (no diagrams)
cargo run -p auto-update-docs -- --generate-diagrams=false

# Analyze changes since specific git ref
cargo run -p auto-update-docs -- --since main

# Commit and push changes
cargo run -p auto-update-docs -- --commit --push

# Create PR with changes
cargo run -p auto-update-docs -- --commit --push --create-pr

# Use different Claude model
cargo run -p auto-update-docs -- --model claude-opus-4-20250514

# Verbose output
cargo run -p auto-update-docs -- --verbose

# Custom docs directory
cargo run -p auto-update-docs -- --docs-dir documentation
```

### Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--repo-root <PATH>` | Repository root directory | `.` |
| `--api-key <KEY>` | Anthropic API key | `$ANTHROPIC_API_KEY` |
| `--docs-dir <PATH>` | Output directory for docs | `docs` |
| `--generate-diagrams` | Generate architecture diagrams | `true` |
| `--update-docs` | Update existing documentation | `true` |
| `--since <REF>` | Analyze changes since git ref | Last 7 days |
| `--commit` | Create git commit with changes | `false` |
| `--push` | Push changes to remote | `false` |
| `--create-pr` | Create pull request | `false` |
| `--model <MODEL>` | Claude model to use | `claude-sonnet-4-5-20250929` |
| `--max-tokens <N>` | Max tokens for Claude | `8000` |
| `--temperature <F>` | Temperature for Claude | `0.3` |
| `--verbose`, `-v` | Verbose output | `false` |
| `--dry-run` | Don't write files | `false` |

## How It Works

### 1. Codebase Analysis

The tool analyzes:
- **Module Discovery**: Finds all Rust crates via Cargo.toml files
- **Dependency Analysis**: Extracts dependencies and relationships
- **Code Changes**: Uses git history to identify recent changes
- **LOC Counting**: Measures lines of code per module
- **Module Classification**: Categorizes as Core, Decoder, Tool, etc.

### 2. Documentation Analysis

Determines what needs updating:
- Scans key documentation files (README, CLAUDE.md, docs/*.md)
- Checks for outdated module counts
- Identifies documents mentioning changed modules
- Finds diagrams needing regeneration
- Prioritizes updates (High/Medium/Low)

### 3. AI Generation

For each documentation update:
- Builds context-rich prompt with codebase state
- Calls Claude API with current doc content
- Claude generates updated documentation
- Preserves existing structure and style
- Maintains technical accuracy

For architecture diagrams:
- Generates multiple diagram types (overview, dependency, data-flow, layers)
- Uses Mermaid syntax for GitHub compatibility
- Creates standalone markdown files
- Includes auto-generation metadata

### 4. Git Operations

If requested:
- Creates new branch for updates
- Commits documentation changes
- Pushes to remote
- Creates pull request with detailed description

## Generated Diagrams

The tool generates four types of architecture diagrams:

### 1. Architecture Overview (`docs/diagrams/architecture-overview.md`)

High-level view showing:
- Core library
- Decoder modules
- Tool modules
- Key relationships

### 2. Dependency Graph (`docs/diagrams/dependency-graph.md`)

Shows dependencies between:
- Internal modules
- External crates
- Hierarchical structure

### 3. Data Flow (`docs/diagrams/data-flow.md`)

Illustrates:
- Raw transaction input
- Decoder processing
- TxIR generation
- Canonical serialization

### 4. Layered Architecture (`docs/diagrams/layered-architecture.md`)

Displays:
- Layer 1: Core types and traits
- Layer 2: Decoder implementations
- Layer 3: Tools and utilities
- Layer 4: Applications

All diagrams are:
- ✅ Valid Mermaid syntax
- ✅ Renderable in GitHub markdown
- ✅ Auto-generated with metadata
- ✅ Version controlled

## CI/CD Integration

### GitHub Actions Workflow

The tool runs automatically via `.github/workflows/auto-update-docs.yml`:

**Schedule**: Nightly at 2:00 AM UTC

**Triggers**:
- Scheduled (cron)
- Push to main (code changes)
- Manual workflow dispatch

**Process**:
1. Checkout repository with full history
2. Build auto-update-docs tool
3. Run documentation analysis and generation
4. Commit changes to new branch
5. Create pull request if changes detected
6. Post summary to workflow run

**Permissions**:
- `contents: write` - For committing changes
- `pull-requests: write` - For creating PRs

### Required Secrets

Add to repository secrets:
- `ANTHROPIC_API_KEY`: Your Anthropic Claude API key

### Manual Triggering

1. Go to **Actions** → **Auto-Update Documentation**
2. Click **Run workflow**
3. Options:
   - **Dry run**: Preview changes without committing
   - **Create PR**: Automatically create pull request

### Workflow Outputs

Each run produces:
- Summary in workflow UI
- List of changed files
- Pull request (if changes detected)
- Issue on failure (for monitoring)

## Configuration

### Claude API Settings

Adjust in CLI or code:

```rust
// Model selection
--model claude-sonnet-4-5-20250929  // Balanced quality/cost
--model claude-opus-4-20250514      // Highest quality
--model claude-haiku-4-20250313     // Fastest/cheapest

// Token limits
--max-tokens 8000   // Default: sufficient for docs
--max-tokens 16000  // For very large documents

// Temperature
--temperature 0.3   // Default: focused/deterministic
--temperature 0.0   // Maximum determinism
--temperature 0.5   // More creative
```

### Documentation Files Checked

The tool automatically checks:

**Root Level**:
- `README.md`
- `ARCHITECTURE.md`
- `ROADMAP.md`
- `CLAUDE.md`

**docs/ Directory**:
- `docs/ARCHITECTURE_REFACTORING.md`
- `docs/TRAIT_BASED_ARCHITECTURE.md`
- `docs/TESTING_STRATEGY.md`
- All other `docs/*.md` files

### Update Priority Logic

**High Priority**:
- Outdated module counts in documentation
- Architecture docs with recent code changes
- Roadmap/status/plan documents

**Medium Priority**:
- Docs mentioning changed modules
- Non-auto-generated Mermaid diagrams
- Documents referencing old phases/versions

**Low Priority**:
- Other markdown files
- General documentation refreshes

## Cost Considerations

Claude API usage costs depend on:
- **Model**: Sonnet 4.5 is balanced for quality/cost
- **Input tokens**: ~10-15k per doc update (depends on doc size + codebase context)
- **Output tokens**: ~2-5k per doc update

**Estimated cost per nightly run**:
- Average: 5-10 documentation updates
- ~50-100k input tokens
- ~10-25k output tokens
- **Cost**: ~$2-5 per run

**Monthly cost**: ~$60-150 for nightly runs

**To reduce costs**:
1. Only run on significant code changes (not nightly)
2. Reduce `--max-tokens` parameter
3. Use Haiku model for simpler updates
4. Limit documentation files checked
5. Only update high-priority docs

## Examples

### Example 1: Manual Update After Refactoring

```bash
# After major refactoring
cargo run -p auto-update-docs -- \
  --since main \
  --commit \
  --verbose

# Review changes
git diff

# Push if satisfied
git push
```

### Example 2: Generate Only Diagrams

```bash
cargo run -p auto-update-docs -- \
  --update-docs=false \
  --generate-diagrams \
  --commit

# Diagrams created in docs/diagrams/
ls docs/diagrams/
```

### Example 3: Dry Run for Testing

```bash
# Test without writing files
cargo run -p auto-update-docs -- \
  --dry-run \
  --verbose

# See what would be updated
# No files modified
```

### Example 4: Create PR Workflow

```bash
# 1. Update docs
cargo run -p auto-update-docs

# 2. Commit changes
git add docs/ *.md
git commit -m "docs: Update architecture documentation"

# 3. Push to branch
git push -u origin update-docs

# 4. Create PR via CLI
gh pr create \
  --title "docs: Update architecture documentation" \
  --body "Auto-generated documentation updates" \
  --label documentation
```

## Development

### Project Structure

```
tools/auto-update-docs/
├── Cargo.toml
├── README.md (this file)
└── src/
    ├── main.rs              # CLI entry point
    ├── analyzer.rs          # Codebase analysis
    ├── claude_api.rs        # Claude API integration
    ├── diagram_generator.rs # Mermaid diagram generation
    ├── doc_updater.rs       # Documentation update logic
    └── git_utils.rs         # Git operations
```

### Adding New Features

1. **New Diagram Types**: Add to `diagram_generator.rs::generate_diagrams()`
2. **Custom Analysis**: Extend `analyzer.rs::analyze_codebase()`
3. **Update Detection**: Modify `doc_updater.rs::check_doc_needs_update()`
4. **Prompts**: Edit prompt templates in `claude_api.rs`

### Testing Locally

```bash
# Build
cargo build -p auto-update-docs

# Run with verbose output
ANTHROPIC_API_KEY=sk-ant-... \
  cargo run -p auto-update-docs -- --verbose --dry-run

# Check generated output
cat docs/diagrams/architecture-overview.md

# Run tests
cargo test -p auto-update-docs
```

### Code Quality

Before committing:

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all --all-targets --all-features -- -D warnings

# Test
cargo test --all
```

## Troubleshooting

### API Key Not Found

```
Error: ANTHROPIC_API_KEY must be set
```

**Solution**: Set environment variable or use `--api-key` flag

```bash
export ANTHROPIC_API_KEY=sk-ant-...
# or
cargo run -p auto-update-docs -- --api-key sk-ant-...
```

### Claude API Rate Limits

```
Error: Claude API error (429): Rate limit exceeded
```

**Solution**: Add delays between requests or reduce update frequency

### Git Push Failed (403)

```
Error: Git push failed: 403 Forbidden
```

**Solution**: Ensure branch name follows pattern: `claude/auto-update-docs-*` with matching session ID

### No Changes Detected

This is normal if:
- Documentation is already up-to-date
- No code changes since last run
- Changes don't affect tracked documentation

### Large Documentation Files

If hitting token limits:

```bash
# Reduce max tokens
cargo run -p auto-update-docs -- --max-tokens 4000

# Or split into multiple runs
cargo run -p auto-update-docs -- --docs-dir docs/section1
cargo run -p auto-update-docs -- --docs-dir docs/section2
```

## Comparison with Other Tools

### vs. Manual Documentation

| Aspect | Manual | auto-update-docs |
|--------|--------|------------------|
| Accuracy | Depends on author | AI-verified against code |
| Freshness | Often outdated | Always current |
| Effort | High | Automated |
| Consistency | Variable | Consistent |
| Diagrams | Manual, stale | Auto-generated |

### vs. ai-refactor-suggest

| Tool | Purpose | Output | Frequency |
|------|---------|--------|-----------|
| `ai-refactor-suggest` | Code improvement suggestions | Issues/reports | Weekly |
| `auto-update-docs` | Documentation updates | Markdown/diagrams | Nightly |

Both tools complement each other:
- `ai-refactor-suggest`: Suggests code improvements
- `auto-update-docs`: Keeps docs in sync with code

## Best Practices

### 1. Review Before Merging

Always review auto-generated PRs:
- ✅ Verify technical accuracy
- ✅ Check diagram correctness
- ✅ Ensure no sensitive info leaked
- ✅ Validate Mermaid syntax

### 2. Customize Prompts

For your specific needs:
- Edit `claude_api.rs::build_doc_update_prompt()`
- Add project-specific context
- Include coding standards
- Reference style guides

### 3. Incremental Updates

Don't update everything at once:
- Start with critical docs (README, CLAUDE.md)
- Add more files gradually
- Monitor API costs
- Adjust frequency as needed

### 4. Version Control Diagrams

Commit generated diagrams:
- Enables easy rollback
- Shows diagram evolution
- Facilitates review
- Supports offline viewing

### 5. Monitor Costs

Track API usage:
- Check monthly bills
- Adjust run frequency
- Optimize prompts
- Use cheaper models for simple updates

## Future Enhancements

Potential improvements:

- [ ] Cache analysis results to avoid re-analyzing unchanged files
- [ ] Incremental updates (only changed docs)
- [ ] Support for other diagram types (PlantUML, Graphviz)
- [ ] Multi-language documentation support
- [ ] Integration with documentation generators (mdBook, Docusaurus)
- [ ] Diff-based updates (show what changed)
- [ ] Custom prompt templates per document type
- [ ] Support for API documentation generation
- [ ] Integration with Verus for formal verification docs
- [ ] Automated changelog generation
- [ ] Link validation and fixing

## Contributing

Contributions welcome! Areas for improvement:

1. **Better Analysis**: Smarter detection of outdated docs
2. **Prompt Engineering**: More effective Claude prompts
3. **Diagram Types**: Additional visualization types
4. **Performance**: Reduce API calls and costs
5. **Testing**: More comprehensive test coverage

## License

Same as parent project: MIT OR Apache-2.0

## References

- [Claude API Documentation](https://docs.anthropic.com/claude/reference/getting-started-with-the-api)
- [Mermaid Documentation](https://mermaid.js.org/)
- [GitHub Actions](https://docs.github.com/en/actions)
- [Project Design Principles](../../CLAUDE.md)

---

**Last Updated**: 2025-11-13
**Version**: 0.1.0
**Status**: Production Ready ✅
