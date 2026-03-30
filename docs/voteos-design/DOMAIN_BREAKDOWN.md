# VoteOS Domain Breakdown

> Defines the modules and domains of the VoteOS system.
> This replaces CivilOS's department-based breakdown with election-specific domains.

---

## Domain Architecture

VoteOS is organized around the election lifecycle, not city departments.

### Module A: Voter Registration & Eligibility
**Responsibility:** Verify and manage voter eligibility for elections.

- Voter eligibility verification (against AxiaSystem identity)
- Registration status tracking
- Voter roll management
- Eligibility disputes and resolutions
- Cross-election eligibility rules

**Key constraint:** VoteOS does NOT onboard citizens. It verifies eligibility of citizens already known to AxiaSystem.

### Module B: Election Management
**Responsibility:** Create, configure, and manage election lifecycles.

- Election creation (type, scope, schedule)
- Election configuration (thresholds, rules, privacy mode)
- Election state machine (DRAFT → PUBLISHED → OPEN → CLOSED → TALLIED → CERTIFIED)
- Election official role management
- Observer access management

### Module C: Ballot Operations
**Responsibility:** Design, distribute, and manage ballots.

- Ballot template creation
- Question/candidate management
- Ballot generation per election
- Ballot distribution to eligible voters
- Ballot versioning and corrections

### Module D: Vote Recording
**Responsibility:** Securely accept and store votes.

- Vote submission validation
- Eligibility gate (no vote without proof)
- Double-vote prevention
- Vote receipt generation
- Ballot secrecy enforcement
- Vote storage (encrypted or separated from identity)

### Module E: Result Aggregation
**Responsibility:** Compute and publish election results.

- Vote counting (deterministic)
- Result computation per question/race
- Threshold evaluation (participation, margin)
- Tie/ambiguity detection and handling
- Provisional result publication
- Final result certification

### Module F: Audit & Oversight
**Responsibility:** Ensure election integrity is independently verifiable.

- Audit trail generation
- Observer access controls
- Recount support
- Evidence reconstruction (tally from votes)
- Compliance reporting
- Contest/dispute management

---

## Module Dependency Order

```
A (Voter Registration) — must exist before D (Vote Recording)
B (Election Management) — must exist before C (Ballot) and D (Vote Recording)
C (Ballot Operations) — must exist before D (Vote Recording)
D (Vote Recording) — must exist before E (Result Aggregation)
E (Result Aggregation) — must exist before F (Audit) for full proofs
F (Audit & Oversight) — depends on all others for evidence
```

### Recommended Build Sequence
1. **Voter Registration** (root dependency — eligibility is the foundation)
2. **Election Management** (lifecycle controls)
3. **Ballot Operations** (content management)
4. **Vote Recording** (the core action)
5. **Result Aggregation** (computation)
6. **Audit & Oversight** (verification layer)

---

## Shared AxiaSystem Dependencies

| VoteOS Need | AxiaSystem Capability |
|-------------|----------------------|
| Verify voter identity | resolve_subject |
| Authenticate voter | authenticate_subject |
| Check voter authority | evaluate_legitimacy |
| Record election action | attest_action |
| Explain election decision | explain_decision |

VoteOS adds election-specific logic ON TOP of these primitives. It does not replace them.
