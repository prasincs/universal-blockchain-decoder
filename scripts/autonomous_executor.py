#!/usr/bin/env python3
"""
Autonomous Claude Code Task Executor

This script automates the execution of tasks from ROADMAP.md by:
1. Parsing roadmap for highest ROI tasks
2. Calculating priority scores
3. Invoking Claude Code to implement tasks
4. Validating success criteria
5. Creating PRs and managing git workflow

Usage:
    python scripts/autonomous_executor.py [--dry-run] [--task-id TASK_ID]

Environment Variables:
    ANTHROPIC_API_KEY: Required for Claude Code API access
    GITHUB_TOKEN: Required for PR creation and auto-merge
"""

import os
import re
import sys
import json
import subprocess
import argparse
from dataclasses import dataclass
from typing import Optional, List, Dict, Tuple
from datetime import datetime
from pathlib import Path


@dataclass
class Task:
    """Represents a task from the roadmap"""
    id: str
    phase: str
    title: str
    status: str  # "⚠️ NEEDS ATTENTION", "🚧 IN PROGRESS", "✅ COMPLETE", etc.
    time_estimate: Optional[str]  # "4-6 hours", "1-2 weeks", etc.
    priority: str  # "CRITICAL", "HIGH", "MEDIUM", "LOW"
    roi_score: float  # Calculated: impact / time
    description: str
    completed_items: List[str]
    remaining_items: List[str]


class ROICalculator:
    """Calculate ROI scores for tasks"""

    PRIORITY_WEIGHTS = {
        "CRITICAL": 100,
        "HIGH": 50,
        "MEDIUM": 25,
        "LOW": 10,
    }

    STATUS_MULTIPLIERS = {
        "🚧 IN PROGRESS": 1.5,  # Prioritize finishing started work
        "⚠️ NEEDS ATTENTION": 1.3,
        "📋 Planned": 1.0,
        "✅ COMPLETE": 0.0,  # Skip completed tasks
    }

    @staticmethod
    def parse_time_estimate(time_str: Optional[str]) -> float:
        """Convert time estimate to hours (e.g., '4-6 hours' -> 5.0, '1-2 weeks' -> 120.0)"""
        if not time_str:
            return 40.0  # Default: 1 week

        time_str = time_str.lower()

        # Extract numbers
        numbers = re.findall(r'\d+', time_str)
        if not numbers:
            return 40.0

        # Calculate average if range
        avg = sum(int(n) for n in numbers) / len(numbers)

        # Convert to hours
        if 'week' in time_str:
            return avg * 40  # 40 hours/week
        elif 'day' in time_str:
            return avg * 8  # 8 hours/day
        elif 'hour' in time_str or 'hr' in time_str:
            return avg
        elif 'month' in time_str:
            return avg * 160  # 160 hours/month
        else:
            return avg  # Assume hours

    @classmethod
    def calculate_roi(cls, task: Task) -> float:
        """
        Calculate ROI score: (priority_weight × status_multiplier) / time_estimate

        Higher score = better ROI (high impact, low time investment)
        """
        priority_weight = cls.PRIORITY_WEIGHTS.get(task.priority, 10)
        status_multiplier = cls.STATUS_MULTIPLIERS.get(task.status, 1.0)
        time_hours = cls.parse_time_estimate(task.time_estimate)

        # Completion percentage boost
        total_items = len(task.completed_items) + len(task.remaining_items)
        if total_items > 0:
            completion_pct = len(task.completed_items) / total_items
            # Boost ROI for nearly complete tasks (80%+ done)
            if completion_pct >= 0.8:
                status_multiplier *= 1.5

        roi = (priority_weight * status_multiplier) / time_hours
        return roi


