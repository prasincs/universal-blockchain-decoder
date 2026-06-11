# Assumptions Review (2026-06)

An adversarial review of the project's own load-bearing assumptions, performed
against the code as it actually is — not as the docs describe it. Each section
states the assumption, what was measured, the verdict, and the improvement.
The measured findings here seeded `loop/BACKLOG.md` and the checks in
`scripts/loop/health_report.py`.

**Method note**: every claim below was verified by direct inspection (grep,
`cargo metadata`, line counts, reading the code), not by reading status docs.
That distinction matters: a parallel survey of this repo that trusted the
ROADMAP/README self-descriptions reported "core TCB target met", "Borsh only,
no JSON hashing", and "validation against upstream libraries" — all three are
false. Self-reported status drifts optimistic; only measured signals can drive
a self-improvement loop (see Assumption 5).

---

## Assumption 1: "Core TCB is < 3000 LOC" — VIOLATED, and partly self-inflicted

**Claimed**: CLAUDE.md and ROADMAP state the trusted core is under 3000 LOC.

**Measured**: `crates/universal-decoder-core/src` is **~4,961 LOC** excluding
the vendored hex crate (6,166 including it).

**Challenge**: the overage isn't organic growth — a large chunk is the
`CanonicalTxIR` mirror hierarchy in `canonical.rs` (~650 LOC of duplicated
structs plus `From` impls). That mirror exists only because `TxIR<'a, V>`
carries a `PhantomData<&'a [u8]>` lifetime that blocks deriving
`BorshSerialize` directly. But the phantom lifetime is ceremony: `TxIR` owns
all of its data (`Vec<u8>`, `String`, owned structs) and borrows nothing. The
lifetime was meant to enforce "decoders can't store raw bytes"
(ENFORCE_TRUE_ENCODING.md), but it doesn't achieve that — decoders store
`raw_bytes` in their own chain-specific types anyway (see Assumption 3).

So the core pays a permanent tax (double type hierarchy, conversion code,
clone-heavy `to_canonical()`, drift risk between the two hierarchies) for a
guarantee it does not actually deliver.

**Improvement**:
1. Drop the phantom lifetime from `TxIR`, derive Borsh directly, delete the
   `Canonical*` mirror. Cuts ~600+ LOC from the TCB and removes a whole class
   of mirror-drift bugs.
2. Keep the LOC budget but enforce it by ratchet (`loop/ratchet.json`):
   core LOC may never increase, and the backlog drives it back under budget.
   A budget nobody measures is a wish, not a constraint.

## Assumption 2: "Canonical serialization is Borsh; JSON is NEVER used for hashing" — VIOLATED BY DESIGN

