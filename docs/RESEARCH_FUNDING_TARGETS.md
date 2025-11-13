# Research Funding Targets: Advancing Formal Verification

**Purpose**: Identify specific academics, research groups, and institutions to fund for advancing the Universal Blockchain Decoder's formal verification methodology and its cross-domain applications.

**Total Estimated Funding Need**: $5M-$15M over 5 years

---

## Tier 1: Core Verus & Verification Experts ($3M-$5M)

### 1. **Bryan Parno** - Carnegie Mellon University
**Role**: Verus co-creator, formal verification pioneer

**Expertise**:
- Co-creator of Verus verification framework
- Ironclad project (verified TLS implementation)
- Distributed systems verification
- Cryptographic protocol verification

**Key Publications**:
- "Verus: Verifying Rust Programs using Linear Ghost Types" (OOPSLA 2023)
- "Ironclad Apps: End-to-End Security via Automated Full-System Verification" (OSDI 2014)

**Why Fund**:
- **Direct Verus expertise** - Can improve Verus for parser verification use cases
- **Track record** - Ironclad demonstrated practical verification at scale
- **Industry connections** - Microsoft Research collaboration
- **Proof automation** - Working on reducing proof burden

**Funding Proposal**:
- **Amount**: $1M-$2M over 3 years
- **Deliverables**:
  - Extend Verus for trait-based verification patterns
  - Automated proof generation from property tests
  - Parser verification methodology paper (SOSP/OSDI)
  - 2-3 PhD students working on Universal Decoder

**Contact**: Carnegie Mellon University, Computer Science Department
**Website**: https://www.andrew.cmu.edu/user/bparno/

---

### 2. **Chris Hawblitzel** - Microsoft Research / UC San Diego
**Role**: Verus co-creator, systems verification expert

**Expertise**:
- Co-creator of Verus
- Dafny verification language
- Verified operating systems (Ironclad, IronFleet)
- Concurrent systems verification

**Key Publications**:
- "Verus: Verifying Rust Programs" (OOPSLA 2023)
- "IronFleet: Proving Practical Distributed Systems Correct" (SOSP 2015)
- "Ironclad Apps" (OSDI 2014)

**Why Fund**:
- **Verus tooling** - Can improve IDE integration, error messages
- **Proof engineering** - Knows how to structure large verification projects
- **Performance** - Experience with zero-overhead verified systems
- **MSR resources** - Access to industrial research infrastructure

**Funding Proposal**:
- **Amount**: $800K-$1.5M over 3 years
- **Deliverables**:
  - Verus IDE tooling improvements
  - Incremental verification infrastructure (RP-4)
  - Parser verification case study paper
  - Integration with Universal Decoder CI/CD

**Contact**: Microsoft Research / UC San Diego
**Website**: https://www.microsoft.com/en-us/research/people/chrishaw/

---

### 3. **Andrea Lattuada** - ETH Zurich / Verus team
**Role**: Verus core developer, linear types expert

**Expertise**:
- Verus implementation and tooling
- Linear types for resource management
- Verified systems programming
- Rust verification

**Key Publications**:
- "Verus: Verifying Rust Programs using Linear Ghost Types" (OOPSLA 2023)
- Work on linear types and verification

**Why Fund**:
- **Hands-on Verus development** - Can add features we need
- **Linear types** - Key for proving no-resource-leak properties
- **Parser verification** - Can develop specialized tactics
- **European collaboration** - Opens EU research funding (Horizon Europe)

**Funding Proposal**:
- **Amount**: $500K-$1M over 3 years
- **Deliverables**:
  - Verus parser verification library
  - Bounded-resource verification tactics
  - Automated proof repair (RP-1)
  - European research consortium formation

**Contact**: ETH Zurich, Department of Computer Science
**Website**: https://andrea.lattuada.me/

---

### 4. **Travis Hance** - Carnegie Mellon University
**Role**: Verus developer, concurrent data structures

**Expertise**:
- Verus core implementation
- Concurrent data structure verification
- Proof automation
- Rust verification patterns

**Key Publications**:
- Verus-related papers and implementation work
- Verified concurrent algorithms

**Why Fund**:
- **Deep Verus knowledge** - Knows internals well
- **Proof patterns** - Can create reusable verification templates
- **Performance verification** - Can help with zero-cost abstraction proofs (RP-5)
- **Mentorship** - Can train PhD students on Verus