class RoadmapParser:
    """Parse ROADMAP.md to extract tasks"""

    def __init__(self, roadmap_path: str = "ROADMAP.md"):
        self.roadmap_path = Path(roadmap_path)
        if not self.roadmap_path.exists():
            raise FileNotFoundError(f"ROADMAP.md not found at {roadmap_path}")

    def parse(self) -> List[Task]:
        """Parse roadmap and return list of tasks with ROI scores"""
        with open(self.roadmap_path, 'r') as f:
            content = f.read()

        tasks = []

        # Find all Phase 3 task sections (most relevant for current work)
        # Pattern: ### 3.X: Task Title STATUS
        pattern = r'###\s+(\d+\.\d+(?:[a-z])?):?\s+(.+?)\s+(✅|🚧|⚠️|📋).*?\n(.*?)(?=\n###|\Z)'

        for match in re.finditer(pattern, content, re.DOTALL):
            phase_id = match.group(1)
            title = match.group(2).strip()
            status_emoji = match.group(3)
            section_content = match.group(4)

            # Map emoji to status text
            status_map = {
                "✅": "✅ COMPLETE",
                "🚧": "🚧 IN PROGRESS",
                "⚠️": "⚠️ NEEDS ATTENTION",
                "📋": "📋 Planned",
            }
            status = status_map.get(status_emoji, "📋 Planned")

            # Extract priority (look for "Priority: CRITICAL" patterns)
            priority_match = re.search(r'\*\*Priority\*\*:\s*(CRITICAL|HIGH|MEDIUM|LOW)', section_content)
            priority = priority_match.group(1) if priority_match else "MEDIUM"

            # Extract time estimate
            time_match = re.search(r'\*\*Time[^:]*\*\*:\s*([^\n]+)', section_content)
            time_estimate = time_match.group(1).strip() if time_match else None

            # Extract completed items (✅ markers)
            completed = re.findall(r'-\s+✅\s+(.+)', section_content)

            # Extract remaining items ([ ] markers or ⏳/🚧 markers)
            remaining = re.findall(r'-\s+\[\s*\]\s+(.+)', section_content)
            remaining.extend(re.findall(r'-\s+⏳\s+(.+)', section_content))

            task = Task(
                id=f"phase-{phase_id}",
                phase=f"Phase {phase_id}",
                title=title,
                status=status,
                time_estimate=time_estimate,
                priority=priority,
                roi_score=0.0,  # Will be calculated
                description=section_content[:500],  # First 500 chars
                completed_items=completed,
                remaining_items=remaining,
            )

            # Calculate ROI
            task.roi_score = ROICalculator.calculate_roi(task)

            tasks.append(task)

        # Sort by ROI score (descending)
        tasks.sort(key=lambda t: t.roi_score, reverse=True)

        return tasks


