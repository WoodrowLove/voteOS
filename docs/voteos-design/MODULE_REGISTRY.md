# VoteOS Module Registry

> Defines all VoteOS modules, their boundaries, responsibilities, and dependencies.

---

## Module Overview

| # | Module | Domain | Key Responsibility |
|---|--------|--------|--------------------|
| 1 | Voter Registry | Eligibility | Who can vote in which elections |
| 2 | Election Management | Lifecycle | Create, configure, control election state |
| 3 | Ballot Operations | Content | Design, generate, distribute ballots |
| 4 | Vote Recording | Casting | Accept, validate, store votes securely |
| 5 | Tally Engine | Computation | Count votes, compute results deterministically |
| 6 | Result Certification | Authority | Certify, publish, export election outcomes |
| 7 | Governance Proposals | Measures | Manage proposals, resolutions, initiatives |
| 8 | Audit & Oversight | Integrity | Evidence reconstruction, observer access, compliance |
| 9 | Election Operations | Administration | Scheduling, staffing, incident management |
| 10 | Integration & Export | Interop | Outcome delivery to CivilOS and external consumers |

---

## Module 1: Voter Registry

**Domain:** Eligibility verification and voter roll management.

**Responsibilities:**
- Verify citizen eligibility for specific elections (age, jurisdiction, standing)
- Maintain voter rolls per election/jurisdiction
- Track registration status (registered, pending, suspended, ineligible)
- Handle eligibility disputes
- Prevent duplicate registration

**Key constraint:** VoteOS does NOT onboard citizens. It verifies eligibility of identities already in AxiaSystem.

**AxiaSystem dependencies:**
- resolve_subject (verify identity exists)
- evaluate_legitimacy (check eligibility authority)

**Domain state:**
- VoterRegistration { voter_ref, election_ref, status, registered_at, eligibility_basis }
- EligibilityRule { election_ref, rule_type, criteria }

---

## Module 2: Election Management

**Domain:** Election lifecycle control.

**Responsibilities:**
- Create elections with type, scope, schedule
- Configure election rules (thresholds, privacy mode, voting period)
- Manage election state machine (DRAFT → PUBLISHED → OPEN → CLOSED → TALLIED → CERTIFIED)
- Enforce state transition rules
- Track election officials and their authority

**Domain state:**
- Election { election_ref, title, type, status, config, created_by, schedule }
- ElectionConfig { privacy_mode, participation_threshold, margin_threshold, voting_method }

---

## Module 3: Ballot Operations

**Domain:** Ballot content management.

**Responsibilities:**
- Create ballot templates (questions, candidates, measures)
- Generate ballots per election
- Manage ballot versions and corrections
- Distribute ballots to eligible voters
- Track ballot issuance status

**Dependencies:** Election Management (election must exist before ballot), Voter Registry (distribution requires eligibility)

**Domain state:**
- BallotTemplate { template_ref, election_ref, items: Vec<BallotItem> }
- BallotItem { item_ref, item_type, title, description, choices }
- BallotIssuance { issuance_ref, voter_ref, ballot_ref, issued_at, status }

---

## Module 4: Vote Recording

**Domain:** Secure vote acceptance and storage.

**Responsibilities:**
- Validate vote submissions (eligible voter, valid ballot, election open)
- Prevent double-voting
- Record votes securely
- Generate vote receipts
- Enforce ballot secrecy (in secret ballot mode)
- Handle spoiled/replacement ballots

**Dependencies:** Voter Registry (eligibility), Ballot Operations (valid ballot), Election Management (election state)

**Domain state:**
- VoteRecord { vote_ref, election_ref, ballot_ref, submitted_at, receipt_hash }
- VoteContent { content_ref, ballot_item_ref, choice } — separated from voter identity in secret ballot mode
- VotingReceipt { receipt_ref, voter_ref, receipt_hash, timestamp }

**Critical invariant:** In secret ballot mode, VoteRecord does NOT contain voter_ref for the vote content. The receipt proves recording without revealing choice.

---

## Module 5: Tally Engine

**Domain:** Deterministic result computation.

**Responsibilities:**
- Count votes per ballot item
- Apply voting method (plurality, ranked choice, approval, etc.)
- Evaluate participation thresholds
- Detect ties and ambiguity
- Produce provisional results
- Support multiple counting methods per election type