**Funding Proposal**:
- **Amount**: $400K-$800K over 3 years
- **Deliverables**:
  - Verus proof pattern library for parsers
  - Zero-cost abstraction verification methodology
  - Verification cookbook for decoder developers
  - 1-2 PhD students

**Contact**: Carnegie Mellon University
**Website**: https://www.andrew.cmu.edu/user/thance/

---

## Tier 2: Parser & Protocol Verification Specialists ($1.5M-$3M)

### 5. **Kathleen Fisher** - Tufts University / DARPA
**Role**: Parser generation and verification pioneer

**Expertise**:
- PADS (Parser Annotations for Data Streams)
- Parser security and correctness
- Data description languages
- DARPA program manager (verified software)

**Key Publications**:
- "From Dirt to Shovels: Fully Automatic Tool Generation from Ad Hoc Data" (POPL 2008)
- PADS data description language
- Parser correctness and security

**Why Fund**:
- **Parser domain expert** - 20+ years in parser verification
- **DARPA connections** - Can help secure DARPA funding ($3M-$10M)
- **Tool generation** - Can auto-generate parsers from specs
- **Security focus** - Understands parser security vulnerabilities

**Funding Proposal**:
- **Amount**: $600K-$1.2M over 3 years
- **Deliverables**:
  - Automatic decoder generation from chain specs
  - Parser security analysis framework
  - DARPA proposal collaboration (10x funding multiplier)
  - Parser verification methodology paper (PLDI)

**Contact**: Tufts University, Computer Science Department
**Website**: https://www.cs.tufts.edu/~kfisher/

---

### 6. **Leonidas Lampropoulos** - University of Maryland
**Role**: Property-based testing & verification

**Expertise**:
- QuickChick (property-based testing in Coq)
- Property-based testing → formal specification
- Automated proof generation
- Testing and verification synergy

**Key Publications**:
- "Generating Good Generators for Inductive Relations" (POPL 2018)
- QuickChick framework
- Automated testing and verification

**Why Fund**:
- **Property testing expert** - Perfect for RP-3 (proptest → Verus pipeline)
- **Proof automation** - Can generate specs from tests
- **Tool building** - QuickChick demonstrates tool-building ability
- **Practical verification** - Focuses on making verification accessible

**Funding Proposal**:
- **Amount**: $500K-$1M over 3 years
- **Deliverables**:
  - Automated Verus spec generation from proptests
  - AI-assisted proof generation tooling
  - Property-based verification methodology paper (ICSE)
  - Integration with ai-refactor-suggest tool

**Contact**: University of Maryland, Computer Science Department
**Website**: https://lemonidas.github.io/

---

### 7. **Alastair Donaldson** - Imperial College London
**Role**: Fuzzing and verification integration

**Expertise**:
- Fuzzing (GraphicsFuzz, CLFuzz)
- Automated test generation
- GPU and compiler verification
- Finding bugs at scale

**Key Publications**:
- "Automated Testing of Graphics Shader Compilers" (OOPSLA 2017)
- GraphicsFuzz project (acquired by Google)
- Metamorphic testing

**Why Fund**:
- **Fuzzing expertise** - Can integrate fuzzing with verification
- **Adversarial testing** - Can find edge cases in parsers
- **Industry impact** - GraphicsFuzz used by Google, Apple, ARM
- **Test generation** - Can generate malicious transaction corpus

**Funding Proposal**:
- **Amount**: $400K-$800K over 3 years
- **Deliverables**:
  - Blockchain transaction fuzzer (structure-aware)
  - Adversarial test corpus (100K+ malicious transactions)
  - Fuzzing-guided verification (verify what fuzzing finds)
  - Security vulnerability paper (IEEE S&P)

**Contact**: Imperial College London, Department of Computing
**Website**: https://www.doc.ic.ac.uk/~afd/

---

## Tier 3: Cross-Domain Application Experts ($1M-$2M)

### 8. **Michael Ernst** - University of Washington
**Role**: Program analysis and verification for practitioners

**Expertise**:
- Checker Framework (pluggable type systems)
- Lightweight verification
- Developer tools for verification
- Practical program analysis

**Key Publications**:
- "Practical Pluggable Types for Java" (ISSTA 2008)
- Checker Framework (widely used in industry)
- Developer-centric verification

**Why Fund**:
- **Practical verification** - Focuses on usable tools
- **Industry adoption** - Checker Framework used by Google, Uber, Amazon
- **IDE integration** - Can make Verus accessible to developers
- **Incremental adoption** - Knows how to introduce verification gradually

