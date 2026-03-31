# VoteOS Standalone System Shape

> How VoteOS looks and operates as a complete independent system.
> This document proves VoteOS doesn't need CivilOS to function.

---

## System Overview

```
┌──────────────────────────────────────────────────────────────┐
│                         VoteOS                                │
│                                                               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │   Voter      │  │   Election   │  │   Ballot            │ │
│  │   Registry   │  │   Management │  │   Operations        │ │
│  │              │  │              │  │                     │ │
│  │  eligibility │  │  lifecycle   │  │  design, issue,     │ │
│  │  rolls       │  │  config      │  │  distribute         │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬──────────────┘ │
│         │                 │                  │                │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼──────────────┐ │
│  │   Vote       │  │   Tally      │  │   Result            │ │
│  │   Recording  │  │   Engine     │  │   Certification     │ │
│  │              │  │              │  │                     │ │
│  │  cast, seal, │  │  count,      │  │  certify, publish,  │ │
│  │  receipt     │  │  verify      │  │  export             │ │
│  └──────────────┘  └──────────────┘  └─────────────────────┘ │
│                                                               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │  Governance  │  │   Audit &    │  │   Election          │ │
│  │  Proposals   │  │   Oversight  │  │   Operations        │ │
│  └─────────────┘  └──────────────┘  └─────────────────────┘ │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Integration & Export Layer                  │  │
│  │  (webhook, result delivery, audit bundle export)        │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │     HTTP API (REST endpoints for all modules)           │  │
│  │     ⚠ ASPIRATIONAL — not yet implemented               │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │     SpineClient (AxiaSystem Bridge)                     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
                     ┌────────▼────────┐
                     │   AxiaSystem    │
                     │   (ICP/IC)      │
                     │                 │
                     │  Identity       │
                     │  Legitimacy     │
                     │  Attestation    │
                     └─────────────────┘
```

---

## Lifecycle of a Typical Election

### Phase 1: Setup
1. Election official creates election (type, scope, schedule)
2. Official configures rules (privacy mode, thresholds, voting method)
3. Ballot template created with questions/candidates
4. Eligibility rules defined for this election
5. Voter roll generated from eligible citizens

### Phase 2: Publication
6. Ballot finalized and locked
7. Election published (visible to voters)
8. Observers registered and scoped

### Phase 3: Voting
9. Election opens at scheduled time
10. Voters authenticate via AxiaSystem
11. Eligibility verified against voter roll
12. Ballot issued to eligible voter
13. Voter casts vote (validated, sealed, receipt generated)
14. Double-vote prevention enforced

### Phase 4: Counting
15. Election closes at scheduled time
16. Tally computed deterministically from recorded votes
17. Participation threshold evaluated
18. Ties and ambiguity detected

### Phase 5: Certification
19. Provisional result reviewed
20. Election official certifies result (attested via AxiaSystem)
21. Result published
22. Contest window opens (if applicable)

### Phase 6: Post-Election
23. Audit trail available for observers
24. Evidence reconstruction possible
25. Recount available if contested
26. Result exported to downstream systems
27. Election archived after retention period

---

## Administrative Roles

| Role | Authority | Assurance Required |
|------|-----------|-------------------|
| Election Administrator | Create elections, configure rules, assign staff | L1 + admin role |
| Election Official | Certify results, manage contests | L1 + official role |
| Ballot Designer | Create/edit ballot templates | L1 |
| Poll Worker | Verify identity at polling, assist voters | L0 + poll_worker role |
| Observer | Monitor process, access audit trail | L0 + observer registration |
| Audit Authority | Initiate recounts, validate evidence | L1 + audit role |
| Voter | Cast votes in eligible elections | L0 (eligibility verified) |

---

## Privacy Modes

### Secret Ballot (Default for Elections)
- Vote content stored separately from voter identity
- Receipt proves vote was recorded, not what was chosen
- Tally computed from anonymous vote pool
- No actor can link voter to choice
- Audit verifies inclusion without revealing individual votes

### Roll Call (For Governance Bodies)
- Each vote attributed to the voter
- Full transparency on who voted how
- Used for council votes, board decisions, committee actions

### Configurable (Per Election)
- Election authority sets privacy mode at creation
- Cannot be changed after publication
- Mode enforced at system level, not policy level

---

## Result Certification

Certification is the binding output of VoteOS:

```
Certification Record:
  election_ref: "elec-2026-mayoral"
  result: { winner: "Jane Smith", votes: 15432, margin: "52.3%" }
  attestation_ref: "att-..." (AxiaSystem attested)
  certified_by: "official-ref-..."
  certified_at: "2026-11-05T21:30:00Z"
  explanation_ref: "exp-..." (audit reasoning)
```

A certified result is:
- Attested via AxiaSystem (tamper-evident)
- Attributable to a specific official
- Independently reconstructible from evidence
- Publishable to external systems

---

## What VoteOS Does NOT Need From CivilOS

| Concern | VoteOS handles it independently |
|---------|-------------------------------|
| Voter identity | AxiaSystem (shared root, not CivilOS) |
| Authentication | AxiaSystem authenticate_subject |
| Election state | VoteOS domain state (own persistence) |
| Ballot content | VoteOS ballot module |
| Vote storage | VoteOS vote recording module |
| Result computation | VoteOS tally engine |
| Certification | VoteOS certification module |
| Audit trail | VoteOS audit module |
| API | VoteOS HTTP API (own server, own port) — **ASPIRATIONAL: not yet implemented** |

VoteOS is a complete system. CivilOS is optional context, not a requirement.
