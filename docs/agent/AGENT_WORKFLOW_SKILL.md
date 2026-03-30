# Agent Workflow Skill

## How the Agent Must Operate Inside the Axia/CivilOS Process

> This document defines the mandatory workflow for any agent session that implements, tests, or modifies platform capabilities. It is not about how to code — it is about how to work inside this process.

---

## 1. Before Any Work

### Read first. Always.

Before writing any code, modifying any file, or proposing any change, the agent MUST read:

1. **PHASE_SUMMARY.md** — What phase is active? What capabilities are in scope?
2. **CAPABILITY_STATUS.md** — What is the current state of each capability?
3. **KNOWN_LIMITATIONS.md** — What is broken, incomplete, or deferred?
4. **The active implementation plan** (Part N for the current capability) — What was approved?
5. **Part 14 — Execution Contracts** — What are the state definitions?

If any of these files cannot be found or read, STOP and report the issue.

### Confirm scope

The agent must be able to answer:
- What capability am I working on?
- What phase does it belong to?
- What is its current state?
- What state am I trying to reach?
- What evidence is required?

If any of these are unclear, ask before proceeding.

---

## 2. The Mandatory Workflow

Every capability implementation follows this exact sequence. No steps may be skipped.

```
[1] READ    → truth ledger + active plan
[2] CONFIRM → scope, capability, target state
[3] PLAN    → produce implementation plan (if not already approved)
[4] WAIT    → plan must be approved before implementation begins
[5] BUILD   → implement only the approved capability
[6] VERIFY  → build passes, no stubs, contract types correct
[7] DEPLOY  → deploy to local replica
[8] TEST    → run evidence-producing checks on real environment
[9] RECORD  → update all 3 truth ledger files
[10] REPORT → summarize honestly with evidence
[11] STOP   → wait for acceptance or correction before proceeding
```

### The workflow is sequential. No parallelism between steps 5-11 and the next capability.

---

## 3. State Progression Rules

### States are ordered. Never skip.

```
Not Started → Implemented → Wired → Live → Proven
```

### Each state has evidence requirements (from Part 14)

| State | Minimum Evidence |
|-------|-----------------|
| Implemented | Code exists, no stubs, contract types defined |
| Wired | Public function callable, canister calls resolve |
| Live | At least one successful real call + one failure path on local replica |
| Proven | Full test suite + schema conformance + limitations documented |

### State transitions require truth ledger updates

No state transition is real until recorded in:
- CAPABILITY_STATUS.md
- PHASE_SUMMARY.md
- KNOWN_LIMITATIONS.md

All three. Every time.

---

## 4. Forbidden Behaviors

### Never do these:

| Forbidden | Why |
|-----------|-----|
| Skip the plan step | Unplanned implementation drifts |
| Start next capability before current is accepted | Phase integrity breaks |
| Claim "complete" or "done" without evidence | Inflation destroys trust |
| Use "production-ready" or "fully working" | These require explicit justification |
| Implement capabilities outside the current phase | Phase gates exist for a reason |
| Modify capabilities already at Proven | Changes require re-verification |
| Auto-promote state without evidence | The Inflation Test must pass |
| Silently ignore test failures | Failures are evidence — document them |
| Expand scope beyond the approved plan | Scope changes require plan revision |

### The Inflation Test

Before making any capability claim, apply:

> "If an independent auditor read this claim and tried to use the capability based on it, would they succeed?"

If the answer is no or maybe, the claim is inflated.

---

## 5. Folder Structure

### Know where things live

| Location | Contains |
|----------|---------|
| `docs/conceptual/` | Parts 1-7 — conceptual architecture (DO NOT MODIFY) |
| `docs/capabilities/` | Parts 8-11 — capability surface + API contracts |
| `docs/repo-mapping/` | Part 12 — AxiaSystem mapping |
| `docs/implementation/` | Parts 13+ — implementation plans, execution reports, truth ledger |
| `docs/agent/` | Agent workflow, session harness, branching policy |
| `.claude/skills/` | Claude skills for validation and enforcement |

### Part numbering

Parts are numbered sequentially. Each capability follows the pattern:
- Even number = Implementation Plan
- Odd number = Implementation Execution / Report

Current parts: 1-23 complete. Next: Part 24.

---

## 6. Truth Ledger System

### Three files. Always in sync.

| File | Purpose | Updated when |
|------|---------|-------------|
| `CAPABILITY_STATUS.md` | Per-capability state + evidence | Every state transition |
| `PHASE_SUMMARY.md` | Phase-level progress | Every state transition + phase completion |
| `KNOWN_LIMITATIONS.md` | What doesn't work | Every implementation + every evidence run |

### Update rules

- Update immediately after each state transition
- Never batch updates across multiple capabilities
- Never update without evidence
- If work is done but ledger is not updated, the work is invalid

---

## 7. How to Detect Scope Drift

Watch for these signals:

| Signal | What it means |
|--------|--------------|
| "While I'm here, I should also..." | Scope creep |
| Modifying a capability that isn't the current target | Out of scope |
| Adding features not in the approved plan | Plan violation |
| Building infrastructure "for later" | Premature optimization |
| Fixing unrelated bugs during implementation | Scope expansion |
| Adding types or functions for future capabilities | Not approved |

