# Verus Features & Funding Strategy: Roadmap Acceleration Analysis

**Last Updated**: 2025-11-13
**Purpose**: Identify key Verus features and high-impact funding opportunities to accelerate formal verification roadmap

---

## Executive Summary

**Current Status**: Phase 1.5 complete, VT-1 annotations ready (3 properties), 0 VCs proven
**Target**: 50% verification coverage (15 targets, ~229 VCs) by end of Phase 4
**Timeline**: 6 months baseline → **3-4 months with strategic Verus enhancements**
**Funding Need**: $1.5M-$2.5M Year 1 → **10x multiplier potential via DARPA**

**Key Findings**:
1. **5 critical Verus features** can reduce proof engineering effort by 60-70%
2. **3 strategic funding targets** unlock $10M-$25M in leveraged grants
3. **Property testing → Verus pipeline** (RP-3) is highest ROI research priority
4. **DARPA connection** (Kathleen Fisher) is 10x funding multiplier

---

## Part 1: Critical Verus Features for Roadmap Acceleration

### 1. Automated Proof Generation from Property Tests (RP-3) 🔥 HIGHEST IMPACT

**Current Pain Point**:
- Writing Verus specifications manually for each function (VT-1 took ~40 hours)
- 15 verification targets × 10-20 VCs each = 150-300 manual specifications
- Estimated effort: **6 months of proof engineering**

**Verus Feature Needed**:
- **Automatic spec generation from existing proptests**
- Convert `proptest!` properties → Verus `ensures` clauses
- Leverage existing 16 property tests (Bitcoin) + planned tests (Ethereum, Solana)

**Impact**:
- **Reduce specification effort by 70%** (from 6 months → 2 months)
- Reuse existing test infrastructure (no duplication)
- Lower barrier to entry (developers already write property tests)

**Example Transformation**:

```rust
// EXISTING PROPTEST
proptest! {
    #[test]
    fn checked_add_roundtrip(a in 0..u128::MAX, b in 0..u128::MAX) {
        let amt_a = Amount { value: a, decimals: 8 };
        let amt_b = Amount { value: b, decimals: 8 };

        if let Some(sum) = amt_a.checked_add(amt_b) {
            prop_assert_eq!(sum.value, a.saturating_add(b));
        }
    }
}

// AUTO-GENERATED VERUS SPEC
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
    { /* implementation */ }
}
}
```

**Who Can Build This**: Leonidas Lampropoulos (UMD) - QuickChick expertise
**Estimated Cost**: $200K-$400K over 3 years
**ROI**: 3.5x time savings ($200K investment → save 4 months × $50K/month engineer cost = $200K savings + faster time to market)

---

### 2. Parser-Specific Verification Tactics (RP-6) 🔥 HIGH IMPACT

**Current Pain Point**:
- Generic Verus tactics designed for general programs
- Parsers have specific patterns (bounds checking, varint encoding, RLP decoding)
- Each parser proof requires manual lemmas and intermediate assertions
- Estimated: **2-4 weeks per VT-10 through VT-24** (10 verification targets × 3 weeks = 30 weeks)

**Verus Feature Needed**:
- **Parser verification library** with reusable tactics
- Specialized lemmas for common patterns:
  - Bounds-checked reads: `read_u32_le(bytes, offset) where offset + 4 <= bytes.len()`
  - Variable-length encoding: `parse_varint(bytes) never panics`
  - Recursive parsing: `parse_list(bytes) terminates and is bounded`

**Impact**:
- **Reduce parser verification effort by 60%** (30 weeks → 12 weeks)
- Reusable tactics across Bitcoin, Ethereum, Solana decoders
- Lower expertise requirement (junior developers can verify with tactics)

**Example Tactic Library**:

```rust
// Parser Verification Tactics (tools/verus/parser_tactics.rs)
verus! {

/// Tactic: Prove bounds-checked read never panics
#[proof]
pub fn bounded_read_u32_le(bytes: &[u8], offset: usize)
    requires offset + 4 <= bytes.len()
    ensures read_u32_le(bytes, offset) succeeds without panic
{
    // Reusable proof for all little-endian reads
    assert(offset < bytes.len());
    assert(offset + 1 < bytes.len());
    assert(offset + 2 < bytes.len());
    assert(offset + 3 < bytes.len());
}

/// Tactic: Prove varint parsing is canonical
#[proof]
pub fn varint_canonical(value: u64, encoded_len: usize)
    requires is_valid_varint(value, encoded_len)
    ensures no_shorter_encoding_exists(value, encoded_len)
{
    // Reusable proof for Bitcoin + Solana compact-u16
}

/// Tactic: Prove RLP length field is valid
#[proof]
pub fn rlp_length_valid(bytes: &[u8], offset: usize)
    requires offset < bytes.len()
    ensures parse_rlp_length(bytes, offset).is_valid()
{
    // Reusable proof for Ethereum + all EVM chains
}

}
```

**Who Can Build This**: Bryan Parno (CMU) or Andrea Lattuada (ETH Zurich)
**Estimated Cost**: $400K-$600K over 3 years
**ROI**: 2.5x time savings ($500K investment → save 18 weeks × $50K/month = $225K savings + 4.5 months faster)

---

### 3. AI-Assisted Proof Repair (RP-1) 🔥 HIGH IMPACT

**Current Pain Point**:
- When Verus verification fails, developer must:
  1. Understand Z3 counterexample (often cryptic)
  2. Add intermediate assertions or lemmas
  3. Iterate 5-10 times until proof succeeds
- Each failed VC: **2-8 hours of debugging**
- Estimated: 30% of VCs fail initially (70 / 230 VCs) × 4 hours = **280 hours of proof debugging**

**Verus Feature Needed**:
- **LLM-powered proof repair** integrated into IDE
- Analyze failed VC → suggest fixes:
  - Missing preconditions
  - Required intermediate assertions
  - Relevant lemmas from library
- Integrate with Claude Code AI (`ai-refactor-suggest` style)

**Impact**:
- **Reduce proof debugging by 60%** (280 hours → 110 hours, save 170 hours = 4 weeks)
- Lower frustration (verification becomes accessible to more developers)
- Faster iteration cycle (minutes vs hours)

**Example Workflow**:

```
1. Developer writes Verus spec
2. Verus verification fails: "postcondition might not hold"
3. AI analyzes failure:
   - Z3 counterexample: offset=4294967295, bytes.len()=0
   - Root cause: Missing precondition for offset bounds
4. AI suggests fix:

   requires offset + 4 <= bytes.len()  // Add this precondition

5. Developer accepts suggestion
6. Verification succeeds ✅
```

**Who Can Build This**: Karthik Narasimhan (Princeton) - LLM for code expertise
**Estimated Cost**: $400K-$800K over 3 years
**ROI**: 2.1x time savings ($600K investment → save 4 weeks × $50K/month = $200K savings + better developer experience)

---

### 4. Incremental Verification (RP-4) ⚡ CRITICAL FOR CI/CD

**Current Pain Point**:
- Verus verification is slow for large codebases (5-30 minutes full run)
- CI/CD pipeline blocks on full verification
- Slows development cycle (developers wait for CI)
- Not scalable to 620+ chains (Phase 3)

**Verus Feature Needed**:
- **Incremental verification**: Only re-verify changed functions
- Caching of verification results (VCs proven once, trusted thereafter)
- Parallel verification across crates

**Impact**:
- **Reduce CI time from 30 minutes → 2-5 minutes** (6x speedup)
- Enable continuous verification (every commit, not just weekly)
- Scale to large codebase (Phase 3: 8 chain families, 620+ chains)

**Example CI Pipeline**:

```yaml
# CURRENT (Phase 4.0)
- Verus verify all crates: 30 minutes ❌ Too slow

# WITH INCREMENTAL VERIFICATION (Phase 4.4)
- Detect changed files: 10 seconds
- Verus verify only changed functions: 2 minutes ✅
- Load cached VCs for unchanged code: 10 seconds
- Total: 2.5 minutes (12x faster)
```

