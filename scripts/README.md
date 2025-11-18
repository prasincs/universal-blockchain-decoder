# CI Scripts

Utility scripts for continuous integration and development workflow.

## CI Monitor (`ci-monitor.sh`)

Automated CI state monitoring and issue resolution script.

### Features

- **Automatic Formatting**: Detects and fixes `cargo fmt` issues
- **Clippy Analysis**: Runs clippy and provides detailed error analysis
- **Documentation Check**: Verifies documentation builds without errors
- **Smart Suggestions**: Analyzes common error patterns and suggests fixes
- **Watch Mode**: Continuous monitoring with auto-refresh
- **Color-coded Output**: Easy-to-read status reporting

### Usage

Basic check (no modifications):
```bash
./scripts/ci-monitor.sh
```

Auto-fix issues where possible:
```bash
./scripts/ci-monitor.sh --fix
```

Check and run tests:
```bash
./scripts/ci-monitor.sh --fix --test
```

Continuous monitoring (watch mode):
```bash
./scripts/ci-monitor.sh --watch
```

### Exit Codes

- `0`: All checks passed
- `1`: Some issues remain or checks failed

### CI Checks Performed

1. **Formatting** (`cargo fmt --check`)
   - Auto-fixable with `--fix` flag
   - Applies consistent code formatting

2. **Clippy Lints** (`cargo clippy`)
   - Detects common Rust anti-patterns
   - Provides suggestions for manual fixes
   - Recognizes patterns like:
     - `needless_borrows_for_generic_args`
     - `unused_imports`
     - `empty_line_after_doc_comments`
     - `useless_vec`

3. **Documentation** (`cargo doc`)
   - Ensures all docs build without errors
   - Catches broken doc links and syntax issues

4. **Tests** (optional with `--test`)
   - Runs unit tests across all crates

### Common Error Patterns

The script automatically detects and provides guidance for:

| Error Pattern | Suggestion |
|--------------|------------|
| `needless_borrows_for_generic_args` | Remove `&` from arguments where indicated |
| `unused_imports` | Remove unused `use` statements |
| `empty_line_after_doc_comments` | Remove blank lines between `///` and code |
| `useless_vec` | Replace `vec![...]` with `[...]` for arrays |

### Integration with Development Workflow

**Pre-commit hook:**
```bash
#!/bin/bash
# .git/hooks/pre-commit
./scripts/ci-monitor.sh --fix
git add -u  # Add auto-fixed files
```

**VS Code task:**
```json
{
  "label": "CI Check",
  "type": "shell",
  "command": "./scripts/ci-monitor.sh",
  "group": "test"
}
```

**GitHub Actions:**
```yaml
- name: Run CI Monitor
  run: ./scripts/ci-monitor.sh
```

### Examples

Check CI state before committing:
```bash
$ ./scripts/ci-monitor.sh

=========================================
  CI State Monitor & Auto-Fixer
=========================================

[INFO] Starting CI checks...

===== Checking: Formatting =====
[INFO] Formatting: ✓ PASSED

===== Checking: Clippy =====
[INFO] Clippy: ✓ PASSED

[INFO] Checking documentation build...
[INFO] Documentation builds cleanly

=========================================
  CI Monitor Summary
=========================================
Issues found:     0
Issues fixed:     0
Issues remaining: 0

[INFO] ✓ All CI checks passed!
```

Auto-fix formatting issues:
```bash
$ ./scripts/ci-monitor.sh --fix

[ERROR] Formatting: ✗ FAILED
[INFO] Attempting to fix formatting issues...
[INFO] Formatting fixes applied
[INFO] Formatting (after fix): ✓ PASSED
[INFO] Issues fixed:     1
```

### Performance

- Average runtime: 2-5 seconds
- Watch mode refresh: 30 seconds
- Safe to run frequently during development

### Requirements

- Rust toolchain (stable)
- `cargo`, `cargo-fmt`, `cargo-clippy`
- Bash shell

### Notes

- The script runs in the repository root
- Temporary files stored in `/tmp/ci-*.log`
- Colored output requires ANSI terminal support
- Watch mode can be stopped with Ctrl+C

---

## Autonomous Executor (`autonomous_executor.py`)

Autonomous task execution system that implements highest ROI tasks from the roadmap automatically.

### Features

