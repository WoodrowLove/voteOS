# Part 14 — Axia Capability Execution Contracts

## Execution Contracts V1

> This document defines the execution truth standard for building AxiaSystem's platform surface. It prevents false completion, vague progress claims, and "compiled = done" thinking. It governs how implementation claims are made from this point forward.

---

## Governing Principle

> **Implemented ≠ Wired ≠ Live ≠ Proven**
>
> These are four different states. No capability may be described as complete unless it satisfies the evidence required for its current target state.

---

## Section 1 — Global Execution State Definitions

### State 1: Implemented

The capability exists in code in a meaningful way.

**Must include:**
- Real code path exists (not placeholder, not stub, not TODO)
- No fake handlers or pass-through mocks in the implementation path
- Contract input/output types are defined and used
- Orchestration logic is written (for composed capabilities)
- Error handling for defined failure modes exists

**Must NOT imply:**
- External integration is functional
- Runtime dependency selection works
- Real data flows through the system
- Behavior is proven correct
- The capability can be called from outside the module

**Evidence required to claim this state:**
- File paths to implementation code
- Function/module names
- Confirmation that no stubs or placeholders remain in the critical path

---

### State 2: Wired

The capability is connected to the intended runtime path.

**Must include:**
- The implementation is invoked from the intended access path (API endpoint, inter-canister call, gateway)
- Dependency paths are connected (the capability can reach the canisters it needs)
- Internal orchestration path is in place (multi-canister flows are wired end-to-end)
- Auth checks are in the call path (even if not yet validated against real sessions)

**Must NOT imply:**
- Successful real execution has occurred
- Stable runtime behavior is proven
- End-to-end correctness across all paths
- Production deployment readiness

**Evidence required to claim this state:**
- Endpoint or entry point exists and routes to implementation
- Dependency connectivity is demonstrated (canister calls resolve, not just compile)
- A call can be made that reaches the implementation code (even if it fails on data)

---

### State 3: Live

The capability executes against real dependencies in a runtime environment.

**Must include:**
- Real runtime environment (local replica, testnet, or production)
- Real dependency connectivity (actual canister calls, not mocked responses)
- Non-simulated execution (real data in, real data out)
- At least one successful end-to-end execution with real-shaped data

**Must NOT imply:**
- Complete coverage of all input variations
- Production readiness
- Sufficient audit proof
- Performance or scale validation
- All failure modes have been exercised

**Evidence required to claim this state:**
- Runtime environment identified (local replica, testnet, etc.)
- At least one successful call with real-shaped input producing contract-conformant output
- Dependency connectivity proof (the capability actually called its dependent canisters)
- Output matches Part 11 response schema

---

### State 4: Proven

The capability has passed its required verification and may be relied upon.

**Must include:**
- Explicit test evidence (unit + integration)
- Runtime evidence (live execution against real dependencies)
- Contract conformance proof (request schema, response schema, error codes, auth levels all match Part 11)
- Failure mode coverage (defined failure modes from Part 11 are tested and produce correct error responses)
- Idempotency verification (for idempotent capabilities)
- Honest limitations documented (what is NOT yet covered, what edge cases remain)

**This is the ONLY state that supports strong completion language.**

**Evidence required to claim this state:**
- All evidence from Implemented + Wired + Live states
- Test results (with specific test names and outcomes)
- Contract conformance checklist (each Part 11 field verified)
- Failure mode test results
- Documented limitations list

---

### State Progression Summary

```
Implemented → Wired → Live → Proven
     │            │        │        │
     │            │        │        └─ "Proven against [environment]"
     │            │        └─ "Live on [environment]"
     │            └─ "Wired to [access path]"
     └─ "Implemented in [file/module]"
```

A capability at state N has satisfied ALL requirements of states 1 through N.

---

## Section 2 — Claim Language Rules

### Approved Language

| State | Approved Claims |
|-------|----------------|
| Implemented | "Implemented but not wired", "Implementation exists in [path]", "Code path complete, not yet connected" |
| Wired | "Wired but not yet live", "Connected to [access path], awaiting runtime proof", "Orchestration path in place" |
| Live | "Live on [environment]", "Executing against real dependencies on [env]", "Live with local-only proof", "Live with limitations: [list]" |
| Proven | "Proven against [environment] with [evidence type]", "Contract-conformant on [environment]", "Verified with [test count] tests covering [scope]" |
| Partial | "Partial support — [what works] implemented, [what doesn't] missing", "Partial — orchestration layer incomplete" |
| Blocked | "Blocked on [dependency]", "Cannot proceed until [prerequisite]" |

