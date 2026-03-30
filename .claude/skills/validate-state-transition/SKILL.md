---
name: validate-state-transition
description: Validate that all evidence requirements are met BEFORE a capability state transition is recorded. Must be called before /update-truth-ledger. Checks required evidence exists, test categories are covered, schema conformance is present, no states are skipped, and runtime validation occurred. Blocks the transition if validation fails.
user-invocable: true
argument-hint: "capability-name proposed-state"
---

# State Transition Validator

You validate that all evidence requirements are met BEFORE a state transition is recorded. You are a gate — if validation fails, the transition is blocked.

## Call this BEFORE /update-truth-ledger

The workflow is: `/validate-state-transition` → if PASS → `/update-truth-ledger`

## Reference

- [Part 14 — Execution Contracts](../../../docs/implementation/14-AXIA_EXECUTION_CONTRACTS_V1.md)
- [CAPABILITY_STATUS.md](../../../docs/implementation/CAPABILITY_STATUS.md)
- [KNOWN_LIMITATIONS.md](../../../docs/implementation/KNOWN_LIMITATIONS.md)

## Validation Checks by Target State

### → Implemented

| Check | How to Verify | Required |
|-------|--------------|----------|
| Code exists | File path identified, function exists | YES |
| No stubs | Grep for TODO/STUB/PLACEHOLDER/unimplemented returns 0 | YES |
| Contract types defined | Request/response types match Part 11 | YES |
| Error handling present | All defined error codes have code paths | YES |
| Build passes | `dfx build` succeeds | YES |

### → Wired

All Implemented checks PLUS:

| Check | How to Verify | Required |
|-------|--------------|----------|
| Public function callable | Function is `public shared` | YES |
| Auth check in call path | Correct auth level enforced | YES |
| Inter-canister calls resolve | Dependent canister refs exist and compile | YES |
| Endpoint reachable | A `dfx canister call` reaches the code | YES |

### → Live

All Wired checks PLUS:

| Check | How to Verify | Required |
|-------|--------------|----------|
| Successful real call | At least one happy-path call on local replica | YES |
| Failure path exercised | At least one error path exercised on local replica | YES |
| Response matches contract | Actual output compared to Part 11 schema | YES |
| Real dependencies hit | Inter-canister calls executed (not mocked) | YES |

### → Proven

All Live checks PLUS:

| Check | How to Verify | Required |
|-------|--------------|----------|
| Full test suite run | All required test categories covered | YES |
| Schema conformance checklist | Field-by-field Part 11 comparison | YES |
| Determinism verified | Same input → same output | YES |
| Limitations documented | KNOWN_LIMITATIONS.md updated | YES |
| No forbidden language | No inflated claims in ledger | YES |

### Required Test Categories for Proven

| Category | Description |
|----------|------------|
| Allow path | Happy path succeeds |
| Deny path (per gate) | Each denial reason exercised |
| Edge case | Boundary conditions |
| Failure/error | Structured errors for invalid input |
| Determinism | Repeated calls produce same result |
| Schema conformance | Field-by-field Part 11 check |

## Output Format

```
## State Transition Validation: [capability] → [proposed state]

### Previous State: [current state in ledger]
### Proposed State: [target]
### Transition Valid: [YES — states are sequential]

### Evidence Checks:
| Check | Status | Evidence |
|-------|--------|----------|
| [check name] | PASS/FAIL/MISSING | [what was found] |

### Test Coverage:
Coverage: [X] / [Y] required categories
Missing: [list]

### Verdict: APPROVED / BLOCKED

### Blocking Reasons (if any):
- [what must be resolved]
```

## Enforcement

If verdict is BLOCKED → do NOT proceed to /update-truth-ledger. Fix the missing evidence first.