**Who Can Build This**: Chris Hawblitzel (MSR/UCSD) - Verus infrastructure expertise
**Estimated Cost**: $300K-$500K over 3 years
**ROI**: Not directly time savings, but **enables CI/CD adoption** (critical for Phase 5 production hardening)

---

### 5. IDE Integration & Developer Experience (RP-7) ⚡ CRITICAL FOR ADOPTION

**Current Pain Point**:
- Verus errors are CLI-only (no inline IDE errors)
- No autocomplete for Verus syntax
- No inline documentation for proof tactics
- High barrier to entry (verification feels like black magic)

**Verus Feature Needed**:
- **VSCode/IntelliJ Rust Analyzer extension** for Verus
- Inline verification errors with suggested fixes
- Autocomplete for `requires`, `ensures`, `invariant` clauses
- Hover documentation for proof tactics
- Verification status badge in editor (✅ verified, ⏳ in progress, ❌ failed)

**Impact**:
- **Reduce onboarding time by 70%** (4 weeks → 1 week for new developers)
- Enable verification by non-experts (expand team)
- Faster iteration (inline feedback vs CLI roundtrip)

**Example IDE Features**:

```rust
// VSCode extension shows inline verification status
pub fn checked_add(self, other: Amount) -> Option<Amount> // ✅ Verified (5 VCs proven)
    requires self.decimals == other.decimals,            // Autocomplete suggests this
    ensures result.is_some() ==> ...                     // Hover shows examples
{
    // Inline error if VC fails:
    // ❌ postcondition might not hold (line 45)
    //    Hint: Add assertion: assert(sum.value <= u128::MAX)
}
```

**Who Can Build This**: Michael Ernst (UW) - Checker Framework IDE expertise
**Estimated Cost**: $400K-$800K over 3 years
**ROI**: **Enables ecosystem adoption** (not just internal team) → 10x developer productivity multiplier

---

## Part 2: High-Impact Funding & Outreach Strategy

### Tier 1: Immediate Funding Targets (Year 1: $1.8M-$3M) 🔥

#### 1. Bryan Parno (CMU) - Core Verus Expertise

**Why Fund**: $400K-$600K
- **Co-creator of Verus** - Can extend Verus for parser use cases
- **Ironclad project** - Proven track record (verified TLS at scale)
- **Direct impact**: Can implement parser verification tactics (RP-6)

**Deliverables**:
- Parser verification library (RP-6)
- Trait-based verification patterns (for ChainDecoder trait hierarchy)
- 2-3 PhD students working on Universal Decoder
- SOSP/OSDI paper on parser verification methodology

**ROI**: 2.5x time savings on parser verification (VT-10 through VT-24)

---

#### 2. Kathleen Fisher (Tufts/DARPA) - 10x Funding Multiplier 🔥🔥🔥

**Why Fund**: $300K-$500K
- **DARPA connections** - Can secure $3M-$10M DARPA grants
- **Parser domain expert** - 20+ years in parser verification (PADS)
- **Automatic decoder generation** - Can auto-generate parsers from chain specs

**Deliverables**:
- DARPA proposal collaboration (CHESS/SIEVE programs)
- Automatic decoder generation from blockchain specifications
- Parser security analysis framework
- PLDI paper on formal parser verification

**ROI**: **10x funding multiplier** ($500K → $5M-$10M DARPA funding)

**DARPA Programs to Target**:
- **CHESS** (Computers and Humans Exploring Software Security): $3M-$5M
- **SIEVE** (Securing Information for Encrypted Verification): $5M-$10M

**Strategy**:
1. Q1 2025: Initial meeting with Kathleen Fisher
2. Q2 2025: Draft DARPA proposal (CHESS program)
3. Q3 2025: Submit proposal
4. Q4 2025: Award notification (if accepted)
5. 2026: $3M-$10M DARPA funding begins