**Dependencies:** Vote Recording (votes to count), Election Management (election rules)

**Domain state:**
- TallyResult { tally_ref, election_ref, item_ref, method, counts, status }
- ItemResult { item_ref, winner, margin, threshold_met, is_ambiguous }

**Critical invariant:** Same votes + same method = same result. Always. Deterministic.

---

## Module 6: Result Certification

**Domain:** Official result attestation and publication.

**Responsibilities:**
- Review and certify tally results
- Publish certified results
- Handle contested results
- Manage certification workflow (official review, attestation, publication)
- Generate certification records

**Dependencies:** Tally Engine (results to certify)

**Domain state:**
- Certification { cert_ref, election_ref, status, certified_by, certified_at, attestation_ref }
- PublishedResult { result_ref, election_ref, publication_time, access_level }

---

## Module 7: Governance Proposals

**Domain:** Structured decision items (measures, resolutions, initiatives).

**Responsibilities:**
- Create governance proposals (referenda, resolutions, citizen initiatives)
- Manage proposal lifecycle (draft, submitted, qualified, on-ballot, decided)
- Validate proposal requirements (signatures, sponsorship, jurisdiction)
- Link proposals to elections
- Track proposal outcomes

**Domain state:**
- Proposal { proposal_ref, type, title, description, sponsor, status, requirements }
- ProposalQualification { qual_ref, proposal_ref, method, evidence, qualified_at }

---

## Module 8: Audit & Oversight

**Domain:** Election integrity verification.

**Responsibilities:**
- Generate audit trails for every election action
- Support evidence reconstruction (tally from votes)
- Manage observer access (register, authorize, scope)
- Handle recounts
- Manage contest/dispute resolution
- Generate compliance reports

**Dependencies:** All other modules (audit covers everything)

**Domain state:**
- AuditTrail { trail_ref, election_ref, entries: Vec<AuditEntry> }
- AuditEntry { entry_ref, action, actor, timestamp, evidence_hash }
- Recount { recount_ref, election_ref, initiated_by, result, status }
- ObserverAccess { observer_ref, election_ref, scope, granted_by }

---

## Module 9: Election Operations

**Domain:** Operational administration.

**Responsibilities:**
- Election scheduling and calendar management
- Polling location/digital endpoint management
- Staff assignment (poll workers, officials)
- Incident reporting and management
- Operational status monitoring

**Domain state:**
- Schedule { schedule_ref, election_ref, events: Vec<ScheduleEvent> }
- StaffAssignment { assignment_ref, election_ref, person_ref, role, location }
- Incident { incident_ref, election_ref, type, description, status }

---

## Module 10: Integration & Export

**Domain:** Outcome delivery to external systems.

**Responsibilities:**
- Export certified results to CivilOS and other consumers
- Publish results via API for external systems
- Generate standard-format result reports
- Support webhook/event notifications
- Manage export audit trail

**Domain state:**
- ExportRecord { export_ref, election_ref, destination, format, exported_at, status }
- Webhook { webhook_ref, target_url, events, status }

---

## Module Dependency Map

```
Module 1 (Voter Registry)
    ↓
Module 2 (Election Management)
    ↓
Module 3 (Ballot Operations) ──→ Module 7 (Governance Proposals)
    ↓
Module 4 (Vote Recording)
    ↓
Module 5 (Tally Engine)
    ↓
Module 6 (Result Certification)
    ↓
Module 10 (Integration & Export)

Module 8 (Audit & Oversight) ──→ spans all modules
Module 9 (Election Operations) ──→ operational layer, parallel
```

---

## Cross-Cutting Concerns

| Concern | How VoteOS Handles It |
|---------|----------------------|
| Identity | AxiaSystem resolve_subject / authenticate_subject |
| Authorization | AxiaSystem evaluate_legitimacy |
| Attestation | AxiaSystem attest_action on every state change |
| Explainability | AxiaSystem explain_decision for audit |
| Privacy | Domain-level ballot secrecy (Module 4) |
| Determinism | Tally Engine is pure computation (Module 5) |
| Auditability | Audit module spans all (Module 8) |
