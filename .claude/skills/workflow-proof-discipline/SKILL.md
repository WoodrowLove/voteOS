---
name: workflow-proof-discipline
description: Enforce strict workflow proof standards. Detects mixed-path tests, validates legitimacy setup, classifies proof levels, and blocks incorrect PASS claims. Use when writing, reviewing, or classifying integration tests for CivilOS workflows.
user-invocable: true
argument-hint: "check | classify <workflow_name> | template <workflow_name>"
---

# Workflow Proof Discipline

You enforce proof quality for CivilOS workflow integration tests. You are a validation skill, not a coding skill. You detect violations, classify proof levels, and block incorrect claims.

## Reference Documents

- [TEST_REGISTRY.md](../../../docs/testing/TEST_REGISTRY.md) — authoritative test status
- [CIVILOS_ORCHESTRATION_AUDIT.md](../../../docs/implementation/CIVILOS_ORCHESTRATION_AUDIT.md) — audit findings

## Provenance

Codified from three independent proof-tightening sessions:
1. onboard_city_worker (PART XX+2/XX+3) — 5-step chain, strict Ok proof
2. issue_driver_license (PART XX+4) — 4-step chain, strict Ok proof
3. execute_city_payment (PART XX+5) — L1 assurance proven, transfer blocked honestly

---

## Modes

### `/workflow-proof check`

Scan test files for violations. For each integration test file in `tests/`:

1. **Mixed-path detection**: Does any test accept both `Ok(...)` AND `Err(...)` as valid outcomes in the same match arm? If yes → VIOLATION.
2. **Silent error swallowing**: Does any test use `Err(e) => println!(...)` without `panic!` or `assert!`? If yes → VIOLATION.
3. **Fake legitimacy**: Does the test use `2vxsx-fae` (anonymous principal) as the happy-path actor? If yes → VIOLATION for happy-path tests.
4. **Hardcoded canister IDs**: Does the test hardcode a canister ID instead of using a constant? Not a violation, but flag as FRAGILE.

Report:
```
Workflow proof check:
  [file]: [PASS / VIOLATION: description]
```

### `/workflow-proof classify <workflow_name>`

Read the test file for the named workflow and classify its proof level using the classification system below. Output:

```
Workflow: <name>
Strict happy-path: YES / NO / BLOCKED
Failure-path: YES / NO
Legitimacy proven: YES / NO (with level if yes)
Full workflow proven: YES / NO / BLOCKED (reason)
Classification: <classification>
```

### `/workflow-proof template <workflow_name>`

Generate a test template for the named workflow following the strict discipline. The template must include:

1. A `setup_legitimate_actor` function using `citizen_onboarding`
2. A strict happy-path test that requires `Ok(...)`
3. A separate legitimacy-denial test
4. JSON roundtrip tests

---

## Invariant Rules

These rules are non-negotiable. Any violation blocks the test from being classified as proven.

### Rule 1: Strict Happy-Path Must Require Ok

```rust
// CORRECT — strict
let outcome = result.expect("STRICT HAPPY PATH: workflow must return Ok(...)");

// WRONG — mixed-path
match result {
    Ok(outcome) => { /* verify */ }
    Err(WorkflowError::PreconditionFailed { .. }) => {
        // "expected for test data" ← THIS IS THE VIOLATION
    }
}
```

A test that accepts denial as a valid happy-path outcome is NOT a happy-path test. It is a mixed-path test and must be split.

### Rule 2: Failure Paths Must Be Separate Tests

Each failure mode gets its own test function with an explicit name:
- `test_<workflow>_legitimacy_denied`
- `test_<workflow>_invalid_department`
- `test_<workflow>_insufficient_funds`

Never combine happy and failure assertions in one test.

### Rule 3: Legitimacy Setup Must Be Real

The test actor must be created through the real AxiaSystem spine:
1. `resolve_subject` to create the identity
2. `authenticate_subject` to get a session
3. Assurance upgrade via `identity.setAssuranceLevel` if the action type requires it

Acceptable assurance levels by action type:
| Action Type | Min Assurance | Upgrade Needed |
|-------------|--------------|----------------|
| operation | L0 | No |
| data_access | L0 | No |
| financial_action | L1 | Yes (L0 → L1) |
| structural_change | L2 | Yes (L0 → L2) |
| governance_action | L1 | Yes (L0 → L1) |

### Rule 4: Environment Blockers Are BLOCKED, Not PASS

If a workflow step fails due to environment constraints (empty wallet, missing canister, etc.):
- Do NOT mark the workflow as STRICT_HAPPY_PATH_PROVEN
- DO mark it as LEGITIMACY_PROVEN_ONLY or STRICT_HAPPY_PATH_BLOCKED
- DO document what prerequisite is missing
- DO NOT hide the blocker by removing the assertion

### Rule 5: All Workflow Artifacts Must Be Verified

A strict happy-path test must verify every output field the workflow claims to produce:
- decision_ref (legitimacy)
- action output (asset_ref, transfer_ref, role_ref, etc.)
- attestation_ref + audit_ref
- explanation text (if workflow includes explain step)

---

## Proof Classifications

| Classification | Meaning | Criteria |
|---------------|---------|----------|
| STRICT_HAPPY_PATH_PROVEN | Full Ok path proven with real actors | All steps complete, Ok required, all artifacts verified |
| STRICT_HAPPY_PATH_BLOCKED | Legitimacy passes but a downstream step fails due to environment | Step 1 proven, step N blocked by external constraint |
| LEGITIMACY_PROVEN_ONLY | Legitimacy gate passes but later steps not reachable | Step 1 Ok, remaining steps not attempted or blocked |
| FAILURE_PATH_PROVEN | At least one failure mode tested separately | Denial or error at specific step verified |
| DENIAL_ONLY | Only legitimacy denial tested | Anonymous principal blocked, no Ok path attempted |
| PARTIAL_PROOF | Some coverage but mixed-path or incomplete | Does not meet strict standards |
| UNPROVEN | No integration test exists | No runtime evidence |

---

## Applying Classifications

When classifying a workflow, read the actual test code. Do not trust comments or claims.

Check:
1. Does a test exist that calls `result.expect(...)` or equivalent strict Ok assertion? → happy-path candidate
2. Does the test create a real user via `citizen_onboarding::execute()`? → legitimacy setup present
3. Does the test verify output artifacts (decision_ref, asset_ref, attestation_ref)? → artifact verification present
4. Does a separate test exist that expects `Err(...)` at a specific step? → failure path present
5. Does any test accept both Ok and Err in the same function? → VIOLATION: mixed-path

---

## Current Workflow Classifications

| Workflow | Classification | Evidence |
|---------|---------------|----------|
| onboard_city_worker | STRICT_HAPPY_PATH_PROVEN + FAILURE_PATH_PROVEN | Real admin, Ok required, step-4 failure tested |
| issue_driver_license | STRICT_HAPPY_PATH_PROVEN + FAILURE_PATH_PROVEN | Real worker + citizen, Ok required, all 4 artifacts |
| execute_city_payment | LEGITIMACY_PROVEN_ONLY + FAILURE_PATH_PROVEN | L1 officer passes gate, transfer BLOCKED (empty wallet) |
| citizen_onboarding | STRICT_HAPPY_PATH_PROVEN (via golden path) | Golden path creates real user end-to-end |
| audit_transparency | STRICT_HAPPY_PATH_PROVEN (via golden path) | Golden path explains real decision |
| All others | DENIAL_ONLY | Anonymous denied at step 1, no Ok path tested |