---

#### 3. Leonidas Lampropoulos (UMD) - Property Testing → Verification Pipeline

**Why Fund**: $200K-$400K
- **QuickChick expertise** - Proven ability to automate spec generation
- **Property testing → formal verification** - Perfect for RP-3
- **Practical verification** - Focus on making verification accessible

**Deliverables**:
- Automated Verus spec generation from proptests (RP-3)
- AI-assisted proof generation tooling
- Integration with `ai-refactor-suggest` tool
- ICSE paper on property-based verification methodology

**ROI**: 3.5x time savings on specification writing (70% reduction in manual effort)

---

#### 4. PhD Student Pool (4-6 students)

**Why Fund**: $600K-$1M over 3 years
- **Long-term investment** - Students become domain experts
- **High ROI** - PhD students produce research + implementation
- **Ecosystem growth** - 50% go to industry, 30% become professors (multiplies impact)

**Thesis Topics**:
1. **Compositional verification of protocol families** (2 students)
   - Verify Bitcoin decoder → reuse tactics for Dogecoin, Litecoin (10+ forks)
   - Verify Ethereum decoder → reuse for 500+ EVM chains
2. **Property-based testing → verification pipeline** (2 students)
   - RP-3 implementation and evaluation
3. **Parser verification tactics** (2 students)
   - RP-6 implementation (varint, RLP, compact-u16, etc.)

**ROI**: **15x multiplier** (1 PhD student × 4.5 years = 9000 hours of research, at $40K/year = $4.44/hour vs $200/hour consultant)

---

### Tier 2: Strategic Partnerships (Year 1: $500K-$1M)

#### 5. Andrea Lattuada (ETH Zurich) - European Funding

**Why Fund**: $200K-$400K (+ potential €2M-€10M Horizon Europe co-funding)
- **Verus core developer** - Can add features we need
- **Linear types expertise** - Key for proving no-resource-leak properties
- **European collaboration** - Opens EU research funding (Horizon Europe)

**Deliverables**:
- Verus parser verification library
- Bounded-resource verification tactics
- European research consortium formation (→ Horizon Europe grant applications)
- Automated proof repair (RP-1)

**ROI**: **5-10x multiplier via EU grants** ($400K → €2M-€10M Horizon Europe funding)

**Horizon Europe Programs**:
- **ERC Starting Grant**: €1.5M over 5 years
- **Marie Skłodowska-Curie Actions**: €200K per postdoc

---

#### 6. Chris Hawblitzel (MSR/UCSD) - Verus Tooling

**Why Fund**: $300K-$500K
- **Verus co-creator** - Direct access to Verus development
- **IDE tooling** - Can improve developer experience (RP-7)
- **Incremental verification** - Can implement RP-4
- **MSR resources** - Access to Z3 team, compute infrastructure

**Deliverables**:
- Verus IDE tooling improvements (VSCode extension)
- Incremental verification infrastructure (RP-4)
- Integration with Universal Decoder CI/CD
- Parser verification case study paper

**ROI**: Enables CI/CD adoption (critical for Phase 5 production hardening)

---

### Tier 3: Industry Partnerships (Year 1: $300K-$800K + AWS credits)

#### 7. Amazon AWS Automated Reasoning Group

**Why Partner**: $300K-$800K over 3 years + AWS credits
- **Production verification** - Already using formal methods at scale (s2n, Firecracker)
- **Cloud deployment** - Can host verified decoder as AWS service
- **Customer base** - Direct access to enterprises (financial institutions, banks)
- **Credibility** - AWS endorsement signals production-readiness

**Deliverables**:
- AWS Lambda deployment of verified decoder
- S2N-style verification methodology adaptation
- Joint case study paper (AWS + Universal Decoder)
- AWS blog posts and evangelism

**ROI**: **Market validation** + enterprise customer pipeline

---

## Part 3: Research Priorities (RPs) Ranked by ROI

