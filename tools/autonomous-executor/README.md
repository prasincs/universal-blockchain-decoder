# Autonomous Executor (Rust Implementation)

Autonomous task execution system written in Rust that implements highest ROI tasks from the roadmap automatically.

## Features

- **ROI-based Task Selection**: Automatically selects tasks using `(Priority × Status × Completion) / Time` formula
- **Pure Rust**: Type-safe, fast, zero Python dependencies
- **Quality Gates**: Enforces cargo fmt, clippy, and test requirements
- **Async Execution**: Uses tokio for concurrent operations
- **Comprehensive Error Handling**: Uses anyhow for context-rich errors

## Usage

### Build

```bash
# From repository root
cargo build --release -p autonomous-executor

# Binary location: target/release/autonomous-executor
```

### List Tasks

```bash
# List all tasks sorted by ROI
./target/release/autonomous-executor list-tasks

# Specify custom roadmap path
./target/release/autonomous-executor list-tasks --roadmap-path ROADMAP.md
```

Output:
```
Tasks (sorted by ROI):

1. phase-3.2: Complete OP Stack
   Status: 🚧 IN PROGRESS | Priority: HIGH | ROI: 28.13
   Time: ~4 hours | Completed: 3/4

2. phase-3.1.x: EVM Test Fixtures
   Status: ⚠️ NEEDS ATTENTION | Priority: HIGH | ROI: 10.83
   Time: 4-6 hours | Completed: 9/16
...
```

### Execute Tasks

```bash
# Execute highest ROI task
export ANTHROPIC_API_KEY="sk-ant-..."
export GITHUB_TOKEN="ghp_..."
./target/release/autonomous-executor execute

# Dry run (preview without executing)
./target/release/autonomous-executor execute --dry-run

# Execute specific task
./target/release/autonomous-executor execute --task-id phase-3.2

# Execute top 3 tasks
./target/release/autonomous-executor execute --top-n 3

# Enable verbose logging
RUST_LOG=debug ./target/release/autonomous-executor execute
```

## Architecture

### Modules

- **cli**: Command-line argument parsing (clap)
- **task**: Task data structure
- **parser**: ROADMAP.md parser (regex-based)
- **roi**: ROI calculator with configurable weights
- **executor**: Task execution orchestrator
- **git**: Git operations helper (fmt, clippy, test, commit, push)

### ROI Calculation

```rust
ROI = (Priority Weight × Status Multiplier × Completion Boost) / Time (hours)
```

**Priority Weights**:
- CRITICAL: 100
- HIGH: 50
- MEDIUM: 25
- LOW: 10

**Status Multipliers**:
- 🚧 IN PROGRESS: 1.5× (finish started work first)
- ⚠️ NEEDS ATTENTION: 1.3× (address blockers quickly)
- 📋 Planned: 1.0×
- ✅ COMPLETE: 0× (skip completed tasks)

**Completion Boost**:
- 80%+ complete: 1.5× (almost done, finish it!)

## Configuration

### ROI Weights

Edit `src/roi.rs`:

```rust
pub fn new() -> Self {
    let mut priority_weights = HashMap::new();
    priority_weights.insert("CRITICAL".to_string(), 100.0);  // ← Adjust here
    priority_weights.insert("HIGH".to_string(), 50.0);
    priority_weights.insert("MEDIUM".to_string(), 25.0);
    priority_weights.insert("LOW".to_string(), 10.0);
    // ...
}
```

### Logging

Set `RUST_LOG` environment variable:

```bash
# Levels: error, warn, info, debug, trace
RUST_LOG=info ./target/release/autonomous-executor execute
RUST_LOG=debug ./target/release/autonomous-executor execute
```

## Testing

```bash
# Run unit tests
cargo test -p autonomous-executor

# Run with output
cargo test -p autonomous-executor -- --nocapture

# Run specific test
cargo test -p autonomous-executor test_parse_time_estimate
```

## Dependencies

- `clap`: CLI argument parsing
- `tokio`: Async runtime
- `reqwest`: HTTP client (for Anthropic API)
- `serde`: Serialization
- `regex`: ROADMAP parsing
- `anyhow`: Error handling
- `log` + `env_logger`: Logging

## GitHub Actions Integration

The tool is integrated into `.github/workflows/autonomous-executor.yml`:

```yaml
- name: Build autonomous executor
  run: cargo build --release -p autonomous-executor

- name: Run autonomous executor
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
    RUST_LOG: info
  run: |
    ./target/release/autonomous-executor execute --top-n 1
```

## Future Enhancements

- [ ] Implement actual Anthropic API calls (placeholder currently)
- [ ] Add PR creation using GitHub API (currently manual)
- [ ] Add retry logic for transient failures
- [ ] Add metrics collection and reporting
- [ ] Add parallel task execution
- [ ] Add task dependency detection
- [ ] Add self-optimization (learn from failures)

## Comparison: Rust vs Python

| Aspect | Rust | Python |
|--------|------|--------|
| Performance | ✅ Fast (~10ms startup) | ⚠️ Slower (~100ms startup) |
| Type Safety | ✅ Compile-time checks | ⚠️ Runtime only |
| Dependencies | ✅ Single binary | ⚠️ Requires Python + pip |
| Error Handling | ✅ Result<T, E> | ⚠️ Exceptions |
| Consistency | ✅ Same toolchain as project | ⚠️ Different ecosystem |
| Iteration Speed | ⚠️ Slower (compile time) | ✅ Fast (interpreted) |

**Verdict**: Rust implementation aligns better with project principles (minimal TCB, type safety, formal verification potential).

## Documentation

- **Main Guide**: `docs/AUTONOMOUS_EXECUTOR.md`
- **Roadmap**: `ROADMAP.md` (task source)
- **Architecture**: `CLAUDE.md` (design principles)

## License

MIT OR Apache-2.0
