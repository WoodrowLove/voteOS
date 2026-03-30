# VoteOS Autonomous Build — Launch Prompt

> Copy this entire prompt into a new Claude Code session pointed at /home/woodrowlove/voteOS

---

## SESSION: VoteOS Wave 1-2 Autonomous Build

You are building VoteOS — a sovereign election and governance legitimacy system.

### Before you write ANY code:

1. Read CLAUDE.md — it has your ecosystem paths, pattern sources, reuse rules
2. Read docs/agent/ITERATION_HARNESS.md — it defines your build loop
3. Read docs/agent/PATTERN_REFERENCE.md — it has the exact code patterns to follow
4. Read docs/voteos-design/MODULE_REGISTRY.md — it defines the modules
5. Read docs/voteos-design/MODULE_SEQUENCE_PLAN.md — it defines the build order
6. Read docs/voteos-design/CAPABILITY_MAP.md — it defines the 100 capabilities
7. Read docs/voteos-design/VOTEOS_COMPLETION_STANDARD.md — it defines done

### Your mission:

Execute Waves 1-2 of VoteOS implementation:

**Wave 1: Foundation**
- Module 1: Voter Registry (12 capabilities)
- Module 2: Election Management (14 capabilities)

**Wave 2: Core Action**
- Module 3: Ballot Operations (10 capabilities)
- Module 4: Vote Recording (11 capabilities)

### How to work:

Follow the ITERATION HARNESS build loop:
1. Read SESSION_STATE.md to know where you are
2. Build the next uncompleted module
3. `cargo build` — must pass
4. `cargo test` — run and classify results
5. Evaluate against MODULE COMPLETION STANDARD
6. Update SESSION_STATE.md honestly
7. Commit + push
8. Proceed to next module

### CRITICAL RULES:

1. **Read ../civilOS/src/spine/client.rs FIRST** — copy the SpineClient pattern exactly
2. **Read ../civilOS/src/domain/store.rs FIRST** — copy the DomainStore pattern exactly
3. **Read ../civilOS/src/error.rs FIRST** — copy the WorkflowError pattern exactly
4. **DO NOT create new AxiaSystem capabilities** — compose from the existing 11
5. **DO NOT create a new identity/legitimacy/attestation system** — use AxiaSystem
6. **Use `requesting_system: "voteos"` in all legitimacy calls** (not "civilos")
7. **Every workflow follows: evaluate → [action] → attest → explain**
8. **Tests must use `.expect()` for happy paths** — no mixed-path assertions
9. **Commit after each completed module** — push to remote
10. **Update SESSION_STATE.md after every iteration** — honest status

### What to build for each module:

For each module in the wave:
1. Domain types (structs, enums) in `src/domain/`
2. Registry wrapping DomainStore in `src/domain/`
3. Dedicated workflows in `src/workflows/`
4. Module-level tests in `tests/`
5. Update `src/lib.rs` to export new modules

### When to stop:

- All Wave 1-2 modules are COMPLETE or CONDITIONALLY_COMPLETE
- OR you hit a blocking dependency requiring human input
- OR 3 consecutive iterations fail to make progress

Report your final state including:
- Which modules are complete
- Which tests pass/fail
- What's blocked and why
- What the next step should be

### Start now. Read the docs, then build Module 1: Voter Registry.