- **ROI-based Task Selection**: Automatically selects tasks using `(Priority × Status) / Time` formula
- **Autonomous Execution**: Uses Claude Code to implement tasks without human intervention
- **Quality Gates**: Enforces fmt, clippy, and test requirements before creating PRs
- **Auto-PR Creation**: Automatically creates and merges PRs when CI passes
- **Scheduled Runs**: Daily execution via GitHub Actions (2 AM UTC)
- **Safety Guardrails**: Rate limiting, rollback support, manual override

### Quick Start

```bash
# Install dependencies (when SDK available)
pip install anthropic

# Set environment variables
export ANTHROPIC_API_KEY="sk-ant-..."
export GITHUB_TOKEN="ghp_..."

# List tasks by ROI
python scripts/autonomous_executor.py --list-tasks

# Execute highest ROI task
python scripts/autonomous_executor.py

# Dry run (preview without executing)
python scripts/autonomous_executor.py --dry-run
```

### Usage

**List all tasks with ROI scores:**
```bash
python scripts/autonomous_executor.py --list-tasks
```

**Execute specific task:**
```bash
python scripts/autonomous_executor.py --task-id phase-3.2
```

**Execute top N tasks:**
```bash
python scripts/autonomous_executor.py --top-n 3
```

**Dry run (preview only):**
```bash
python scripts/autonomous_executor.py --dry-run
```

**GitHub Actions (automated):**
```bash
# Set secrets in GitHub: ANTHROPIC_API_KEY
# Runs daily at 2 AM UTC automatically

# Manual trigger:
gh workflow run autonomous-executor.yml
```

### ROI Scoring Formula

```
ROI = (Priority Weight × Status Multiplier × Completion Boost) / Time (hours)
```

**Priority Weights:**
- CRITICAL: 100
- HIGH: 50
- MEDIUM: 25
- LOW: 10

**Status Multipliers:**
- 🚧 IN PROGRESS: 1.5× (finish started work first)
- ⚠️ NEEDS ATTENTION: 1.3× (address blockers quickly)
- 📋 Planned: 1.0×
- ✅ COMPLETE: 0× (skip completed tasks)

**Completion Boost:**
- 80%+ complete: 1.5× (almost done, finish it!)

### Example Output

```bash
$ python scripts/autonomous_executor.py --list-tasks

Tasks (sorted by ROI):

1. phase-3.2: Complete OP Stack
   Status: 🚧 IN PROGRESS | Priority: HIGH | ROI: 28.13
   Time: ~4 hours | Completed: 3/4

2. phase-3.1.x: EVM Test Fixtures
   Status: ⚠️ NEEDS ATTENTION | Priority: HIGH | ROI: 10.83
   Time: 4-6 hours | Completed: 9/16

3. phase-1.5.2: Property Tests
   Status: 🚧 IN PROGRESS | Priority: MEDIUM | ROI: 0.94
   Time: ongoing | Completed: 16/50
```

### Safety Guardrails

✅ **Quality checks required:**
- `cargo fmt --all` (no formatting changes)
- `cargo clippy --all --all-targets --all-features -- -D warnings` (zero warnings)
- `cargo test --all` (100% pass rate)

✅ **Branch protection:**
- Only creates `claude/**` branches
- Never pushes directly to `main`
- Requires CI to pass before merge

✅ **Rate limiting:**
- Max 1 task per day (scheduled runs)
- Max 3 tasks per manual run
- 2-hour timeout per task

✅ **Rollback available:**
- All changes in PRs
- Easy to revert if needed

### Configuration

**Tune ROI weights** (`scripts/autonomous_executor.py`):
```python
PRIORITY_WEIGHTS = {
    "CRITICAL": 100,  # ← Adjust here
    "HIGH": 50,
    "MEDIUM": 25,
    "LOW": 10,
}
```

**Change schedule** (`.github/workflows/autonomous-executor.yml`):
```yaml
schedule:
  - cron: '0 2 * * *'  # Daily at 2 AM UTC
  # More frequent: '0 */6 * * *' (every 6 hours)
  # Less frequent: '0 2 * * 1' (weekly on Monday)
```

### Monitoring

```bash
# List recent workflow runs
gh run list --workflow=autonomous-executor.yml

# View logs for specific run
gh run view <run-id> --log

# View PRs created by autonomous executor
gh pr list --label autonomous
```

### Documentation

- **Full Guide**: `docs/AUTONOMOUS_EXECUTOR.md` (comprehensive documentation)
- **Roadmap**: `ROADMAP.md` (task source)
- **Architecture**: `CLAUDE.md` (design principles)

---

**See also**: `../CLAUDE.md` for overall project guidelines