### Forbidden Language

The following terms are **forbidden** unless accompanied by explicit justification referencing this document's evidence requirements:

| Forbidden | Why |
|-----------|-----|
| "Complete" | Implies Proven state without evidence |
| "Done" | Ambiguous — which state? |
| "Integrated" | Implies Wired + Live without distinguishing |
| "Production-ready" | Implies Proven + deployment validation + scale testing |
| "Fully working" | Implies all paths, all failure modes, all edge cases |
| "Connected to AxiaSystem" | Implies Live without specifying which dependencies |
| "Platform capability exists" | Implies Proven without evidence |
| "Triad works" | Implies Proven end-to-end triad resolution |

### The Inflation Test

Before making any capability claim, apply this test:

> **"If an independent auditor read this claim and tried to use the capability based on it, would they succeed?"**
>
> If the answer is no or maybe, the claim is inflated. Rewrite it with the correct state and limitations.

---

## Section 3 — Capability-Level Execution Contracts (Phase 1)

---

### Capability: resolve_subject

#### A. What Counts as Implemented

- [ ] Subject orchestration module exists with real code (not stubs)
- [ ] Module accepts the Part 11 request schema: identification (type + material), verification_level, financial_account (link + account_reference), context
- [ ] Module calls user canister (registerUserComplete or ensureIdentityAndWallet)
- [ ] Module calls identity canister (ensureIdentity, getIdentity, setAssuranceLevel)
- [ ] Module calls wallet canister (ensureWallet, getFullWalletProfile)
- [ ] Module composes the Part 11 response schema: subject_ref, resolution_type, identity, standing, financial_account, legitimacy
- [ ] Error handling exists for: INVALID_IDENTIFICATION, VERIFICATION_FAILED, ACCOUNT_LINK_FAILED, SYSTEM_UNAVAILABLE
- [ ] No placeholders in the orchestration path

#### B. What Counts as Wired

- [ ] POST /v1/resolve_subject endpoint exists and routes to the orchestration module
- [ ] Orchestration module can reach user, identity, and wallet canisters (inter-canister call paths resolve)
- [ ] System-level auth check is in the call path
- [ ] A request can be sent to the endpoint and reaches the orchestration code (even if it fails on missing data)

#### C. What Counts as Live

- [ ] Endpoint called on a running environment (local replica minimum)
- [ ] A new subject can be created: request with new identification material → response with subject_ref + financial_account
- [ ] An existing subject can be resolved: request with known identification → same subject_ref returned (idempotency proven)
- [ ] Response matches Part 11 schema exactly (all fields present, correct types)
- [ ] User, identity, and wallet canisters all received real calls (not mocked)

#### D. What Counts as Proven

- [ ] All Live criteria satisfied
- [ ] Unit tests for orchestration logic (at least: happy path, missing identification, verification failure, account link failure)
- [ ] Integration test: create new subject end-to-end
- [ ] Integration test: resolve existing subject (idempotency)
- [ ] Integration test: system unavailable handling (canister unreachable)
- [ ] Response schema validated field-by-field against Part 11
- [ ] All 4 error codes produce correct error responses
- [ ] Standing correctly reflects user active status + identity assurance level
- [ ] Limitations documented (e.g., "verification_level enhanced not yet supported")

#### E. Minimum Evidence Required

- Unit tests: 4+ (happy path, each error type)
- Integration tests: 3+ (create, resolve, failure)
- Runtime proof: 1+ successful end-to-end on local replica
- Schema proof: field-by-field Part 11 comparison
- Limitations doc: explicit list of what is not yet covered

#### F. Forbidden Claims Before Proof

- Cannot claim "subject resolution works" without idempotency proof
- Cannot claim "triad creation is complete" without wallet linkage verification
- Cannot claim "canonical subject reference" without demonstrating stability across resolve calls
- Cannot claim "standing is accurate" without showing assurance level integration

---

