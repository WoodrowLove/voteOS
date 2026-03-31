# VoteOS Architecture Positioning

> Definitive statement of what VoteOS is, how it stands in the ecosystem,
> and why it must never be reduced to a feature of another system.

---

## What VoteOS IS

VoteOS is a **sovereign decision legitimacy system**.

It is a complete, independent platform for:
- **Elections** — public office, referenda, recalls, initiatives
- **Governance proposals** — resolutions, measures, policy questions
- **Certification** — attested, auditable decision outcomes
- **Audit** — independent evidence reconstruction and verification
- **Privacy-aware voting** — ballot secrecy with vote verification

VoteOS enforces a strict truth standard:
- Every vote must be traceable to an eligible voter
- Every tally must be reconstructible from evidence
- Every result must be certified with attestation
- Every step must be independently auditable
- Ambiguity is a legitimate outcome, never suppressed

VoteOS is **neutral**. It does not care about outcomes. It cares about process integrity.

---

## What VoteOS IS NOT

| VoteOS is NOT | Why this matters |
|---------------|-----------------|
| A CivilOS module or submodule | VoteOS is a sovereign sibling, not a child component |
| A generic admin panel | It enforces election-specific integrity rules that admin panels don't have |
| A simple poll or survey app | Polls don't require eligibility proof, attestation, or audit trails |
| A hidden execution engine | VoteOS certifies decisions — it does not implement policy |
| A duplicate citizen identity database | AxiaSystem owns identity; VoteOS verifies eligibility |
| A governance feature set inside CivilOS | CivilOS has its own governance module for operational decisions; VoteOS handles democratic elections with stricter rules |
| A tightly coupled subsystem | VoteOS and CivilOS interoperate but neither owns the other |

---

## Why VoteOS Must Be Sovereign

### 1. Trust separation
Election outcomes must not be influenced by the system that executes their consequences. If VoteOS were inside CivilOS, the operational system could theoretically influence the decision system. Sovereignty prevents this.

### 2. Stricter integrity requirements
Elections require stronger guarantees than city operations:
- Ballot secrecy (no equivalent in permits or DMV)
- Double-vote prevention (no equivalent in city services)
- Tally reconstructibility (results must be independently verifiable)
- Ambiguity handling (ties are not errors)

### 3. Independent deployment
A jurisdiction may need elections without city operations (e.g., a school board, homeowners association, tribal government). VoteOS must work without CivilOS.

### 4. Regulatory independence
Election systems may be subject to different regulatory requirements than operational systems. Keeping them separate makes compliance cleaner.

---

## How VoteOS Stands on Its Own

Even without CivilOS, VoteOS is a complete system:

```
VoteOS + AxiaSystem = Fully functional election platform

Citizens → AxiaSystem (identity/legitimacy)
Elections → VoteOS (lifecycle, ballots, votes, tallies, certification)
Audit → VoteOS (evidence, observers, recounts)
Results → VoteOS (certified, attested, published)
```

No CivilOS component is required for VoteOS to function.

---

## Ecosystem Position

```
                      AxiaSystem
                  (Shared Truth Root)
                 /         |         \
            CivilOS     VoteOS    [Future Systems]
          (City Ops)  (Elections)  (Education, Health, etc.)
```

Each system:
- Consumes AxiaSystem for identity, legitimacy, attestation
- Owns its domain-specific state
- Publishes outcomes as attested records
- Does not directly call sibling systems

---

## The Decision / Execution Boundary (Fundamental)

This is the most important architectural principle in VoteOS:

**VoteOS decides. It never executes.**

| VoteOS produces | Someone else acts on |
|----------------|---------------------|
| "Budget referendum passed with 67% yes" | CivilOS implements budget change |
| "Jane Smith elected as mayor" | CivilOS updates governance structure |
| "Recall failed — insufficient votes" | No action needed |
| "Resolution 2026-03 approved by council" | CivilOS processes resolution |

VoteOS is not involved in what happens after certification.
Its job ends when the result is certified and published.
