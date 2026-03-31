# VoteOS Corrected Build Order

> Produced: 2026-03-30 (ground-truth realignment session)
> Previous plan: MODULE_SEQUENCE_PLAN.md (5 waves)
> This document: Refined order based on ground-truth audit

---

## Assessment of Previous Wave Plan

Original 5-wave plan:
1. Voter Registry + Election Management → DONE
2. Ballot Operations + Vote Recording → DONE
3. Tally Engine + Result Certification
4. Governance Proposals + Audit & Oversight
5. Election Operations + Integration & Export

**Verdict: The wave order is fundamentally correct. No re-sequencing of module dependencies needed.**

What changes: what each wave must include, and a trust gate is inserted between Waves 3 and 4.

---

## Corrected Wave Definitions

### Wave 3: Tally Engine + Result Certification (NEXT)

**Modules:** 5 (Tally Engine) + 6 (Result Certification)

**Why this is next:** The trust core is incomplete without deterministic tallying and result certification. Modules 1–4 handle input (who votes, how). Modules 5–6 handle output (what the result is, who certifies it).

#### Module 5 — Tally Engine (9 capabilities)

Domain types needed:
- `TallyResult` — per-election result container
- `TallyEntry` — per-item result (winner, vote counts, percentages)
- `TallyMethod` — computation strategy
- `TallyAuditEntry` — audit trail for computation steps
- `AmbiguityRecord` — tie detection and declaration

Key capabilities:
- `compute_tally` — deterministic aggregation of sealed vote contents
- `apply_voting_method` — plurality first, ranked choice later
- `evaluate_thresholds` — participation + margin thresholds
- `detect_tie` — explicit tie handling
- `handle_ambiguity` — ambiguity is legitimate, never suppressed
- `verify_tally_determinism` — same input → same output proof

Critical constraint: The tally engine is **pure computation**. It takes sealed vote contents from `VoteRegistry::sealed_contents()` and produces a deterministic result. No side effects, no randomness.

#### Module 6 — Result Certification (8 capabilities)

Domain types needed:
- `CertificationRecord` — who certified, when, evidence
- `CertificationStatus` — Pending, Certified, Contested, Recertified
- `Contest` — challenge to a certified result
- `ContestResolution` — adjudication outcome

Key capabilities:
- `certify_result` — governance action, elevated legitimacy
- `contest_result` — challenge mechanism
- `resolve_contest` — adjudication
- `publish_result` — make certified result available
- `generate_result_bundle` — evidence package for audit

**Wave 3 gate criteria:**
- [ ] Tally computed deterministically from sealed vote contents
- [ ] Plurality method proven (same votes → same result, repeated)
- [ ] Tie detection works (equal counts detected and declared)
- [ ] Threshold evaluation works (participation + margin)
- [ ] Ambiguity declared, not suppressed
- [ ] Result certified with AxiaSystem attestation
- [ ] Contest mechanism works
- [ ] Election transitions: Closed → Tallied → Certified

---

### Wave 3.5: End-to-End Domain Proof (NEW — trust gate)

**Not a module wave. A proof wave.**

Before Wave 4, VoteOS must demonstrate a complete election lifecycle at domain level:

1. Create election (Draft)
2. Register voters (verify eligibility)
3. Create ballot template, add items, finalize
4. Issue ballots to registered voters
5. Cast votes (double-vote prevention, secrecy)
6. Seal votes
7. Close election
8. Compute tally (deterministic)
9. Certify result
10. Verify audit trail reconstructs result

Single integration test proving the trust core works end-to-end at domain level. Does not require AxiaSystem.

**Gate criteria:**
- [ ] One complete election lifecycle: creation → certification
- [ ] Tally matches manually computed expected result
- [ ] Audit trail complete for every step
- [ ] Double-vote prevention holds
- [ ] Secrecy holds (content not linkable to voter)

---

### Wave 4: Audit & Oversight (REVISED — promoted to solo wave)

**Module:** 8 (Audit & Oversight)

**Why revised:** Original plan paired Audit with Governance Proposals. But Audit is trust-core; Governance Proposals are not. Audit must come immediately after computation is proven.

Wave 4 must include:
- Cross-module audit trail reconstruction
- Observer access (process monitoring, no individual votes)
- Recount capability (rerun tally from sealed votes)
- Evidence bundle export (full chain for external audit)
- Dispute resolution tracking

**Gate criteria:**
- [ ] Audit trail reconstructs result independently
- [ ] Observer monitors state without accessing individual votes
- [ ] Recount produces identical result
- [ ] Evidence bundle is self-contained and verifiable

---

### Wave 5: Governance Proposals + Integration & Export

**Modules:** 7 (Governance Proposals) + 10 (Integration & Export)

Non-core extensions built on proven trust core.

**Gate criteria:**
- [ ] Proposal lifecycle works (submit → review → ballot → vote → certify)
- [ ] At least one export format works (JSON bundle or webhook)
- [ ] CivilOS delivery path defined (via AxiaSystem attestation)

---

### Wave 6: Election Operations + Runtime Boundary

**Modules:** 9 (Election Operations) + API layer + deployment

Includes non-module work:
- HTTP API server (axum routes wrapping workflow functions)
- CLI for election officials
- Deployment configuration
- Health checks and monitoring
- Operator documentation

---

## Summary

| Wave | Modules | Purpose | Status |
|------|---------|---------|--------|
| 1 | 1 + 2 | Foundation | BUILD_COMPLETE |
| 2 | 3 + 4 | Core action | BUILD_COMPLETE |
| **3** | **5 + 6** | **Results** | **NEXT** |
| **3.5** | **—** | **E2E domain proof** | **TRUST GATE** |
| **4** | **8** | **Audit** | Trust verification |
| 5 | 7 + 10 | Extensions | Non-blocking |
| 6 | 9 + API | Operations + runtime | System boundary |

---

## Changes from Original Plan

1. Modules 1–4 downgraded from CONDITIONALLY_COMPLETE → BUILD_COMPLETE
2. Wave 3.5 inserted as trust gate (end-to-end domain proof)
3. Module 8 (Audit) promoted to solo Wave 4 (trust-critical)
4. Module 7 (Governance) moved to Wave 5 (non-core)
5. Wave 6 added for runtime boundary (API + deployment)
6. Each wave gate criteria made explicit