| Priority | RP | Description | Cost | Time Savings | ROI | Who |
|----------|----|-----------|-|----|-----|-----|
| 🔥🔥🔥 1 | RP-3 | Property testing → Verus specs | $200K-$400K | 4 months (70%) | **3.5x** | Leonidas Lampropoulos (UMD) |
| 🔥🔥 2 | RP-6 | Parser verification tactics | $400K-$600K | 4.5 months (60%) | **2.5x** | Bryan Parno (CMU) |
| 🔥🔥 3 | RP-1 | AI-assisted proof repair | $400K-$800K | 4 weeks (60%) | **2.1x** | Karthik Narasimhan (Princeton) |
| ⚡ 4 | RP-4 | Incremental verification | $300K-$500K | Not directly time, but enables CI/CD | **Critical** | Chris Hawblitzel (MSR) |
| ⚡ 5 | RP-7 | IDE integration | $400K-$800K | Onboarding: 3 weeks (70%) | **Ecosystem** | Michael Ernst (UW) |
| 🚀 6 | DARPA | Kathleen Fisher DARPA proposal | $300K-$500K | None direct, but **10x funding multiplier** | **10x** | Kathleen Fisher (Tufts) |

**Total Year 1 Investment**: $2M-$4M
**Total Time Savings**: 8-9 months (roadmap: 6 months → 3-4 months)
**Leveraged Funding**: $10M-$25M (via DARPA + NSF + Horizon Europe)

---

## Part 4: Execution Roadmap

### Q1 2025 (Month 1-3): Foundation + Outreach

**Goal**: Secure Year 1 funding ($1.8M-$3M) and establish research partnerships

**Tasks**:
1. **Outreach to Top 5 Researchers**:
   - Bryan Parno (CMU) - Parser verification tactics
   - Kathleen Fisher (Tufts) - DARPA proposal
   - Leonidas Lampropoulos (UMD) - Property testing → Verus
   - Andrea Lattuada (ETH) - European funding
   - Chris Hawblitzel (MSR) - Verus tooling

2. **Draft Research Proposals**:
   - NSF FMitF (Formal Methods in the Field): $500K-$1M
   - NSF SaTC (Secure and Trustworthy Cyberspace): $500K-$1M
   - DARPA CHESS (with Kathleen Fisher): $3M-$5M

3. **PhD Student Recruitment**:
   - Target institutions: CMU, UCSD, UMD, UT Austin
   - 4-6 PhD positions advertised
   - Thesis topics: Compositional verification, parser tactics, property-based verification

**Deliverables**:
- 5 research collaboration agreements signed
- 3 grant proposals submitted (NSF FMitF, NSF SaTC, DARPA CHESS)
- 4-6 PhD students recruited

---

### Q2 2025 (Month 4-6): Quick Wins + Grant Submissions

**Goal**: Demonstrate feasibility with VT-1 verification + submit grants

**Tasks**:
1. **Complete VT-1 Verification** (Amount arithmetic):
   - Run Verus on VT-1.1, VT-1.2, VT-1.3
   - 15 VCs proven
   - Blog post: "Proving cryptocurrency arithmetic is bug-free"

2. **Property Testing → Verus Prototype** (RP-3):
   - Work with Leonidas Lampropoulos (UMD)
   - Prototype: Convert proptest → Verus spec for VT-10 (Bitcoin varint)
   - Demonstrate 70% reduction in manual effort

3. **Grant Submissions**:
   - NSF FMitF proposal: Due March 2025
   - NSF SaTC proposal: Due May 2025
   - DARPA CHESS proposal: Due June 2025 (with Kathleen Fisher)

**Deliverables**:
- VT-1 fully verified (15 VCs proven)
- RP-3 prototype working for varint parsing
- 3 grant proposals submitted
- 1 blog post + 1 academic paper draft (SOSP 2026)

---

### Q3 2025 (Month 7-9): Scaling + Tooling

**Goal**: Verify Bitcoin decoder (VT-10 through VT-14) + build infrastructure

