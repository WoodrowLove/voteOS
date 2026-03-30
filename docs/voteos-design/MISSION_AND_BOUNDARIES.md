# VoteOS Mission and Boundaries

> Defines what VoteOS is, what it is not, and where its authority begins and ends.

---

## Mission

VoteOS exists to make democratic decisions **legitimate, auditable, and honest**.

It manages the complete lifecycle of structured decision-making:
- Who is eligible to participate
- What is being decided
- How votes are recorded
- How results are computed
- How outcomes are certified
- How the entire process can be independently verified

---

## What VoteOS IS

| Capability | Description |
|-----------|-------------|
| Election platform | Creates, configures, and runs elections of all types |
| Governance proposal system | Manages measures, referenda, resolutions for formal decision |
| Eligibility engine | Verifies voter rights against AxiaSystem identity truth |
| Vote recording system | Securely accepts, validates, and stores votes |
| Tally engine | Deterministically computes results from evidence |
| Certification authority | Produces attested, auditable election outcomes |
| Audit platform | Enables independent verification of every step |
| Privacy enforcer | Separates ballot content from voter identity when required |

---

## What VoteOS IS NOT

| Not This | Why |
|----------|-----|
| Identity system | AxiaSystem owns identity. VoteOS consumes it. |
| City operations platform | CivilOS handles permits, DMV, finance, public safety. |
| Policy execution engine | VoteOS certifies decisions. Others implement them. |
| Social media poll | VoteOS elections have eligibility, attestation, and legal weight. |
| Survey tool | Surveys don't require eligibility proof or audit trails. |
| General governance module | CivilOS has its own governance module for operational decisions. VoteOS handles democratic elections. |
| Replacement for in-person voting | VoteOS is a digital election system. It does not mandate the elimination of paper ballots. |

---

## The Decision / Execution Boundary

This is the most important architectural boundary in VoteOS:

```
┌─────────────────────────────────┐     ┌─────────────────────────────────┐
│         DECISION DOMAIN         │     │        EXECUTION DOMAIN         │
│          (VoteOS)               │     │     (CivilOS / Other)           │
│                                 │     │                                 │
│  - Who can vote?                │     │  - What policy changes?         │
│  - What's on the ballot?        │     │  - What budget shifts?          │
│  - How are votes counted?       │ ──► │  - Who takes office?            │
│  - What was the result?         │     │  - What programs start/stop?    │
│  - Is the result certified?     │     │  - What gets built?             │
│                                 │     │                                 │
│  VoteOS produces:               │     │  CivilOS consumes:              │
│    CERTIFIED OUTCOME            │     │    CERTIFIED OUTCOME            │
└─────────────────────────────────┘     └─────────────────────────────────┘
```

VoteOS **never** implements the outcome of an election.
It only certifies what was decided.

---

## Relationship to AxiaSystem

AxiaSystem provides:
- **Identity resolution** — who is this person? (resolve_subject)
- **Authentication** — prove you are who you claim (authenticate_subject)
- **Legitimacy evaluation** — are you authorized for this action? (evaluate_legitimacy)
- **Attestation** — tamper-evident record of action (attest_action)
- **Explanation** — audit-ready reasoning (explain_decision)
- **Asset registration** — register election artifacts as tracked assets (register_asset)

VoteOS consumes these. It does NOT own, replicate, or override them.

---

## Relationship to CivilOS

| Aspect | CivilOS | VoteOS |
|--------|---------|--------|
| Domain | City operations | Democratic decisions |
| Identity source | AxiaSystem | AxiaSystem (same) |
| Citizens | Onboards citizens | Verifies eligibility of existing citizens |
| Data | City records (DMV, finance, permits) | Election records (ballots, votes, results) |
| Governance | Operational governance (approvals, budgets) | Democratic governance (elections, referenda) |
| Outputs | City services | Certified election outcomes |
| Coupling | None — reads AxiaSystem | None — reads AxiaSystem |
| Co-deployment | Optional | Optional |

When both are deployed for the same city:
- Citizens onboarded via CivilOS already exist in AxiaSystem
- VoteOS uses those identities without re-onboarding
- CivilOS may consume VoteOS certified results
- Neither system calls the other directly

---

## Authority Model

| Actor | VoteOS Authority |
|-------|-----------------|
| Voter | Cast votes in elections where eligible |
| Election Official | Create/manage elections, certify results |
| Poll Worker | Assist operations, verify identity at polling |
| Observer | Monitor process, access audit trail |
| Audit Authority | Initiate recounts, validate evidence |
| System (VoteOS) | Compute tallies, generate receipts |
| CivilOS | NONE over VoteOS (read-only result consumer) |
| AxiaSystem | Identity/legitimacy truth provider |

---

## Election Types Supported

| Type | Description | Privacy Mode |
|------|-------------|-------------|
| General Election | Public offices, representatives | Secret ballot |
| Referendum | Yes/no policy questions | Secret ballot |
| Resolution | Governance body decision | Roll call (transparent) |
| Recall | Remove elected official | Secret ballot |
| Initiative | Citizen-proposed measure | Secret ballot |
| Advisory | Non-binding opinion | Configurable |
| Internal Governance | Organization/committee decisions | Configurable |

---

## Privacy Modes

| Mode | Ballot Secrecy | Vote Verification | Tally Transparency |
|------|---------------|-------------------|-------------------|
| Secret Ballot | Yes — vote content not linked to voter | Receipt proves recording, not choice | Full aggregate result published |
| Roll Call | No — each vote is attributed to voter | Full attribution | Full individual + aggregate |
| Configurable | Per-election setting by authority | Per-election setting | Per-election setting |
