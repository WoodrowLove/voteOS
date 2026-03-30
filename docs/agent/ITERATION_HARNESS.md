# VoteOS Iteration Harness — Self-Checking Build Loop

> This document defines the autonomous iteration protocol.
> The agent follows this loop WITHOUT external prompting between iterations.
> It acts as its own manager: build → test → evaluate → decide → iterate or report.

---

## The Build Loop

```
┌─────────────────────────────────────────────────┐
│                 START ITERATION                   │
│                                                   │
│  1. Read SESSION_STATE.md                         │
│  2. Identify next uncompleted module/wave         │
│  3. Execute implementation for that module        │
│  4. Run: cargo build                              │
│  5. Run: cargo test                               │
│  6. Evaluate results against MODULE COMPLETION    │
│  7. Update SESSION_STATE.md with honest status    │
│  8. Commit + push                                 │
│  9. DECISION GATE:                                │
│     - All tests pass + module complete? → NEXT    │
│     - Tests fail? → FIX then re-evaluate          │
│     - Blocked by environment? → REPORT + STOP     │
│     - Wave gate criteria met? → ADVANCE WAVE      │
│                                                   │
│  10. Loop back to step 1                          │
└─────────────────────────────────────────────────┘
```

---

## Decision Rules (The Agent's "Manager")

### After each module implementation:

**IF** `cargo build` fails:
→ Fix compilation errors. Do not commit broken code.

**IF** `cargo test` has failures:
→ Fix test failures. Classify the failure:
- Code bug → fix and re-test
- Environment dependency → mark as BLOCKED, document, continue

**IF** module passes all 6 completion layers:
→ Mark module as COMPLETE in SESSION_STATE.md
→ Commit with descriptive message
→ Push to remote
→ Proceed to next module

**IF** module passes 5/6 layers (missing one):
→ Mark as CONDITIONALLY_COMPLETE
→ Document what's missing
→ Proceed to next module (don't block on non-critical gaps)

**IF** module is blocked by something external:
→ Mark as BLOCKED with exact reason
→ Document in SESSION_STATE.md
→ Skip to next module that can proceed
→ If ALL remaining modules are blocked → STOP and report

---

## Wave Gate Rules

### At the end of each wave (2 modules):

Check the wave gate criteria from MODULE_SEQUENCE_PLAN.md.

**IF** all gate criteria met:
→ Log: "Wave N COMPLETE — advancing to Wave N+1"
→ Update SESSION_STATE.md
→ Commit
→ Continue

**IF** gate criteria partially met:
→ Log: "Wave N CONDITIONAL — criteria X not met: [reason]"
→ If the missing criteria is non-blocking for the next wave → continue
→ If it's blocking → STOP and report

---

## Proof Classification Rules (Apply to every test)

After writing tests, classify each one:

| Classification | When to Use |
|---------------|-------------|
| STRICT_HAPPY_PATH_PROVEN | test uses `.expect()`, real AxiaSystem setup, all artifacts verified |
| STRICT_HAPPY_PATH_BLOCKED | legitimacy passes but downstream blocked (environment) |
| LEGITIMACY_PROVEN_ONLY | step 1 passes, later steps unreachable |
| FAILURE_PATH_PROVEN | denial/error at specific step verified |
| DENIAL_ONLY | only unauthorized access tested |
| UNPROVEN | no test exists yet |

**RULE: Never classify as PROVEN if the test uses `match Ok/Err` instead of `.expect()`.**

---

## Commit Discipline

Every commit must:
1. Build cleanly (`cargo build` passes)
2. Not introduce test regressions
3. Have a descriptive message following the pattern:
   ```
   feat: Module N (Name) — [summary]

   [details of what was built]
   [test status]
   [any known limitations]

   Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
   ```
4. Push to remote after each module completion

---

## Session State Protocol

SESSION_STATE.md must be updated after EVERY iteration with:
- Current phase
- Last completed gate
- Module status table (NOT_STARTED → DESIGN_COMPLETE → ... → COMPLETE)
- Next action
- Any blockers

This is how the agent resumes if context is lost.

---

## When to STOP

The agent STOPS and reports when:
1. All modules in the current wave are COMPLETE or CONDITIONALLY_COMPLETE
2. A blocking dependency prevents further progress
3. A design decision requires human input (privacy model, algorithm choice)
4. 3 consecutive iterations fail to make progress

STOP message format:
```
ITERATION STOP — [reason]
Modules completed: [list]
Modules blocked: [list]
Next required action: [exact step]
```

---

## Reuse Rules (CRITICAL)

Before creating ANY new capability or infrastructure:

1. **Check AxiaSystem** — does this capability already exist in the 11 locked primitives?
2. **Check the Rust Bridge** — is there already a binding for this?
3. **Check the pattern reference** — is there a proven CivilOS pattern for this?
4. **Compose first** — can the need be met by composing existing capabilities?
5. **Only then create** — if none of the above work, create new VoteOS-specific logic

**NEVER duplicate what AxiaSystem already provides.**
**NEVER create a new identity system, legitimacy system, or attestation system.**