class ClaudeCodeExecutor:
    """Execute tasks using Claude Code API"""

    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY environment variable required")

    def execute_task(self, task: Task, dry_run: bool = False) -> Tuple[bool, str]:
        """
        Execute a task using Claude Code

        Returns:
            (success: bool, message: str)
        """
        print(f"\n{'='*80}")
        print(f"Executing Task: {task.title}")
        print(f"Phase: {task.phase}")
        print(f"Priority: {task.priority}")
        print(f"ROI Score: {task.roi_score:.2f}")
        print(f"Time Estimate: {task.time_estimate}")
        print(f"{'='*80}\n")

        if dry_run:
            print("[DRY RUN] Would execute task, but skipping...")
            return True, "Dry run - no actual execution"

        # Construct prompt for Claude Code
        prompt = self._build_prompt(task)

        # Create branch name
        branch_name = f"claude/automate-{task.id}-{datetime.now().strftime('%Y%m%d-%H%M%S')}"

        try:
            # 1. Create and checkout branch
            self._run_command(f"git checkout -b {branch_name}")

            # 2. Invoke Claude Code (using subprocess to call claude CLI)
            # Note: This assumes 'claude' CLI is available
            # Alternative: Use Anthropic API directly
            result = self._invoke_claude_code(prompt)

            if not result:
                return False, "Claude Code execution failed"

            # 3. Run pre-commit checks
            if not self._run_quality_checks():
                return False, "Quality checks failed"

            # 4. Run tests
            if not self._run_tests():
                return False, "Tests failed"

            # 5. Commit changes
            commit_msg = f"automate: {task.title}\n\n{task.description[:200]}\n\nGenerated by autonomous executor"
            self._run_command(f'git add -A')
            self._run_command(f'git commit -m "{commit_msg}"')

            # 6. Push branch
            self._run_command(f'git push -u origin {branch_name}')

            return True, f"Successfully completed task on branch {branch_name}"

        except Exception as e:
            return False, f"Error executing task: {str(e)}"

    def _build_prompt(self, task: Task) -> str:
        """Build prompt for Claude Code"""
        prompt = f"""# Autonomous Task Execution

You are working autonomously to complete a task from the Universal Blockchain Decoder roadmap.

## Task Details

**Phase**: {task.phase}
**Title**: {task.title}
**Priority**: {task.priority}
**Time Estimate**: {task.time_estimate}
**Status**: {task.status}

## Description

{task.description}

## Completed Items

{chr(10).join(f'- ✅ {item}' for item in task.completed_items)}

## Remaining Items

{chr(10).join(f'- [ ] {item}' for item in task.remaining_items)}

## Instructions

1. Implement the remaining items listed above
2. Follow the design principles in CLAUDE.md
3. Run `cargo fmt --all && cargo clippy --all --all-targets --all-features -- -D warnings` before committing
4. Write comprehensive tests (unit + property + integration)
5. Update documentation if needed
6. Mark todos as completed in TodoWrite as you finish each item

## Success Criteria

- ✅ All remaining items completed
- ✅ All tests pass (`cargo test --all`)
- ✅ No clippy warnings
- ✅ Code formatted (`cargo fmt --all`)
- ✅ Documentation updated (if needed)

## Important

- Follow the pre-commit checks in CLAUDE.md
- Use TodoWrite to track your progress
- Commit with clear, descriptive messages
- Push to a branch starting with 'claude/'

Begin implementation now. Work autonomously until all success criteria are met.
"""
        return prompt

    def _invoke_claude_code(self, prompt: str) -> bool:
        """Invoke Claude Code with prompt (placeholder - needs actual implementation)"""
        # TODO: Implement actual Claude Code invocation
        # Option 1: Use 'claude' CLI if available
        # Option 2: Use Anthropic API directly
        # Option 3: Use Claude Code SDK

        print("[PLACEHOLDER] Claude Code invocation would happen here")
        print(f"Prompt:\n{prompt[:200]}...\n")

        # For now, return True to test workflow
        # In production, this would actually invoke Claude Code
        return True

    def _run_command(self, cmd: str) -> str:
        """Run shell command and return output"""
        print(f"Running: {cmd}")
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        if result.returncode != 0:
            raise RuntimeError(f"Command failed: {cmd}\n{result.stderr}")
        return result.stdout

    def _run_quality_checks(self) -> bool:
        """Run cargo fmt and clippy"""
        try:
            print("\n[Quality Checks] Running cargo fmt...")
            self._run_command("cargo fmt --all")

            print("[Quality Checks] Running cargo clippy...")
            self._run_command("cargo clippy --all --all-targets --all-features -- -D warnings")

            return True
        except Exception as e:
            print(f"[Quality Checks] FAILED: {e}")
            return False

    def _run_tests(self) -> bool:
        """Run test suite"""
        try:
            print("\n[Tests] Running cargo test...")
            # Allow 10 minutes for tests
            result = subprocess.run(
                "cargo test --all",
                shell=True,
                capture_output=True,
                text=True,
                timeout=600
            )

            if result.returncode != 0:
                print(f"[Tests] FAILED:\n{result.stderr}")
                return False

            print("[Tests] PASSED ✅")
            return True
        except subprocess.TimeoutExpired:
            print("[Tests] FAILED: Timeout after 10 minutes")
            return False
        except Exception as e:
            print(f"[Tests] FAILED: {e}")
            return False


