# VoteOS Interoperability Hooks

> Defines the specific integration points between VoteOS, AxiaSystem, and CivilOS.

---

## AxiaSystem Integration Points

### Consumed by VoteOS

| AxiaSystem Capability | VoteOS Use | Module |
|----------------------|-----------|--------|
| resolve_subject | Verify voter identity exists | Voter Registry |
| authenticate_subject | Authenticate voter at vote time | Vote Recording |
| evaluate_legitimacy | Check voter eligibility, official authority | All modules |
| attest_action | Record every election state change | All modules |
| explain_decision | Audit-ready reasoning for every decision | All modules |
| register_asset | Register elections/ballots as tracked assets | Election Mgmt, Ballot |

### VoteOS-Specific Legitimacy Actions

| Action Type | Min Assurance | Used By |
|------------|---------------|---------|
| operation (cast_vote) | L0 | Vote Recording |
| operation (register_voter) | L0 | Voter Registry |
| governance_action (create_election) | L1 | Election Management |
| governance_action (certify_result) | L1 | Result Certification |
| governance_action (initiate_recount) | L1 + role | Audit & Oversight |
| data_access (resolve_voter_record) | L0 | Voter Registry |

---

## CivilOS Integration Points

### VoteOS → CivilOS (Result Export)

| What | How | When |
|------|-----|------|
| Certified election result | Attested record via AxiaSystem | After certification |
| Proposal outcome | Attested record | After certification |
| Elected official identity | Subject_ref + role | After certification |

CivilOS reads these as attested records from AxiaSystem, not through direct API calls to VoteOS.

### CivilOS → VoteOS (Identity)

| What | How | When |
|------|-----|------|
| Citizen exists | AxiaSystem resolve_subject | At voter registration |
| Citizen assurance level | AxiaSystem evaluate_legitimacy | At eligibility check |
| Citizen standing | AxiaSystem subject standing | At eligibility check |

VoteOS reads these from AxiaSystem, not from CivilOS directly.

---

## External Observer Integration

| Consumer | Access Method | Scope |
|----------|-------------|-------|
| Party observers | Observer API (Module 8) | Process monitoring, no individual votes |
| Media | Published results API (Module 10) | Certified results only |
| Regulatory bodies | Audit bundle export (Module 8) | Full evidence package |
| Academic researchers | Anonymized statistics (Module 10) | Aggregate data only |

---

## Webhook Events (Module 10)

| Event | Payload | Consumers |
|-------|---------|-----------|
| election.published | election_ref, title, schedule | CivilOS, media |
| election.opened | election_ref, voting_period | All subscribers |
| election.closed | election_ref, vote_count | All subscribers |
| result.certified | election_ref, result_summary | CivilOS, media, officials |
| result.contested | election_ref, contest_reason | Officials, auditors |
| recount.completed | election_ref, recount_result | Officials, auditors |

---

## Identity Persistence Flow

```
Step 1: City deploys CivilOS
        → Citizens onboarded to AxiaSystem
        → Maria gets subject_ref "abc123..."

Step 2: City deploys VoteOS
        → VoteOS calls resolve_subject("abc123...")
        → AxiaSystem confirms Maria exists
        → VoteOS checks eligibility criteria
        → Maria added to voter roll

Step 3: Maria votes
        → VoteOS authenticates via AxiaSystem
        → VoteOS checks she's on voter roll
        → Vote recorded

Step 4: Result certified
        → VoteOS attests result via AxiaSystem
        → CivilOS governance module reads attested result
        → Policy implemented

No system called another directly.
AxiaSystem was the shared truth throughout.
```