### Capability: authenticate_subject

#### A. What Counts as Implemented

- [ ] Unified auth module exists with real code
- [ ] Module accepts Part 11 request schema: subject_ref, credentials (type + primary + secondary), device_context, requested_scope
- [ ] Module calls user canister (validateLogin)
- [ ] Module calls identity canister (verifySession or creates session)
- [ ] Module produces Part 11 response schema: session (session_ref, subject_ref, authenticated_at, expires_at, scope), standing (status, valid_to_proceed)
- [ ] Error handling exists for: INVALID_CREDENTIALS, SUBJECT_NOT_FOUND, ACCOUNT_SUSPENDED, DEVICE_REJECTED, SYSTEM_UNAVAILABLE
- [ ] Session storage exists (session_ref maps to subject + expiration + scope)

#### B. What Counts as Wired

- [ ] POST /v1/authenticate_subject endpoint exists and routes to auth module
- [ ] Auth module can reach user and identity canisters
- [ ] No prior authentication required (public endpoint)
- [ ] A request can be sent and reaches the auth code

#### C. What Counts as Live

- [ ] Endpoint called on running environment
- [ ] A known subject can authenticate with valid credentials → session_ref returned
- [ ] Invalid credentials produce INVALID_CREDENTIALS error
- [ ] Suspended subject produces ACCOUNT_SUSPENDED error
- [ ] Session_ref is usable (can be passed to subsequent capability calls)
- [ ] Response matches Part 11 schema exactly

#### D. What Counts as Proven

- [ ] All Live criteria satisfied
- [ ] Unit tests: happy path, invalid credentials, subject not found, account suspended, system unavailable
- [ ] Integration test: authenticate → receive session → use session in subsequent call
- [ ] Integration test: expired session handling
- [ ] Session expiration works (session_ref becomes invalid after expires_at)
- [ ] Non-idempotency verified (each successful auth creates a new session)
- [ ] All 5 error codes produce correct responses
- [ ] Limitations documented

#### E. Minimum Evidence Required

- Unit tests: 5+ (happy path, each error type)
- Integration tests: 3+ (auth flow, session usage, session expiration)
- Runtime proof: 1+ successful auth → session → use on local replica
- Schema proof: field-by-field Part 11 comparison
- Session lifecycle proof: creation + usage + expiration

#### F. Forbidden Claims Before Proof

- Cannot claim "authentication works" without session lifecycle proof (create + use + expire)
- Cannot claim "sessions are valid" without demonstrating session_ref accepted by another capability
- Cannot claim "device context binding" without showing device_context influences auth behavior
- Cannot claim "standing confirmed" without showing suspended subjects are rejected

---

### Capability: resolve_system_state

#### A. What Counts as Implemented

- [ ] System state module exists with real code
- [ ] Module accepts Part 11 request schema: actor (subject_ref + session_ref), query (scope + target_ref + detail_level)
- [ ] Module routes by scope: platform_health, dependency_status, asset_state, sync_context, all
- [ ] Module calls NamoraAI (getSystemHealth) for platform_health
- [ ] Module calls individual canister healthCheck() endpoints for dependency_status
- [ ] Module calls asset_registry for asset_state scope
- [ ] Module provides sync_context (authoritative timestamp + ordering reference)
- [ ] Error handling for: INVALID_SCOPE, TARGET_NOT_FOUND, ACCESS_DENIED, SYSTEM_UNAVAILABLE

#### B. What Counts as Wired

- [ ] POST /v1/resolve_system_state endpoint exists and routes to system state module
- [ ] Module can reach NamoraAI, admin2, and at least 2 other canisters for health
- [ ] Subject-authenticated auth check is in the call path
- [ ] Scope routing reaches the correct internal path for each scope value

#### C. What Counts as Live

- [ ] Endpoint called on running environment
- [ ] platform_health scope returns real health data from NamoraAI
- [ ] dependency_status scope returns real status from at least 2 canisters
- [ ] sync_context scope returns an authoritative timestamp
- [ ] all scope returns combined data
- [ ] Response matches Part 11 schema
- [ ] Read-only verified (no state changes from any query)

#### D. What Counts as Proven