When drift is detected: STOP, note what was tempting, and return to the approved scope.

Exception: Upstream fixes that are directly blocking the current capability (e.g., wallet ID bug, admin2 identity ID) are acceptable and should be documented.

---

## 8. How to Summarize Work

Every work session must end with a structured summary:

```
## Session Summary

### Objective: [what was attempted]
### Capability: [which capability]
### Starting state: [state before this session]
### Ending state: [state after this session]

### Files changed: [list]
### Evidence produced: [test results, runtime outputs]
### Limitations found: [new limitations discovered]
### Truth ledger updated: [yes/no, which files]

### Proposed state: [Implemented/Wired/Live/Proven]
### Confidence: [High/Medium/Low]
### Blocking issues: [any]
### Next recommended move: [what should happen next]
```

---

## 9. Upstream Fix Protocol

When a blocking issue is found in AxiaSystem substrate during implementation:

1. **Document the issue** — root cause, file, line, impact
2. **Fix at the correct layer** — don't patch downstream; fix the source
3. **Verify the fix** — build passes, behavior confirmed
4. **Document the fix** — in the capability's evidence and KNOWN_LIMITATIONS.md
5. **Note it is pre-existing** — distinguish upstream bugs from implementation bugs

Upstream fixes discovered: wallet_service.mo getWalletByOwner ID bug, admin2 hardcoded identity canister ID, all hardcoded canister IDs needing configurable setters.

---

## 10. Phase Gate Protocol

Before starting any new phase:

1. ALL capabilities in the current phase must be at Proven
2. Phase completion must be explicitly accepted
3. The next phase's gate must be recorded as open in PHASE_SUMMARY.md
4. The first step in the new phase is always a planning document, not code

Phase gates are not optional. They are hard stops.

---

## 11. Execution Authority Model

At the start of EVERY session, the agent MUST declare its authority level.

### Authority Levels

| Level | Meaning | Allowed Actions |
|-------|---------|----------------|
| `READ_ONLY` | Inspection and analysis only | Read files, run queries, produce reports. NO file modifications. |
| `PLAN_ONLY` | Design and planning only | Read files, create plan documents in `docs/`. NO canister code changes. |
| `IMPLEMENT_ALLOWED` | Full implementation | Read, write, build, deploy, test. Only within approved scope. |
| `VERIFY_ONLY` | Testing and validation only | Read files, run tests, deploy to replica. NO code modifications. |

### Mandatory Declaration

The agent must output at session start:

```
Authority Level: [READ_ONLY | PLAN_ONLY | IMPLEMENT_ALLOWED | VERIFY_ONLY]
```

### Enforcement

- If Authority Level is not `IMPLEMENT_ALLOWED`, ANY code change is a **violation**
- If Authority Level is `PLAN_ONLY`, only files in `docs/` may be created or modified
- If Authority Level is `VERIFY_ONLY`, only test scripts may be executed; no source code changes
- Authority level cannot be self-promoted during a session — it must be set at start

---

## 12. Capability Boundary Contract

Before starting work on any capability, the agent MUST declare:

```
Target Capability: [name]
Allowed Files: [list of files that may be modified]
Restricted System Areas: [areas that must NOT be touched]
```

### Enforcement

- Any modification outside the declared `Allowed Files` is a **violation**
- `Restricted System Areas` are hard boundaries — no exceptions without explicit approval
- Upstream fixes that directly block the current capability are the ONLY exception, and must be documented

### Default Restricted Areas

Unless explicitly unlocked:
- Capabilities already at Proven state
- Canister code for non-active capabilities
- Governance/admin bootstrap logic (unless the active capability requires it)
- Bridge code (Rust) unless the active capability is bridge-related

---

## 13. Test Coverage Contract

Every capability implementation must define and satisfy required test categories before reaching Proven.

### Required Test Categories

| Category | What it proves | Required for Proven? |
|----------|---------------|---------------------|
| Allow path | Happy path succeeds | YES |
| Deny path (per gate) | Each denial reason is exercised | YES |
| Edge case | Boundary conditions handled | YES |
| Failure/error | Structured errors for invalid input | YES |
| Determinism | Same input → same output | YES |
| Schema conformance | Field-by-field Part 11 check | YES |
| Idempotency (if applicable) | Repeated calls behave correctly | YES (for idempotent capabilities) |
| Downstream usability | Output is consumed by next capability | RECOMMENDED |

### Reporting Format

```
Test Coverage: [X] / [Y] required categories
Missing: [list of uncovered categories]
```

### Enforcement

- Missing any required category → cannot reach Proven
- Partial coverage may support Live but must be documented in KNOWN_LIMITATIONS.md

### Gate Coverage Requirement

For capabilities with a gate-based evaluation model (like evaluate_legitimacy), each gate must have at least one dedicated runtime test proving it fires correctly.

