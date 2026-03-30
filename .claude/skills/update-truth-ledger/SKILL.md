---
name: update-truth-ledger
description: Record a capability state transition in the truth ledger system. Use after completing implementation work on any capability to update CAPABILITY_STATUS.md, PHASE_SUMMARY.md, and KNOWN_LIMITATIONS.md. Enforces Part 14 execution contracts — no state can be skipped, no claim made without evidence, no transition recorded without updating all three files. Also use to audit whether truth artifacts are current.
user-invocable: true
argument-hint: "capability-name new-state"

---

# Truth Ledger Updater

You enforce Part 14's rule: **"No state transition is real until it is recorded in truth artifacts."** Your job is to update the three truth ledger files when a capability reaches a new state, and to reject inflated claims.

## Truth Ledger Files

All three MUST be updated on every state transition:

- [CAPABILITY_STATUS.md](../../../docs/implementation/CAPABILITY_STATUS.md) — per-capability state + evidence
- [PHASE_SUMMARY.md](../../../docs/implementation/PHASE_SUMMARY.md) — phase-level progress
- [KNOWN_LIMITATIONS.md](../../../docs/implementation/KNOWN_LIMITATIONS.md) — what doesn't work

## Governing Rules

Read before every update:
- [Part 14 — Execution Contracts](../../../docs/implementation/14-AXIA_EXECUTION_CONTRACTS_V1.md)

## State Definitions

| State | Meaning | Required Evidence |
|-------|---------|-------------------|
| Not Started | No code exists | — |
| Implemented | Real code, no stubs, contract types defined | File paths, function names, no stubs confirmation |
| Wired | Connected to runtime path, dependencies reachable | Endpoint exists, canister calls resolve |
| Live | Executes against real dependencies | At least one successful real call, response matches Part 11 |
| Proven | Tested, contract-conformant, limitations documented | Unit + integration tests, schema conformance, failure modes tested |

## State Transition Rules

### Rule 1: States cannot be skipped
```
Not Started → Implemented → Wired → Live → Proven
```
A capability at state N must have satisfied ALL requirements of states 1 through N.

### Rule 2: Evidence must exist for the claimed state
Before updating, verify that the evidence described in Part 14 Section 3 for the specific capability and state actually exists. Do not trust narrative — check files.

### Rule 3: All three files must be updated atomically
A state transition recorded in CAPABILITY_STATUS.md but not reflected in PHASE_SUMMARY.md and KNOWN_LIMITATIONS.md is incomplete.

## Update Process

When invoked with `$ARGUMENTS` (capability name + new state):

### Step 1 — Verify the transition is valid
- Read CAPABILITY_STATUS.md for current state
- Confirm the new state is exactly one step forward (no skipping)
- If the transition skips a state, REJECT and explain why

### Step 2 — Verify evidence exists
- For **Implemented**: Grep for the implementation files. Confirm no TODO/STUB/PLACEHOLDER in critical path.
- For **Wired**: Verify endpoint exists (grep for route/endpoint). Verify canister call paths.
- For **Live**: Look for test output, runtime logs, or actual response data.
- For **Proven**: Look for test files, test results, schema conformance checks, limitations doc.

### Step 3 — Apply the Inflation Test
> "If an independent auditor read this claim and tried to use the capability based on it, would they succeed?"

If the answer is no or maybe, REJECT the transition.

### Step 4 — Update CAPABILITY_STATUS.md
- Update Current State
- Fill in the evidence section for the new state
- Update Known Limitations (add any new ones discovered)
- Update Next Required State

### Step 5 — Update PHASE_SUMMARY.md
- Update the capability's state in the phase capabilities table
- Recalculate progress percentage
- Update Progress Summary with honest current state
- Update Blockers if any have changed
- Update Next Actions

### Step 6 — Update KNOWN_LIMITATIONS.md
- Add any new limitations discovered during this work
- Remove any limitations that have been resolved
- Update impact assessments if they've changed

### Step 7 — Report
Output what was updated and what evidence was found.

## Audit Mode

When invoked without a new state (just a capability name or "audit"):

- Read all three truth ledger files
- Check whether the recorded states are still accurate
- Flag any claims that appear inflated based on current code
- Flag any missing updates (e.g., new code exists but CAPABILITY_STATUS.md still says Not Started)
- Report discrepancies

## Forbidden Language Check

On every update, scan CAPABILITY_STATUS.md and PHASE_SUMMARY.md for forbidden terms:
- "Complete", "Done", "Integrated", "Production-ready", "Fully working"

If found without explicit Part 14 justification, flag for correction.

## Output Format

```
## Truth Ledger Update: [capability] → [new state]

### Transition Valid: YES / NO / REJECTED
### Evidence Found:
- [list of evidence verified]

### Files Updated:
- CAPABILITY_STATUS.md: [what changed]
- PHASE_SUMMARY.md: [what changed]
- KNOWN_LIMITATIONS.md: [what changed]

### New Limitations Discovered:
- [any new limitations found during verification]

### Forbidden Language Check: CLEAN / VIOLATIONS FOUND
```
