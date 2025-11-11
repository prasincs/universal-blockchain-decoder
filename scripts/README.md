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

**See also**: `../CLAUDE.md` for overall project guidelines
