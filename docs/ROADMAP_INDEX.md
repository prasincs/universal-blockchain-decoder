# Universal Blockchain Decoder - Roadmap Index

> **Three domains, one vision**: Building credible, useful, and sustainable multi-chain transaction analysis

**Last Updated**: 2025-11-16

---

## Overview

This project has three parallel tracks:

```
┌─────────────────────────────────────────────────────────┐
│                     VISION                               │
│  Open, verifiable, privacy-aware transaction analysis   │
│             across all blockchains                       │
└─────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  TECHNICAL   │  │   PRODUCT    │  │  MARKETING   │
│              │  │              │  │              │
│ • Decoders   │  │ • Use cases  │  │ • Content    │
│ • Verification│ │ • Adoption   │  │ • Conferences│
│ • Security   │  │ • Validation │  │ • Community  │
│ • Performance│  │ • PMF        │  │ • Awareness  │
└──────────────┘  └──────────────┘  └──────────────┘
```

---

## The Three Roadmaps

### 📐 [Technical Roadmap](./ROADMAP_TECHNICAL.md)

**Focus**: Software engineering, security, performance

**Key milestones**:
- Q1 2025: Complete 1 formal proof + benchmarks (CRITICAL for credibility)
- Q2 2025: Differential fuzzing + corpus evaluation (security)
- Q3 2025: Complete all major decoders (coverage)
- Q4 2025: External audit + API stability (production-ready)

**Blockers**:
- Academic papers require: proofs + benchmarks
- Production use requires: audit + fuzzing
- v1.0 requires: verification + stability

**Primary audience**: Developers, security researchers, formal methods community

---

### 🎯 [Product Roadmap](./ROADMAP_PRODUCT.md)

**Focus**: Use cases, adoption, market validation

**Key milestones**:
- Q1 2025: 20 user interviews + competitive analysis (validation)
- Q2 2025: 2-3 pilot integrations (proof of value)
- Q3 2025: Evidence of product-market fit (PMF)
- Q4 2025: Self-service onboarding + ecosystem integrations (scale)

**Blockers**:
- Adoption requires: validated use cases
- PMF requires: happy users
- Scale requires: frictionless onboarding

**Primary audience**: Potential users (explorers, compliance, indexers, researchers)

---

### 📣 [Marketing Roadmap](./ROADMAP_MARKETING.md)

**Focus**: Content, conferences, community building

**Key milestones**:
- Q1 2025: 4 blog posts + social presence (awareness)
- Q2 2025: 1 conference talk + guest posts (credibility)
- Q3 2025: Academic paper submitted (academic reputation)
- Q4 2025: Self-sustaining community (longevity)

**Blockers**:
- Content requires: technical milestones complete
- Conferences require: proof + evaluation
- Community requires: active users

**Primary audience**: Broader blockchain community, academics, enthusiasts

---

## How They Connect

### The Dependency Graph

```
TECHNICAL                  PRODUCT                   MARKETING
    │                         │                         │
    │                         │                         │
    ▼                         │                         │
Formal proof ────────────────┼────────────────────────►│
    │                         │                   Blog post 4
    │                         │                         │
    ▼                         │                         │
Benchmarks ──────────────────┼────────────────────────►│
    │                         │                   Conference
    │                         │                   submission
    │                         │                         │
    │                         ▼                         │
    │                   User interviews ────────────────►│
    │                         │                   Case studies
    │                         │                         │
    ▼                         ▼                         │
Decoders complete ──────► Pilot integrations ─────────►│
    │                         │                   User stories
    │                         │                         │
    ▼                         ▼                         ▼
Security audit ──────────► Production use ────────► Community
    │                         │                    growth
    │                         │                         │
    ▼                         ▼                         ▼
  v1.0                      PMF                   Sustainability
```

### Key Interdependencies

**Technical → Product**:
- Users need working decoders before piloting
- Production requires security audit
- PMF requires performance acceptable

**Technical → Marketing**:
- Blog post 4 (verification) requires proof complete
- Conference talks require benchmarks + evaluation
- Academic paper requires full technical completion

**Product → Marketing**:
- Case studies require pilot users
- User testimonials require happy users
- Community growth requires adoption

**Marketing → Product**:
- Content drives awareness → user discovery
- Conferences generate leads → pilot candidates
- Community provides feedback → product improvement

**Marketing → Technical**:
- User feedback → feature prioritization
- Bug reports → quality improvement
- Community contributions → faster development

---

## Q1 2025 Focus (Jan-Mar)

**Theme**: **Credibility**

**Critical path**:
1. ✅ Technical: Complete 1 formal proof (4 weeks) ⭐ HIGHEST PRIORITY
2. ✅ Technical: Benchmark suite (4 weeks) ⭐ CRITICAL
3. ✅ Product: User discovery interviews (8 weeks)
4. ✅ Marketing: Blog post series (4 posts)

**Why credibility first**:
- Can't write academic paper without proofs/benchmarks
- Can't attract pilots without credibility
- Can't speak at conferences without evaluation

**Success criteria**:
- At least 1 complete formal proof
- Benchmark results published
- 20 user interviews completed
- 4 blog posts published
- 100+ GitHub stars

---

## Q2 2025 Focus (Apr-Jun)

**Theme**: **Security & Validation**

