# Session Harness

## How Each Long-Running Agent Session Starts, Runs, and Ends

> This document defines the protocol for agent sessions that do real implementation work. It ensures every session is scoped, tracked, and recoverable.

---

## Session Start Checklist

Every implementation session MUST begin with these steps:

### Step 1 — Load context

Read the following files in order:

```
docs/implementation/PHASE_SUMMARY.md
docs/implementation/CAPABILITY_STATUS.md
docs/implementation/KNOWN_LIMITATIONS.md
```

Record internally:
- Current phase
- Current capability being worked on
- Current state of that capability
- Target state for this session

### Step 2 — Confirm branch

Check: what branch am I on?

```
git branch --show-current
git status
```

If not on the correct agent branch, switch before proceeding.

If uncommitted changes exist from a prior session, assess them before continuing.

### Step 3 — Confirm objective

The agent must state (to itself or the user):

```
Session objective: [what I am trying to accomplish]
Approved capability: [which capability]
Current state: [state from truth ledger]
Target state: [what I aim to reach]
Allowed scope: [what files/canisters I may modify]
```

If the objective is unclear, ask.

### Step 4 — Confirm the plan exists

If the current capability does not have an approved implementation plan, the session objective is "produce the plan" — not "implement."

Plans must be approved before implementation begins.

### Step 5 — Check for blockers

Read KNOWN_LIMITATIONS.md for the current capability. Are there any blockers that would prevent progress?

If yes, the session objective may need to shift to resolving the blocker.

---

## During the Session

### Work within scope

- Modify only files related to the approved capability
- Do not add features from future phases
- Do not refactor unrelated code
- Exception: upstream fixes that directly block the current capability

### Produce evidence continuously

- Build after each significant change
- Test on local replica when implementation is complete
- Capture command output for evidence

### Update truth ledger at each state transition

When the capability moves from one state to the next:

1. Update CAPABILITY_STATUS.md with evidence
2. Update PHASE_SUMMARY.md with progress
3. Update KNOWN_LIMITATIONS.md with any new discoveries

### Watch for drift

If you notice yourself:
- Modifying files outside the approved scope
- Adding types or functions for future capabilities
- Fixing bugs in unrelated canisters (unless they block you)
- Expanding the implementation beyond the plan

Stop and re-read the approved plan.

---

## Session End Protocol

Every session MUST end with:

### Step 1 — Verify truth ledger is current

All three files must reflect the actual state of work:
- CAPABILITY_STATUS.md — correct state for the capability
- PHASE_SUMMARY.md — correct phase progress
- KNOWN_LIMITATIONS.md — any new limitations recorded

### Step 2 — Produce session summary

```
## Session Summary

### Date: [YYYY-MM-DD]
### Session objective: [what was attempted]
### Capability: [which capability]
### Starting state: [state at session start]
### Ending state: [state at session end]

### Files changed:
- [list all modified files]

### Evidence produced:
- [build results]
- [test results]
- [runtime outputs]

### Truth ledger updated: [yes/no]
### Proposed state transition: [from → to]
### Confidence: [High/Medium/Low]

### Blocking issues:
- [any issues that prevent further progress]

### Next recommended move:
- [what should happen in the next session]
```

### Step 3 — Commit work (if on agent branch)

If the session produced meaningful progress:

```
git add [specific files]
git commit -m "capability: [name] — [state reached] — [brief description]"
```

Do not commit:
- Partial work that doesn't compile
- Files outside the approved scope
- Experimental changes that weren't tested

### Step 4 — Stop cleanly

Do not start the next capability in the same session unless explicitly instructed.

The session ends when:
- The current capability reaches a new state
- A blocker is found that requires user input
- The approved scope is complete
- The user requests a stop

---

## Session Recovery

If a session is interrupted or resumed:

### Step 1 — Read truth ledger

The truth ledger is the source of recovery. Read all three files.

### Step 2 — Assess partial work

Check:
- Are there uncommitted changes in the working tree?
- Does the last truth ledger entry match the actual code state?
- Were any tests started but not completed?

### Step 3 — Resume or rollback

If partial work is consistent:
- Resume from the last recorded state
- Do not re-do work that is already evidenced

If partial work is inconsistent:
- Identify the discrepancy
- Roll back to the last consistent state
- Re-start from there

### Step 4 — Never assume prior session context

Even if you "remember" what happened before, verify against the truth ledger. Memory is not truth. Files are truth.

---

## What the Harness Must Fail On

The session is invalid if any of these occur:

| Failure Condition | Why |
|------------------|-----|
| Truth ledger not read at session start | Agent may work on wrong capability |
| No approved plan for the current capability | Implementation without planning |
| State skipped (e.g., Not Started → Live) | Part 14 violation |
| Forbidden claim language without evidence | Inflation |
| Next capability started without acceptance of current | Phase integrity |
| Files modified outside approved scope | Scope violation |
| Session ends without summary | No audit trail |
| Truth ledger not updated after state change | Work is invalid |

---

## Session Record Format

