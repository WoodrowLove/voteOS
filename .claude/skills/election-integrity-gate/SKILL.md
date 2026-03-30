# Election Integrity Gate Skill

> Enforces election-specific integrity constraints that are stricter than general platform discipline.
> This skill is invoked before any election state transition or result publication.

---

## Core Election Integrity Rules

### Rule 1: No Vote Without Eligibility Proof
Before any vote is recorded:
- [ ] Voter identity resolved via AxiaSystem
- [ ] Voter eligibility verified for this specific election
- [ ] Voter has not already voted in this election (no double-voting)
- [ ] Election is in OPEN state

Violation → REJECT vote, log reason, do not record.

### Rule 2: No Tally Without Complete Evidence
Before any result is computed:
- [ ] All recorded votes are accounted for
- [ ] Vote count matches submission count
- [ ] No orphan votes (votes without matching eligible voter)
- [ ] No phantom votes (voters counted without vote record)

Violation → BLOCK result publication, flag for audit.

### Rule 3: No Winner Declaration Without Threshold
Before any winner is declared:
- [ ] Participation threshold met (if defined for election type)
- [ ] Margin threshold met (if defined for election type)
- [ ] Tie handling rules applied if margin is within threshold
- [ ] Ambiguity declared if rules cannot determine winner

Violation → Result status = AMBIGUOUS, not WINNER.

### Rule 4: No Silent State Transition
Every election state change must have:
- [ ] Actor attribution (who triggered it)
- [ ] Legitimacy evaluation (were they authorized)
- [ ] Attestation (tamper-evident record)
- [ ] Explanation (audit-ready reasoning)

Violation → State change rejected, logged as unauthorized attempt.

### Rule 5: No Privacy Compromise
For secret ballot elections:
- [ ] Vote content is not linked to voter identity in storage
- [ ] Vote receipt proves recording without revealing choice
- [ ] Audit trail proves inclusion without revealing individual votes
- [ ] Observer access does not expose individual choices

Violation → HALT election, escalate to audit authority.

---

## Election Lifecycle States

```
DRAFT → PUBLISHED → OPEN → CLOSED → TALLIED → CERTIFIED → ARCHIVED
                                                    ↓
                                               CONTESTED → RECOUNTED → CERTIFIED
```

### State Transition Rules
| From | To | Requires |
|------|----|----------|
| DRAFT | PUBLISHED | Election official + complete ballot |
| PUBLISHED | OPEN | Scheduled time OR manual open by authority |
| OPEN | CLOSED | Scheduled time OR manual close by authority |
| CLOSED | TALLIED | Automatic computation from recorded votes |
| TALLIED | CERTIFIED | Election official attestation |
| TALLIED | CONTESTED | Contest filed within deadline |
| CONTESTED | RECOUNTED | Audit authority approval |
| RECOUNTED | CERTIFIED | Audit authority attestation |
| CERTIFIED | ARCHIVED | Retention period passed |

### Forbidden Transitions
- OPEN → DRAFT (cannot un-open an election)
- CLOSED → OPEN (cannot re-open a closed election)
- CERTIFIED → TALLIED (cannot un-certify)
- ARCHIVED → any state (terminal)

---

## Proof Classifications for Elections

| Classification | Meaning |
|---------------|---------|
| ELECTION_PROVEN | Full lifecycle tested: create → open → vote → close → tally → certify |
| TALLY_PROVEN | Result computation verified against known vote set |
| ELIGIBILITY_PROVEN | Voter verification chain tested against AxiaSystem |
| PRIVACY_PROVEN | Ballot secrecy verified (vote not linkable to voter) |
| AUDIT_PROVEN | Audit trail reconstructs result from evidence |
| INTEGRITY_BLOCKED | One or more integrity rules violated |
