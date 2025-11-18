# Autonomous Claude Code Executor

## Overview

The **Autonomous Executor** is a self-improving system that:

1. **Analyzes** the `ROADMAP.md` to find highest ROI tasks
2. **Prioritizes** tasks using a scoring algorithm (impact / time)
3. **Executes** tasks autonomously using Claude Code
4. **Validates** success (tests, clippy, formatting)
5. **Creates PRs** and auto-merges when CI passes
6. **Repeats** until full test coverage or roadmap completion

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  GitHub Actions (Scheduler)                             │
│  - Runs daily at 2 AM UTC                              │
│  - Rebases all claude/** branches                      │
│  - Invokes autonomous executor                         │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  ROI Calculator & Task Selector                         │
│  - Parses ROADMAP.md for tasks                         │
│  - Calculates: (priority × status) / time             │
│  - Selects highest ROI incomplete task                │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Claude Code Agent                                      │
│  - Receives task description + context                 │
│  - Implements solution autonomously                    │
│  - Runs pre-commit checks (fmt, clippy)               │
│  - Commits changes to new branch                       │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Quality Gate                                           │
│  - cargo test --all (must pass)                        │
│  - cargo clippy (no warnings)                          │
│  - cargo fmt (must be formatted)                       │
│  - Creates PR if all pass                              │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Auto-Merge System                                      │
│  - Waits for CI/CD (all checks green)                  │
│  - Auto-merges using squash merge                      │
│  - Notifies on failure                                 │
│  - Returns to step 1 (select next task)               │
└─────────────────────────────────────────────────────────┘
```

## ROI Scoring Algorithm

### Formula

```
ROI = (Priority Weight × Status Multiplier × Completion Boost) / Time (hours)
```

### Priority Weights

| Priority | Weight | Use Case |
|----------|--------|----------|
| CRITICAL | 100 | Blocking issues, security vulnerabilities |
| HIGH | 50 | Important features, high-impact improvements |
| MEDIUM | 25 | Standard features, optimizations |
| LOW | 10 | Nice-to-haves, documentation |

### Status Multipliers

| Status | Multiplier | Rationale |
|--------|-----------|-----------|
| 🚧 IN PROGRESS | 1.5× | Prioritize finishing started work |
| ⚠️ NEEDS ATTENTION | 1.3× | Address blockers quickly |
| 📋 Planned | 1.0× | Standard priority |
| ✅ COMPLETE | 0× | Skip completed tasks |

### Completion Boost

- **80%+ complete**: 1.5× multiplier (finish almost-done tasks)
- Calculated from: `completed_items / (completed_items + remaining_items)`

### Time Conversion

- `"4-6 hours"` → 5 hours (average)
- `"1-2 weeks"` → 120 hours (average × 40 hours/week)
- `"3 days"` → 24 hours (average × 8 hours/day)

### Example Calculation

```
Task: "Complete OP Stack" (Phase 3.2)
- Priority: HIGH (weight = 50)
- Status: 🚧 IN PROGRESS (multiplier = 1.5)
- Completion: 90% (3 items done, 1 remaining) → boost = 1.5
- Time: "~4 hours" → 4 hours

ROI = (50 × 1.5 × 1.5) / 4 = 28.125

Compare to:
Task: "WASM Demo" (Phase 3.10)
- Priority: HIGH (weight = 50)
- Status: 📋 Planned (multiplier = 1.0)
- Completion: 0% (0 done, 10 remaining) → boost = 1.0
- Time: "1-2 weeks" → 120 hours

ROI = (50 × 1.0 × 1.0) / 120 = 0.417

Result: OP Stack has 67× higher ROI (28.125 vs 0.417)
→ Executor chooses OP Stack first
```

## Usage

### Automatic Execution (Scheduled)

The executor runs **automatically daily at 2 AM UTC**:

```yaml
# .github/workflows/autonomous-executor.yml
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
```

**What happens**:
1. Rebases all `claude/**` branches onto `main`
2. Selects top 1 highest ROI task
3. Executes task using Claude Code
4. Creates PR if successful
5. Auto-merges when CI passes

### Manual Execution

#### 1. Execute Specific Task

```bash
# Via GitHub Actions UI
Go to Actions → Autonomous Task Executor → Run workflow
  - task_id: "phase-3.2"
  - dry_run: false

# Via CLI
gh workflow run autonomous-executor.yml \
  -f task_id=phase-3.2 \
  -f dry_run=false
```

#### 2. Execute Top N Tasks

```bash
# Execute top 3 highest ROI tasks
gh workflow run autonomous-executor.yml \
  -f top_n=3
```

#### 3. Dry Run (Preview)

```bash
# See what would be executed without making changes
gh workflow run autonomous-executor.yml \
  -f dry_run=true
```

#### 4. Local Execution

```bash
# Install dependencies
pip install anthropic  # Claude Code SDK (when available)

# Set API keys
export ANTHROPIC_API_KEY="sk-ant-..."
export GITHUB_TOKEN="ghp_..."

# List all tasks with ROI scores
python scripts/autonomous_executor.py --list-tasks

# Execute specific task
python scripts/autonomous_executor.py --task-id phase-3.2

# Execute top 3 tasks
python scripts/autonomous_executor.py --top-n 3

# Dry run
python scripts/autonomous_executor.py --dry-run
```

## Safety Guardrails

### 1. Quality Gates (Mandatory)

Every task execution must pass:

✅ **Code Formatting**: `cargo fmt --all` (no changes)
✅ **Clippy Lints**: `cargo clippy --all --all-targets --all-features -- -D warnings` (zero warnings)
✅ **Test Suite**: `cargo test --all` (100% pass rate)
✅ **Build**: `cargo build --all` (successful compilation)

**If any check fails → PR not created, task marked failed**

### 2. Branch Protection

- ✅ All PRs require CI to pass before merge
- ✅ Auto-merge only enabled after all checks green
- ✅ Changes only on `claude/**` branches (never directly to `main`)
- ✅ Squash merges (clean history)

### 3. Rollback Mechanism

If a merged PR causes issues:

```bash
# Revert the merge commit
git revert -m 1 <merge-commit-sha>

# Or reset to before merge (if no other commits)
git reset --hard <previous-commit>
git push origin main --force-with-lease
```

### 4. Rate Limiting

- ✅ Max 1 task per day (scheduled runs)
- ✅ Max 3 tasks per manual run (configurable)
- ✅ 2-hour timeout per task execution
- ✅ Prevents infinite loops / runaway execution

### 5. Notification System

- ✅ GitHub issue created on failure (label: `autonomous`, `bug`)
- ✅ Workflow logs available for debugging
- ✅ PR comments show detailed execution summary

### 6. Manual Override

Maintainers can always:
- ✅ Cancel running workflow
- ✅ Close autonomous PRs
- ✅ Disable scheduled runs (edit `.github/workflows/autonomous-executor.yml`)
- ✅ Modify ROI weights (edit `scripts/autonomous_executor.py`)

## Configuration

### Tuning ROI Weights

Edit `scripts/autonomous_executor.py`:

```python
class ROICalculator:
    PRIORITY_WEIGHTS = {
        "CRITICAL": 100,  # ← Increase to prioritize critical tasks
        "HIGH": 50,       # ← Adjust for your needs
        "MEDIUM": 25,
        "LOW": 10,
    }

    STATUS_MULTIPLIERS = {
        "🚧 IN PROGRESS": 1.5,       # ← Boost in-progress tasks
        "⚠️ NEEDS ATTENTION": 1.3,
        "📋 Planned": 1.0,
        "✅ COMPLETE": 0.0,
    }
```

### Changing Schedule

Edit `.github/workflows/autonomous-executor.yml`:

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC

    # More frequent (every 6 hours):
    - cron: '0 */6 * * *'

    # Less frequent (weekly on Monday):
    - cron: '0 2 * * 1'