**Funding Proposal**:
- **Amount**: $400K-$800K over 3 years
- **Deliverables**:
  - Verus IDE plugins (VSCode, IntelliJ)
  - Incremental verification adoption strategy
  - Developer UX study for formal verification
  - Tool paper (FSE/ASE)

**Contact**: University of Washington, Paul G. Allen School
**Website**: https://homes.cs.washington.edu/~mernst/

---

### 9. **Emina Torlak** - University of Washington
**Role**: Solver-aided verification and synthesis

**Expertise**:
- Rosette (solver-aided programming)
- Program synthesis
- SMT solvers for verification
- Automated reasoning

**Key Publications**:
- "Growing Solver-Aided Languages with Rosette" (Onward! 2013)
- "A Lightweight Symbolic Virtual Machine for Solver-Aided Host Languages" (PLDI 2014)
- Rosette framework (widely used)

**Why Fund**:
- **Proof automation** - Can automate proof search with solvers
- **Synthesis** - Can synthesize decoder implementations from specs
- **Tool expertise** - Rosette demonstrates practical tool building
- **Constraint solving** - Can optimize verification performance

**Funding Proposal**:
- **Amount**: $300K-$700K over 3 years
- **Deliverables**:
  - SMT-based proof automation for Verus
  - Decoder synthesis from chain specifications
  - Solver-aided verification toolkit
  - PLDI paper on automated verification

**Contact**: University of Washington, Paul G. Allen School
**Website**: https://homes.cs.washington.edu/~emina/

---

### 10. **Kevin Fisher** - Johns Hopkins Applied Physics Lab
**Role**: Medical device verification and FDA certification

**Expertise**:
- Medical device software verification
- FDA regulatory compliance (510(k), PMA)
- Safety-critical systems
- HL7/FHIR protocol security

**Key Publications**:
- Medical device security research
- FDA guidance on software verification

**Why Fund**:
- **Medical domain** - Can apply methodology to HL7/FHIR parsers
- **Regulatory expertise** - Knows FDA requirements
- **Safety-critical** - Experience with life-critical systems
- **Market access** - Can help penetrate medical device market

**Funding Proposal**:
- **Amount**: $300K-$600K over 3 years
- **Deliverables**:
  - Verified HL7 message parser
  - FDA pre-market approval pathway documentation
  - Medical device verification case study
  - IEEE EMBS paper (medical engineering)

**Contact**: Johns Hopkins Applied Physics Lab
**Website**: https://www.jhuapl.edu/

---

## Tier 4: AI-Assisted Verification ($800K-$1.5M)

### 11. **Karthik Narasimhan** - Princeton University
**Role**: AI for code generation and verification

**Expertise**:
- Large language models for code
- Program synthesis
- Automated reasoning with LLMs
- CodeT5, CodeBERT applications

**Key Publications**:
- "Evaluating Large Language Models for Code" (multiple papers)
- AI-assisted programming research

**Why Fund**:
- **LLM for proofs** - Can improve AI-assisted proof generation
- **Claude integration** - Can optimize prompts for ai-refactor-suggest
- **Cost reduction** - LLMs can reduce proof engineering cost
- **Accessibility** - Makes verification accessible to non-experts

**Funding Proposal**:
- **Amount**: $400K-$800K over 3 years
- **Deliverables**:
  - LLM fine-tuning for Verus proof generation
  - AI-assisted proof repair (RP-1)
  - Evaluation of LLM verification quality
  - ICLR/NeurIPS paper on AI for verification

**Contact**: Princeton University, Computer Science Department
**Website**: https://www.cs.princeton.edu/~karthikn/

---

### 12. **Swarat Chaudhuri** - University of Texas at Austin
**Role**: Program synthesis and neuro-symbolic AI

**Expertise**:
- Program synthesis
- Neuro-symbolic reasoning
- Automated verification
- Combining learning and logic

**Key Publications**:
- "Neurosymbolic Programming" (Foundations and Trends, 2021)
- Program synthesis with ML
- Automated reasoning

**Why Fund**:
- **Neuro-symbolic** - Combines ML and formal methods
- **Synthesis** - Can generate verified parsers
- **Probabilistic verification** - Relevant to RP-2
- **Innovation** - Cutting-edge approach

**Funding Proposal**:
- **Amount**: $400K-$700K over 3 years
- **Deliverables**:
  - Neuro-symbolic proof generation
  - Probabilistic verification methodology (RP-2)
  - Learned proof tactics for parsers
  - POPL paper on ML-assisted verification

