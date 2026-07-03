#!/usr/bin/env python3
"""Measured health report for the self-improvement loop.

Produces loop/report.json plus a human-readable summary on stdout. Every
number here is MEASURED from the repository — never read from ROADMAP.md or
other self-reported status docs (see docs/ASSUMPTIONS_REVIEW.md, Assumption 5).

Deliberately dependency-light (Python stdlib only) so it still runs when the
Cargo workspace itself is broken — "does the workspace build" is one of the
signals, so the reporter must not depend on it.

Usage:
    scripts/loop/health_report.py            # static checks + cargo metadata
    scripts/loop/health_report.py --build    # also: cargo check --workspace --all-targets
    scripts/loop/health_report.py --test     # also: cargo test on reference crates
    scripts/loop/health_report.py --update-ratchet
        Rewrite loop/ratchet.json with current values where they IMPROVED.
        Never loosens a ratchet; regressions still fail.

Exit codes: 0 ok, 1 ratchet regression or hard failure (workspace broken).
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
CORE = REPO / "crates" / "universal-decoder-core"
RATCHET_PATH = REPO / "loop" / "ratchet.json"
REPORT_PATH = REPO / "loop" / "report.json"

# Reference decoders: depth-first targets the loop must keep green.
REFERENCE_CRATES = [
    "universal-decoder-core",
    "decoder-bitcoin",
    "decoder-ethereum",
    "decoder-solana",
]

# Upstream chain libraries. If one of these appears in a decoder's
# dev-dependencies it MUST be imported by at least one test — otherwise it is
# supply-chain risk with zero validation value (Assumption 4).
UPSTREAM_LIBS = [
    "bitcoin",
    "alloy-primitives",
    "alloy-consensus",
    "alloy-eips",
    "alloy-rlp",
    "pallas-codec",
    "pallas-primitives",
    "pallas-traverse",
    "tonlib-core",
    "solana-transaction-status",
    "solana-sdk",
    "algonaut",
    "algonaut_core",
    "algonaut_transaction",
    "starknet-crypto",
]


def run(cmd, timeout=3600):
    """Run a command in the repo root, return (ok, combined_output)."""
    try:
        p = subprocess.run(
            cmd,
            cwd=REPO,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return p.returncode == 0, (p.stdout + p.stderr)
    except (subprocess.TimeoutExpired, OSError) as e:
        return False, str(e)


def rust_loc(root: Path, exclude_dirs=("vendored",)) -> int:
    total = 0
    for f in root.rglob("*.rs"):
        if any(part in exclude_dirs for part in f.parts):
            continue
        total += sum(1 for _ in f.open(encoding="utf-8", errors="replace"))
    return total


def decoder_crates():
    return sorted(
        d for d in (REPO / "crates").iterdir() if d.is_dir() and d.name.startswith("decoder-")
    )


def dev_deps(cargo_toml: Path):
    """Crude TOML section parse: names listed under [dev-dependencies]."""
    names, in_section = [], False
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_section = stripped == "[dev-dependencies]"
            continue
        if in_section and stripped and not stripped.startswith("#"):
            m = re.match(r"([A-Za-z0-9_-]+)\s*=", stripped)
            if m:
                names.append(m.group(1))
    return names


def crate_uses_lib(crate_dir: Path, lib: str) -> bool:
    """Does any test/bench/example code actually import this library?"""
    ident = lib.replace("-", "_")
    pattern = re.compile(rf"\b(use|extern crate)\s+{re.escape(ident)}\b")
    candidates = []
    for sub in ("tests", "benches", "examples"):
        candidates.extend((crate_dir / sub).rglob("*.rs") if (crate_dir / sub).is_dir() else [])
    # #[cfg(test)] modules inside src also count.
    candidates.extend((crate_dir / "src").rglob("*.rs") if (crate_dir / "src").is_dir() else [])
    for f in candidates:
        if pattern.search(f.read_text(encoding="utf-8", errors="replace")):
            return True
    return False


def check_resolve(report):
    ok, out = run(["cargo", "metadata", "--no-deps", "--format-version", "1"], timeout=600)
    report["workspace_resolves"] = ok
    if not ok:
        report["workspace_resolve_error"] = out[-2000:]
    return ok


def check_core_loc(report):
    report["core_loc_non_vendored"] = rust_loc(CORE / "src")
    report["core_loc_budget"] = 3000  # CLAUDE.md claim; currently violated, ratchet enforces non-growth


def check_raw_bytes_storage(report):
    """Decoders whose chain-specific type stores the original bytes.

    Stored bytes make encode(decode(x)) == x vacuous (Assumption 3). Heuristic:
    a `raw_bytes` (or `original_bytes`) struct field in src/. Decoders should
    reconstruct bytes from parsed fields instead.
    """
    field = re.compile(r"^\s*(?:pub\s+)?(?:raw_bytes|original_bytes)\s*:\s*Vec<u8>", re.M)
    offenders = []
    for crate in decoder_crates():
        src = crate / "src"
        if not src.is_dir():
            continue
        for f in src.rglob("*.rs"):
            if field.search(f.read_text(encoding="utf-8", errors="replace")):
                offenders.append(crate.name)
                break
    report["raw_bytes_storing_decoders"] = sorted(offenders)
    report["raw_bytes_storing_count"] = len(offenders)


def check_dead_validation_deps(report):
    """Upstream chain libs declared as dev-deps but never imported by tests."""
    dead, live = [], []
    for crate in decoder_crates():
        toml = crate / "Cargo.toml"
        if not toml.is_file():
            continue
        for dep in dev_deps(toml):
            if dep in UPSTREAM_LIBS:
                (live if crate_uses_lib(crate, dep) else dead).append(f"{crate.name}:{dep}")
    report["dead_validation_deps"] = sorted(dead)
    report["dead_validation_deps_count"] = len(dead)
    report["differential_test_deps"] = sorted(live)
    report["differential_decoders_count"] = len({entry.split(":")[0] for entry in live})


def check_json_in_canonical(report):
    """String fields in the canonical (hashed) representation (Assumption 2)."""
    canonical = CORE / "src" / "canonical.rs"
    count = 0
    if canonical.is_file():
        text = canonical.read_text(encoding="utf-8")
        # Free-form String fields inside Canonical* structs; these carry JSON.
        count = len(re.findall(r":\s*(?:Option<\s*)?String", text))
    report["string_fields_in_canonical"] = count


def check_fuzz_targets(report):
    dirs = sorted(p.parent.parent.name for p in (REPO / "crates").glob("*/fuzz/Cargo.toml"))
    report["crates_with_fuzz_targets"] = dirs


def check_fixtures(report):
    counts = {}
    for crate in decoder_crates():
        fixtures = crate / "tests" / "fixtures"
        if fixtures.is_dir():
            n = sum(
                1
                for f in fixtures.rglob("*")
                if f.is_file() and f.suffix in (".json", ".hex", ".bin", ".raw")
            )
            if n:
                counts[crate.name] = n
    report["fixture_counts"] = counts
    report["fixture_total"] = sum(counts.values())


def locked_versions():
    """name -> set of versions pinned in Cargo.lock."""
    lock = REPO / "Cargo.lock"
    versions = {}
    if not lock.is_file():
        return versions
    name = None
    for line in lock.read_text(encoding="utf-8").splitlines():
        m = re.match(r'name = "([^"]+)"', line)
        if m:
            name = m.group(1)
            continue
        m = re.match(r'version = "([^"]+)"', line)
        if m and name:
            versions.setdefault(name, set()).add(m.group(1))
            name = None
    return versions


def check_upstream_updates(report):
    """Upstream drift signal: are our differential oracles outdated?

    A new upstream release is adversarial-testing work: bump the dev-dep and
    re-run the differential suite — disagreements with the new version are
    exactly the findings this project exists to produce. Network-dependent,
    so informational (not ratcheted); skipped silently when offline.
    """
    import urllib.error
    import urllib.request

    locked = locked_versions()
    outdated, checked = [], 0
    for lib in UPSTREAM_LIBS:
        if lib not in locked:
            continue
        url = f"https://crates.io/api/v1/crates/{lib}"
        req = urllib.request.Request(
            url, headers={"User-Agent": "universal-blockchain-decoder health_report"}
        )
        try:
            with urllib.request.urlopen(req, timeout=8) as resp:
                data = json.load(resp)
        except (urllib.error.URLError, OSError, ValueError):
            report["upstream_update_check"] = "skipped (crates.io unreachable)"
            return
        checked += 1
        latest = data.get("crate", {}).get("max_stable_version")
        if latest and latest not in locked[lib]:
            ours = ", ".join(sorted(locked[lib]))
            outdated.append(f"{lib}: locked {ours} -> latest {latest}")
    report["upstream_update_check"] = f"ok ({checked} crates checked)"
    report["upstream_outdated"] = sorted(outdated)


def check_build(report):
    ok, out = run(["cargo", "check", "--workspace", "--all-targets"], timeout=3600)
    report["workspace_builds"] = ok
    if not ok:
        errors = [line for line in out.splitlines() if line.startswith("error")]
        report["build_errors"] = errors[:30]
    return ok


def check_tests(report):
    cmd = ["cargo", "test", "-q"]
    for c in REFERENCE_CRATES:
        cmd += ["-p", c]
    ok, out = run(cmd, timeout=3600)
    report["reference_tests_pass"] = ok
    if not ok:
        report["reference_test_failures"] = [
            line for line in out.splitlines() if "FAILED" in line or line.startswith("error")
        ][:30]
    return ok


# Ratchets: metric name -> direction ("max" = must not increase,
# "min" = must not decrease). The loop may only move these one way.
RATCHETS = {
    "core_loc_non_vendored": "max",
    "raw_bytes_storing_count": "max",
    "dead_validation_deps_count": "max",
    "string_fields_in_canonical": "max",
    "differential_decoders_count": "min",
    "fixture_total": "min",
}


def apply_ratchet(report, update: bool, accept_regression: str | None = None):
    """Compare metrics against one-way baselines.

    `accept_regression` (a mandatory human-readable reason) rebaselines any
    regressed metrics instead of failing. This exists for exactly one
    situation: merging upstream work (e.g. main) that legitimately moves a
    metric the wrong way. The acceptance is recorded in loop/ratchet.json
    under "accepted_regressions" as an audit trail - the regression stays
    visible, it does not get laundered into a normal baseline update.
    """
    ratchet = json.loads(RATCHET_PATH.read_text()) if RATCHET_PATH.is_file() else {}
    regressions, improved = [], {}
    for metric, direction in RATCHETS.items():
        current = report.get(metric)
        baseline = ratchet.get(metric)
        if current is None:
            continue
        if baseline is None:
            improved[metric] = current
            continue
        if direction == "max":
            if current > baseline:
                regressions.append(f"{metric}: {baseline} -> {current} (must not increase)")
            elif current < baseline:
                improved[metric] = current
        else:
            if current < baseline:
                regressions.append(f"{metric}: {baseline} -> {current} (must not decrease)")
            elif current > baseline:
                improved[metric] = current
    accepted = []
    if regressions and accept_regression:
        import datetime

        for metric, direction in RATCHETS.items():
            current, baseline = report.get(metric), ratchet.get(metric)
            if current is None or baseline is None:
                continue
            regressed = current > baseline if direction == "max" else current < baseline
            if regressed:
                ratchet[metric] = current
                accepted.append(
                    {
                        "metric": metric,
                        "from": baseline,
                        "to": current,
                        "reason": accept_regression,
                        "accepted_at": datetime.date.today().isoformat(),
                    }
                )
        ratchet.setdefault("accepted_regressions", []).extend(accepted)
        regressions = []
    if update and (improved or accepted):
        RATCHET_PATH.parent.mkdir(parents=True, exist_ok=True)
        ratchet.update(improved)
        RATCHET_PATH.write_text(json.dumps(ratchet, indent=2, sort_keys=True) + "\n")
    report["ratchet_regressions"] = regressions
    report["ratchet_improvements"] = improved
    if accepted:
        report["ratchet_accepted_regressions"] = accepted
    return regressions


def summarize(report):
    def flag(ok):
        return "PASS" if ok else "FAIL"

    lines = ["# Health report (measured)", ""]
    lines.append(f"workspace resolves: {flag(report.get('workspace_resolves'))}")
    if "workspace_builds" in report:
        lines.append(f"workspace builds (all targets): {flag(report['workspace_builds'])}")
    if "reference_tests_pass" in report:
        lines.append(f"reference crate tests: {flag(report['reference_tests_pass'])}")
    lines.append(
        f"core LOC (non-vendored): {report['core_loc_non_vendored']}"
        f" (budget {report['core_loc_budget']})"
    )
    lines.append(
        f"decoders storing raw bytes (vacuous roundtrip): "
        f"{report['raw_bytes_storing_count']} {report['raw_bytes_storing_decoders']}"
    )
    lines.append(
        f"dead 'validation' dev-deps: {report['dead_validation_deps_count']} "
        f"{report['dead_validation_deps']}"
    )
    lines.append(
        f"decoders with real differential tests: {report['differential_decoders_count']} "
        f"{report['differential_test_deps']}"
    )
    lines.append(f"String/JSON fields in canonical form: {report['string_fields_in_canonical']}")
    lines.append(f"fixture files (per-crate total): {report['fixture_total']}")
    lines.append(f"crates with fuzz targets: {report['crates_with_fuzz_targets']}")
    lines.append(f"upstream update check: {report.get('upstream_update_check', 'not run')}")
    for entry in report.get("upstream_outdated", []):
        lines.append(f"  OUTDATED ORACLE: {entry}")
    if report["ratchet_regressions"]:
        lines.append("")
        lines.append("RATCHET REGRESSIONS:")
        lines.extend(f"  - {r}" for r in report["ratchet_regressions"])
    if report.get("ratchet_improvements"):
        lines.append("")
        lines.append(f"ratchet improvements recorded: {report['ratchet_improvements']}")
    for acc in report.get("ratchet_accepted_regressions", []):
        lines.append(
            f"ACCEPTED REGRESSION {acc['metric']}: {acc['from']} -> {acc['to']} ({acc['reason']})"
        )
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--build", action="store_true", help="run cargo check --workspace")
    ap.add_argument("--test", action="store_true", help="run cargo test on reference crates")
    ap.add_argument("--offline", action="store_true", help="skip the crates.io freshness check")
    ap.add_argument("--update-ratchet", action="store_true")
    ap.add_argument(
        "--accept-regression",
        metavar="REASON",
        help="rebaseline regressed metrics with an audited reason "
        "(ONLY for merges of upstream work; implies --update-ratchet)",
    )
    args = ap.parse_args()
    if args.accept_regression:
        args.update_ratchet = True

    report = {}
    hard_fail = not check_resolve(report)
    check_core_loc(report)
    check_raw_bytes_storage(report)
    check_dead_validation_deps(report)
    check_json_in_canonical(report)
    check_fuzz_targets(report)
    check_fixtures(report)
    if not args.offline:
        check_upstream_updates(report)
    if args.build and not hard_fail:
        hard_fail |= not check_build(report)
    if args.test and not hard_fail:
        hard_fail |= not check_tests(report)

    regressions = apply_ratchet(
        report, update=args.update_ratchet, accept_regression=args.accept_regression
    )

    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(summarize(report))
    print(f"\nfull report: {REPORT_PATH.relative_to(REPO)}")

    sys.exit(1 if (hard_fail or regressions) else 0)


if __name__ == "__main__":
    main()
