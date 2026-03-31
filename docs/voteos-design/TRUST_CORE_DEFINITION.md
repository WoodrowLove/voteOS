# VoteOS Trust Core Definition

> Produced: 2026-03-30 (ground-truth realignment session)
> Purpose: Define the minimum set of modules that make VoteOS trustworthy as a decision legitimacy system

---

## What "Trust Core" Means

The trust core is the minimum system slice that, once proven, makes VoteOS trustworthy enough to certify real election outcomes. Feature count does not matter. What matters:

> "This election result was produced by a deterministic process, from verified votes, cast by eligible voters, counted without error, and independently auditable."

---

## The Trust Core: 7 Modules

### Tier 1 — Foundation (Modules 1–2)
*Who votes and what they're voting on*

| Module | Trust Role | Why Core |
|--------|-----------|----------|
| 1. Voter Registry | Eligibility gate | Without verified eligibility, no vote has legitimacy |
| 2. Election Management | Lifecycle authority | Without strict state machine, election boundaries are ambiguous |

### Tier 2 — Action (Modules 3–4)
*What voters choose from and how they choose*

| Module | Trust Role | Why Core |
|--------|-----------|----------|
| 3. Ballot Operations | Content integrity | Without ballot integrity, voters may choose from tampered options |
| 4. Vote Recording | Secrecy + double-vote prevention | Without these, the election is fundamentally broken |

### Tier 3 — Computation (Modules 5–6)
*What the result is and who certifies it*

| Module | Trust Role | Why Core |
|--------|-----------|----------|
| 5. Tally Engine | Deterministic computation | Without deterministic tallying, the same votes could yield different results |
| 6. Result Certification | Authority + finality | Without certification, results are claims, not commitments |

### Tier 4 — Verification (Module 8)
*Can anyone independently verify the result?*

| Module | Trust Role | Why Core |
|--------|-----------|----------|
| 8. Audit & Oversight | Independent verification | Without audit capability, trust is based on authority, not evidence |

---

## Why These 7 and Not the Full 10

| Module | Role | Why NOT Core |
|--------|------|-------------|
| 7. Governance Proposals | Extended election types | Features, not trust. General elections work without proposals. |
| 9. Election Operations | Operational support | Supports running elections, not result trustworthiness. |
| 10. Integration & Export | External connectivity | Distribution concern. Results are trustworthy before export. |

---

## Trust Chain

Each link depends on the one before it:

```
1. Voter Registry           ← Who is eligible?
   ↓
2. Election Management      ← What is being decided? Is voting open?
   ↓
3. Ballot Operations        ← What are the choices? Untampered?
   ↓
4. Vote Recording           ← Did eligible voters cast valid votes? Secrecy maintained?
   ↓
5. Tally Engine             ← What is the deterministic result?
   ↓
6. Result Certification     ← Who certifies? Is there finality?
   ↓
8. Audit & Oversight        ← Can anyone independently verify all of the above?
```

Breaking any link breaks the chain.

---

## Trust Core Proofs Required

| Proof | Module | Standard |
|-------|--------|----------|
| ELIGIBILITY_PROVEN | 1 | Ineligible voters cannot register; requires AxiaSystem identity |
| LIFECYCLE_PROVEN | 2 | Invalid transitions rejected; all transitions attested |
| BALLOT_INTEGRITY_PROVEN | 3 | Content unchanged after finalization (hash verification) |
| DOUBLE_VOTE_PREVENTION_PROVEN | 4 | No voter votes twice per election |
| SECRECY_PROVEN | 4 | Vote content not linkable to voter in secret ballot mode |
| TALLY_DETERMINISM_PROVEN | 5 | Same sealed votes → same result, always |
| AMBIGUITY_HANDLED | 5 | Ties and ambiguity declared, never suppressed |
| CERTIFICATION_CHAIN_PROVEN | 6 | Result traceable from individual votes through tally to certification |
| AUDIT_RECONSTRUCTION_PROVEN | 8 | Result independently reproducible from evidence alone |

**Current proof state:**
- ELIGIBILITY_PROVEN: Domain level only ✓
- LIFECYCLE_PROVEN: Domain level only ✓
- BALLOT_INTEGRITY_PROVEN: Domain level only ✓
- DOUBLE_VOTE_PREVENTION_PROVEN: Domain level only ✓
- SECRECY_PROVEN: Structural level ✓ (VoteRecord has no voter_ref)
- TALLY_DETERMINISM_PROVEN: NOT STARTED
- AMBIGUITY_HANDLED: NOT STARTED
- CERTIFICATION_CHAIN_PROVEN: NOT STARTED
- AUDIT_RECONSTRUCTION_PROVEN: NOT STARTED

**5 of 9 proofs demonstrated at domain level. 4 require modules 5, 6, and 8.**

---

## Minimum Viable Trust Core

Can demonstrate election integrity in a controlled environment:
- Modules 1–6 implemented with domain tests
- One complete election lifecycle proven at domain level:
  register → create election → ballot → issue → cast vote → seal → tally → certify

## Full Trust Core

Ready for real elections:
- Modules 1–6 + 8 proven against live AxiaSystem
- All 9 proofs at workflow level
- API boundary exists and tested
- Audit reconstruction independently verified
- Operator documentation complete

---

## Decision / Execution Boundary (Invariant)

The trust core produces **certified decisions**. It never implements them.

VoteOS says: *"Candidate A won with 52.3% in a valid election. Certified, attested, auditable."*

CivilOS (or any consumer) acts on that. VoteOS has no authority over what happens next. The system that counts votes must not be the system that implements the winner's authority.
