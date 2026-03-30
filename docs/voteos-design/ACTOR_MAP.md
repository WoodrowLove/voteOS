# VoteOS Actor Map

> Defines who interacts with VoteOS and what they can do.

---

## Primary Actors

### Voter
- A citizen with verified eligibility for a specific election
- Identity established via AxiaSystem (may have been onboarded through CivilOS)
- Can: view elections, verify eligibility, cast votes, verify vote receipt
- Cannot: create elections, modify ballots, view other voters' choices, certify results

### Election Official
- Authorized administrator for election lifecycle management
- Requires elevated assurance (L1+ via AxiaSystem)
- Can: create elections, configure ballots, open/close elections, certify results
- Cannot: cast votes in elections they administer, modify recorded votes, suppress results

### Poll Worker
- Authorized operational staff during election execution
- Can: verify voter identity at polling, assist with ballot access, report incidents
- Cannot: modify election configuration, access vote content, certify results

### Observer
- Authorized transparency role (party representatives, media, public monitors)
- Can: view election state, monitor process, access audit trail
- Cannot: cast votes, modify anything, access individual vote content

### Audit Authority
- Independent verification role (separate from election officials)
- Can: initiate recount, verify evidence, validate tally, resolve contests
- Cannot: modify votes, change election configuration, certify on behalf of officials

---

## System Actors

### VoteOS System
- Automated processes: tally computation, threshold evaluation, receipt generation
- All system actions are deterministic and attested

### AxiaSystem
- Source of truth for identity, legitimacy, and attestation
- VoteOS consumes but does not control AxiaSystem

### CivilOS (When Co-deployed)
- May consume VoteOS certified results
- Does NOT influence VoteOS election processes
- Citizen identities onboarded via CivilOS carry into VoteOS automatically

---

## Role-Action Matrix

| Action | Voter | Official | Poll Worker | Observer | Auditor |
|--------|-------|----------|-------------|----------|---------|
| Create election | | X | | | |
| Configure ballot | | X | | | |
| Open/close election | | X | | | |
| Verify eligibility | X | X | X | | |
| Cast vote | X | | | | |
| View vote receipt | X | | | | |
| Monitor process | | X | | X | |
| Report incident | | | X | X | |
| Certify result | | X | | | |
| Contest result | X | | | | |
| Initiate recount | | | | | X |
| Verify audit trail | | | | X | X |
| Access individual votes | | | | | |

Note: **No actor can access individual vote content** in secret ballot mode. This is a system invariant.
