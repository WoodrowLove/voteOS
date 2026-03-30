# VoteOS Module Execution Sequence

> The order in which VoteOS modules should be built.
> Order is driven by dependency logic, not code convenience.

---

## Execution Order

### Wave 1: Foundation (Modules 1-2)

**Module 1: Voter Registry** — Build first.
- Root dependency for all voting operations
- Nothing can vote without verified eligibility
- Establishes the AxiaSystem integration pattern (resolve_subject for eligibility)
- Proves the shared-truth model works

**Module 2: Election Management** — Build second.
- Controls the lifecycle everything else depends on
- No ballots without elections, no votes without open elections
- Establishes the state machine pattern

### Wave 2: Ballot + Vote (Modules 3-4)

**Module 3: Ballot Operations** — Build third.
- Requires election to exist (Module 2)
- Defines what voters choose from
- Establishes ballot integrity patterns

**Module 4: Vote Recording** — Build fourth.
- Requires eligibility (Module 1), election open (Module 2), ballot issued (Module 3)
- The core action of the entire system
- Must prove double-vote prevention and ballot secrecy

### Wave 3: Results (Modules 5-6)

**Module 5: Tally Engine** — Build fifth.
- Requires votes to exist (Module 4)
- Pure computation — deterministic, reconstructible
- Must be independently verifiable

**Module 6: Result Certification** — Build sixth.
- Requires tally (Module 5)
- Authority layer — certification is the system's binding output
- Must be attested via AxiaSystem

### Wave 4: Governance + Audit (Modules 7-8)

**Module 7: Governance Proposals** — Build seventh.
- Extends the ballot system with proposal lifecycle
- Links citizen initiatives to election ballots
- Can be built after the core vote→tally→certify chain works

**Module 8: Audit & Oversight** — Build eighth.
- Spans all modules — needs them to exist
- Evidence reconstruction, observer access, recounts
- The trust-verification layer

### Wave 5: Operations + Integration (Modules 9-10)

**Module 9: Election Operations** — Build ninth.
- Operational layer (scheduling, staffing, incidents)
- Not on the critical path but needed for real deployment

**Module 10: Integration & Export** — Build tenth.
- Delivers outcomes to CivilOS and external systems
- Requires certification (Module 6) to have something to export
- Completes the interoperability story

---

## Build Order Rationale

```
Wave 1: Can you verify who can vote? Can you create an election?
Wave 2: Can you issue a ballot? Can you record a vote?
Wave 3: Can you count votes? Can you certify the result?
Wave 4: Can you manage proposals? Can you audit everything?
Wave 5: Can you operate elections? Can you deliver outcomes?
```

Each wave builds on the previous. No wave can be skipped.

---

## Gate Criteria Per Wave

### Wave 1 Gate
- [ ] Voter eligibility verified against AxiaSystem
- [ ] Election lifecycle state machine working (DRAFT → CERTIFIED)
- [ ] At least one election type configurable

### Wave 2 Gate
- [ ] Ballot template with items created and issued
- [ ] Vote recorded securely
- [ ] Double-vote prevention proven
- [ ] Ballot secrecy enforced in secret ballot mode

### Wave 3 Gate
- [ ] Tally computed deterministically from recorded votes
- [ ] Same votes always produce same result
- [ ] Tie/ambiguity correctly handled
- [ ] Result certified with attestation

### Wave 4 Gate
- [ ] Proposal lifecycle works (draft → qualified → on-ballot → decided)
- [ ] Audit trail reconstructs election result from evidence
- [ ] Observer access works without exposing individual votes

### Wave 5 Gate
- [ ] Election operations support real deployment
- [ ] Certified results exportable to CivilOS
- [ ] Webhook/notification system works