```
Gate Coverage:
- [gate name] → tested (test #N)
- [gate name] → tested (test #N)
- [gate name] → NOT TESTED (blocker: ...)
```

Missing gate coverage blocks Proven for gate-based capabilities.

---

## 14. Mandatory Execution Chain

Skills must be called in order. Skipping a required chain step is a violation.

### Before ANY implementation session

```
/axia-workflow start → /upstream-integrity-check [capability]
```

Both must pass before code changes begin. If either reports BLOCKED, implementation cannot start.

### Before ANY state transition

```
/validate-state-transition [capability] [state] → /update-truth-ledger [capability] [state]
```

`/validate-state-transition` must return APPROVED before `/update-truth-ledger` is called. If validation returns BLOCKED, the transition is rejected.

### Before ANY session end

```
/axia-workflow check → /axia-workflow summary
```

Drift check must run before the summary is produced. Violations found in the check must be addressed or documented in the summary.

### Enforcement

If any required chain step is skipped → **violation**. The downstream step is invalid without its prerequisite.

---

## 15. Auto-Stop Conditions

The agent must IMMEDIATELY STOP work if ANY of the following conditions are detected. These are not warnings — they are hard stops.

| Condition | Action |
|-----------|--------|
| Authority violation (code change in PLAN_ONLY) | STOP. Report violation. Await instruction. |
| Scope violation (file outside declared scope modified) | STOP. Report violation. Await instruction. |
| Non-goal violation (action touches declared non-goal) | STOP. Report violation. Await instruction. |
| Missing required evidence for claimed state | STOP. Revert claim. Gather evidence. |
| Upstream integrity failure (canister unreachable, ID mismatch) | STOP. Fix upstream. Re-run integrity check. |
| System intent violation (any of the 10 invariants) | STOP. Flag which invariant. Await instruction. |
| State skip detected (e.g., Not Started → Live) | STOP. Cannot proceed. Must follow sequential states. |

### After STOP

1. Report what triggered the stop
2. Report the current honest state
3. Do NOT continue work until the condition is resolved or the user explicitly overrides

### Auto-stop is not optional

The agent cannot reason its way past an auto-stop. Even if the agent believes the violation is minor, it must stop and report.

---

## 16. Confidence Scoring

Every state transition proposal and session summary must include a confidence score.

| Level | Meaning |
|-------|---------|
| **High** | All required test categories covered. All paths exercised. Schema conformance verified. No known gaps. |
| **Medium** | Most paths tested. Some edge cases untested. Known limitations documented but not all exercised. |
| **Low** | Partial evidence only. Major paths untested. Significant gaps remain. |

### Reporting

```
Confidence: [High | Medium | Low]
Basis: [brief justification]
```

### Enforcement

- High confidence required for Proven state
- Medium confidence acceptable for Live state
- Low confidence acceptable only for Implemented or Wired

---

## 17. Workflow Proof Discipline

> Added: 2026-03-29. Codified from three independent proof-tightening sessions.

### Mandatory Enforcement Rules

These rules apply to ALL CivilOS workflow integration tests. Violations block Proven claims.

1. **No workflow may be marked Proven without strict happy-path proof.** A strict happy-path test requires `Ok(...)` and panics on any `Err(...)`. Mixed-path tests that accept denial as a valid outcome are invalid.

2. **Mixed-path tests must be rewritten.** If a test accepts both `Ok(...)` and `Err(PreconditionFailed)` in the same function, it is not a proof — it is a mixed-path test. Split into separate happy-path and failure-path tests.

3. **Failure paths must be tested separately.** Each failure mode (legitimacy denial, invalid input, insufficient funds) gets its own explicitly-named test function.

4. **Legitimacy must be proven with real setup.** The test actor must be created via `resolve_subject` + `authenticate_subject`. Assurance must be upgraded via `identity.setAssuranceLevel` if the action type requires it. Anonymous principals (`2vxsx-fae`) are only valid in denial-path tests.

5. **Environment blockers must result in BLOCKED classification, not PASS.** If a workflow step fails due to environment constraints (empty wallet, missing canister), the workflow proof is BLOCKED. The legitimacy proof may still be valid if step 1 passed.

### Execution Chain Addition

Before marking any workflow as tested or proven:
```
→ /axia-workflow check
→ /workflow-proof check
→ /workflow-proof classify <workflow_name>
```

### Proof Classification Reference

| Classification | Meaning |
|---------------|---------|
| STRICT_HAPPY_PATH_PROVEN | Full Ok path proven with real actors, all artifacts verified |
| STRICT_HAPPY_PATH_BLOCKED | Legitimacy passes, downstream step blocked by environment |
| LEGITIMACY_PROVEN_ONLY | Legitimacy gate passes, later steps not reachable |
| FAILURE_PATH_PROVEN | At least one failure mode tested in separate test |
| DENIAL_ONLY | Only anonymous denial tested, no Ok path attempted |
| PARTIAL_PROOF | Some coverage but mixed-path or incomplete |
| UNPROVEN | No integration test exists |

### Skill Reference

Full enforcement rules: `.claude/skills/workflow-proof-discipline/SKILL.md`
