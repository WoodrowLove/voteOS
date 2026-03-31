# VoteOS ↔ CivilOS Relationship

> Definitive architecture statement on how these two sovereign siblings coexist.

---

## Fundamental Principle

**VoteOS and CivilOS are sovereign sibling systems.**

Neither owns the other. Neither is a module of the other.
Both depend on AxiaSystem. Neither depends on each other.

```
        AxiaSystem (Source of Truth)
       /                            \
    CivilOS                       VoteOS
    (City Operations)             (Election Integrity)
    - permits, DMV, finance       - elections, ballots, votes
    - public safety, assets       - tallies, certification
    - citizen services            - governance proposals
    - operational governance      - democratic governance
```

---

## Interaction Model

VoteOS and CivilOS do NOT call each other's APIs directly.

Instead, they interact through shared AxiaSystem truth:

### Shared AxiaSystem Reads
Both systems read from the same AxiaSystem for:
- Citizen identity (resolve_subject)
- Authentication (authenticate_subject)
- Legitimacy evaluation (evaluate_legitimacy)
- Attestation (attest_action)

### Attested Record Exchange
VoteOS publishes certified results as AxiaSystem-attested records.
CivilOS may read those records when implementing governance decisions.

No real-time coupling. No direct API calls between systems.

---

## Concrete Interaction Examples

### Example 1: Budget Referendum

```
1. CivilOS governance module identifies need for budget vote
2. Election official creates referendum in VoteOS
3. Citizens (already in AxiaSystem via CivilOS onboarding) vote in VoteOS
4. VoteOS tallies: 67% yes, 33% no
5. VoteOS certifies result (attested via AxiaSystem)
6. CivilOS reads certified result from AxiaSystem
7. CivilOS finance module implements budget change
8. VoteOS is not involved in implementation
```

### Example 2: Mayoral Election

```
1. Election official creates election in VoteOS
2. Citizens verify eligibility against AxiaSystem identity
3. Voting occurs through VoteOS
4. VoteOS certifies: Jane Smith elected
5. CivilOS governance module reads certified result
6. CivilOS updates governance structure (new mayor)
7. VoteOS does not care what CivilOS does with the result
```

### Example 3: Voter Eligibility Check

```
1. Citizen Maria was onboarded by CivilOS two years ago
2. Maria's identity exists in AxiaSystem: subject_ref "abc123..."
3. City deploys VoteOS
4. VoteOS calls evaluate_legitimacy for Maria
5. AxiaSystem confirms: identity valid, assurance L0, standing active
6. VoteOS checks additional eligibility (age, jurisdiction, registration)
7. Maria added to voter roll
8. VoteOS never called CivilOS — it went straight to AxiaSystem
```

### Example 4: Council Resolution

```
1. Council member submits resolution via VoteOS governance proposals
2. Resolution goes through VoteOS lifecycle (draft → submitted → on-ballot)
3. Council votes in roll-call mode (transparent, attributed)
4. VoteOS tallies: 7 yes, 2 no, 1 abstain
5. VoteOS certifies resolution as passed
6. CivilOS operational module reads certified resolution
7. CivilOS implements the operational change
```

---

## Data Ownership Boundaries

| Data | Owned By | Created By | Consumed By |
|------|----------|-----------|-------------|
| Citizen identity (subject_ref) | AxiaSystem | CivilOS or VoteOS (whoever onboards first) | Both |
| Assurance level | AxiaSystem | CivilOS or VoteOS | Both |
| Citizen standing | AxiaSystem | CivilOS or VoteOS | Both |
| City department roles | CivilOS | CivilOS | CivilOS only |
| DMV/finance/permit records | CivilOS | CivilOS | CivilOS only |
| Voter eligibility | VoteOS | VoteOS | VoteOS only |
| Voter rolls | VoteOS | VoteOS | VoteOS only |
| Election state | VoteOS | VoteOS | VoteOS (CivilOS reads result) |
| Ballot content | VoteOS | VoteOS | VoteOS only |
| Individual votes | VoteOS | VoteOS | VoteOS only (secret) |
| Certified results | VoteOS | VoteOS | CivilOS + public |
| Governance proposals | VoteOS | VoteOS | CivilOS reads outcomes |

---

## What CivilOS CANNOT Do to VoteOS

| Action | Allowed? | Why |
|--------|----------|-----|
| Read certified results | YES | Published, attested records |
| Modify election state | NO | VoteOS is sovereign |
| Access individual votes | NO | Ballot secrecy |
| Override certification | NO | Only VoteOS officials can certify |
| Create elections | NO | VoteOS owns election lifecycle |
| Block voter registration | NO | Eligibility is VoteOS + AxiaSystem |
| Influence tallies | NO | Tally is deterministic from votes |

---

## What VoteOS CANNOT Do to CivilOS

| Action | Allowed? | Why |
|--------|----------|-----|
| Implement policy changes | NO | That's CivilOS's job |
| Modify city department structure | NO | CivilOS owns operations |
| Access city financial records | NO | CivilOS data boundary |
| Issue permits or licenses | NO | CivilOS domain |
| Manage city employees | NO | CivilOS domain |

---

## Communication Protocol

```
VoteOS → AxiaSystem: attest_action(certified_result)
CivilOS → AxiaSystem: read attested record

No VoteOS → CivilOS API call ever.
No CivilOS → VoteOS API call ever.
```

AxiaSystem is the mailbox. Both systems post to it and read from it.