**Claimed**: docs/CANONICAL_SERIALIZATION.md, CLAUDE.md ("NEVER use JSON for
hashing or signature verification").

**Measured**: the canonical, hashed representation embeds free-form JSON:
- `CanonicalTxMetadata.extra: String` — "JSON string for extra data"
  (`canonical.rs:76`)
- `CanonicalSignature.metadata: Option<String>` — JSON (`canonical.rs:90`)
- `CanonicalGenericOperation.metadata: String` — JSON (`canonical.rs:167`)

Per TXIR_DATA_COMPLETENESS.md, for EVM chains `extra` carries **signed
transaction fields**: nonce, gas limit, gas price, chain id, the entire
EIP-2930 access list. Borsh-encoding a JSON string is not canonical: key
order, whitespace, and number formatting are producer-dependent. Two decoders
(or two versions of one decoder) can emit semantically identical TxIRs with
different `canonical_hash()` values. The malleability scenario the docs warn
about is reproduced inside the canonical format itself.

Secondary issue: `CanonicalTxMetadata.size: usize` is in the canonical struct;
prefer an explicitly sized integer (`u64`) in anything canonical.

**Improvement**: promote the data that matters out of JSON into typed Borsh
fields. Concretely: a per-family typed extension enum
(`enum ChainExtra { Evm(EvmExtra), Utxo(UtxoExtra), ... , Opaque(Vec<u8>) }`)
or, minimally, a `BTreeMap<String, Vec<u8>>` (sorted keys, byte values) so
encoding is order-independent. `extra` as display-JSON can remain — but
outside the canonical/hashing path.

## Assumption 3: "Re-encoding is mandatory and verified: encode(decode(x)) == x" — TRUE BUT VACUOUS FOR SEVERAL DECODERS

**Claimed**: the roundtrip property is the project's central correctness
property; property tests enforce it.

**Measured**: multiple decoders implement `ChainEncoder::to_bytes()` as
`Ok(self.raw_bytes.clone())` — Bitcoin among them. The repo's own
ENFORCE_TRUE_ENCODING.md calls this pattern "CHEATING!" and proposes a
type-level fix, but the migration was never executed. With stored bytes, the
roundtrip property is satisfied by construction and **tests nothing about
decode correctness**: a decoder that parsed every field wrong would still
pass.

**Challenge**: the roundtrip property is the right *idea* but the wrong sole
oracle. Even with true re-encoding it only proves injectivity of the parse,
not semantic correctness (you can roundtrip bytes you've mislabeled).

**Improvement**, in order of value:
1. **Differential decoding** (see Assumption 4) is the real oracle for
   semantic correctness — compare parsed fields against an independent
   upstream implementation on real fixtures.
2. Execute the ENFORCE_TRUE_ENCODING migration for the reference decoders
   (Ethereum already reconstructs RLP; Bitcoin should reconstruct from
   fields).
3. Add a **mutation check** to property tests: decode, perturb one semantic
   field, re-encode — output bytes MUST change. This catches stored-bytes
   cheating dynamically, without trusting greps. (A decoder returning stored
   bytes fails immediately.)
4. Until (2) lands, the health report counts `raw_bytes`-storing decoders and
   ratchets the count downward.

## Assumption 4: "We adversarially test against upstream dependencies" — ~5% IMPLEMENTED

**Claimed**: dev-dependencies on rust-bitcoin, alloy, pallas (Cardano),
tonlib-core (TON), solana-transaction-status, algonaut are declared "for test
validation ONLY", implying differential testing.

**Measured**:
- Only **Bitcoin** has a real differential test
  (`crates/decoder-bitcoin/tests/bitcoin_core_vectors.rs` imports `bitcoin`).
- `pallas-*`, `tonlib-core`, `solana-transaction-status` are declared and
  **never imported by any test**.
- `alloy-*` for Ethereum is **commented out** ("version conflicts").
- `algonaut` was declared, never used — and when upstream yanked all
  `algonaut_client 0.4.x` releases, it made the **entire workspace
  unresolvable** (no `Cargo.lock` was committed, so resolution happened fresh
  every build). The unused dependency carried all of the supply-chain risk
  and none of the validation value.

**Challenge**: this is the project's founding premise, and it is the least
implemented part of the codebase. Worse, the half-measure (declared deps,
no tests) is strictly negative: compile time, audit surface, and a build
break.

**Improvement**:
1. A dev-dependency on an upstream chain library is only allowed if at least
   one test imports it — enforced by the `dead_validation_deps` check in the
   health report.
2. Differential tests live behind their own test files per crate so upstream
   breakage is quarantined to that crate, and `Cargo.lock` is committed
   (done) so yanks/new releases can't brick resolution.
3. The differential corpus, not the roadmap, becomes the loop's primary
   work-generator: every disagreement with upstream on a real fixture is an
   auto-generated backlog item.

## Assumption 5: "The roadmap can drive autonomous execution (ROI = parsed status emojis)" — UNSOUND SIGNAL

**Claimed**: `tools/autonomous-executor` selects tasks by parsing ROADMAP.md
status markers and priorities, computing ROI from self-reported completion
percentages. Its Anthropic API integration is itself still a placeholder.

**Measured drift between self-reported and actual state**:
- ROADMAP says fuzzing infrastructure is complete; FUZZING_RESULTS.md shows
  Ethereum/EVM/core fuzz targets **don't compile** (API drift).
- Decoders marked "production-ready" store raw bytes (vacuous roundtrip).
- "Validation against upstream" claimed; one chain actually does it.
- The workspace did not build at all at review time.

**Challenge**: a self-improvement loop that consumes self-reported status will
Goodhart itself — it optimizes the document, not the code. The executor would
happily report ROI on tasks whose "completion" numbers are fiction.

**Improvement**: invert the data flow. The loop's input is the **measured
health report** (`scripts/loop/health_report.py` → `loop/report.json`);
ROADMAP.md becomes narrative documentation, not machine-readable truth.
Ratchets (`loop/ratchet.json`) make regressions on measured metrics a hard
failure, so the loop can only move metrics in one direction. See
`docs/SELF_IMPROVEMENT_LOOP.md`.

## Assumption 6: "Supply chain is secured by minimal deps + vendoring" — PARTIAL

**Measured**:
- No `Cargo.lock` was committed (fixed in this change) — every build
  re-resolved against live crates.io, which is how a yanked dev-dependency
  took the workspace down.
- The *core* is genuinely minimal (serde, borsh, thiserror, sha2/sha3 +
  vendored hex) — this claim holds.
- The *workspace* transitively pulls 80+ external crates (tonlib-core,
  winterfell, pallas, starknet-crypto, reqwest, octocrab, git2, ...). The
  "minimal dependencies" story is true only for the core crate, and docs
  should say so precisely.

**Improvement**: commit the lockfile (done), scope the minimal-deps claim to
the core, and let the health report track the workspace dependency count so
growth is at least visible.

## Assumption 7: "Breadth proves the type system" — BACKWARDS

**Claimed (implicitly, by the 39-decoder roster)**: more chains = more
validation of the universal TxIR hypothesis.

**Challenge**: TxIR's real hypothesis — "one IR can faithfully represent
every chain family" — is stress-tested by *depth* (exotic transactions:
Taproot script-path spends, EIP-4844 blobs, Solana address-lookup-tables,
Cardano multi-asset + Plutus, TON multi-message), not by adding a 40th
thin decoder that routes everything into `Operation::Generic` and a JSON
`extra` blob. The escape hatches (`Generic` + JSON metadata) currently absorb
exactly the cases that would falsify the type system, which means breadth
*hides* counter-evidence rather than providing it. 28 TODOs and several stub
decoders (Mina "actual parsing TODO", Filecoin CID, Arbitrum RLP) add
maintenance surface without testing anything.

**Improvement**: freeze new-chain additions until the three reference
decoders (Bitcoin, Ethereum, Solana) pass: true re-encoding, differential
tests vs upstream, compiling fuzz targets, and typed (non-JSON) extras. Track
"fraction of fixture data landing in Generic/extra" per chain — that number
is the honest measure of how universal TxIR actually is, and it should
ratchet down.

## Assumption 8: "Quality gates run in CI" — GATES EXIST, KEY ONES MISSING

**Measured**: CI runs fmt/clippy/tests/audit/docs — good. But:
- Fuzz targets are not compile-checked on PRs (only nightly fuzzing), which
  is how three fuzz suites silently rotted.
- Nothing checks that "validation" dev-deps are used, that the core LOC
  budget holds, or that decoders don't store raw bytes.
- `tests/fixtures/` at the repo root is empty scaffolding (.gitkeep +
  READMEs, one JSON template) while real fixtures live per-crate — dead
  structure that misleads contributors.

**Improvement**: the health report covers these as cheap static checks and is
designed to run as a CI job (`--fast` mode needs no full build). Fuzz targets
get a `cargo check` gate. Root `tests/fixtures/` either becomes the shared
corpus or is deleted (backlog item).

---

## Summary table

| # | Assumption | Verdict | Primary fix |
|---|-----------|---------|-------------|
| 1 | Core < 3000 LOC | Violated (~4,961) | Delete Canonical* mirror; ratchet LOC |
| 2 | Borsh-only canonical form | Violated by design | Typed extras, no JSON in hashed path |
| 3 | Roundtrip property is enforced | Vacuous where bytes are stored | True re-encoding + mutation check |
| 4 | Adversarial upstream testing | ~1 of 6 chains real | Differential harness as loop signal |
| 5 | Roadmap status drives automation | Unsound (status drifts) | Measured report drives the loop |
| 6 | Supply chain secured | Partial; no lockfile | Lockfile committed; scope claims |
| 7 | Breadth validates TxIR | Backwards | Depth on 3 reference chains; track Generic/extra usage |
| 8 | CI gates quality | Key gates missing | Health report in CI; fuzz compile gate |

**What survives the review** (worth keeping, explicitly):
- The TxIR-as-IR concept and trait-based extension model are sound.
- The core crate's dependency discipline is real.
- Pure-Rust parsing with upstream libs as *test-only* oracles is the right
  architecture — it just needs to actually be done.
- The roundtrip property is the right cheap invariant once re-encoding is
  real, and is ideal for fuzzing.