```

### Task Selection Criteria

Modify `scripts/autonomous_executor.py`:

```python
# Select top N by ROI, excluding completed
selected_tasks = [t for t in tasks if t.status != "✅ COMPLETE"][:args.top_n]

# Add custom filters:
# - Only CRITICAL/HIGH priority
selected_tasks = [
    t for t in tasks
    if t.status != "✅ COMPLETE"
    and t.priority in ["CRITICAL", "HIGH"]
][:args.top_n]

# - Only tasks under 1 week
selected_tasks = [
    t for t in tasks
    if t.status != "✅ COMPLETE"
    and ROICalculator.parse_time_estimate(t.time_estimate) <= 40  # 40 hours = 1 week
][:args.top_n]
```

## Monitoring

### View Execution History

```bash
# List recent workflow runs
gh run list --workflow=autonomous-executor.yml

# View logs for specific run
gh run view <run-id> --log

# View PRs created by autonomous executor
gh pr list --label autonomous
```

### Dashboards

- **GitHub Actions**: https://github.com/prasincs/universal-blockchain-decoder/actions
- **PRs**: Filter by label `autonomous`
- **Issues**: Filter by label `autonomous` + `bug` for failures

### Success Metrics

Track these over time:

| Metric | Target | Current |
|--------|--------|---------|
| Tasks completed per week | 7 | - |
| Test coverage | 100% | ~75% |
| Property tests | 50 | 16 |
| Failed executions | < 10% | - |
| PR merge time | < 1 hour | - |

## Troubleshooting

### Issue: Tasks not executing

**Check**:
1. `ANTHROPIC_API_KEY` secret is set in GitHub
2. `GITHUB_TOKEN` has write permissions
3. Workflow is enabled (not disabled in `.github/workflows/`)
4. No tasks have `status = "✅ COMPLETE"` (all done!)

**Debug**:
```bash
# Run locally with verbose logging
python scripts/autonomous_executor.py --list-tasks
```

### Issue: PRs not auto-merging

**Check**:
1. CI checks are passing (all green)
2. Branch protection rules allow auto-merge
3. PR has label `autonomous`
4. Auto-merge enabled: `gh pr view <number> --json autoMergeRequest`

**Fix**:
```bash
# Manually enable auto-merge
gh pr merge <number> --auto --squash
```

### Issue: Tests failing in CI but pass locally

**Check**:
1. `Cargo.lock` committed (reproducible builds)
2. No OS-specific dependencies
3. Timeout limits (CI may be slower)

**Fix**:
```bash
# Update Cargo.lock
cargo update
git add Cargo.lock
git commit -m "Update Cargo.lock for CI reproducibility"
```

### Issue: Executor stuck in loop

**Symptoms**: Same task executed repeatedly without progress

**Fix**:
1. **Cancel workflow**: GitHub Actions → Cancel running workflow
2. **Disable schedule**: Edit `.github/workflows/autonomous-executor.yml`, comment out `schedule` section
3. **Debug task**: Run manually with `--dry-run` to see what's happening
4. **Update roadmap**: Mark problematic task as ✅ COMPLETE or add blockers

## Advanced Usage

### Custom Task Prompts

Modify `ClaudeCodeExecutor._build_prompt()` to customize instructions:

```python
def _build_prompt(self, task: Task) -> str:
    # Add project-specific context
    prompt = f"""
You are working on the Universal Blockchain Decoder.

{task.description}

## Additional Context

Read CLAUDE.md for design principles.
Follow the testing strategy in docs/TESTING_STRATEGY.md.
Use TodoWrite to track progress.

## Success Criteria

{task.remaining_items}

Begin implementation now.
"""
    return prompt