**Contact**: UT Austin, Computer Science Department
**Website**: https://www.cs.utexas.edu/~swarat/

---

## Tier 5: Systems & Performance Verification ($1M-$2M)

### 13. **Xi Wang** - University of Washington
**Role**: Systems verification, OS and network

**Expertise**:
- Operating system verification
- Network protocol verification
- Bug finding in systems software
- Verified compilation

**Key Publications**:
- "Jitk: A Trustworthy In-Kernel Interpreter Infrastructure" (OSDI 2014)
- "Using Crash Hoare Logic for Certifying the FSCQ File System" (SOSP 2015)

**Why Fund**:
- **Systems expertise** - Can apply to blockchain systems
- **Network protocols** - Relevant for P2P protocol verification
- **Performance** - Understands systems performance constraints
- **Practical systems** - Focus on real-world systems

**Funding Proposal**:
- **Amount**: $500K-$1M over 3 years
- **Deliverables**:
  - Verified blockchain P2P protocol stack
  - Network protocol family verification methodology
  - Zero-overhead verification case study
  - OSDI/SOSP paper

**Contact**: University of Washington, Paul G. Allen School
**Website**: https://homes.cs.washington.edu/~xi/

---

### 14. **Ranjit Jhala** - University of California, San Diego
**Role**: Liquid types, refinement types, verification

**Expertise**:
- Liquid types (refinement types for Haskell)
- Automated verification
- SMT-based verification
- Type systems for verification

**Key Publications**:
- "Liquid Types" (PLDI 2008)
- "Refinement Types for Haskell" (ICFP 2014)
- LiquidHaskell tool

**Why Fund**:
- **Refinement types** - Similar to Verus linear types
- **Automated verification** - Reduces annotation burden
- **Type systems** - Can improve Verus type system
- **UCSD connection** - Local to Verus development

**Funding Proposal**:
- **Amount**: $500K-$1M over 3 years
- **Deliverables**:
  - Refinement type extensions for Verus
  - Automated invariant inference for parsers
  - Type-driven verification methodology
  - PLDI paper on refinement types for Rust

**Contact**: UC San Diego, Computer Science and Engineering
**Website**: https://ranjitjhala.github.io/

---

## Research Institutions & Labs

### 15. **CMU Software Engineering Institute (SEI)**
**Focus**: Critical infrastructure security and verification

**Relevant Programs**:
- Assured Software Development
- Formal Methods for Security
- Critical Infrastructure Protection

**Why Partner**:
- **Government connections** - Can help with DARPA, NSF funding
- **Industry relationships** - Works with DoD, financial sector
- **Applied research** - Focus on practical deployment
- **Certification** - Expertise in security certification

**Funding Proposal**:
- **Amount**: $500K-$1M over 3 years
- **Deliverables**:
  - Industry deployment case studies
  - Security certification guidance (Common Criteria)
  - Government adoption strategy
  - Technical reports and standards documents

**Contact**: https://www.sei.cmu.edu/

---

### 16. **MIT CSAIL - Programming Languages & Software Engineering Group**
**Focus**: Programming languages, verification, synthesis

**Key Faculty**:
- Armando Solar-Lezama (program synthesis)
- Martin Rinard (program analysis)
- Adam Chlipala (verified compilers)

**Why Partner**:
- **World-class PL research** - Multiple verification experts
- **Compiler verification** - Adam Chlipala's Fiat-Crypto
- **Synthesis** - Can auto-generate verified parsers
- **Infrastructure** - Access to research infrastructure

**Funding Proposal**:
- **Amount**: $800K-$1.5M over 3 years
- **Deliverables**:
  - Verified parser synthesis framework
  - Cryptographic protocol verification extensions
  - PLDI/POPL papers (3-5)
  - PhD student collaboration (2-3 students)

**Contact**: https://www.csail.mit.edu/research/programming-languages-software-engineering

---

### 17. **INRIA (France) - Prosecco Team**
**Focus**: Cryptographic protocol verification

**Key Researchers**:
- Karthik Bhargavan (HACL*, EverCrypt)
- Bruno Blanchet (ProVerif)

**Why Partner**:
- **HACL* experience** - Already verified crypto in production (Firefox, Linux kernel)
- **F* verification** - Similar to Verus
- **European funding** - Access to Horizon Europe grants (€2M-€10M)
- **International collaboration** - Expands research network

