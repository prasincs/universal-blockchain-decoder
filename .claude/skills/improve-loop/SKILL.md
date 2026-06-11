---
name: improve-loop
description: Execute one iteration of the measured self-improvement loop - run the health report, take the top backlog item, fix it, verify, ratchet, commit. Use when asked to run the improvement loop, self-improve, or work through the backlog.
---

# improve-loop: one iteration of the self-improvement loop

Execute EXACTLY ONE iteration. Design and rules: `docs/SELF_IMPROVEMENT_LOOP.md`.
Findings that seeded the system: `docs/ASSUMPTIONS_REVIEW.md`.

## Procedure

1. **Measure first.**
   ```bash
   python3 scripts/loop/health_report.py
   ```
   - Exit non-zero (broken workspace or ratchet regression) → fixing that IS
     this iteration; skip step 2.
   - Compare output against `loop/report.json` from the last commit; note any
     drift.
   - If `upstream_outdated` is non-empty for a library that has differential
     tests, append a "bump + re-run differential suite" item to the backlog
     (P1, upstream-updates section) before selecting.

2. **Select** the topmost `[ ]` item in `loop/BACKLOG.md` whose "Blocked by"
   (if any) is resolved. Mark it `[~]`. Do not invent a different task; if you
   discover something new, append it to the backlog with a mechanical
   acceptance check and continue with the selected item.

3. **Implement** the smallest change that satisfies the item's acceptance
   check. Stay within the item's scope.

4. **Verify** — all of:
   ```bash
   # the item's own acceptance command (from the backlog entry)
   cargo fmt --all
   cargo clippy --all --all-targets --all-features -- -D warnings
   cargo test -p universal-decoder-core -p decoder-bitcoin -p decoder-ethereum -p decoder-solana
   python3 scripts/loop/health_report.py --update-ratchet
   ```
   The last command must exit 0. If your change should have moved a metric,
   confirm `loop/report.json` shows it. Run the full `cargo test --all` when
   the change touches shared crates.

5. **Record**: mark the item `[x]` with a one-line note and date; commit the
   code change together with `loop/BACKLOG.md`, `loop/report.json`, and
   `loop/ratchet.json`. Commit message: what changed and which metric moved
   (e.g. "Bitcoin true re-encoding: raw_bytes_storing_count 24 -> 23").

6. **Stop** after one item. Report: item completed, metric movement, anything
   appended to the backlog.

## Hard rules (from docs/SELF_IMPROVEMENT_LOOP.md — read them if unsure)

- NEVER weaken/delete/ignore a test or fixture to get green.
- NEVER hand-edit `loop/ratchet.json`; only `--update-ratchet` writes it.
- NEVER satisfy a metric by deleting what it measures, unless the backlog
  item says deletion is the fix.
- A differential-test disagreement with an upstream library is a deliverable:
  capture a minimal repro fixture and file it in the backlog before deciding
  which side is wrong.
- If the selected item turns out to be ambiguous or needs a decision the
  backlog doesn't specify (e.g. canonical format break), stop and ask instead
  of guessing — or in unattended mode, skip to the next item and note why.