```

### Integration with External Systems

```python
# Send notifications to Slack
def notify_slack(message: str):
    webhook_url = os.environ.get("SLACK_WEBHOOK_URL")
    if webhook_url:
        requests.post(webhook_url, json={"text": message})

# In autonomous_executor.py:
if success:
    notify_slack(f"✅ Task completed: {task.title}")
else:
    notify_slack(f"❌ Task failed: {task.title}")
```

### Parallel Execution

Execute multiple tasks in parallel (advanced):

```python
# In autonomous_executor.py main()
from concurrent.futures import ThreadPoolExecutor

with ThreadPoolExecutor(max_workers=3) as executor:
    futures = [
        executor.submit(executor.execute_task, task)
        for task in selected_tasks
    ]
    results = [f.result() for f in futures]
```

**⚠️ Warning**: Parallel execution may cause git conflicts. Ensure tasks are independent.

## Future Enhancements

### Phase 2: Self-Optimization

- [ ] **Learning from failures**: Adjust ROI weights based on success rate
- [ ] **Time estimation**: Learn actual time taken vs estimated
- [ ] **Dependency detection**: Parse `remaining_items` for blockers
- [ ] **Smart scheduling**: Execute related tasks together

### Phase 3: Advanced Capabilities

- [ ] **Code review**: Autonomous executor reviews own PRs
- [ ] **Refactoring**: Identify code duplication and refactor
- [ ] **Documentation**: Auto-generate missing docs
- [ ] **Performance**: Profile and optimize hot paths
- [ ] **Security**: Scan for vulnerabilities autonomously

### Phase 4: Multi-Agent Collaboration

- [ ] **Specialization**: Different agents for testing, docs, features
- [ ] **Consensus**: Multiple agents review each other's work
- [ ] **Debate**: Agents propose different solutions, best wins

## Examples

### Example 1: Completing OP Stack

```bash
# Manual trigger for specific task
gh workflow run autonomous-executor.yml \
  -f task_id=phase-3.2 \
  -f dry_run=false