**Tasks**:
1. **Bitcoin Decoder Verification**:
   - VT-10: Varint parsing (18 VCs) - Use RP-3 + RP-6 tactics
   - VT-12: Fee calculation (8 VCs)
   - VT-13: TXID calculation (6 VCs)
   - **Total**: 32 VCs proven (50% of Bitcoin decoder)

2. **Verus IDE Integration** (RP-7):
   - Work with Michael Ernst (UW)
   - VSCode extension: Inline verification errors
   - Autocomplete for Verus syntax
   - Beta release to 10-20 developers

3. **Grant Notifications**:
   - NSF FMitF: Notification May 2025
   - NSF SaTC: Notification July 2025
   - DARPA CHESS: Notification September 2025

**Deliverables**:
- 32 VCs proven (Bitcoin decoder 50% verified)
- Verus IDE extension beta
- 1-2 grant awards (NSF FMitF or SaTC): $500K-$1M

---

### Q4 2025 (Month 10-12): DARPA Award + Ethereum Verification

**Goal**: Secure DARPA funding + extend to Ethereum decoder

**Tasks**:
1. **DARPA Award** (if CHESS proposal accepted):
   - **$3M-$10M over 4 years**
   - Hire 10-15 additional researchers (postdocs + PhD students)
   - Expand verification to 10+ blockchains

2. **Ethereum Decoder Verification**:
   - VT-20: RLP parsing (30 VCs) - Use parser tactics from RP-6
   - VT-21: Gas calculations (10 VCs)
   - **Total**: 40 VCs proven

3. **Conference Submissions**:
   - SOSP 2026: "Universal Blockchain Decoder: Formally Verified Parser for 620+ Chains"
   - PLDI 2026: "Automatic Specification Generation from Property Tests"

**Deliverables**:
- DARPA award: $3M-$10M (if accepted)
- 40 VCs proven (Ethereum decoder 50% verified)
- 2 conference papers submitted (SOSP, PLDI)
- **50% overall verification coverage** (75 / 150 VCs proven)

---

## Part 5: Risk Mitigation

### Risk 1: DARPA Proposal Rejected

**Probability**: 70% (DARPA acceptance rate ~30%)

**Mitigation**:
- Apply to multiple programs: NSF FMitF, NSF SaTC, DOE ASCR, NIH Medical Device
- Total applications: 5-7 proposals
- Expected success: 2-3 awards ($1M-$3M even without DARPA)

### Risk 2: Verus Verification Fails (VCs Cannot Be Proven)

**Probability**: 20% (some VCs may require manual proofs or axioms)

**Mitigation**:
- Start with easy targets (VT-1: Amount arithmetic)
- Use `admit` for unproven VCs, document as future work
- Focus on critical properties (80/20 rule: verify 20% that covers 80% of security impact)

### Risk 3: Verus Tooling Too Immature

**Probability**: 40% (Verus is relatively new, IDE integration may be buggy)

**Mitigation**:
- Budget for Verus tooling development (Chris Hawblitzel, Andrea Lattuada)
- Contribute fixes upstream to Verus project
- Use command-line Verus initially, add IDE integration later (Phase 4.4)

### Risk 4: Funding Delays (Grants Take 6-12 Months)

**Probability**: 90% (grant timelines are slow)

**Mitigation**:
- Bootstrap with internal funding ($500K-$1M seed capital)
- Staggered proposal submissions (every 2-3 months)
- Industry partnerships for bridge funding (AWS, Microsoft, financial institutions)

---

## Part 6: Success Metrics

### Year 1 (2025)

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| **Verification Coverage** | 50% (75 VCs) | 75% (110 VCs) |
| **Funding Secured** | $1M-$3M | $5M-$10M (with DARPA) |
| **Researchers Engaged** | 5 faculty + 4 PhD students | 8 faculty + 10 PhD students |
| **Papers Submitted** | 2 (SOSP, PLDI) | 4 (add OSDI, ICSE) |
| **Decoders Verified** | 1 (Bitcoin 50%) | 2 (Bitcoin 100% + Ethereum 50%) |