**Critical path**:
1. Technical: Differential fuzzing (4 weeks)
2. Technical: Corpus evaluation (4 weeks)
3. Product: 2-3 pilot integrations (12 weeks)
4. Marketing: Conference talk + guest posts

**Why security & validation**:
- Users need trust before production deployment
- Pilots validate product-market fit
- Fuzzing prevents embarrassing bugs

**Success criteria**:
- 10M+ fuzz iterations, 0 bugs
- >99.9% corpus success rate
- 2-3 active pilots
- 1 conference talk delivered

---

## Q3 2025 Focus (Jul-Sep)

**Theme**: **Completeness & PMF**

**Critical path**:
1. Technical: Complete all major decoders
2. Technical: External security audit (if funded)
3. Product: Evidence of PMF (5+ production users)
4. Marketing: Academic paper submission

**Why completeness**:
- Users need coverage of their chains
- Security audit requires stable codebase
- PMF unlocks growth

**Success criteria**:
- 20+ decoders complete
- Security audit passed (if funded)
- 5+ production integrations
- Academic paper submitted

---

## Q4 2025 Focus (Oct-Dec)

**Theme**: **Sustainability**

**Critical path**:
1. Technical: API stabilization + v1.0 release
2. Product: Self-service onboarding
3. Marketing: Self-sustaining community

**Why sustainability**:
- v1.0 signals production-ready
- Self-service enables scale
- Community ensures longevity

**Success criteria**:
- v1.0.0 released
- Self-service docs complete
- 200+ Discord members
- 10+ active contributors

---

## Resource Allocation (If Funded)

**Total budget**: ~$200K/year (ideal)

**Technical** (60% = $120K):
- Security audit: $80K (one-time)
- Developer time: $40K (part-time)

**Product** (20% = $40K):
- User research: $10K
- Pilot support: $20K
- Integration help: $10K

**Marketing** (20% = $40K):
- Content creation: $20K
- Events/conferences: $15K
- Community: $5K

**Realism**: Likely running on $0 budget (volunteers) until grants secured.

---

## Risk Matrix

### Critical Risks (Could Kill Project)

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| No users adopt (no PMF) | High | Medium | User interviews early, pivot if needed |
| Security bug in production | High | Low | Extensive fuzzing, audit before v1.0 |
| Can't complete formal proofs | High | Medium | Start simple, get Verus community help |
| No funding for audit | Medium | Medium | Self-fund or delay v1.0 |

### Medium Risks (Slowdown)

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Performance not competitive | Medium | Low | Profile and optimize (Phase 5.2) |
| Too few chains supported | Medium | Low | Focus on coverage over perfection |
| Community doesn't grow | Medium | Medium | Content marketing, be helpful |

### Low Risks (Acceptable)

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Competitors copy approach | Low | High | Embrace (OSS means copying is fine) |
| Scope creep (encoding requests) | Low | Medium | Stay focused, say no |
| Academic paper rejected | Low | Medium | Revise and resubmit |

---

## Decision Framework

**When making trade-offs, prioritize**:

1. **Credibility > Speed** (better to be slow and correct than fast and wrong)
2. **Users > Features** (solve real problems, not hypothetical ones)
3. **Core > Extensions** (minimal TCB, extensible periphery)
4. **Security > Convenience** (airgapped, verified, audited)
5. **Open > Closed** (transparency builds trust)

**Example decisions**:
- Delay v1.0 for security audit? **Yes** (credibility > speed)
- Add transaction encoding? **No** (core > extensions)
- Skip formal verification? **Never** (security > convenience)

---

## Success Definition

**By end of 2025, we'll know we succeeded if**:

✅ **Technical**: v1.0 released with at least 1 fully verified component
✅ **Product**: 20+ production users who would be "very disappointed" if we shut down
✅ **Marketing**: 2000+ GitHub stars, self-sustaining community, 1 academic paper

**We'll know we failed if**:

❌ No users after 1 year (no PMF)
❌ Critical security bug in production (trust destroyed)
❌ Unable to complete formal proofs (credibility claim false)

---

## Appendix: Related Documents

### Core Documentation
- [ROADMAP_FAQ.md](./ROADMAP_FAQ.md) - Honest answers to hard questions
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical architecture
- [CLAUDE.md](../CLAUDE.md) - Core design principles

### Domain Roadmaps
- [ROADMAP_TECHNICAL.md](./ROADMAP_TECHNICAL.md) - Engineering roadmap
- [ROADMAP_PRODUCT.md](./ROADMAP_PRODUCT.md) - Product & adoption
- [ROADMAP_MARKETING.md](./ROADMAP_MARKETING.md) - Content & community

### Specific Plans
- [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) - Testing pyramid
- [FORMAL_VERIFICATION.md](./FORMAL_VERIFICATION.md) - Verification plan
- [GIT_SUBTREE_VENDORING.md](./GIT_SUBTREE_VENDORING.md) - Dependency strategy

---

**Questions? Concerns? Criticisms?**

We welcome all feedback. Open an issue or start a discussion on GitHub.

**Transparency commitment**: We'll update these roadmaps quarterly and be honest about progress, setbacks, and pivots.

---

**Last Updated**: 2025-11-16
**Status**: Domain-separated roadmaps complete
**Next Update**: 2026-01-16 (quarterly review)