- [ ] All Live criteria satisfied
- [ ] Unit tests: each scope routing path, invalid scope, target not found
- [ ] Integration test: platform_health against real NamoraAI
- [ ] Integration test: dependency_status against multiple real canisters
- [ ] Integration test: sync_context produces consistent timestamps
- [ ] Integration test: all scope composes correctly
- [ ] All 4 error codes produce correct responses
- [ ] Idempotency verified (same query → same result at same time)
- [ ] Limitations documented

#### E. Minimum Evidence Required

- Unit tests: 5+ (each scope + error paths)
- Integration tests: 4+ (each scope against real canisters)
- Runtime proof: all scopes produce data on local replica
- Schema proof: field-by-field Part 11 comparison per scope
- Read-only proof: state unchanged after queries

#### F. Forbidden Claims Before Proof

- Cannot claim "system observability works" without proving all scope paths return real data
- Cannot claim "dependency monitoring active" without real canister health check evidence
- Cannot claim "sync context available" without demonstrating authoritative timestamps
- Cannot claim "asset state queryable" without asset_registry integration proof

---

## Section 4 — Phase 1 Completion Contract

### Phase 1 IS complete when ALL of the following are true:

- [ ] `resolve_subject` is at **Proven** state
- [ ] `authenticate_subject` is at **Proven** state
- [ ] `resolve_system_state` is at **Proven** state
- [ ] All three produce Part 11 contract-conformant responses
- [ ] All three handle their defined failure modes correctly
- [ ] All three have documented limitations
- [ ] Session from `authenticate_subject` is accepted by `resolve_system_state`
- [ ] Subject from `resolve_subject` can authenticate via `authenticate_subject`
- [ ] Cross-capability flow proven: resolve → authenticate → query system state
- [ ] Truth artifacts updated (status docs, phase summary, capability tracker)
- [ ] No forbidden claim language remains in any documentation

### Phase 1 is NOT complete if any of these are true:

- Only contracts/schemas exist (no implementation)
- Only isolated unit tests pass (no integration evidence)
- Only one or two capabilities are proven (all three required)
- Stubbed or mocked integrations are in the critical path
- Canister calls are simulated rather than real
- Repo compiles but no runtime proof exists
- Agent claims completion without evidence artifacts
- Cross-capability flow is untested (each works alone but not together)
- Limitations are undocumented

---

## Section 5 — Required Evidence Types

### A. Code Evidence

| What | How to Verify |
|------|--------------|
| Implementation exists | File paths + function names in implementation tracking doc |
| No stubs in critical path | Grep for TODO, STUB, PLACEHOLDER, unimplemented in implementation files |
| Contract types used | Request/response types match Part 11 schemas |

### B. Contract Evidence

| What | How to Verify |
|------|--------------|
| Request schema alignment | Field-by-field comparison: Part 11 request schema vs. actual accepted input |
| Response schema alignment | Field-by-field comparison: Part 11 response schema vs. actual output |
| Auth behavior alignment | Verify correct auth level enforced (system-level, public, subject-authenticated) |
| Failure mode alignment | Each defined error code from Part 11 is produced by the correct condition |

### C. Runtime Evidence

| What | How to Verify |
|------|--------------|
| Real execution path | Call trace showing request → orchestration → canister calls → response |
| Dependency connectivity | Evidence that dependent canisters received and processed calls |
| Actual output | Response body from a real call, compared to Part 11 schema |
| Environment identified | Which environment (local replica, testnet, production) |

### D. Test Evidence

| What | How to Verify |
|------|--------------|
| Unit tests | Test names, pass/fail status, what they cover |
| Integration tests | Test names, environment, pass/fail status, what cross-boundary paths they exercise |
| Edge case tests | Specific failure modes and boundary conditions tested |

### E. Verification Evidence

| What | How to Verify |
|------|--------------|
| Health/status checks | Canister health endpoints responding |
| Idempotency proof | Same input called twice → same output, no duplicate side effects |
| Session lifecycle | Session created → used → expired → rejected |
| Traceable proof artifacts | Specific test run IDs, timestamps, output files |

---

## Section 6 — Phase Truth Ledger Update Rule

After each capability reaches a new state (Implemented, Wired, Live, Proven), the following truth artifacts **must** be updated:

### Required Updates