class AutoMergeManager:
    """Manage automatic PR merging when tests pass"""

    def __init__(self, github_token: Optional[str] = None):
        self.github_token = github_token or os.environ.get("GITHUB_TOKEN")
        if not self.github_token:
            raise ValueError("GITHUB_TOKEN environment variable required")

    def create_pr(self, branch: str, task: Task) -> Optional[str]:
        """Create PR using GitHub CLI"""
        try:
            title = f"[Autonomous] {task.title}"
            body = f"""## Automated Task Completion

**Phase**: {task.phase}
**Priority**: {task.priority}
**ROI Score**: {task.roi_score:.2f}

### Description

{task.description}

### Completed Items

{chr(10).join(f'- ✅ {item}' for item in task.completed_items)}

### Changes

This PR was automatically generated by the autonomous executor system.

### Checklist

- [x] All tests pass
- [x] Code formatted (cargo fmt)
- [x] No clippy warnings
- [x] Documentation updated

---

*Generated by autonomous_executor.py*
"""

            cmd = f'gh pr create --title "{title}" --body "{body}" --head {branch}'
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

            if result.returncode != 0:
                print(f"Failed to create PR: {result.stderr}")
                return None

            # Extract PR URL from output
            pr_url = result.stdout.strip()
            print(f"\n✅ PR created: {pr_url}")
            return pr_url

        except Exception as e:
            print(f"Error creating PR: {e}")
            return None

    def enable_auto_merge(self, pr_url: str) -> bool:
        """Enable auto-merge for PR (requires all checks to pass)"""
        try:
            cmd = f'gh pr merge {pr_url} --auto --squash'
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

            if result.returncode != 0:
                print(f"Failed to enable auto-merge: {result.stderr}")
                return False

            print(f"✅ Auto-merge enabled for {pr_url}")
            return True

        except Exception as e:
            print(f"Error enabling auto-merge: {e}")
            return False


def main():
    parser = argparse.ArgumentParser(description="Autonomous Claude Code Task Executor")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done without executing")
    parser.add_argument("--task-id", type=str, help="Execute specific task ID (e.g., 'phase-3.2')")
    parser.add_argument("--top-n", type=int, default=1, help="Execute top N highest ROI tasks (default: 1)")
    parser.add_argument("--list-tasks", action="store_true", help="List all tasks with ROI scores and exit")

    args = parser.parse_args()

    print("=" * 80)
    print("Universal Blockchain Decoder - Autonomous Task Executor")
    print("=" * 80)
    print()

    # 1. Parse roadmap
    print("[1/5] Parsing ROADMAP.md...")
    parser = RoadmapParser()
    tasks = parser.parse()
    print(f"Found {len(tasks)} tasks")

    # List tasks if requested
    if args.list_tasks:
        print("\nTasks (sorted by ROI):\n")
        for i, task in enumerate(tasks[:20], 1):  # Show top 20
            print(f"{i}. {task.id}: {task.title}")
            print(f"   Status: {task.status} | Priority: {task.priority} | ROI: {task.roi_score:.2f}")
            print(f"   Time: {task.time_estimate} | Completed: {len(task.completed_items)}/{len(task.completed_items) + len(task.remaining_items)}")
            print()
        return 0

    # 2. Select task(s)
    print("\n[2/5] Selecting tasks...")

    if args.task_id:
        # Find specific task
        selected_tasks = [t for t in tasks if t.id == args.task_id]
        if not selected_tasks:
            print(f"ERROR: Task '{args.task_id}' not found")
            return 1
    else:
        # Select top N by ROI, excluding completed
        selected_tasks = [t for t in tasks if t.status != "✅ COMPLETE"][:args.top_n]

    if not selected_tasks:
        print("No tasks to execute (all completed or none match criteria)")
        return 0

    print(f"Selected {len(selected_tasks)} task(s) for execution:")
    for task in selected_tasks:
        print(f"  - {task.title} (ROI: {task.roi_score:.2f}, Priority: {task.priority})")

    # 3. Execute tasks
    print("\n[3/5] Executing tasks...")

    executor = ClaudeCodeExecutor()
    auto_merge = AutoMergeManager()

    results = []
    for task in selected_tasks:
        success, message = executor.execute_task(task, dry_run=args.dry_run)
        results.append((task, success, message))

        if success and not args.dry_run:
            # Create PR and enable auto-merge
            branch = message.split("branch ")[-1]
            pr_url = auto_merge.create_pr(branch, task)
            if pr_url:
                auto_merge.enable_auto_merge(pr_url)

    # 4. Report results
    print("\n[5/5] Execution Summary:")
    print("=" * 80)

    successful = sum(1 for _, success, _ in results if success)
    failed = len(results) - successful

    print(f"\nTotal: {len(results)} | Successful: {successful} | Failed: {failed}\n")

    for task, success, message in results:
        status = "✅ SUCCESS" if success else "❌ FAILED"
        print(f"{status}: {task.title}")
        print(f"  {message}\n")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
