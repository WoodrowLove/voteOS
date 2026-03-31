# VoteOS Operational Intelligence Layer

> Read-only observational layer for system behavior and pilot reporting.

---

## Purpose

The intelligence layer provides structured, aggregated views of VoteOS system state without exposing election truth data. It exists to answer:

- What is the system doing right now?
- How did the pilot go?
- Are there problems that need attention?

## What Is Included

| Data | Source | Example |
|------|--------|---------|
| Election counts by status | ElectionRegistry | 3 Draft, 2 Open, 5 Certified |
| Proposal counts | ProposalRegistry | 2 Draft, 1 Certified |
| Operational state | OperationsRegistry | 1 paused, 0 incidents |
| Audit results | AuditRegistry | 3 verified, 0 failed |
| Export counts | ExportRegistry | 2 exports generated |
| Runtime config | AppState | persistence=true, auth=true |
| Adoption metrics | Adoption pipeline output | 95% normalization rate |
| Shadow validation | Validation pipeline output | 1 match, 0 mismatches |
| Key findings | Deterministic rules | "All metrics within thresholds" |

## What Is Intentionally Excluded

- Individual vote content
- Voter identity linkage
- Ballot selections
- Raw tally data per voter
- Any data that could compromise ballot secrecy

## API Endpoints

### GET /api/system/insights

Returns a `SystemSnapshot` with real-time counts from all registries.
No auth required (observability endpoint).

### GET /api/system/pilot-report

Returns a `PilotReport` with adoption, reconciliation, shadow validation, and audit summaries.
No auth required.

## How External Systems Use This

### SunnyJaymes / Control Plane

Poll `/api/system/insights` to monitor:
- Are elections progressing through lifecycle?
- Are any elections paused or incident-flagged?
- Has audit verification passed?

Poll `/api/system/pilot-report` to assess:
- Did the pilot succeed?
- What's the normalization/reconciliation rate?
- Were there shadow validation mismatches?

### Operator Dashboard

Use `/api/system/insights` to display:
- Active election count
- Certification count
- Operational alerts (paused/incidents)

## Key Findings Rules

The pilot report generates findings deterministically:

| Condition | Finding |
|-----------|---------|
| Normalization rate < 90% | "Low normalization rate" |
| Invalid records > 0 | "N invalid records detected" |
| Ambiguous reconciliations > 0 | "N ambiguous reconciliations" |
| Identity miss rate > 20% | "High identity miss rate" |
| Shadow mismatches > 0 | "N shadow validation mismatches" |
| Audit failures > 0 | "N audit verifications FAILED" |
| All clean | "All pilot metrics within acceptable thresholds" |
| No data | "No pilot data processed yet" |

## Design Principles

1. **Read-only**: Never mutates domain state
2. **Aggregated**: Only counts, rates, statuses — no raw records
3. **Deterministic**: Same inputs always produce same output
4. **Separated**: Intelligence data is distinct from election truth data
5. **Observable**: No auth required for intelligence endpoints
