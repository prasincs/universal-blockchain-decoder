# Concepts Review (2026-07)

A second review pass, one level deeper than `docs/ASSUMPTIONS_REVIEW.md`.
That audit asked "are the project's claims true?"; this one asks **"are the
underlying concepts right, even if they were implemented perfectly?"** Every
verdict cites code as evidence. Concept changes are breaking design
decisions — the improvements here are proposals with a recommendation, filed
in `loop/BACKLOG.md` under "Concept decisions (need sign-off)".

---

## C1. TxIR conflates three layers that have different epistemic status

TxIR mixes, in one hashed structure:

1. **Syntax** — what the bytes say (nonce, outputs, signatures). Total,
   decidable, verifiable by re-encoding. This is the decoder's actual job.
2. **Interpretation** — what the transaction *means* (`Operation::Transfer`
   vs `ContractCall`). A heuristic classification that can be wrong or
   version-dependent, never verifiable against bytes.
3. **Effects** — what the transaction *does to state* (`StateDeltas`). Not a
   function of the bytes at all: it requires the chain state (UTXO set,
   account balances, gas market, contract code).

Layer 3 is the clearest category error, and the decoders prove it by
fabricating data:

- Bitcoin: `value: 0, // Requires UTXO set`
  (`decoder-bitcoin/src/types.rs:338`) — a raw Bitcoin transaction does not
  contain the values of the inputs it spends. The canonical hash covers a
  placeholder indistinguishable from a genuine zero-value input.
- Zcash: same, twice (`decoder-zcash/src/types.rs:290,442`).
- Cosmos: `balance_change: -1` / `1` (`decoder-cosmos/src/lib.rs:437-497`)
  — a **direction flag** crammed into a field typed as an i128 balance
  delta. Semantic garbage, Borsh-serialized and hashed as "canonical".
- Ethereum: `balance_change: -(self.value as i128)`
  (`decoder-ethereum/src/types.rs:869`) — ignores gas, so it is not the
  balance change.
- Aptos: `-gas_cost` guessed from gas limit (`decoder-aptos/src/lib.rs:327`).
- XRP and Stellar: sender delta = **fee only** — the payment amount and the
  recipient's delta are simply absent (`decoder-xrp/src/types.rs:279-289`,
  `decoder-stellar/src/lib.rs:324-341`).
- Solana: `balance_change: 0` for writable-*signed* accounts only; writable
  unsigned accounts omitted entirely (`decoder-solana/src/types.rs:337-351`).
- TON: `balance_change: 0` even though `total_fees` was parsed and then
  discarded (`decoder-ton/src/lib.rs:248-256`).
- Cosmos additionally abuses `outputs` — a UTXO field — for IBC/WASM
  messages on an account chain (`decoder-cosmos/src/lib.rs:502,565`).

Every decoder faced the same impossible demand — "fill in effects you cannot
know" — and each invented a different lie. **That is a concept failing, not
an implementation failing.** No amount of decoder work fixes it, because the
field is not computable from the input.

And these are not internal artifacts: the CLI prints `state_deltas` counts
and the canonical hash (`universal-decoder-cli/src/main.rs:663-677`), and
the WASM bindings serialize full `state_deltas` — fabricated zero input
values, `±1` sentinels, fee-only deltas — to JavaScript callers as if they
were real (`universal-decoder-wasm/src/lib.rs:544-565`). The fabrications
are the shipped product surface.

**Improvement (recommended)**: split the IR by epistemic status.

- `DecodedTx` (layer 1): only byte-derivable facts. Hashed, differential-
  tested, roundtrip-verified. This is the trusted product.
- Interpretation (layer 2): keep `operations`, but versioned as a *lens*
  (`interpretation_version`), excluded from the canonical hash or hashed
  separately — two implementations can then disagree on classification
  without disagreeing on what was decoded.
- Effects (layer 3): delete `StateDeltas` from TxIR. UTXO in/out points are
  layer-1 facts and stay (prev_tx, index, script, output values); *balances,
  input values, account_changes* are not. If effect prediction is ever
  wanted, it is a separate function `fn effects(tx, state) -> StateDeltas`
  whose signature admits it needs state.

## C2. `canonical_hash` is not currently a well-defined concept

`CanonicalSerialize::canonical_hash()` = SHA-256 over Borsh of the full TxIR.
For that hash to mean anything, the input must be a *deterministic function
of the transaction bytes*. Today it also covers:

- **Fabricated effects** (C1) — implementation-specific lies.
- **Display strings**: `Address.human_readable: Option<String>` — bech32 vs
  checksummed-hex rendering changes the hash; `ChainRef.name: String` —
  "Arbitrum One" vs "arbitrum-one" changes the hash; `Amount.decimals` — for
  tokens, decimals is *contract state* (off-chain knowledge), not bytes.
- **Free-form JSON** (`extra`, signature metadata — Assumptions Review #2),
  and even Rust `Debug` output (C3 below). Worse: three decoders write
  strings into the "JSON" `extra` field that are not JSON at all — Cosmos
  `"memo: {…}, gas: {…}"` (`decoder-cosmos/src/lib.rs:129`), Polkadot
  `"pallet: {…}, call: {…}"`, TON `"lt:{…},outmsg_cnt:{…}"` — hand-built
  with `format!`, so adversarial memo content can even make them malformed.
- **`metadata.tx_hash`**, which IS byte-derivable but is wrong in 5 of 8
  surveyed decoders: Zcash leaves it **empty** (`types.rs:231`), TON stores
  the **previous** transaction's hash (`lib.rs:232`), Polkadot uses
  Blake2b-**512** where the real extrinsic hash is Blake2b-256
  (`lib.rs:45-49`), Stellar hashes ASCII tag strings plus signatures instead
  of the XDR envelope discriminant (`types.rs:443-465`), and Bitcoin returns
  internal byte order rather than the display txid. Nothing caught this
  because no differential oracle checks txids yet — the canonical hash then
  hashes these wrong hashes.

So two *correct* decoders of the same transaction produce different
"canonical" hashes — and the one implementation we have often hashes
incorrect or absent data. The concept needs a definition before it needs code:
**the canonical hash is a cross-implementation fingerprint of what was
decoded**, and therefore its domain must be exactly the byte-derivable
fields, with a written per-version field list (a `CANONICAL_HASH.md` spec).
Everything else — display names, decimals, chain nicknames, JSON — lives
outside the hashed set. (Prerequisite for the differential-testing endgame:
"decoders agree" should eventually be checkable as hash equality, which is
only possible once the hash stops covering opinions.)

## C3. The `Generic` escape hatch silently drops security-critical content

`Operation::Generic { op_type: String, data: Vec<u8>, metadata: String }` is
where everything outside the 4 named operations lands. Two problems, one
conceptual:

NEAR (`decoder-near/src/lib.rs:309-321`): `AddKey`, `DeleteKey`,
`DeleteAccount` — the actions that grant or revoke *control of an account* —
map to `Generic` with `op_type` derived from `format!("{:?}", action)`,
**`data: vec![]`**, and `metadata: format!("{:?}", action)`. The public key
being added does not appear in the TxIR at all; the "canonical" metadata is
Rust Debug formatting, which is not stable across compiler or type changes.

The conceptual problem: **the fallback is lossy precisely where fidelity
matters most.** Exotic, dangerous operations (permission changes, module
upgrades) are exactly what a decode-for-verification IR exists to surface,
and exactly what the escape hatch reduces to an unstructured label. A signer
reviewing this TxIR would see less than a block explorer shows.

The deeper version of the same failing: the Operation vocabulary has **no
mapping specification**, so the eight surveyed decoders invented eight
philosophies —

- Zcash: **everything is `Generic`**, including plain transparent UTXOs
  (`"UTXO_Input"`/`"UTXO_Output"` with JSON-as-bytes payloads) — 100%
  escape hatch.
- Cosmos: **everything is `Transfer`** — governance votes become Transfers,
  IBC acknowledgements become zero-amount self-Transfers, contract
  instantiation becomes a Transfer (`decoder-cosmos/src/lib.rs:272-384`);
  `Stake`/`ContractCall`/`Generic` are never used, even though Cosmos has
  the most staking-shaped messages of any surveyed chain.
- Solana: **everything is `ContractCall`** with empty `method` — even
  System-program transfers are never recognized as Transfers
  (`decoder-solana/src/types.rs:314-327`).
- TON: **zero operations, hardcoded `vec![]`** despite parsed messages
  (`decoder-ton/src/lib.rs:263`).
- Bitcoin: one `Transfer` per output with an always-empty `from`.

The same on-chain reality maps to disjoint TxIR shapes depending on which
decoder author's taste you get. An IR whose vocabulary has no normative
mapping rules isn't an IR; it's five dialects sharing a schema. The
improvement therefore has two halves: the no-information-destruction rule
above, **and a written mapping spec per ChainFamily** (what MUST become
`Transfer`, what MUST NOT, when `Generic` is allowed) — testable, because
differential fixtures can assert the expected variant.

**Improvement**: `Generic` must obey a *no-information-destruction rule*:
`data` carries the complete chain-native encoding of the operation (so the
TxIR is at worst un-interpreted, never un-faithful), `op_type` comes from a
registry of stable identifiers (chain-namespaced, e.g. `near:AddKey`), and
Debug/JSON strings are banned from it. Add a lint/test: no decoder may emit
`Generic` with empty `data` for an action that has a payload.

## C4. Chain identity has no namespace, so identity is ambiguous

`ChainIdentity::chain_id() -> u64` with doc guidance "CAIP-2, SLIP-44, or
custom" (`chain.rs:33-38`) — i.e., three incompatible registries at once. In
the code today: Bitcoin=0 and Solana=501 look like SLIP-44, Ethereum=1 and
Arbitrum=42161 are EVM chain-ids. Those namespaces collide (SLIP-44 0 is
Bitcoin; EVM chain-id 0 is unassigned/test; SLIP-44 1 is "testnet all
coins"; EVM 1 is Ethereum mainnet). A u64 cannot carry identity across
registries, and `ChainRef.name: String` inside the hashed form makes chain
identity partly a matter of spelling.

**Improvement**: adopt a CAIP-2-shaped identity — `(namespace, reference)`
like `("eip155", "42161")`, `("bip122", <genesis-hash-prefix>)` — as the
hashed identity; keep the u64 and display name as convenience metadata
outside the hash. This is also what makes the 2000-chain EVM registry sound:
one namespace, upstream-defined references.

## C5. The `ChainFamily` taxonomy mixes orthogonal axes

`{Utxo, Account, Instruction, Privacy, Actor, Other}` (`chain.rs:62-90`)
mixes the **state model** (UTXO vs account) with **transaction packaging**
(instruction lists — but Cosmos txs are also message lists and are filed
under "Account") and **privacy** (Zcash is UTXO *and* shielded; the
`TxIR.privacy` field already models that axis properly — the taxonomy
double-counts it). Classifications that mix axes force arbitrary choices
and then get baked into the canonical hash via `ChainRef.family`, where they
can never be re-filed without changing every historical hash.

**Improvement**: family = state model only (`Utxo | Account | Hybrid`);
everything else becomes orthogonal, non-hashed capability metadata. Better
still: drop family from the per-transaction hashed data entirely — it is
static chain metadata, not a fact about this transaction.

## C6. `AuthorizationPackage` assumes one scheme and "everything is a signature"

`signature_scheme: SignatureScheme` is a single value per transaction
(`ir.rs:183`), but real transactions mix schemes: a Taproot script-path
spend can carry Schnorr and ECDSA material; Cosmos multisigs mix key types;
Bitcoin witness stacks contain non-signature items (scripts, control blocks,
arbitrary data) that are not "signatures" at all. The concept "a transaction
has *a* signature scheme and a flat list of signatures with `key_index`
links" is an account-chain generalization that doesn't survive contact with
UTXO witnesses.

The survey shows every decoder bending under it: Bitcoin crams the **entire
witness stack** — pubkeys, redeem scripts, control blocks — into
`signatures` and hardcodes `Ecdsa` even for Taproot/Schnorr spends
(`decoder-bitcoin/src/types.rs:274-305`); Zcash labels Sapling (RedJubjub)
authorization `Ecdsa` while extracting nothing; Polkadot correctly maps
Sr25519 to `Schnorr` but then mislabels the *key* as `Ed25519`
(`decoder-polkadot/src/lib.rs:170`); XRP hardcodes `Ecdsa` even for
`ED`-prefixed Ed25519 keys and ignores `Signers` multisig arrays entirely;
Stellar maps N signatures to a hardcoded **single** public key, a mismatch
that core's own `verify_structure` would reject
(`decoder-stellar/src/lib.rs:129-132`).

**Improvement**: scheme moves onto each `Signature`; witnesses are
represented as witnesses (per-input stacks of typed items: signature |
pubkey | script | control-block | opaque), from which signatures are one
projection.

## C7. Dead and ceremonial concepts are inflating the "minimal" core

Measured (grep across all decoder crates):

| Concept | Evidence | Verdict |
|---|---|---|
| `TxVerifier` (verify signatures *from TxIR*) | 0 implementations | Conceptually impossible: sighashes are computed over chain-native serialization, which TxIR doesn't preserve. Verification belongs on the chain-specific type. Delete. |
| `DecoderPlugin` (`Box<dyn Any>` registry) | 0 implementations | Contradicts the stated static-dispatch/zero-cost/verifiability ethos on its face. Delete. |
| `TxIR<'a, const V: u8>` version parameter | All 45 usages are `V = 1`; `Canonicalizer` hardcodes `TxIR<'a, 1>` (`traits.rs:109`) | Type-level versioning that cannot actually vary is ceremony; worse, the trait pins it, so it *can't* evolve without breaking every decoder anyway. Replace with a runtime `format_version` field governed by the C2 spec. |
| `PhantomData<&'a [u8]>` lifetime | Borrows nothing; owns all data | Already flagged in Assumptions #1 — it exists to prevent byte-storage it doesn't prevent, and it forces the 650-LOC Canonical* mirror. Delete. |
| `hooks.rs` (LoggingHook, SizeLimitHook, registry) | Middleware inside the TCB | Logging middleware does not belong in a "small, reviewable, formally verifiable" core. Move out or delete. |
| `TxHashable::to_canonical_bytes() -> Vec<u8>` | Duplicates `CanonicalSerialize::to_canonical_bytes() -> Result<...>` with a diverging signature | Two competing canonical-bytes concepts in one core. Keep one. |

Deleting these is also the honest path back toward the 3000-LOC budget —
shrink by removing concepts that don't earn their keep, not by code golf.

## C8. What "universal" can honestly mean

The strong reading — *one type system that losslessly represents every
chain's transactions* — is falsified by this codebase's own evidence:
fabricated effects (C1), lossy fallbacks (C3), JSON escape valves, and 24
decoders keeping the original bytes around because TxIR can't reproduce
them. The defensible product is different and still valuable:

1. **Per-chain faithful decode** (chain-specific types), verified by
   roundtrip + differential testing. This part is real today and is where
   the trust story lives.
2. **A versioned semantic summary** (TxIR) with an explicit
   **no-fabrication rule** — every field is traceable to bytes or absent —
   and **measured coverage**: % of transaction content that survives into
   typed TxIR rather than `Generic`/`extra`, per chain, ratcheted upward.

"Universal" then means *universally honest*: the summary never claims more
than the bytes support, and its blind spots are quantified instead of
hidden. That reframing turns the escape hatches from falsifier-absorbers
(Assumptions #7) into the project's own falsification meter.

## What survives this review

- **Trait-based decoder registration** (`ChainDecoder`/`Canonicalizer`/
  `ChainEncoder`): right shape, keep.
- **privacy.rs** is the best-designed module in core: composable primitives
  instead of enums, orthogonal to state model, optional. It should be the
  template for operations/witnesses — though it currently has only 2
  consumers (zcash, aleo), so it needs the same measured-coverage treatment.
- **Borsh for the canonical layer** — right tool; the problem is what's fed
  into it (C1/C2), not the encoding.
- **The roundtrip property and differential oracles** — the verification
  concepts are sound; C1/C2 are about keeping unverifiable content from
  free-riding on their credibility.

## Migration sketch (TxIR v2, one deliberate break)

Order matters — this stacks with the open backlog items so the format
breaks once, not four times:

1. Adopt the C2 hash spec (byte-derivable fields only) + typed extras
   (Assumptions #2) + CAIP-2 identity (C4).
2. Delete `StateDeltas`-as-effects (C1), single `signature_scheme` (C6),
   and the dead concepts (C7) — including the phantom lifetime, which
   collapses the Canonical* mirror.
3. Enforce the `Generic` no-information-destruction rule (C3) and add the
   Generic-rate / fabrication-rate metrics to the health report.

Each step is a backlog item under "Concept decisions (need sign-off)" in
`loop/BACKLOG.md`; none should be executed by the loop until the design is
signed off, because every one changes canonical hashes.
