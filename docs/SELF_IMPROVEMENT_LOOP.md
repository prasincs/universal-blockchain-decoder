# Self-Improvement Loop

How this project improves itself autonomously without lying to itself.

## Why the previous approach was unsound

`tools/autonomous-executor` selected work by parsing ROADMAP.md status emojis
and self-reported completion percentages into an "ROI" score. The 2026-06
review (`docs/ASSUMPTIONS_REVIEW.md`) showed why that can't work: the repo's
self-reported status had drifted far from reality (fuzz targets that don't
compile were "complete"; decoders whose roundtrip test is vacuous were
"production-ready"; "validation against upstream" existed for 1 chain out of
6 that declared it; the workspace didn't build at all). A loop that consumes
self-reported status optimizes the documents, not the code.

**Principle: the loop reads only measured signals and writes only
mechanically-verifiable improvements.**

## The structure

```
scripts/loop/health_report.py   The sensor. Measures the repo (build, tests,
                                LOC budget, raw-bytes cheating, dead deps,
                                JSON-in-canonical, differential coverage,
                                fixtures, fuzz targets). Stdlib Python only,
                                so it runs even when the workspace is broken.
loop/report.json                Latest measurement (committed, diffable).
loop/ratchet.json               One-way baselines. "max" metrics may never
                                increase, "min" metrics may never decrease.
                                A regression makes the report exit non-zero.
loop/BACKLOG.md                 Prioritized work items derived from measured
                                findings, each with a mechanical acceptance
                                check. The loop takes the top unblocked item.
.claude/skills/improve-loop/    The actor. A Claude Code skill that executes
                                exactly one iteration. Runnable interactively
                                (/improve-loop), on a schedule (/loop
                                /improve-loop), or from CI.
docs/ASSUMPTIONS_REVIEW.md      The audit that seeded all of the above.
```

## Signal sources

The loop consumes three kinds of measured signal; nothing else generates work:

1. **Internal invariants** (offline, every run): build/tests, core LOC budget,
   raw-bytes cheating, dead validation deps, JSON-in-canonical, fixture and
   fuzz-target inventory.
2. **Known on-chain transactions**: real transactions fetched by txid via
   `scripts/loop/fetch_corpus.py`. Fixtures are self-certifying — the txid is
   recomputed locally from the raw bytes (Keccak-256 for Ethereum,
   double-SHA256 over the witness-stripped serialization for Bitcoin) before
   writing, and stored in the sidecar for independent re-verification. The
   differential tests decode each fixture with both our decoder and the
   upstream library; any field-level disagreement is a finding. Corpus depth
   (exotic transaction shapes), not chain count, is what stress-tests TxIR.
3. **Upstream dependency updates**: the health report compares every locked
   upstream oracle against the latest stable release on crates.io
   (`upstream_outdated`). A new upstream release is adversarial-testing work:
   bump the dev-dep, re-run the differential suite, and treat disagreements
   with the new version as findings. This check is informational (network-
   dependent, not ratcheted) and skipped silently when offline.

## One iteration

1. **Measure**: `python3 scripts/loop/health_report.py` (add `--build`/
   `--test` when touching code). If it exits non-zero on entry, fixing THAT
   is the iteration.
2. **Select**: topmost unblocked `[ ]` item in `loop/BACKLOG.md`. No item may
   be invented mid-iteration; new findings become new backlog entries instead.
3. **Implement**: smallest change satisfying the item's acceptance check.
4. **Verify**: the item's own check, plus `cargo fmt --all`,
   `cargo clippy --all --all-targets --all-features -- -D warnings`, plus the
   health report with `--update-ratchet` (records improvements, fails on
   regressions).
5. **Record**: tick the item, re-commit `loop/report.json` + `loop/ratchet.json`
   with the change. The commit message states which metric moved and how.
6. **Stop.** One item per iteration keeps every commit reviewable and keeps a
   bad iteration from compounding.

## Anti-Goodhart rules (non-negotiable)

The loop improves the *code*, and the metrics merely observe it. Therefore:

1. **Never weaken a test, assertion, or fixture to make a check pass.**
   Deleting/`#[ignore]`-ing a failing test is a regression, not a fix.
2. **Never edit `loop/ratchet.json` by hand.** Only
   `health_report.py --update-ratchet` writes it, and only in the improving
   direction. The single exception is merging upstream work that
   legitimately moves a metric the wrong way:
   `--accept-regression "<reason>"` rebaselines the regressed metrics AND
   records `{metric, from, to, reason, date}` under `accepted_regressions`
   in the ratchet file — the regression stays on the record and should have
   a backlog item driving it back.
3. **Never satisfy a metric by deleting the thing it measures** (e.g.,
   deleting a decoder to lower `raw_bytes_storing_count`) unless the backlog
   item explicitly says deletion is the fix (dead deps are an example where
   it is).
4. **Status claims live in `loop/report.json` only.** Don't write
   "production-ready" / "complete" into ROADMAP/README; cite measurements.
5. **A vacuous pass is a fail.** If a property can't distinguish a correct
   implementation from `Ok(self.raw_bytes.clone())`, strengthen the property
   (mutation testing) before counting it.
6. **Upstream disagreement is a finding, not an annoyance.** When a
   differential test disagrees with upstream, the iteration's output is a
   minimal reproduction + backlog item — whichever side is wrong, that's the
   adversarial value this project exists to produce.

## Running it

Interactive (one iteration):

    /improve-loop

Recurring (e.g. while leaving a session running):

    /loop 30m /improve-loop

CI gate (cheap static mode, fails on ratchet regression):

    python3 scripts/loop/health_report.py

Full measurement before/after a work session:

    python3 scripts/loop/health_report.py --build --test --update-ratchet

## Relationship to existing automation

- `tools/autonomous-executor`: its quality gates (fmt/clippy/test) are kept;
  its ROADMAP-prose ROI selection is superseded by `loop/BACKLOG.md` ordering.
  Backlog item exists to repoint or retire it.
- `.github/workflows/autonomous-executor.yml`, `ai-refactor-suggest.yml`:
  candidates for consolidation onto the health report; until then they must
  not write to `loop/`.
- Nightly fuzzing stays; a backlog item adds compile-checks of fuzz targets
  to PR CI so they cannot rot silently again (that's how three fuzz suites
  died last time).
