# VoteOS System Intent

> The foundational truth contract for VoteOS.
> Every design decision, capability, and operational behavior must be traceable to these invariants.

---

## What VoteOS Is

VoteOS is a sovereign decision, election, and governance legitimacy system.

It manages the entire lifecycle of democratic decision-making:
- voter eligibility verification
- ballot creation and distribution
- secure vote recording
- result aggregation and certification
- audit trail for election validity

VoteOS is NOT a city operating system. That is CivilOS.
VoteOS is NOT a general-purpose governance tool.

VoteOS is the system that ensures **decisions are legitimate, auditable, and honest**.

---

## What VoteOS Is NOT

- NOT a CivilOS module or submodule
- NOT a replacement for CivilOS governance capabilities
- NOT a social media voting platform
- NOT a poll or survey system
- NOT a system that makes decisions — it records and certifies them
- NOT a system that executes policy — it validates democratic authority for policy

---

## Core Invariants

### 1. No Election Without Eligibility Proof

Every vote recorded in VoteOS must be traceable to a verified eligible voter.
Eligibility is determined by AxiaSystem identity truth, not by VoteOS itself.

No anonymous votes. No unverifiable voters. No assumed eligibility.

### 2. No Tally Without Reconstructible Evidence

Every election result must be independently reconstructible from recorded votes.
The tally is a computation over evidence, not an assertion.

If the evidence cannot reproduce the tally, the tally is invalid.

### 3. No Ambiguous Winner Resolution

If an election produces an ambiguous result (tie, contested, insufficient participation), VoteOS must:
- declare the ambiguity explicitly
- not choose a winner
- not resolve the ambiguity silently

Ambiguity is a legitimate outcome, not a bug.

### 4. No Hidden Authority Over Election State

Every state change in an election lifecycle must be:
- attributable to a specific actor
- evaluated for legitimacy
- attested
- explainable

No election can be opened, closed, invalidated, or certified without recorded authority.

### 5. No Silent Vote Acceptance or Rejection

Every vote submission must produce an explicit outcome:
- accepted (with receipt)
- rejected (with reason)
- deferred (with explanation)

No vote may be silently dropped, duplicated, or modified.

### 6. No Privacy Mode Ambiguity

VoteOS must distinguish clearly between:
- **ballot secrecy**: voter's choice is not linked to their identity in the tally
- **vote verification**: voter can confirm their vote was recorded correctly
- **eligibility transparency**: voter's right to vote is provable

These are not contradictions. They are separate concerns that must be independently satisfied.

### 7. Explicit Separation: Decision vs. Execution

VoteOS decides nothing. It records decisions made by voters.

The distinction:
- **Decision formation**: voters choose (VoteOS records this)
- **Operational execution**: city implements the decision (CivilOS or other systems act on it)

VoteOS produces certified outcomes. It does not execute policy.

### 8. Truth Over Convenience

If the truth is complex, ambiguous, or uncomfortable:
- record it accurately
- do not simplify it for convenience
- do not round results
- do not suppress minority outcomes

### 9. Deterministic Behavior

Same input → same output. Always.

No randomness in tallying. No non-deterministic eligibility checks.
No results that change based on timing or order of processing.

### 10. Legitimacy Spine

Every protected operation in VoteOS follows:

```
evaluate_legitimacy → [action] → attest_action → explain_decision
```

This is inherited from AxiaSystem. No VoteOS operation bypasses this chain.

---

## Relationship to AxiaSystem

VoteOS depends on AxiaSystem for:
- **Identity truth**: who is this person? (resolve_subject)
- **Authentication**: is this person who they claim to be? (authenticate_subject)
- **Legitimacy evaluation**: is this person authorized for this action? (evaluate_legitimacy)
- **Attestation**: create tamper-evident record of action (attest_action)
- **Explanation**: generate audit explanation of decision (explain_decision)

VoteOS does NOT:
- maintain its own identity database
- authenticate users independently
- define its own legitimacy model

It consumes AxiaSystem truth and adds election-specific domain logic on top.

---

## Relationship to CivilOS

VoteOS and CivilOS are sovereign siblings:
- Both depend on AxiaSystem
- Both can be deployed for the same city
- A citizen onboarded in CivilOS is already known to AxiaSystem
- When VoteOS is deployed, that citizen's identity carries over automatically
- VoteOS does NOT re-onboard citizens — it verifies eligibility against existing identity

CivilOS may consume VoteOS outcomes:
- "The budget referendum passed" → CivilOS governance module implements it
- "The new mayor was elected" → CivilOS updates governance structure

VoteOS remains neutral. It does not care what CivilOS does with the result.

---

## Citizen Identity Persistence

When a city deploys CivilOS first:
- Citizens are onboarded via AxiaSystem (resolve_subject)
- Identity truth is established (assurance levels, standing, roles)

When the same city later deploys VoteOS:
- Those citizens already exist in AxiaSystem
- VoteOS verifies their eligibility using existing identity
- No duplicate onboarding, no parallel identity systems
- The citizen's assurance level, standing, and roles carry over

This is the shared-truth model. AxiaSystem is the single source of identity truth for both systems.