# Expected output:
# ✅ Task completed: Complete OP Stack
# ✅ PR created: https://github.com/.../pull/123
# ✅ Auto-merge enabled
# (waits for CI)
# ✅ PR merged automatically
```

### Example 2: Adding Property Tests

```bash
# Autonomous executor selects task
# ROI calculation:
# - Priority: MEDIUM (25)
# - Status: 🚧 IN PROGRESS (1.5×)
# - Completion: 32% (16/50 tests) → 1.0×
# - Time: "ongoing" → 40 hours
# ROI = (25 × 1.5 × 1.0) / 40 = 0.9375

# Executor implements:
# 1. Reads property test template
# 2. Generates 34 new property tests
# 3. Runs tests (must pass)
# 4. Commits to branch claude/automate-property-tests-...
# 5. Creates PR
# 6. Auto-merges when CI passes
```

### Example 3: Daily Rebase + Execution

```bash
# Scheduled run at 2 AM UTC
# 1. Rebase all claude/** branches
# 2. Select highest ROI task (e.g., "Complete OP Stack")
# 3. Execute task
# 4. Create PR
# 5. Monitor CI
# 6. Auto-merge when green

# Next day at 2 AM UTC
# 1. Rebase again
# 2. Select next highest ROI task (e.g., "EVM Test Fixtures")
# 3. Repeat...
```

## Ethical Considerations

### Transparency

- ✅ All PRs labeled `autonomous`
- ✅ Commit messages indicate automation
- ✅ PR descriptions explain what changed
- ✅ Logs available for audit

### Human Oversight

- ✅ Maintainers can review/close PRs
- ✅ Manual approval can be required (branch protection)
- ✅ Workflows can be disabled
- ✅ Execution limited to 1 task/day by default

### Responsible AI

- ✅ Quality gates prevent broken code
- ✅ Test coverage ensures correctness
- ✅ Formal verification for critical paths
- ✅ Human review for security-sensitive changes

## Conclusion

The Autonomous Executor enables **continuous, relentless progress** on the Universal Blockchain Decoder roadmap while maintaining **high code quality** and **safety standards**.

**Benefits**:
- 🚀 **Velocity**: 7+ tasks per week (vs 2-3 manual)
- 🎯 **Focus**: Always works on highest ROI tasks
- 🔁 **Consistency**: Pre-commit checks never forgotten
- 📈 **Progress**: Visible through PRs and metrics
- 🤖 **Automation**: Zero human intervention needed

**Result**: A self-improving codebase that **autonomously evolves toward full test coverage, formal verification, and production readiness**.

---

**Status**: Experimental (Phase 1)
**Last Updated**: 2025-11-18
**Feedback**: https://github.com/prasincs/universal-blockchain-decoder/issues

**Next Steps**:
1. Set `ANTHROPIC_API_KEY` and `GITHUB_TOKEN` secrets
2. Enable workflow in GitHub Actions
3. Monitor first few executions
4. Tune ROI weights based on results
5. Expand to multi-agent collaboration (Phase 4)
