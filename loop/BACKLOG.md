# Improvement Backlog (measured, prioritized)

Work items for the self-improvement loop. Every item was derived from a
**measured** finding (see `loop/report.json` and `docs/ASSUMPTIONS_REVIEW.md`),
not from self-reported status. Each item has acceptance criteria expressed as
a command whose output must change — if you can't verify it mechanically, it
doesn't belong here.

Protocol for the loop: take the topmost UNBLOCKED item, do it, verify, update
this file and the ratchet, commit. One item per iteration. See
`docs/SELF_IMPROVEMENT_LOOP.md`.

Status: `[ ]` open · `[x]` done · `[~]` in progress

---

## P0 — the loop cannot run without these

- [x] **Workspace unresolvable: yanked `algonaut_client 0.4.x`**
  Removed unused `algonaut*` dev-deps from `decoder-algorand`.
  Verify: `cargo metadata --no-deps` exits 0. (Done 2026-06.)

- [x] **No committed `Cargo.lock`**
  Every build re-resolved against live crates.io; one upstream yank broke the
  workspace. Lockfile now committed.
  Verify: `git ls-files Cargo.lock` non-empty. (Done 2026-06.)

- [x] **No measured health signal**
  Added `scripts/loop/health_report.py` + `loop/ratchet.json`.
  Verify: `python3 scripts/loop/health_report.py` exits 0. (Done 2026-06.)

## P1 — make the founding premise true (differential testing)

One item per chain. Pattern to copy:
`crates/decoder-bitcoin/tests/bitcoin_core_vectors.rs`.
Each test decodes real mainnet fixtures with BOTH our decoder and the
upstream library, and asserts field-level agreement (not just "both parsed").
Each closed item must raise `differential_decoders_count` and reduce
`dead_validation_deps_count` in the health report.

- [ ] **Ethereum vs alloy** — re-add `alloy-primitives`/`alloy-rlp` as
  dev-deps (they were commented out for "version conflicts"; resolve them or
  pin), differential-test legacy/EIP-1559/EIP-2930/EIP-4844 fixtures:
  nonce, gas fields, to, value, input, access list, v/r/s, tx hash.
  Verify: `cargo test -p decoder-ethereum --test alloy_differential`.
- [ ] **Solana vs solana-transaction-status** (dep declared, unused).
  Verify: `cargo test -p decoder-solana` includes a differential test file.
- [ ] **Cardano vs pallas** (3 deps declared, unused).
- [ ] **TON vs tonlib-core** (dep declared, unused).
- [ ] **BNB vs alloy** (deps declared, unused) — or delete the deps if the
  EVM differential test covers it.
- [ ] **Policy**: dead `UPSTREAM_LIBS` dev-deps for chains nobody is testing
  get DELETED, not kept "for later" — declared-but-unused deps carry yank
  risk with zero value (this already broke the build once).

## P1 — canonical form must actually be canonical

- [ ] **Remove JSON from the hashed path** (`string_fields_in_canonical: 5`).
  Replace `CanonicalTxMetadata.extra: String`,
  `CanonicalSignature.metadata: Option<String>`,
  `CanonicalGenericOperation.{op_type, metadata}` with typed Borsh data:
  per-family extra enum or `BTreeMap<String, Vec<u8>>`. Also change
  `size: usize` to `u64`.
  Verify: ratchet metric `string_fields_in_canonical` drops; add a test that
  two TxIRs built with differently-ordered extra data hash identically.
  Note: this is a canonical-format break — bump TxIR version `V`.

## P2 — make the roundtrip property non-vacuous

`raw_bytes_storing_count` is 24/39. Execute ENFORCE_TRUE_ENCODING.md instead
of re-litigating it. Reference decoders first:

- [ ] **Add a mutation property test helper to decoder-test-utils**: decode →
  perturb one semantic field → re-encode → bytes MUST differ. This makes
  stored-bytes cheating fail dynamically; apply to reference decoders first.
- [ ] **Bitcoin true re-encoding** — reconstruct legacy/SegWit/Taproot bytes
  from parsed fields; delete the `raw_bytes` field.
  Verify: `raw_bytes_storing_count` drops to 23; all 123 Bitcoin Core vectors
  still roundtrip; mutation test passes.
- [ ] **Ethereum true re-encoding** — RLP reconstruction reportedly exists;
  remove the stored field, make `to_bytes()` use it unconditionally.
- [ ] **Solana true re-encoding** — rebuild message + signatures layout.
- [ ] Remaining 21 decoders: one item each, generated from
  `loop/report.json:raw_bytes_storing_decoders` as the list shrinks.

## P2 — restore rotted infrastructure

- [ ] **Fuzz targets don't compile** (Ethereum, EVM, core — API drift, see
  FUZZING_RESULTS.md). Fix them, then add a CI job that `cargo check`s every
  `crates/*/fuzz` so they can't rot silently again.
  Verify: `cd crates/decoder-ethereum/fuzz && cargo check`.
- [ ] **Run the health report in CI** — add a job calling
  `python3 scripts/loop/health_report.py` (static mode) on every PR; ratchet
  regressions fail the build.
- [ ] **Ethereum sender recovery** — `from` is a zeroed placeholder in TxIR;
  `k256` is already a dependency. Implement ECDSA recovery from (v, r, s).
  Verify: differential test asserts recovered sender == alloy's.

## P3 — shrink the TCB and the dead structure

- [ ] **Delete the `Canonical*` mirror hierarchy** (~650 LOC): drop the
  phantom lifetime from `TxIR` (it owns all its data; the lifetime prevents
  nothing and forces the mirror), derive Borsh directly.
  Verify: `core_loc_non_vendored` drops by ≥500; canonical hashes of existing
  fixtures preserved or version-bumped deliberately.
  Blocked by: "Remove JSON from the hashed path" (do one format break, once).
- [ ] **Root `tests/fixtures/` is empty scaffolding** (.gitkeep + READMEs).
  Either make it the shared cross-decoder corpus the docs describe, or delete
  it and point docs at per-crate fixtures.
- [ ] **Repoint `tools/autonomous-executor`** at `loop/report.json` +
  `loop/BACKLOG.md` instead of parsing ROADMAP.md prose ROI, or retire it in
  favor of the `improve-loop` skill.
- [ ] **Track TxIR expressiveness honestly**: add per-chain "% of fixture
  data landing in `Operation::Generic` / `extra`" to the health report and
  ratchet it downward. New-chain additions stay frozen until reference chains
  pass differential + true-encoding gates (Assumption 7).
- [ ] **Reconcile docs with measurements**: ROADMAP/README claims that
  contradict `loop/report.json` (e.g. "production-ready", "validation against
  upstream") get rewritten to cite measured status.