**Funding Proposal**:
- **Amount**: $400K-$800K over 3 years (+ EU co-funding)
- **Deliverables**:
  - Cryptographic signature verification for TxIR
  - F* to Verus translation tools
  - Cross-language verification methodology (RP-3)
  - Horizon Europe grant applications

**Contact**: https://prosecco.inria.fr/

---

## Postdoctoral & PhD Student Funding

### **Postdoc Positions** (4-6 positions)
**Cost**: $80K-$100K per postdoc per year
**Total**: $1M-$1.8M over 3 years

**Research Areas**:
1. Verus parser verification tactics (2 postdocs)
2. AI-assisted proof generation (1 postdoc)
3. Cross-domain applications (medical, financial, IoT) (2 postdocs)
4. Performance verification and optimization (1 postdoc)

**Hosting Institutions**: CMU, UW, UCSD, UT Austin

---

### **PhD Student Support** (10-15 students)
**Cost**: $40K-$50K per student per year
**Total**: $1.8M-$3.4M over 4.5 years (average PhD duration)

**Thesis Topics**:
1. Compositional verification of protocol families (2 students)
2. Canonical serialization verification (2 students)
3. Property-based testing → verification pipeline (2 students)
4. Automated proof repair and maintenance (2 students)
5. Cross-domain applications (medical, IoT, finance) (3 students)
6. Zero-cost abstraction verification (2 students)
7. AI-assisted verification (2 students)

**Target Institutions**: CMU, UC San Diego, University of Washington, UT Austin, MIT, Princeton

---

## Industry Research Partnerships

### 18. **Microsoft Research**
**Relevant Groups**: RiSE (Research in Software Engineering), Programming Languages

**Key Researchers**:
- Rustan Leino (Dafny)
- K. Rustan M. Leino (verification methodology)

**Why Partner**:
- **Dafny to Verus** - Can share verification patterns
- **Industry scale** - Verification at Windows/Azure scale
- **Resources** - Compute infrastructure for large-scale verification
- **Z3 SMT solver** - Core dependency for Verus

**Funding Proposal**:
- **Amount**: $0 (in-kind collaboration) to $500K (joint research)
- **Deliverables**:
  - Z3 optimizations for parser verification
  - Dafny/Verus interoperability
  - Industrial case studies
  - MSR technical reports

**Contact**: https://www.microsoft.com/en-us/research/

---

### 19. **Amazon Web Services (AWS) Automated Reasoning Group**
**Focus**: Formal verification for cloud services

**Key Projects**:
- s2n (verified TLS implementation)
- Firecracker (verified microVM)
- Cedar (verified authorization)

**Why Partner**:
- **Production verification** - Already using formal methods at scale
- **Cloud deployment** - Can host verified decoder as AWS service
- **Customer base** - Direct access to enterprises
- **Funding** - Can fund research + provide AWS credits

**Funding Proposal**:
- **Amount**: $300K-$800K over 3 years + AWS credits
- **Deliverables**:
  - AWS Lambda deployment of verified decoder
  - S2N-style verification methodology adaptation
  - Joint case study paper
  - AWS blog posts and evangelism

**Contact**: https://aws.amazon.com/security/provable-security/

---

## Funding Strategy & Timeline

### **Phase 1 (Year 1): Core Infrastructure** - $1.5M-$2.5M
**Focus**: Verus improvements, parser verification methodology

**Target Researchers**:
- Bryan Parno (CMU) - $400K
- Chris Hawblitzel (MSR/UCSD) - $300K
- Andrea Lattuada (ETH) - $200K
- Leonidas Lampropoulos (UMD) - $200K
- 3-4 postdocs - $300K
- 5-6 PhD students - $400K-$600K

**Deliverables**:
- Verus parser verification library
- Automated spec generation from property tests
- First verified decoder (Bitcoin)
- 2-3 conference papers

---

### **Phase 2 (Year 2): Scaling & Applications** - $2M-$3.5M
**Focus**: Cross-domain applications, AI-assisted verification

**Target Researchers**:
- Kathleen Fisher (Tufts/DARPA) - $400K
- Karthik Narasimhan (Princeton) - $300K
- Alastair Donaldson (Imperial) - $200K
- Michael Ernst (UW) - $200K
- Kevin Fisher (JHU APL) - $200K
- 2-3 postdocs - $200K-$300K
- 5-7 PhD students - $500K-$700K

