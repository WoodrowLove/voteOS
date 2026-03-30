---
name: spine-validator
description: Validate that any action capability implementation correctly follows the full platform composition chain — resolve → evaluate → act → attest. Use when reviewing or implementing Phase 4+ action capabilities (execute_transfer, register_asset, record_governance_decision) to ensure they consume the decision spine and do not bypass legitimacy or attestation.
user-invocable: true
argument-hint: "capability-name-or-file-path"
---

# Platform Spine Validator

You verify that action capabilities correctly follow the proven composition chain. This is critical for Phase 4+ capabilities that perform real mutations.

## The Proven Spine (Phases 1-3)

```
resolve_subject → authenticate_subject → evaluate_legitimacy → [action] → attest_action
                                                                              ↓
                                                                      explain_decision (on demand)
```

Every action capability MUST consume this chain. No exceptions.

## What to Check

For any action capability (execute_transfer, register_asset, record_governance_decision, or custom):

### 1. Legitimacy Gate

- Does the capability require a `decision_ref` from `evaluate_legitimacy`?
- Is the decision_ref validated (looked up in decision records)?
- Does validation confirm the decision was `proceed`?
- Is subject alignment checked (decision.subject == request.subject)?

**VIOLATION if:** An action capability can execute without a valid `proceed` decision from evaluate_legitimacy.

### 2. Attestation Integration

- Does the capability produce an `action_ref` that can be passed to `attest_action`?
- Is the action designed to be attestable (stable ref, clear action_type)?
- Does the documentation/plan indicate attestation as a post-action step?

**WARNING if:** An action capability produces results that cannot be attested.

### 3. Subject Continuity

- Does the capability use the same `subject_ref` throughout?
- Does subject_ref trace back to `resolve_subject`?
- Is session_ref from `authenticate_subject` carried through?

**VIOLATION if:** Subject identity is lost or replaced mid-chain.

### 4. System Intent Alignment

- Does the action respect the 10 invariants from SYSTEM_INTENT.md?
- Is the action deterministic?
- Are side effects explicit?
- Is the action auditable?

### 5. Error Propagation

- If legitimacy was denied, does the action refuse to execute?
- If the decision_ref is invalid, does the action return a structured error?
- Does the action handle SYSTEM_UNAVAILABLE from upstream?

## Output Format

```
## Spine Validation: [capability]

### Legitimacy Gate: PRESENT / MISSING / PARTIAL
- decision_ref required: [YES/NO]
- decision validated: [YES/NO]
- proceed check: [YES/NO]
- subject alignment: [YES/NO]

### Attestation Integration: READY / NOT READY / PARTIAL
- action_ref produced: [YES/NO]
- attestable action_type: [YES/NO]
- attestation documented in plan: [YES/NO]

### Subject Continuity: MAINTAINED / BROKEN
- subject_ref consistent: [YES/NO]
- session_ref carried: [YES/NO]

### System Intent: ALIGNED / VIOLATION
- [any violations]

### Error Propagation: CORRECT / INCOMPLETE
- denied handling: [YES/NO]
- invalid ref handling: [YES/NO]

### Verdict: SPINE VALID / SPINE BROKEN / NEEDS WORK
```

## When to Use

- During Phase 4+ capability planning (validate the plan follows the spine)
- During Phase 4+ capability implementation (validate the code follows the spine)
- During code review of any action capability
- When a new action type is proposed
