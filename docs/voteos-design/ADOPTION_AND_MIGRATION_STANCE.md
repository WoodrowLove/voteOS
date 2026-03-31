# VoteOS Adoption and Migration Stance

> Whether VoteOS needs a wrapper/adoption subsystem like CivilOS's migration wrapper.

---

## Assessment: YES — VoteOS needs a migration/adoption layer.

### Why

Real-world adoption of VoteOS means replacing existing election infrastructure.
Municipalities, states, school boards, and organizations already have:

- **Voter rolls** — existing registered voter databases
- **Election configurations** — rules, districts, precincts, deadlines
- **Historical results** — past election outcomes and audit records
- **Office registries** — elected officials, terms, succession
- **Observer/party registrations** — credentialed observers, party affiliations
- **Administrative accounts** — election administrators, poll workers

These cannot be discarded. They must be migrated or reconciled.

Ignoring migration is what killed many promising election technology projects.

---

## Wrapper Architecture (Planned — Future Phase)

The VoteOS wrapper should follow the same proven CivilOS pattern:

```
Legacy Election System
         │
    ┌────▼─────┐
    │  Adapter  │  ← connects to legacy source (DB, API, file)
    └────┬─────┘
    ┌────▼─────────┐
    │  Normalizer   │  ← transforms legacy schema to VoteOS shapes
    └────┬─────────┘
    ┌────▼──────────────┐
    │  Reconciler        │  ← maps legacy voters to AxiaSystem identities
    └────┬──────────────┘
    ┌────▼──────────────┐
    │  Cutover Controller│  ← SHADOW → PARALLEL → VOTEOS_PRIMARY
    └────┬──────────────┘
    ┌────▼──────────────┐
    │  Audit Comparison  │  ← compares legacy vs VoteOS outcomes
    └────┬──────────────┘
         │
    VoteOS (new authority)
```

### Wrapper Subsystems (Planned)

| Subsystem | Purpose | CivilOS Equivalent |
|-----------|---------|-------------------|
| Legacy Voter Roll Adapter | Connect to existing voter databases | Legacy Adapter Layer |
| Voter Identity Reconciler | Map legacy voter IDs to AxiaSystem subject_refs | Identity Reconciler |
| Election Config Normalizer | Transform legacy election rules to VoteOS format | Schema Normalizer |
| Office/Role Importer | Import elected officials, terms, roles | (No direct equivalent) |
| Shadow Tally Comparison | Run VoteOS tally alongside legacy, compare results | Audit Comparison Engine |
| Cutover Controller | Manage per-module transition state | Cutover Controller |

### Migration-Specific Challenges

| Challenge | VoteOS-Specific Concern |
|-----------|----------------------|
| Voter roll format | Every jurisdiction has different voter DB schema |
| Historical results | Past elections may use different counting methods |
| Privacy constraints | Migrating voter data must preserve privacy laws |
| Certification chain | Historical certifications may not have AxiaSystem attestation |
| In-progress elections | Cannot migrate mid-election; must time migration |
| Observer credentials | Existing observer registrations need mapping |

---

## Migration Phases (Recommended)

### Phase M1: Data Assessment
- Inventory legacy voter rolls, election configs, historical data
- Classify what can be migrated vs. what needs manual re-entry
- Identify privacy and compliance constraints

### Phase M2: Identity Reconciliation
- Map legacy voter records to AxiaSystem identities
- Handle duplicates, conflicts, inactive voters
- Establish canonical identity links
- (Follows exact pattern proven in CivilOS identity collision resolution)

### Phase M3: Shadow Run
- Configure VoteOS with imported election rules
- Run VoteOS tally alongside legacy system for a test election
- Compare results (tally comparison engine)
- Do NOT use VoteOS results as official until verified

### Phase M4: Parallel Operation
- Both systems active for real elections
- VoteOS results compared against legacy for confidence
- Operator monitors discrepancies
- Metrics must meet confidence threshold before cutover

### Phase M5: VoteOS Primary
- VoteOS becomes the official election system
- Legacy system receives mirror copies as backup
- Cutover controlled per-election or per-jurisdiction

### Phase M6: Legacy Decommission
- Legacy system retired
- Historical data preserved in VoteOS or archive
- Full audit trail maintained

---

## When the Wrapper Should Be Built

The wrapper is NOT needed for Wave 1-5 implementation.

It should be planned as **Wave 6 or later**, after the core VoteOS system is proven.

Build order rationale:
1. First: prove VoteOS works correctly (Waves 1-5)
2. Then: prove VoteOS can migrate from legacy (Wrapper wave)
3. Finally: prove VoteOS can replace legacy under real conditions (Pilot)

This matches the CivilOS build sequence where the wrapper was built AFTER the core system was proven and BEFORE pilot deployment.

---

## Key Principle: Migration Is Not Optional

For VoteOS to be adopted by real jurisdictions, migration is a first-class concern.
It must be planned explicitly, not hand-waved.

The proven CivilOS wrapper pattern (adapter → normalizer → reconciler → cutover → audit) provides the blueprint. VoteOS should adapt it for election-specific data, not reinvent it.
