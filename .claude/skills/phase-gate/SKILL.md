---
name: phase-gate
description: Verify whether a CivilOS rollout phase's exit criteria are met before proceeding to the next phase. Use before starting work on any new phase to confirm all prerequisites are satisfied at runtime — not just in code. Enforces the "never skip phases" rule from Part 3. Also use to audit the current state of any phase.
user-invocable: true
argument-hint: "phase-number"

---

# CivilOS Rollout Phase Gate

You enforce the **"never skip phases"** rule from Part 3. Before any new phase begins, you verify that all prerequisite phases have met their exit criteria — at runtime, not just in code structure.

## Reference Documents

Always read before evaluating:
- [Part 3 — Rollout Phases](../../../docs/conceptual/03-rollout-phases.md)
- [Part 10 — Production Capabilities](../../../docs/conceptual/10-AXIA_CAPABILITIES_V2.md)
- [Part 11 — API Contracts](../../../docs/conceptual/11-AXIA_API_CONTRACTS_V1.md)
- [Part 13 — Implementation Sequencing](../../../docs/implementation/13-AXIA_CAPABILITY_IMPLEMENTATION_SEQUENCE_V1.md)
- [Part 14 — Execution Contracts](../../../docs/implementation/14-AXIA_EXECUTION_CONTRACTS_V1.md)

### Truth Ledger (MUST read current state before any gate check)
- [CAPABILITY_STATUS.md](../../../docs/implementation/CAPABILITY_STATUS.md)
- [PHASE_SUMMARY.md](../../../docs/implementation/PHASE_SUMMARY.md)
- [KNOWN_LIMITATIONS.md](../../../docs/implementation/KNOWN_LIMITATIONS.md)

## Execution State Definitions (from Part 14)

A capability's state determines gate eligibility:
- **Not Started** — no code exists
- **Implemented** — code exists, no stubs, contract types defined
- **Wired** — connected to runtime path, dependencies reachable
- **Live** — executes against real dependencies, at least one successful call
- **Proven** — tested, contract-conformant, limitations documented

**A phase gate only opens when ALL capabilities in that phase are at Proven state.**

## Phase Definitions and Exit Criteria

### Phase 0 — Platform Readiness
**Owner:** AxiaSystem (not CivilOS)
**Exit criteria:**
- [ ] Platform is callable (at least one endpoint responds)
- [ ] Platform capabilities are identifiable (capability list is discoverable)
- [ ] Platform is usable by an external system (auth works, responses are structured)
**Verification method:** Attempt a real call to `resolve_system_state` or health endpoint.

### Phase 1 — CivilOS Foundation
**Owner:** CivilOS
**Exit criteria:**
- [ ] A city can exist inside CivilOS (city registration works)
- [ ] Users can belong to departments (department assignment works)
- [ ] Roles and permissions are defined (role system is functional)
- [ ] Access is partitioned correctly (cross-department isolation proven)
**Verification method:** Create a city, add departments, assign users with roles, verify partition isolation.

### Phase 2 — Observability Layer
**Owner:** CivilOS
**Exit criteria:**
- [ ] Operators can see system state (dashboard or probe returns data)
- [ ] System dependencies are visible (platform status is queryable)
- [ ] External connectivity is provable (real probe, not mocked)
- [ ] No false claims of integration (local-only mode is distinguishable)
**Verification method:** Call `resolve_system_state`, verify real data. Check local vs connected mode.

### Phase 3 — Platform Connectivity
**Owner:** CivilOS + Platform
**Exit criteria:**
- [ ] CivilOS can prove it can reach the platform (real call succeeds)
- [ ] Real vs local behavior is distinguishable (mode is visible)
- [ ] Integration is no longer hypothetical (at least one bounded real call path)
**Verification method:** Execute a real platform call (e.g., `resolve_system_state` against live platform). Verify mode indicator.

### Phase 4 — Additional External Composition
**Exit criteria:**
- [ ] Multiple external dependencies are visible
- [ ] System can prove reachability to each
- [ ] System remains LOCAL_ONLY in behavior (no enforcement yet)

### Phase 5 — Controlled Composition
**Exit criteria:**
- [ ] Operators can see a unified system view
- [ ] System state is understandable (composed, not raw)
- [ ] Multiple dependencies are contextualized together

### Phase 6 — First Real Capability Consumption
**Exit criteria:**
- [ ] CivilOS performs real actions using the platform (e.g., `resolve_subject`)
- [ ] Actions are traceable and auditable
- [ ] System begins to move beyond observation
**Verification method:** Call `resolve_subject` with real data. Verify `attest_action` produces valid attestation.

### Phase 7 — Institutional Workflows
**Exit criteria:**
- [ ] Departments can operate real workflows
- [ ] CivilOS becomes operationally useful
- [ ] Workflows follow composition pattern (resolve → evaluate → act → attest)

### Phase 8 — Policy and Enforcement Integration
**Exit criteria:**
- [ ] System decisions are policy-aware (`evaluate_legitimacy` in real paths)
- [ ] Enforcement becomes real and reliable
- [ ] All enforcement is visible, explainable, auditable (`explain_decision` works)

### Phase 9 — City Deployment Model
**Exit criteria:**
- [ ] A new city can be onboarded with a defined process
- [ ] System is reproducible and scalable
- [ ] No shared data leakage across cities

### Phase 10 — Distribution and Packaging
**Exit criteria:**
- [ ] A city can install CivilOS
- [ ] System boots and verifies itself
- [ ] Dependencies are visible
- [ ] System is production-ready

## Evaluation Process

When invoked with `$ARGUMENTS` (a phase number):

1. **Read the current codebase** — scan for evidence of each exit criterion
2. **Check runtime truth** — look for test results, probe outputs, real call evidence (not just code structure)
3. **Check all prerequisite phases** — verify that phases 0 through N-1 are satisfied before approving phase N
4. **Flag any false completion** — compiled code or passing unit tests do NOT count as phase completion

## Anti-Patterns to Flag

| Anti-pattern | Why it fails |
|---|---|
| Building workflows before observability | Can't verify system state supports the workflow |
| Claiming integration before real connectivity | False completion, brittle at runtime |
| Adding enforcement before visibility | Enforcement becomes opaque and unauditable |
| Packaging before system truth established | Distributes unverified assumptions |
| Mocked platform calls treated as real | Hides integration gaps |

## Output Format

```
## Phase Gate: Phase [N] — [Phase Name]

### Prerequisites (Phases 0 to N-1)
| Phase | Status | Evidence |
|-------|--------|----------|
| 0     | PASS/FAIL/NOT_STARTED | [what was checked] |
| ...   | ...    | ...      |

### Exit Criteria for Phase [N]
| Criterion | Status | Evidence |
|-----------|--------|----------|
| [criterion] | MET / NOT MET / PARTIAL | [what was found] |

### Verdict: GATE OPEN | GATE BLOCKED | PREREQUISITES NOT MET

### Blockers (if any)
- [What must be done before this phase can proceed]

### Risks
- [Any false completion signals detected]
```