1. **Capability Status Tracker** — a single file tracking each capability's current state, last evidence date, and known limitations
2. **Phase Summary** — updated to reflect overall phase progress
3. **Implementation tracking doc** — file paths, module names, evidence references for the capability
4. **Known Limitations List** — what is explicitly not yet supported for each capability

### Update Rule

> **No state transition is real until it is recorded in truth artifacts.**
>
> An agent may not claim a capability has moved to a new state unless the truth artifacts have been updated in the same work session.

### Required Tracking Files (create before implementation begins)

| File | Purpose | Location |
|------|---------|----------|
| `CAPABILITY_STATUS.md` | Per-capability state tracker | `docs/implementation/` |
| `PHASE_SUMMARY.md` | Phase-level progress | `docs/implementation/` |
| `KNOWN_LIMITATIONS.md` | Per-capability limitation list | `docs/implementation/` |

If these files do not yet exist, they **must** be created before the first capability reaches Implemented state.

---

## Section 7 — Review Gate Before Phase 2

### Hard Gate

> **Phase 2 (`evaluate_legitimacy`) must not begin full implementation until Phase 1 is Proven according to this document.**

### What is Allowed Before Gate Clears

- Preliminary design exploration for `evaluate_legitimacy`
- Policy engine architecture sketching
- Decision record store schema design
- Reading and understanding existing admin2/governance/identity authorization patterns

### What is NOT Allowed Before Gate Clears

- Building the policy evaluation engine
- Creating the decision record store
- Wiring `evaluate_legitimacy` to an endpoint
- Claiming any implementation progress on Phase 2 capabilities

### Why This Gate Exists

`evaluate_legitimacy` depends on:
- Subject resolution working correctly (identity context must be reliable)
- Authentication working correctly (session context must be trustworthy)
- System state being observable (legitimacy evaluation should be aware of platform health)

If Phase 1 is not proven, Phase 2 will be built on unverified assumptions about identity, auth, and system state — recreating the failure pattern of building on unproven foundations.

---

## Section 8 — Recommended Verification Pattern

Every capability implementation should follow this sequence:

```
Step 1 — Implement
    Write the orchestration code.
    Define contract types.
    Handle failure modes.
    → Update status to "Implemented in [path]"
    → Update CAPABILITY_STATUS.md

Step 2 — Verify Contract Alignment
    Compare request schema field-by-field to Part 11.
    Compare response schema field-by-field to Part 11.
    Verify error codes match.
    Verify auth level matches.
    → Document any deviations or limitations

Step 3 — Wire Runtime Path
    Connect endpoint to implementation.
    Connect implementation to dependent canisters.
    Verify call path resolves (not just compiles).
    → Update status to "Wired to [access path]"
    → Update CAPABILITY_STATUS.md

Step 4 — Test Local Proof
    Execute on local replica (or equivalent).
    Verify at least one happy path produces correct output.
    Verify at least one failure path produces correct error.
    → Update status to "Live on [environment]"
    → Update CAPABILITY_STATUS.md

Step 5 — Document Limitations
    List what is not yet covered.
    List known edge cases.
    List any contract deviations with rationale.
    → Update KNOWN_LIMITATIONS.md

Step 6 — Mark State Honestly
    Apply the Inflation Test.
    Use only approved claim language.
    → Update CAPABILITY_STATUS.md with final state

Step 7 — Update Truth Artifacts
    Update PHASE_SUMMARY.md.
    Commit evidence references.
    → No state transition is real until recorded
```

### This pattern applies to ALL capabilities in ALL phases.

---

## Summary

| Rule | Description |
|------|-------------|
| **States are distinct** | Implemented ≠ Wired ≠ Live ≠ Proven |
| **Claims require evidence** | No state transition without matching evidence type |
| **Language is controlled** | Forbidden terms cannot be used without explicit justification |
| **Phase gates are hard** | Phase 2 cannot begin until Phase 1 is Proven |
| **Truth artifacts are mandatory** | Every state change is recorded or it didn't happen |
| **Limitations are first-class** | Documenting what doesn't work is as important as documenting what does |
| **The Inflation Test governs all claims** | "Would an independent auditor succeed based on this claim?" |

> **This document governs how all implementation claims are made from this point forward. No capability, no phase, and no milestone may be described as complete without satisfying these contracts.**