Each session should produce a record that can be stored or referenced:

```
{
  "session_date": "YYYY-MM-DD",
  "objective": "...",
  "capability": "...",
  "phase": N,
  "start_state": "...",
  "end_state": "...",
  "files_changed": [...],
  "evidence": [...],
  "ledger_updated": true/false,
  "proposed_transition": "... → ...",
  "blockers": [...],
  "next_move": "..."
}
```

This record is the proof that the session was disciplined and scoped.

---

## Standard Session Header (MANDATORY)

The agent MUST output this complete header at the start of every implementation session. If any section is incomplete, the session MUST NOT proceed.

```
## Session Header

### Session Context
- Phase: [N — name]
- Capability: [name]
- Authority Level: [READ_ONLY | PLAN_ONLY | IMPLEMENT_ALLOWED | VERIFY_ONLY]
- Current State: [Not Started | Implemented | Wired | Live | Proven]
- Target State: [next state to reach]

### Scope Declaration
- Allowed Files: [explicit list of files that may be modified]
- Allowed System Surface: [which canisters/modules are in scope]
- Restricted Areas: [what must NOT be touched]

### Capability Boundaries
- Upstream Dependencies: [capabilities this one depends on]
- Downstream Dependencies: [capabilities that will depend on this one]
- No-Touch Areas: [capabilities/files explicitly excluded]

### Explicit Non-Goals
- [thing 1 that this session will NOT do]
- [thing 2]
- [thing 3]

### Risk Awareness
- [known risk 1 for this capability]
- [known risk 2]

### Substrate Awareness
- [key AxiaSystem components that will be used]
- [known substrate limitations]

### Truth Ledger Impact
- CAPABILITY_STATUS.md: [what will change]
- PHASE_SUMMARY.md: [what will change]
- KNOWN_LIMITATIONS.md: [what may be added]

### Next Step: [first concrete action after header validation]
```

### Enforcement

If the header is incomplete or missing any section → **STOP the session**. Do not proceed with partial context.

---

## Explicit Non-Goals System

Every session MUST include a list of explicit non-goals — things this session will NOT do.

### Purpose

Non-goals prevent scope drift by making boundaries visible before work begins.

### Examples of good non-goals

- "Will NOT implement explain_decision (that is a separate capability)"
- "Will NOT modify governance canister logic"
- "Will NOT add historical querying to decision records"
- "Will NOT fix unrelated canister bugs unless they block this capability"

### Enforcement

If the agent takes an action that touches a declared non-goal → **STOP and report violation**.

Non-goals cannot be silently removed during a session. If a non-goal needs to be revised, it must be explicitly acknowledged and justified.

---

## Mid-Session Checkpoint Protocol

At every major step during implementation, the agent MUST pause and check:

### Checkpoint Template

```
Checkpoint:
- Still within declared scope? [YES/NO]
- Authority level still valid? [YES/NO]
- Any new unknowns discovered? [list or NONE]
- Any non-goal touched? [YES/NO]
- Truth ledger still accurate? [YES/NO]
```

### When to checkpoint

- After completing a major implementation step (e.g., types defined, gates built, function wired)
- After any unexpected error or substrate discovery
- After any file modification outside the primary implementation file
- Before running tests on local replica
- Before updating the truth ledger

### Enforcement

If any checkpoint answer is NO → **STOP and assess** before continuing.

If a new unknown is discovered that changes the implementation approach → document it and reassess scope before proceeding.

---

## Failure Condition Updates

The session is invalid if any of these additional conditions occur:

| Failure Condition | Why |
|------------------|-----|
| Session header not output at start | No context established |
| Authority level not declared | No permission model |
| Non-goals not declared | No drift boundaries |
| Authority level violated (e.g., code change in PLAN_ONLY) | Permission breach |
| Non-goal action taken without acknowledgment | Scope violation |
| Checkpoint skipped at major step | No drift detection |
| Upstream inconsistency found but not documented | Silent failure |

---

## Session Memory Compression

Long sessions accumulate context. After major milestones, the agent must compress to prevent context bloat and token inefficiency.

### When to compress

- After a capability reaches a new state (Implemented, Wired, Live, Proven)
- After resolving a complex upstream fix
- After any evidence run producing >20 lines of output
- When the agent notices it is re-explaining prior decisions

### Compression protocol

After a milestone, produce a compressed state summary:

```
## Compressed State

### Decisions made:
- [key decision 1]
- [key decision 2]

### Constraints active:
- [constraint 1]
- [constraint 2]

### Open questions:
- [question 1]

### Discard:
- Intermediate reasoning that led to decisions (decisions are final)
- Raw test output already captured in truth ledger
- Substrate analysis details already recorded in implementation plan
```

### What to retain

- Current capability state and evidence summary
- Active constraints and non-goals
- Known blockers and open questions
- Truth ledger current values

### What to discard

- Intermediate reasoning chains (the decision matters, not the path to it)
- Redundant re-statements of prior context
- Raw command output already recorded in evidence
- Substrate exploration details already in the plan document