**Deliverables**:
- Verified SWIFT/HL7 parsers (cross-domain)
- AI-assisted proof generation tool
- Adversarial test corpus
- DARPA proposal submission
- 4-6 conference papers

---

### **Phase 3 (Year 3-5): Ecosystem & Standardization** - $1.5M-$4M
**Focus**: Industry adoption, standardization, PhD graduations

**Target Researchers**:
- Xi Wang (UW) - $500K
- Ranjit Jhala (UCSD) - $400K
- Emina Torlak (UW) - $300K
- Swarat Chaudhuri (UT Austin) - $300K
- CMU SEI - $400K
- MIT CSAIL - $800K
- INRIA Prosecco - $300K
- 2-3 postdocs - $200K-$300K
- PhD student continuations - $800K-$1.2M

**Deliverables**:
- ISO/IEC standardization proposal
- 10+ verified decoder families
- Industry deployment case studies
- 8-12 PhD theses
- 10-15 conference papers
- ACM Software System Award nomination

---

## Grant Application Targets

### **NSF Grants** (3-5 proposals)
- **FMitF** (Formal Methods in the Field): $500K-$1M each
- **SaTC** (Secure and Trustworthy Cyberspace): $500K-$1M each
- **CPS** (Cyber-Physical Systems): $500K-$1M for IoT/medical

**Target Amount**: $2M-$5M over 5 years

---

### **DARPA Programs** (1-2 proposals)
- **CHESS** (Computers and Humans Exploring Software Security)
- **SIEVE** (Securing Information for Encrypted Verification and Evaluation)

**Target Amount**: $3M-$10M per program (if accepted)

---

### **DOE Grants** (1-2 proposals)
- **ASCR** (Advanced Scientific Computing Research)
- Focus: High-assurance software for scientific computing

**Target Amount**: $1M-$3M over 5 years

---

### **NIH Grants** (1 proposal)
- **Medical Device Cyber Security Program**
- Focus: Verified HL7/FHIR parsers

**Target Amount**: $1M-$3M over 5 years

---

### **European Union (Horizon Europe)** (1-2 proposals)
- **ERC Starting Grant** (for early-career researchers)
- **Marie Skłodowska-Curie Actions** (postdoc fellowships)

**Target Amount**: €1M-€3M over 5 years

---

## Return on Investment (ROI)

### **Research Funding ROI**
- **Direct funding**: $5M-$15M
- **Leveraged funding** (grants): $10M-$25M
- **Total research budget**: $15M-$40M

### **Publications ROI**
- **Target**: 15-20 papers at top-tier venues
- **Citations**: 500-2000 citations (5-year estimate)
- **Impact**: Establishes Universal Decoder as standard methodology

### **Talent ROI**
- **PhD graduates**: 10-15 experts in verified parsers
- **Postdocs**: 4-6 early-career researchers
- **Industry pipeline**: 50% go to tech companies (Google, Microsoft, Amazon)
- **Academic pipeline**: 30% become professors (multiplies impact)

### **Intellectual Property ROI**
- **Patents**: 3-5 patents on verification methodology
- **Open source**: All code MIT/Apache (maximizes adoption)
- **Standards**: ISO/IEC standard → regulatory requirement

### **Commercial ROI**
- **Industry adoption**: 3+ financial institutions using methodology
- **Market validation**: Verification becomes competitive requirement
- **Revenue acceleration**: Research prestige → easier enterprise sales

---

## Summary: Recommended Immediate Funding

### **Top 5 Priority Targets** (Year 1) - $1.8M-$3M

1. **Bryan Parno (CMU)** - $400K-$600K
   - Core Verus expertise, proven track record

2. **Chris Hawblitzel (MSR/UCSD)** - $300K-$500K
   - Verus tooling and IDE integration

3. **Leonidas Lampropoulos (UMD)** - $200K-$400K
   - Property testing → verification pipeline (RP-3)

4. **Kathleen Fisher (Tufts)** - $300K-$500K
   - DARPA connections, parser domain expertise

5. **PhD Student Pool (4-6 students)** - $600K-$1M
   - Long-term investment, high ROI

**Next Steps**:
1. Reach out to researchers with project overview
2. Schedule collaboration meetings
3. Draft grant proposals (NSF FMitF, DARPA)
4. Submit funding applications Q2 2025
5. Begin research collaborations Q3 2025

---

**Last Updated**: 2025-11-13
**Contact**: [Your contact info]