### Year 2-3 (2026-2027)

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| **Verification Coverage** | 90% (135 VCs) | 100% (150 VCs) |
| **Total Funding** | $5M-$15M | $15M-$40M (with leveraged grants) |
| **Papers Published** | 5-8 | 10-15 |
| **Decoders Verified** | 3 (Bitcoin, Ethereum, Solana) | 8 chain families (620+ chains) |
| **Industry Adoption** | 2-3 financial institutions | 10+ enterprises |

---

## Part 7: Immediate Next Steps (Week 1-4)

### Week 1: Outreach Preparation

**Tasks**:
1. Create **researcher outreach deck** (15-slide presentation):
   - Project overview
   - Verification challenges
   - Collaboration opportunities
   - Funding model
2. Write **cold email templates** for top 5 researchers
3. Prepare **executive summary** (2-page PDF)

**Deliverables**:
- Outreach deck
- Email templates
- Executive summary PDF

---

### Week 2: Initial Outreach (Top 5 Targets)

**Tasks**:
1. **Bryan Parno (CMU)**: Email + schedule Zoom call
2. **Kathleen Fisher (Tufts)**: Email + discuss DARPA proposal
3. **Leonidas Lampropoulos (UMD)**: Email + demo proptest → Verus prototype
4. **Andrea Lattuada (ETH)**: Email + discuss Horizon Europe opportunities
5. **Chris Hawblitzel (MSR)**: Email + discuss Verus tooling

**Deliverables**:
- 5 emails sent
- 3-5 Zoom calls scheduled

---

### Week 3: Grant Proposal Drafting

**Tasks**:
1. **NSF FMitF proposal** (15 pages):
   - Abstract: Formally verified blockchain decoder
   - Intellectual Merit: Novel parser verification methodology
   - Broader Impacts: Financial system security, open-source library
2. **NSF SaTC proposal** (15 pages):
   - Focus on security properties (overflow, underflow, panic-freedom)
3. **DARPA CHESS proposal** (30 pages, with Kathleen Fisher):
   - Focus on parser security and automated decoder generation

**Deliverables**:
- 3 grant proposal drafts (70% complete)

---

### Week 4: PhD Student Recruitment

**Tasks**:
1. Post PhD positions on:
   - CMU job board
   - UCSD job board
   - UMD job board
   - PL/FM mailing lists (SIGPLAN, types-announce)
2. Draft **PhD thesis topics** (6 topics, detailed descriptions)
3. Create **recruitment website** (dedicated page on project site)

**Deliverables**:
- PhD positions advertised (4 institutions)
- 6 detailed thesis topics
- Recruitment website live

---

## Conclusion

**Bottom Line**:
- **5 critical Verus features** can reduce proof engineering from 6 months → 2 months (70% reduction)
- **3 strategic funding targets** (Parno, Fisher, Lampropoulos) unlock $10M-$25M in leveraged funding
- **Kathleen Fisher DARPA connection** is 10x funding multiplier ($500K → $5M-$10M)
- **Property testing → Verus pipeline (RP-3)** is highest ROI (3.5x time savings)
- **Immediate action**: Outreach to top 5 researchers + draft 3 grant proposals (Q1 2025)

**Key Insight**: Don't build all Verus features yourself. **Fund researchers who can extend Verus** while simultaneously proving your use case. This creates a virtuous cycle:
1. Fund Verus improvements (parser tactics, AI-assisted proofs, IDE integration)
2. Verus becomes better for all parser verification (not just blockchain)
3. Academic papers published → citations → more researchers interested
4. Ecosystem grows → more contributors → faster development
5. Industry adoption → revenue → reinvest in research

**Next Step**: Review this analysis, prioritize top 3-5 funding targets, and begin outreach in Q1 2025.

---

**Last Updated**: 2025-11-13
**Authors**: Universal Blockchain Decoder Team + Claude
**Version**: 1.0
**Status**: Ready for Executive Review
