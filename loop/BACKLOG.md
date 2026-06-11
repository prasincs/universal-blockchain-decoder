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

- [x] **Ethereum vs alloy** — added `alloy-consensus`/`alloy-eips`/
  `alloy-primitives` (1.8.3, pinned by Cargo.lock) as dev-deps; the old
  "version conflicts" no longer reproduce.
  `tests/alloy_differential.rs` compares type, chain_id, nonce, gas fields,
  to, value, input, access list, v/r/s parity, tx hash, and recovered sender.
  Verify: `cargo test -p decoder-ethereum --test alloy_differential`.
  (Done 2026-06; `differential_decoders_count` 2 -> 3.)
  **Findings produced**: 2 of 6 "real" fixtures are defective —
  `eth_eip2930.hex` is an unsigned 8-field payload, `eth_erc20_transfer.hex`
  has corrupt RLP (declares 176 payload bytes, contains 175). Both decoders
  agree on rejection; the tests document this. Replacement items below.
- [ ] **Solana vs solana-transaction-status** (dep declared, unused).
  Verify: `cargo test -p decoder-solana` includes a differential test file.
- [ ] **Cardano vs pallas** (3 deps declared, unused).
- [ ] **TON vs tonlib-core** (dep declared, unused).
- [ ] **BNB vs alloy** (deps declared, unused) — or delete the deps if the
  EVM differential test covers it.
- [ ] **Policy**: dead `UPSTREAM_LIBS` dev-deps for chains nobody is testing
  get DELETED, not kept "for later" — declared-but-unused deps carry yank
  risk with zero value (this already broke the build once).

## P1 — corpus of KNOWN on-chain transactions

The differential suite is only as strong as its corpus, and the corpus rotted
(two defective "real" fixtures, above). `scripts/loop/fetch_corpus.py` fetches
transactions by txid and writes self-certifying fixtures: the txid is
recomputed locally from the raw bytes before writing, and stored in the
sidecar so anyone can re-verify against a block explorer. Requires network
egress to an RPC/Esplora endpoint (blocked in the current sandbox; run from a
dev machine, commit the fixtures, verification re-runs offline in tests).

- [x] **Fix `parse_eip4844` field layout** — proper 14-field parse with
  `max_fee_per_blob_gas` + `blob_versioned_hashes` (new typed fields on
  `EthereumTransaction`), mandatory `to`. Fixing it exposed a SECOND 4844
  bug: `signing_hash()` used type byte 0x02 without blob fields, so sender
  recovery was wrong for every type-3 tx — also fixed. Oracle:
  `differential_eip4844_alloy_generated` builds a signed blob tx with
  alloy's encoder and asserts field-level agreement incl. tx hash and
  recovered sender. (Done 2026-06. Mainnet type-3 fixture still open below —
  fetching needs RPC egress this sandbox doesn't have.)
- [ ] **Replace `eth_erc20_transfer.hex`** with a verified mainnet ERC-20
  transfer: `fetch_corpus.py ethereum 0x<txid> --name eth_erc20_transfer_v2`,
  flip its test from `assert_both_reject` to `assert_agreement`.
- [ ] **Replace `eth_eip2930.hex`** with a real SIGNED mainnet EIP-2930 tx
  (they are rare; any tx with type 0x1 works), same procedure.
- [ ] **Add EIP-4844 blob tx fixture** (type 0x3, the canonical-bytes form
  without sidecar blobs as returned by eth_getRawTransactionByHash).
- [ ] **Bitcoin corpus depth**: add a Taproot key-path spend, a Taproot
  script-path spend, and a large multi-input SegWit tx via
  `fetch_corpus.py bitcoin <txid>`; wire into the existing differential test.
- [ ] **Corpus integrity test**: a test per decoder that recomputes each
  fixture's txid from its raw bytes and compares against the sidecar's
  `txid` field, so corpus corruption fails offline CI (this is what would
  have caught `eth_erc20_transfer.hex` years earlier).

## P1 — upstream dependency updates as a signal

The health report now queries crates.io for newer stable versions of every
locked upstream oracle (`upstream_outdated` in `loop/report.json`).

- [ ] **Policy**: when `upstream_outdated` lists a library that has
  differential tests, bumping it and re-running the suite IS a backlog item
  (treat each entry as auto-generated work). Disagreements after a bump are
  findings: minimal repro fixture + backlog entry before deciding which side
  is wrong.
- [ ] *(auto-generated 2026-06)* **Bump alloy 1.8.3 -> 2.x** in
  decoder-ethereum dev-deps; re-run `alloy_differential`.
- [ ] *(auto-generated 2026-06)* **Bump rust-bitcoin 0.31 -> 0.32** in
  decoder-bitcoin dev-deps; re-run `bitcoin_core_vectors`.
- [ ] **Re-pin cadence**: `cargo update` of the locked graph on a schedule
  (e.g. monthly), gated by the full test suite + health report, so the
  committed Cargo.lock doesn't fossilize.

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
- [ ] **TON `real_transactions` integration tests fail on main** — 3 of 5
  tests in `crates/decoder-ton/tests/real_transactions.rs` panic (e.g.
  `test_real_ton_state_init` at :86). Nobody noticed because CI's
  integration-test job only runs `cargo test -p universal-decoder-core
  --tests` — decoder crates' integration tests are NOT in CI (same gap that
  let the fuzz targets rot). Fix the TON tests AND extend CI to run
  `cargo test --workspace` (with the known-failing autonomous-executor item
  resolved or that tool retired first).
  Verify: `cargo test -p decoder-ton --test real_transactions`.
- [ ] **Workspace fails clippy on current stable (1.94)** — pre-existing
  `unnecessary_unwrap` / `large_enum_variant` errors in `decoder-optimism`
  (src/types.rs:397-403, enum at :13) and `decoder-evm`
  (src/registry.rs:177-178) fire under `-D warnings` with clippy 1.94 but
  evidently not in CI's toolchain. Fix the lints (don't allow-list them) and
  pin/refresh the CI toolchain so local and CI clippy agree.
  Verify: `cargo clippy -p decoder-optimism -p decoder-evm --all-targets -- -D warnings`.
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
  favor of the `improve-loop` skill. Its `roi::tests::test_parse_time_estimate`
  is failing on main (expects 60.0, gets 120.0 — the ROADMAP prose it parses
  drifted again, which is Assumption 5 in miniature); fix or retire with it.
- [ ] **Track TxIR expressiveness honestly**: add per-chain "% of fixture
  data landing in `Operation::Generic` / `extra`" to the health report and
  ratchet it downward. New-chain additions stay frozen until reference chains
  pass differential + true-encoding gates (Assumption 7).
- [ ] **Reconcile docs with measurements**: ROADMAP/README claims that
  contradict `loop/report.json` (e.g. "production-ready", "validation against
  upstream") get rewritten to cite measured status.
