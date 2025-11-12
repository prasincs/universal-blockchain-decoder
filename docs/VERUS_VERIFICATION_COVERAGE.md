# Verus Formal Verification & Coverage Tracking: Complete Guide

## Executive Summary

This document provides a comprehensive guide to using **Verus** for formal verification of the Universal Blockchain Decoder and implementing **verification coverage tracking and display**.

**Key Findings**:
- ✅ Verus supports machine-readable output (`--output-json`) for CI/CD integration
- ✅ Verification coverage can be tracked via proof obligations and SMT statistics
- ✅ No built-in coverage dashboard (manual implementation required)
- ✅ Best practices from Asterinas, Microsoft, Amazon projects available

---

## Table of Contents

1. [What is Verus?](#what-is-verus)
2. [How Verus Works](#how-verus-works)
3. [Verification Coverage: Concepts](#verification-coverage-concepts)
4. [Tracking Verification Coverage](#tracking-verification-coverage)
5. [Displaying Coverage](#displaying-coverage)
6. [Implementation Strategy](#implementation-strategy)
7. [CI/CD Integration](#cicd-integration)
8. [Real-World Examples](#real-world-examples)

---

## What is Verus?

**Verus** is an **SMT-based formal verification tool for Rust** developed by researchers from Carnegie Mellon University, Microsoft Research, and MPI-SWS.

### Core Capabilities

```
┌─────────────────────────────────────────────────────┐
│              Verus Architecture                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Rust Code + Specifications (requires/ensures)     │
│              ↓                                      │
│  Verus Compiler (Extended Rust syntax)             │
│              ↓                                      │
│  Verification Conditions (VCs)                     │
│              ↓                                      │
│  Z3 SMT Solver (Automated Theorem Prover)         │
│              ↓                                      │
│  ✅ Verified   OR   ❌ Verification Error          │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Key Features

1. **Native Rust Syntax**: Write specifications in Rust (no new language to learn)
2. **Linear Ghost Types**: Leverage Rust's ownership for proof reasoning
3. **Automated Proving**: Z3 SMT solver handles proofs automatically
4. **Systems-Oriented**: Designed for low-level systems code (pointers, concurrency)
5. **Production-Ready**: Used at Microsoft, Amazon, and in OSDI 2024 papers

### What Makes Verus Special

Unlike traditional theorem provers (Coq, Isabelle), Verus:
- ✅ Verifies **actual executable Rust code** (not extracted models)
- ✅ **Zero runtime overhead** (verification is compile-time only)
- ✅ Supports **unsafe Rust** (pointers, concurrency)
- ✅ Leverages **Rust's type system** (ownership, lifetimes)
- ✅ **Automated proofs** (manual proofs only when needed)

---

## How Verus Works

### 1. Specification Language

Verus extends Rust with **specification constructs**:

```rust
verus! {

// Executable function with formal specification
pub fn divide(a: i32, b: i32) -> (result: i32)
    requires b != 0,                    // Precondition
    ensures result * b == a,            // Postcondition
{
    a / b
}

// Specification function (ghost code, compile-time only)
spec fn is_sorted(v: &Vec<i32>) -> bool {
    forall|i: int, j: int|
        0 <= i < j < v.len() ==> v[i] <= v[j]
}

// Proof function (ghost code, proves properties)
proof fn sorted_implies_first_is_min(v: &Vec<i32>)
    requires
        v.len() > 0,
        is_sorted(v),
    ensures
        forall|i: int| 0 <= i < v.len() ==> v[0] <= v[i]
{
    // Proof body (can use assert, lemmas)
}

} // verus!
```

### 2. Verification Modes

Verus has three function modes:

| Mode | Purpose | Compiles to Binary | Runs Z3 Verification |
|------|---------|-------------------|---------------------|
| `exec` | Executable code | ✅ Yes | ✅ Yes |
| `spec` | Specifications | ❌ No (ghost) | ❌ No (axiomatized) |
| `proof` | Proofs | ❌ No (ghost) | ✅ Yes |

**Example**:
```rust
verus! {

// exec: Real code that runs in production
fn factorial(n: u64) -> (result: u64)
    requires n <= 20,  // Prevent overflow
    ensures result > 0,
{
    if n == 0 { 1 } else { n * factorial(n - 1) }
}

// spec: Mathematical definition (ghost code)
spec fn factorial_spec(n: nat) -> nat
    decreases n
{
    if n == 0 { 1 } else { n * factorial_spec(n - 1) }
}

// proof: Prove factorial matches spec
proof fn factorial_correct(n: u64)
    requires n <= 20,
    ensures factorial(n) == factorial_spec(n as nat),
{
    // Proof by induction (Z3 often auto-proves)
}

} // verus!
```

### 3. The Verification Pipeline

```
Step 1: Write Code + Specs
   ↓
┌────────────────────────────────────────┐
│  fn decode(bytes: &[u8]) -> Result    │
│    requires bytes.len() >= 4          │
│    ensures result.is_ok() ==> ...     │
└────────────────────────────────────────┘
   ↓
Step 2: Verus Generates Verification Conditions (VCs)
   ↓
┌────────────────────────────────────────┐
│  VC1: Array access is in bounds       │
│  VC2: No integer overflow              │
│  VC3: Postcondition holds              │
│  VC4: Loop invariant preserved         │
└────────────────────────────────────────┘
   ↓
Step 3: Z3 SMT Solver Attempts to Prove Each VC
   ↓
┌────────────────────────────────────────┐
│  Z3 Output:                            │
│    VC1: ✅ Verified                    │
│    VC2: ✅ Verified                    │
│    VC3: ❌ Failed (counterexample)     │
│    VC4: ✅ Verified                    │
└────────────────────────────────────────┘
   ↓
Step 4: Report Results to User
   ↓
✅ Verification succeeded (all VCs proved)
   OR
❌ Verification failed (show failing VC + error)
```

### 4. Proof Obligations (VCs)

Every function generates **verification conditions** (proof obligations):

**Example: Bounds Checking**
```rust
verus! {

fn get_u32(bytes: &[u8], offset: usize) -> (result: u32)
    requires offset + 4 <= bytes.len(),  // Precondition
    ensures result <= u32::MAX,           // Always true (tautology)
{
    // Verus generates VCs:
    // VC1: offset < bytes.len()     (from bytes[offset])
    // VC2: offset+1 < bytes.len()   (from bytes[offset+1])
    // VC3: offset+2 < bytes.len()   (from bytes[offset+2])
    // VC4: offset+3 < bytes.len()   (from bytes[offset+3])
    //
    // All VCs discharged by precondition: offset + 4 <= bytes.len()

    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

} // verus!
```

**Example: Overflow Checking**
```rust
verus! {

fn checked_add(a: u64, b: u64) -> (result: Option<u64>)
    ensures
        result.is_some() ==> result.unwrap() == a + b,  // Correct value
        result.is_none() ==> a + b > u64::MAX,           // Overflow case
{
    // Verus generates VCs:
    // VC1: If a + b <= u64::MAX, then Some(a+b) satisfies postcondition
    // VC2: If a + b > u64::MAX, then None satisfies postcondition
    //
    // Z3 proves both VCs automatically

    a.checked_add(b)
}

} // verus!
```

---

## Verification Coverage: Concepts

### What is Verification Coverage?

**Traditional Code Coverage** (testing):
```
Lines Executed / Total Lines = 85%
```

**Verification Coverage** (formal methods):
```
Verified Properties / Total Safety Properties = 60%
```

### Types of Verification Coverage

#### 1. **Function Coverage**

**Metric**: Percentage of functions with formal specifications

```rust
// ✅ Verified (has spec)
fn decode(bytes: &[u8]) -> Result<Tx>
    requires bytes.len() >= 4,
    ensures ...
{ ... }

// ❌ Unverified (no spec)
fn helper_function(x: i32) -> i32 {
    x + 1
}
```

**Coverage**: `1 verified / 2 total = 50%`

#### 2. **Property Coverage**

**Metric**: Critical safety properties proven

For blockchain decoder, track:
- ✅ Panic-freedom (no unwrap, no out-of-bounds)
- ✅ Overflow safety (checked arithmetic)
- ✅ Canonicalization injectivity
- ❌ Signature verification correctness (not yet proven)

**Coverage**: `3 properties / 4 properties = 75%`

#### 3. **Line Coverage** (Least Useful)

**Metric**: Lines of code executed during verification

⚠️ **Not the same as test coverage!** Verus proves **all possible executions**, not just tested paths.

#### 4. **Proof Obligation Coverage** (Most Granular)

**Metric**: Verification conditions (VCs) proven

```
Total VCs generated: 247
VCs proven by Z3:    235
VCs assumed (admit): 12
-------------------------------
Coverage: 95.1%
```

### Coverage Metrics Comparison

| Metric | Granularity | Usefulness | Ease of Tracking |
|--------|-------------|------------|------------------|
| Function Coverage | Coarse | ⭐⭐⭐ Medium | ✅ Easy (count functions with specs) |
| Property Coverage | Medium | ⭐⭐⭐⭐⭐ High | ⚠️ Manual (document critical properties) |
| VC Coverage | Fine | ⭐⭐⭐⭐ High | ✅ Easy (parse Verus output) |
| Line Coverage | Very Fine | ⭐ Low | ❌ Hard (not directly supported) |

**Recommendation**: Focus on **Property Coverage** (business logic) + **VC Coverage** (technical metric)

---

## Tracking Verification Coverage

### 1. Verus Output Formats

Verus provides **machine-readable output** for tracking:

#### Basic Output (Human-Readable)

```bash
$ verus crates/universal-decoder-core/src/ir.rs

verification results:: 12 verified, 0 errors
```

#### JSON Output (Machine-Readable)

```bash
$ verus --output-json crates/universal-decoder-core/src/ir.rs
```

**Example JSON output**:
```json
{
  "verification_results": [
    {
      "function": "Amount::checked_add",
      "status": "verified",
      "time_ms": 42,
      "smt_rlimit": 1250,
      "vcs_count": 3,
      "vcs_verified": 3
    },
    {
      "function": "BitcoinDecoder::decode",
      "status": "failed",
      "time_ms": 128,
      "smt_rlimit": 5000,
      "vcs_count": 15,
      "vcs_verified": 14,
      "error": "postcondition might not hold"
    }
  ],
  "summary": {
    "total_functions": 2,
    "verified": 1,
    "failed": 1,
    "total_vcs": 18,
    "vcs_verified": 17
  }
}
```

#### Performance Profiling

```bash
# Detailed timing breakdown
$ verus --time crates/universal-decoder-core/src/ir.rs

# Expanded timing (per VC)
$ verus --time-expanded crates/universal-decoder-core/src/ir.rs
```

**Output**:
```
Function: Amount::checked_add
  VC1 (ensures clause 1): 12ms (rlimit: 400)
  VC2 (ensures clause 2): 8ms (rlimit: 300)
  VC3 (overflow check): 22ms (rlimit: 550)
  Total: 42ms
```

### 2. Tracking Verification Targets

**Approach**: Create a **verification manifest** documenting critical properties.

**File**: `docs/VERIFICATION_TARGETS.md`

```markdown
# Verification Targets for Universal Blockchain Decoder

## Phase 1: Core Library (universal-decoder-core)

### Target: VT-1 - Amount Arithmetic Safety

**Priority**: CRITICAL
**Status**: ✅ Verified
**Properties**:
- ✅ VT-1.1: checked_add overflow detection
- ✅ VT-1.2: checked_sub underflow detection
- ✅ VT-1.3: checked_mul overflow detection
- ❌ VT-1.4: Decimal conversion correctness (TODO)

**Files**: `crates/universal-decoder-core/src/ir.rs:120-180`
**VCs**: 12 / 12 proven
**Last Verified**: 2025-01-15

---

### Target: VT-2 - Canonicalization Injectivity

**Priority**: CRITICAL
**Status**: ⏳ In Progress
**Properties**:
- ✅ VT-2.1: Borsh encoding is deterministic
- ⏳ VT-2.2: decode(encode(x)) == x (in progress)
- ❌ VT-2.3: encode(x) == encode(y) ==> x == y (TODO)

**Files**: `crates/universal-decoder-core/src/traits.rs:45-90`
**VCs**: 8 / 15 proven
**Blocked By**: Borsh library verification
**Last Verified**: 2025-01-20

---

## Phase 2: Bitcoin Decoder (decoder-bitcoin)

### Target: VT-10 - Bitcoin Parsing Safety

**Priority**: HIGH
**Status**: ❌ Not Started
**Properties**:
- ❌ VT-10.1: Varint parsing never panics
- ❌ VT-10.2: Input parsing bounds-checked
- ❌ VT-10.3: Output value doesn't overflow

**Files**: `crates/decoder-bitcoin/src/parsing.rs`
**VCs**: 0 / 47 proven
**Last Verified**: N/A

---

## Summary

| Phase | Targets | Verified | In Progress | Todo | Coverage |
|-------|---------|----------|-------------|------|----------|
| Core  | 5       | 2        | 1           | 2    | 40%      |
| Bitcoin | 3     | 0        | 0           | 3    | 0%       |
| Ethereum | 3    | 0        | 0           | 3    | 0%       |
| **Total** | **11** | **2** | **1**     | **8** | **18%**  |

**Next Action**: Complete VT-2.2 (canonicalization roundtrip)
```

### 3. Automated Coverage Tracking Script

**File**: `tools/verus-coverage.sh`

```bash
#!/bin/bash
# Verus Verification Coverage Tracker

set -euo pipefail

VERUS_BIN="${VERUS_BIN:-verus}"
OUTPUT_DIR="target/verus-coverage"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$OUTPUT_DIR"

echo "==> Running Verus verification with coverage tracking..."

# Verify each crate and capture JSON output
for crate in crates/universal-decoder-core crates/decoder-bitcoin; do
    crate_name=$(basename "$crate")
    echo "  -> Verifying $crate_name..."

    $VERUS_BIN --output-json "$crate/src/lib.rs" \
        > "$OUTPUT_DIR/${crate_name}_${TIMESTAMP}.json" 2>&1 \
        || echo "    [WARN] Verification failed for $crate_name"
done

# Parse JSON results and generate coverage report
python3 tools/parse-verus-coverage.py \
    "$OUTPUT_DIR" \
    > "$OUTPUT_DIR/coverage_report_${TIMESTAMP}.md"

echo "==> Coverage report: $OUTPUT_DIR/coverage_report_${TIMESTAMP}.md"

# Generate summary
total_vcs=$(jq '[.summary.total_vcs] | add' "$OUTPUT_DIR"/*.json)
verified_vcs=$(jq '[.summary.vcs_verified] | add' "$OUTPUT_DIR"/*.json)
coverage=$(echo "scale=2; $verified_vcs * 100 / $total_vcs" | bc)

echo ""
echo "=== Verification Coverage Summary ==="
echo "Total VCs:     $total_vcs"
echo "Verified VCs:  $verified_vcs"
echo "Coverage:      ${coverage}%"
```

**File**: `tools/parse-verus-coverage.py`

```python
#!/usr/bin/env python3
"""Parse Verus JSON output and generate coverage report."""

import json
import sys
from pathlib import Path
from datetime import datetime

def parse_coverage(json_dir: Path):
    """Parse all JSON files and compute coverage."""

    total_functions = 0
    verified_functions = 0
    failed_functions = 0
    total_vcs = 0
    verified_vcs = 0

    crate_results = {}

    # Parse all JSON files
    for json_file in json_dir.glob("*.json"):
        try:
            with open(json_file) as f:
                data = json.load(f)

            crate_name = json_file.stem.split('_')[0]

            # Aggregate stats
            total_functions += data['summary']['total_functions']
            verified_functions += data['summary']['verified']
            failed_functions += data['summary']['failed']
            total_vcs += data['summary']['total_vcs']
            verified_vcs += data['summary']['vcs_verified']

            crate_results[crate_name] = data

        except Exception as e:
            print(f"Error parsing {json_file}: {e}", file=sys.stderr)

    # Generate Markdown report
    print("# Verus Verification Coverage Report")
    print(f"\n**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"\n## Summary\n")
    print(f"- **Total Functions**: {total_functions}")
    print(f"- **Verified**: {verified_functions} ({100*verified_functions//total_functions}%)")
    print(f"- **Failed**: {failed_functions}")
    print(f"- **Total VCs**: {total_vcs}")
    print(f"- **Verified VCs**: {verified_vcs} ({100*verified_vcs//total_vcs}%)")

    print(f"\n## Per-Crate Breakdown\n")
    print("| Crate | Functions | Verified | Failed | VCs | Coverage |")
    print("|-------|-----------|----------|--------|-----|----------|")

    for crate_name, data in crate_results.items():
        summary = data['summary']
        func_coverage = 100 * summary['verified'] // summary['total_functions']
        vc_coverage = 100 * summary['vcs_verified'] // summary['total_vcs']
        print(f"| {crate_name} | {summary['total_functions']} | "
              f"{summary['verified']} | {summary['failed']} | "
              f"{summary['total_vcs']} | {vc_coverage}% |")

    print(f"\n## Failed Verifications\n")

    for crate_name, data in crate_results.items():
        failures = [r for r in data['verification_results'] if r['status'] == 'failed']
        if failures:
            print(f"### {crate_name}\n")
            for failure in failures:
                print(f"- **{failure['function']}**: {failure['error']}")
                print(f"  - VCs: {failure['vcs_verified']}/{failure['vcs_count']}")
                print(f"  - Time: {failure['time_ms']}ms\n")

if __name__ == "__main__":
    json_dir = Path(sys.argv[1])
    parse_coverage(json_dir)
```

---

## Displaying Coverage

### 1. Coverage Badge (README.md)

**Manual Badge** (update after each run):

```markdown
[![Verification Coverage](https://img.shields.io/badge/verification-78%25-yellow)]()
```

**Automated Badge** (via GitHub Actions):

```yaml
# .github/workflows/verus-coverage.yml
name: Verus Coverage

on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Verus
        run: |
          git clone https://github.com/verus-lang/verus.git /tmp/verus
          cd /tmp/verus && ./tools/get-z3.sh
          echo "/tmp/verus/source/target-verus/release" >> $GITHUB_PATH

      - name: Run Verification
        run: ./tools/verus-coverage.sh

      - name: Extract Coverage
        id: coverage
        run: |
          COVERAGE=$(jq '[.summary.vcs_verified, .summary.total_vcs] |
                        .[0] * 100 / .[1]' target/verus-coverage/*.json |
                        head -1)
          echo "coverage=$COVERAGE" >> $GITHUB_OUTPUT

      - name: Create Coverage Badge
        uses: schneegans/dynamic-badges-action@v1.6.0
        with:
          auth: ${{ secrets.GIST_SECRET }}
          gistID: YOUR_GIST_ID
          filename: verus-coverage.json
          label: Verification
          message: ${{ steps.coverage.outputs.coverage }}%
          color: ${{ steps.coverage.outputs.coverage > 80 && 'green' ||
                    steps.coverage.outputs.coverage > 50 && 'yellow' || 'red' }}
```

### 2. Dashboard (HTML Report)

**File**: `tools/generate-dashboard.py`

```python
#!/usr/bin/env python3
"""Generate HTML dashboard for Verus verification coverage."""

import json
from pathlib import Path

HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <title>Verus Verification Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ display: flex; gap: 20px; margin-bottom: 30px; }}
        .card {{ border: 1px solid #ddd; padding: 20px; border-radius: 8px; }}
        .verified {{ color: green; }}
        .failed {{ color: red; }}
    </style>
</head>
<body>
    <h1>Verus Verification Dashboard</h1>

    <div class="summary">
        <div class="card">
            <h3>Overall Coverage</h3>
            <h1>{overall_coverage}%</h1>
            <p>{verified_vcs} / {total_vcs} VCs</p>
        </div>
        <div class="card">
            <h3>Functions</h3>
            <p class="verified">✅ Verified: {verified_functions}</p>
            <p class="failed">❌ Failed: {failed_functions}</p>
        </div>
    </div>

    <canvas id="coverageChart" width="400" height="200"></canvas>

    <script>
        const ctx = document.getElementById('coverageChart');
        new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: {crate_labels},
                datasets: [{{
                    label: 'Verification Coverage (%)',
                    data: {crate_coverage},
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                scales: {{ y: {{ beginAtZero: true, max: 100 }} }}
            }}
        }});
    </script>
</body>
</html>
"""

def generate_dashboard(json_dir: Path, output_file: Path):
    # ... parse JSON files ...
    # ... compute metrics ...

    html = HTML_TEMPLATE.format(
        overall_coverage=overall_coverage,
        verified_vcs=verified_vcs,
        total_vcs=total_vcs,
        verified_functions=verified_functions,
        failed_functions=failed_functions,
        crate_labels=json.dumps(crate_names),
        crate_coverage=json.dumps(crate_coverages)
    )

    output_file.write_text(html)

if __name__ == "__main__":
    generate_dashboard(Path("target/verus-coverage"),
                      Path("target/verus-coverage/dashboard.html"))
```

### 3. Terminal Dashboard (Live)

```bash
#!/bin/bash
# tools/verus-watch.sh - Live verification dashboard

watch -n 10 '
echo "=== Verus Verification Status ==="
echo ""
verus --output-json crates/*/src/lib.rs 2>&1 |
    jq -r ".summary |
           \"Functions: \(.verified)/\(.total_functions) verified\n\" +
           \"VCs: \(.vcs_verified)/\(.total_vcs) proven\n\" +
           \"Coverage: \((.vcs_verified * 100 / .total_vcs))%\""
echo ""
echo "Last updated: $(date)"
'
```

**Output**:
```
=== Verus Verification Status ===

Functions: 12/15 verified
VCs: 235/247 proven
Coverage: 95%

Last updated: 2025-01-15 14:32:10
```

---

## Implementation Strategy

### Phase 1: Foundation (Week 1-2)

**Goal**: Install Verus, verify first function, set up tracking

**Tasks**:
1. Install Verus toolchain
2. Verify simple function (Amount::checked_add)
3. Set up `VERIFICATION_TARGETS.md`
4. Create coverage tracking script

**Deliverable**: 1 verified function, coverage infrastructure

### Phase 2: Core Library (Week 3-6)

**Goal**: Verify critical properties in universal-decoder-core

**Targets**:
- VT-1: Amount arithmetic (overflow, underflow)
- VT-2: Canonicalization determinism
- VT-3: Error propagation safety

**Deliverable**: 5 verified functions, 40% core coverage

### Phase 3: Bitcoin Decoder (Week 7-12)

**Goal**: Verify Bitcoin transaction parsing

**Targets**:
- VT-10: Varint parsing safety
- VT-11: Input/output parsing bounds
- VT-12: Fee calculation overflow safety

**Deliverable**: 10 verified functions, Bitcoin decoder proven

### Phase 4: Dashboard & CI/CD (Week 13-14)

**Goal**: Automate coverage tracking and display

**Tasks**:
1. Set up GitHub Actions for Verus
2. Generate HTML dashboard
3. Add coverage badge to README
4. Document verification process

**Deliverable**: Automated verification on every commit

---

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/verus.yml`

```yaml
name: Formal Verification (Verus)

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  verify:
    name: Verus Verification
    runs-on: ubuntu-latest
    timeout-minutes: 60

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Cache Verus
        uses: actions/cache@v3
        with:
          path: |
            ~/.verus
            /tmp/verus
          key: ${{ runner.os }}-verus-${{ hashFiles('tools/install-verus.sh') }}

      - name: Install Verus
        run: |
          if [ ! -d "/tmp/verus" ]; then
            git clone https://github.com/verus-lang/verus.git /tmp/verus
            cd /tmp/verus
            ./tools/get-z3.sh
          fi
          echo "/tmp/verus/source/target-verus/release" >> $GITHUB_PATH

      - name: Verify Core Library
        run: |
          verus --output-json --time \
            crates/universal-decoder-core/src/lib.rs \
            > verus-core.json
        continue-on-error: true

      - name: Verify Bitcoin Decoder
        run: |
          verus --output-json --time \
            crates/decoder-bitcoin/src/lib.rs \
            > verus-bitcoin.json
        continue-on-error: true

      - name: Generate Coverage Report
        run: |
          python3 tools/parse-verus-coverage.py . \
            > verification-report.md

      - name: Upload Coverage Report
        uses: actions/upload-artifact@v3
        with:
          name: verus-coverage
          path: |
            verus-*.json
            verification-report.md

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('verification-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## Verus Verification Results\n\n' + report
            });

      - name: Check Coverage Threshold
        run: |
          COVERAGE=$(jq '[.summary.vcs_verified * 100 / .summary.total_vcs] |
                        add / length' verus-*.json)
          echo "Verification coverage: $COVERAGE%"

          # Fail if coverage drops below 80%
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "ERROR: Verification coverage below 80%"
            exit 1
          fi
```

### Pre-Commit Hook

**File**: `.git/hooks/pre-commit`

```bash
#!/bin/bash
# Verify modified Rust files before commit

MODIFIED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -z "$MODIFIED_FILES" ]; then
    exit 0
fi

echo "Running Verus verification on modified files..."

for file in $MODIFIED_FILES; do
    if verus --output-json "$file" > /dev/null 2>&1; then
        echo "  ✅ $file verified"
    else
        echo "  ❌ $file verification failed"
        echo ""
        echo "Run 'verus $file' to see errors"
        exit 1
    fi
done

echo "All modified files verified successfully!"
```

---

## Real-World Examples

### Example 1: Asterinas OS (Page Tables)

**Project**: Formally verified OS kernel page tables
**Approach**: 14 verification targets, 11 verified

```rust
// From asterinas/vostd
verus! {

/// Verification Target: fvt10-pt-cursor-navigation
/// Proves: Cursor navigation is memory-safe
proof fn cursor_navigation_safe(cursor: &mut PageTableCursor)
    requires
        cursor.is_valid(),
        cursor.current_level() > 0,
    ensures
        cursor.pop_level().is_ok(),
{
    // Proof that pop_level() never panics
    // and maintains page table invariants
}

} // verus!
```

**Coverage Tracking**: Manual tracking in README + verification target IDs

### Example 2: Microsoft Ironclad (Key-Value Store)

**Project**: Verified distributed key-value store
**Properties Proven**:
- Linearizability
- Crash consistency
- Byzantine fault tolerance

**Coverage**: ~15,000 lines of verified code

### Example 3: Amazon (Cryptography)

**Project**: Verified cryptographic primitives
**Properties**: Constant-time execution, correctness proofs

---

## Recommended Approach for Universal Blockchain Decoder

### 1. Start Small

**Week 1**: Verify `Amount::checked_add`

```rust
verus! {

impl Amount {
    pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
        requires self.decimals == other.decimals,
        ensures
            result.is_some() ==> {
                let sum = result.unwrap();
                sum.value == self.value + other.value &&
                sum.decimals == self.decimals
            },
            result.is_none() ==> self.value + other.value > u128::MAX,
    {
        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount {
                value: sum,
                decimals: self.decimals,
            }),
            None => None,
        }
    }
}

} // verus!
```

**Run**:
```bash
$ verus --output-json crates/universal-decoder-core/src/ir.rs
# Expect: 3 VCs proven
```

### 2. Build Coverage Infrastructure

**Week 2**: Set up tracking

1. Create `docs/VERIFICATION_TARGETS.md`
2. Write `tools/verus-coverage.sh`
3. Add GitHub Actions workflow
4. Generate first coverage report

### 3. Incremental Verification

**Months 2-4**: Verify critical paths

Priority order:
1. ✅ Amount arithmetic (high value, easy)
2. ✅ Borsh canonicalization (critical, medium)
3. ✅ Bitcoin varint parsing (medium value, medium)
4. ✅ Fee calculations (high value, medium)
5. ⏳ Full Bitcoin decoder (high value, hard)

### 4. Coverage Goals

| Milestone | Coverage Target | Timeline |
|-----------|----------------|----------|
| v0.1 | 20% (Amount + core) | Month 1 |
| v0.2 | 50% (Core + Bitcoin partial) | Month 3 |
| v0.3 | 75% (Core + Bitcoin full) | Month 6 |
| v1.0 | 90% (Core + 2 decoders) | Month 12 |

---

## Conclusion

### Key Takeaways

1. **Verus is Production-Ready**: Used at Microsoft, Amazon, in OSDI papers
2. **Coverage is Trackable**: `--output-json` + custom scripts
3. **No Built-in Dashboard**: Manual implementation required
4. **Start Small**: Verify critical functions first, expand incrementally
5. **Automate Early**: CI/CD integration from day 1

### Next Steps

1. ✅ Read this document
2. ⏭️ Install Verus (see `docs/FORMAL_VERIFICATION.md`)
3. ⏭️ Verify first function (`Amount::checked_add`)
4. ⏭️ Set up coverage tracking
5. ⏭️ Add to CI/CD pipeline

### Resources

- **Verus GitHub**: https://github.com/verus-lang/verus
- **Tutorial**: https://verus-lang.github.io/verus/guide/
- **Zulip Chat**: https://verus-lang.zulipchat.com/
- **Asterinas Example**: https://github.com/asterinas/vostd
- **Paper**: "Verus: A Practical Foundation for Systems Verification" (SOSP 2024)

---

**Last Updated**: 2025-01-15
**Version**: 1.0
**Maintainer**: Universal Blockchain Decoder Team
