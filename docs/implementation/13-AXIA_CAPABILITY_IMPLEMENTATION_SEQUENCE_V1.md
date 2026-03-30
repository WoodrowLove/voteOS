# Part 13 — Axia Capability Implementation Sequencing

## Implementation Sequence V1

> This document defines the controlled implementation sequence for transforming AxiaSystem into a platform-grade capability layer. It determines what gets built first, what depends on what, what can reuse existing logic, and what must be deferred. Based on ground truth from Part 12.

---

## Governing Principle

> You are not sequencing code. You are sequencing **platform capability emergence**.
>
> - Identity must exist before legitimacy
> - Legitimacy must exist before action
> - Action must exist before attestation
> - Attestation must exist before explanation

---

## Section 1 — Capability Implementation Matrix

---

### Capability: resolve_subject

**Current Status:** Partial
**Business Importance:** High
**Technical Complexity:** Medium

**Dependencies:**
- None. This is the root capability.

**Supporting Canisters / Modules:**
- `user` — createUser, registerUserComplete, ensureIdentityAndWallet, getCompleteUserInfo
- `identity` — createIdentity, ensureIdentity, getIdentity, setAssuranceLevel
- `wallet` — createWallet, ensureWallet, getFullWalletProfile
- `types/triad_shared.mo` — TriadIdentity, LinkProof, VerificationLevel
- `integration/triad_coordinator.mo` — CrossSystemOperation tracking

**Reuse vs New Build:**
Compose. The triad primitives (user + identity + wallet) exist with `ensureIdentityAndWallet` and `registerUserComplete` providing partial orchestration. Needs a thin orchestration layer that:
1. Accepts the contract's input shape (identification material + verification level + financial linkage intent)
2. Routes to the correct `ensure*` functions
3. Composes the contract's output shape (subject_ref + standing + financial_account + legitimacy)

**Required New Abstractions:**
- Subject orchestration module (thin — routes to existing canisters)
- Response normalization layer (maps existing canister outputs to contract schema)
- Standing concept (aggregating user.isActive + identity.assuranceLevel into contract's standing model)

**Risk Level:** Low — all primitives exist, orchestration is straightforward

**Why This Order Matters:**
Every other capability requires a `subject_ref`. Nothing works without identity. This is the foundation.

---

### Capability: authenticate_subject

**Current Status:** Partial
**Business Importance:** High
**Technical Complexity:** Medium

**Dependencies:**
- `resolve_subject` — subjects must exist before they can authenticate

**Supporting Canisters / Modules:**
- `user` — validateLogin (credential verification)
- `identity` — verifySession (session scope validation)
- `wallet` — SessionScope types
- `utils/authz.mo` — requireAuthenticated, requireCaller
- `ai_router` — session management (creation, validation, token rotation)

**Reuse vs New Build:**
Compose. `validateLogin` handles credential verification and `verifySession` handles session validation. Needs:
1. Unified session model that combines user auth + identity session + scope assignment
2. Session context object matching the contract response
3. Device context binding (registerDevice exists but is not in the auth flow)

**Required New Abstractions:**
- Unified session service (composes user auth + identity session into one session_ref)
- Session storage with expiration and scope tracking
- Device context integration into auth flow
- Lockout mechanism (attempts tracking, lockout_until)

**Risk Level:** Medium — session unification across canisters requires careful state management

**Why This Order Matters:**
Every subsequent capability requires `session_ref` from authentication. Cannot test or use any capability without auth.

---

### Capability: resolve_system_state

**Current Status:** Partial
**Business Importance:** Medium
**Technical Complexity:** Low

**Dependencies:**
- `authenticate_subject` — requires session for authenticated queries
- Can be partially built without auth (health endpoints can be unauthenticated during development)

**Supporting Canisters / Modules:**
- `namora_ai` — getSystemHealth, getInsights
- `admin2` — healthCheck
- `escrow` — healthCheck
- `payout` — healthCheck
- `nft` — healthCheck
- `bridge_orchestrator` — health.rs (bridge health aggregation)
- `asset_registry` — getAsset, getAssetsByOwner (asset state reads)

**Reuse vs New Build:**
Compose. NamoraAI's `getSystemHealth()` already returns overall + component status. Needs:
1. Scope-based routing (platform_health, dependency_status, asset_state, sync_context, all)
2. Aggregation of individual canister healthCheck() endpoints into dependency_status
3. Asset state read integration from asset_registry
4. Sync context (authoritative timestamp) — requires new lightweight service

**Required New Abstractions:**
- Scope router (routes to NamoraAI, individual canisters, asset_registry, or sync service based on scope parameter)
- Sync context provider (can be lightweight — IC system time + monotonic ordering counter)
- Response normalization to contract schema

**Risk Level:** Low — mostly read-only aggregation of existing endpoints

**Why This Order Matters:**
Part 3 mandates "truth before action" — observability must exist before the system performs real operations. Operators need to see system state before capabilities are exercised.

---

### Capability: evaluate_legitimacy

**Current Status:** Partial (critical gap)
**Business Importance:** Critical
**Technical Complexity:** High

**Dependencies:**
- `resolve_subject` — must resolve actor identity before evaluating legitimacy
- `authenticate_subject` — must have valid session context
- `resolve_system_state` — legitimacy evaluation should be aware of system health (degrade gracefully)

**Supporting Canisters / Modules:**
- `admin2` — defineRole, grantRole, listGrants, setFlag, getFlag, setEmergency
- `governance` — isIssuer, isDomainAdmin, isLegitimacyAdmin, grantDomainRole
- `identity` — setAssuranceLevel, getClaim, verifySession
- `ai_router` — policy evaluation, circuit breaker
- `utils/authz.mo` — requireAuthenticated, requireCaller, requireSystemCaller

**Reuse vs New Build:**
New (with reuse of existing role/authority data). This is the most significant new work in the entire system. Existing pieces:
- Admin2 provides role definitions and grants
- Governance provides domain authority checks
- Identity provides assurance levels and claims
- AI router provides request-level policy

What is missing:
- A **unified evaluation engine** that composes all of these into a single (actor, action, target, context) → (proceed / denied / requires_approval) function
- **Decision persistence** — evaluations must be stored with a `decision_ref` for later explanation
- **Approval-gate logic** — determining when an action requires additional approval and what level
- **Policy rule storage** — structured policy rules beyond feature flags
- **Data access policy** — currently no concept of data-level authorization beyond canister-local checks

**Required New Abstractions:**
- **Policy evaluation engine** — the core new module. Takes (actor, action, target, context), composes admin2 roles + governance authority + identity assurance + policy flags, returns unified decision
- **Decision record store** — persists every evaluation with decision_ref, full request, decision, policy references, timestamp
- **Policy rule store** — structured policy definitions (beyond admin2 feature flags) that the engine evaluates
- **Approval chain resolver** — when `requires_approval` is returned, determines the approval chain based on action type, department, city context

**Risk Level:** High — this is the most complex new capability, touches the most existing systems, and has the highest correctness requirements. An incorrect legitimacy evaluation is worse than no evaluation.

**Why This Order Matters:**
This is the platform's core invariant. Every mutating operation must pass through legitimacy evaluation. Without it, the platform cannot enforce its fundamental promise: no illegitimate actions. Building action capabilities (transfer, governance, assets) before legitimacy means those actions would operate without authorization — violating the composition pattern.

---

### Capability: attest_action

**Current Status:** Missing
**Business Importance:** High
**Technical Complexity:** High

**Dependencies:**
- `resolve_subject` — must know who is attesting
- `authenticate_subject` — must have valid session
- `evaluate_legitimacy` — actions being attested should have been evaluated (though attestation itself may not require re-evaluation)

**Supporting Canisters / Modules:**
- `governance` — audit trail with integrity hashing (pattern reference)
- `admin2` — AuditEvent, tailAudit (operational logging pattern)
- `namora_ai` — Audit + Trace systems (trace pattern)
- `xrpl_bridge` — aegis_audit.rs (bridge audit pattern)
- `types/triad_shared.mo` — CorrelationContext, FlowStep (correlation types)

**Reuse vs New Build:**
New. No attestation service exists. However, the system has multiple audit/trace patterns that inform the design:
- Governance's integrity hashing provides a tamper-evidence pattern
- Admin2's AuditEvent provides an event structure pattern
- Bridge's aegis_audit provides a cross-system audit pattern
- CorrelationContext provides a correlation ID pattern

What must be built:
- Cryptographic signature generation for actions
- Canonical `audit_ref` generation (globally unique, cross-system)
- Timestamp binding from authoritative source
- Signature verification endpoint (for independent verification)
- Attestation record storage

**Required New Abstractions:**
- **Attestation service** (new canister or module) — core signing + audit ref generation
- **Signature store** — immutable storage of attestation records
- **Verification endpoint** — allows external parties to verify attestations without platform access
- **Timestamp authority** — may share infrastructure with `resolve_system_state` sync context

**Risk Level:** High — cryptographic operations must be correct. An attestation service with bugs undermines the entire trust model. However, the scope is well-defined and does not depend on complex cross-canister state.

**Why This Order Matters:**
The composition pattern requires attestation as the final step: resolve → evaluate → act → attest. Without attestation, actions are untraceable and unprovable. Must exist before any action capabilities are exposed externally.

---

### Capability: explain_decision

**Current Status:** Missing
**Business Importance:** Medium
**Technical Complexity:** Medium

**Dependencies:**
- `evaluate_legitimacy` — hard dependency. Decisions must be stored before they can be explained. Cannot be built or tested without the decision record store from `evaluate_legitimacy`.

**Supporting Canisters / Modules:**
- Decision record store (created as part of `evaluate_legitimacy`)
- `admin2` — tailAudit (pattern for audit retrieval)

**Reuse vs New Build:**
New (but depends on `evaluate_legitimacy` infrastructure). The decision record store created for `evaluate_legitimacy` contains the raw evaluation data. This capability adds:
1. Human-readable explanation generation from stored policy evaluation results
2. Machine-readable breakdown with rule references and context factors
3. Detail level support (summary, detailed, audit_grade)

**Required New Abstractions:**
- **Explanation generator** — transforms stored decision records into human-readable and machine-readable explanations
- **Rule reference resolver** — maps policy rule IDs to human-readable names and descriptions

**Risk Level:** Medium — depends on the quality of decision records from `evaluate_legitimacy`. If decision records are well-structured, explanation generation is straightforward. If not, this becomes a significant rework.

**Why This Order Matters:**
Explanation is the "why" behind every authorization decision. It serves operators who need to understand denials, auditors who need to verify compliance, and agents who need to reason about alternative approaches. However, it cannot exist until decisions are being recorded.

---

### Capability: execute_transfer

**Current Status:** Partial
**Business Importance:** High
**Technical Complexity:** High

**Dependencies:**
- `resolve_subject` — must know sender and receiver identities
- `authenticate_subject` — must have valid session
- `evaluate_legitimacy` — transfer must be authorized (decision_ref required)
- `attest_action` — completed transfers must be attested

**Supporting Canisters / Modules:**
- `payment` — initiatePayment, approvePayment, completePayment, refundPayment
- `payout` — initiatePayout, executePayout
- `escrow` — createEscrow, releaseEscrow, cancelEscrow, refund requests
- `split_payment` — initiateSplitPayment, executeSplitPayment
- `wallet` — creditWallet, debitWallet, getWalletBalance
- `token` — transfer, mintToUser, bridgeMintToUser, bridgeBurnFromUser
- `subscriptions` — createSubscription, renewSubscription
- `treasury` — allocateFunds, withdrawFunds
- `bridge_orchestrator` — cross-chain payment routing
- `modules/refund_module.mo` — unified refund logic

**Reuse vs New Build:**
Compose (heavy). All financial primitives exist across 7+ canisters. Needs:
1. **Transfer type router** — accepts `execute_transfer` contract input and routes to the correct canister(s) based on transfer type
2. **Unified transfer_ref generation** — currently each canister generates its own IDs
3. **Compliance pre-check integration** — currently no compliance check on the transfer path
4. **Decision_ref linkage** — transfers must reference their legitimacy evaluation
5. **Response normalization** — map diverse canister responses to contract schema

**Required New Abstractions:**
- **Transfer orchestration module** — the router/coordinator that unifies all transfer types
- **Transfer reference service** — generates canonical, cross-type transfer_ref values
- **Compliance gate** — pre-transfer compliance check (may be part of `evaluate_legitimacy` or standalone)

**Risk Level:** High — financial operations must be correct. The diversity of existing canisters (7+) means the orchestration layer must handle many edge cases. Escrow lifecycle (place, release, return, dispute) adds complexity. Recurring payment scheduling adds state management.

**Why This Order Matters:**
This is the first real action capability. It depends on identity (who is transferring), legitimacy (is this allowed), and attestation (prove it happened). Building it before those dependencies exist would create unauthorized, untraceable financial operations.

---

### Capability: resolve_financial_state

**Current Status:** Partial
**Business Importance:** High
**Technical Complexity:** Medium

**Dependencies:**
- `resolve_subject` — must resolve target identity
- `authenticate_subject` — must have valid session
- `execute_transfer` — not a hard dependency, but financial state is most meaningful after transfers exist

**Supporting Canisters / Modules:**
- `wallet` — getWalletBalance, getWalletOverview, getFullWalletProfile, getTransactionHistory
- `treasury` — getTreasuryStats, getBalance
- `payment` — getPaymentStatus, getPaymentHistory, getPayment
- `token` — getBalanceOf, getBalancesForUser
- `escrow` — getEscrow, listEscrows
- `payout` — getPayoutDetails
- `split_payment` — getSplitPaymentDetails

**Reuse vs New Build:**
Compose. All data exists. Needs:
1. **Scope-based router** — routes to correct canisters based on scope parameter (account, department, treasury, transaction)
2. **Response aggregation** — combines multi-canister data into contract schema
3. **Transaction status unification** — currently each payment type has its own status query; needs unified lookup by reference

**Required New Abstractions:**
- **Financial state aggregator** — scope router + response normalizer
- **Transaction reference resolver** — given any transfer_ref (from execute_transfer), determines which canister owns it and retrieves status

**Risk Level:** Medium — read-only aggregation is lower risk than writes, but financial data must be accurate. Stale or inconsistent data across canisters is a concern.

**Why This Order Matters:**
Financial reads support transfer workflows (pre-validation, post-confirmation), operator dashboards, and compliance reporting. Should be available when transfers are available.

---

### Capability: register_asset

**Current Status:** Partial (strongest existing area)
**Business Importance:** Medium
**Technical Complexity:** Medium

**Dependencies:**
- `resolve_subject` — must know asset owner identity
- `authenticate_subject` — must have valid session
- `evaluate_legitimacy` — asset operations must be authorized

**Supporting Canisters / Modules:**
- `asset` — registerAssetTriad, transferAssetTriad, deactivateAssetTriad, batchTransferAssetsTriad
- `asset_registry` — registerAssetTriad, transferAssetTriad, getAsset, getAssetsByOwner
- `nft` — createNFT, transferNFT, transferNFTSafe, lockNFTForBridge

**Reuse vs New Build:**
Expose with adaptation. Triad-compliant functions are the closest to contract-ready of any capability. Needs:
1. **Canonical asset canister resolution** — asset and asset_registry have overlapping functions; must decide which is canonical
2. **Encumbrance system** — no lien/hold/restriction concept exists; needs addition
3. **Provenance query** — asset history exists as transfer events but is not exposed as a provenance chain
4. **Response enrichment** — add encumbrances array, provenance_length, operation_completed to responses

**Required New Abstractions:**
- **Encumbrance module** — new concept for asset liens, holds, and restrictions
- **Provenance query** — new function to retrieve ordered ownership history

**Risk Level:** Medium — the triad-compliant foundation is solid, but adding encumbrances is new domain logic that must integrate cleanly with existing transfer/deactivation flows.

**Why This Order Matters:**
Asset registration is valuable for CivilOS departments (property, permits, licenses) but is not on the critical path for the core legitimacy execution engine. Safe to build after the action layer is established.

---

### Capability: establish_governance_context

**Current Status:** Partial
**Business Importance:** Medium
**Technical Complexity:** Medium

**Dependencies:**
- `resolve_subject` — must know participant identities
- `authenticate_subject` — must have valid session
- `evaluate_legitimacy` — governance setup must be authorized

**Supporting Canisters / Modules:**
- `governance` — createProposal, isIssuer, isDomainAdmin, grantDomainRole, revokeDomainRole
- Proposal modules: UpgradeProposals, GeneralProposals, TriadGovernance

**Reuse vs New Build:**
Compose from existing + refactor. Proposal creation and domain role checking exist. Needs:
1. **Eligibility query** — take participant list, return per-participant eligibility based on governance context
2. **Contract-aligned response** — governance_ref, rules, eligibility results, threshold
3. **Governance canister refactoring** — separate proposal/voting from system administration (product registry, release registry, cycles treasury, config registry, health monitoring)
4. **Deadline and threshold exposure** — internal proposal parameters exposed as contract fields

**Required New Abstractions:**
- **Eligibility resolver** — given governance context + participants, returns eligibility per participant
- **Governance canister separation** — refactor overloaded governance canister into focused modules

**Risk Level:** Medium — governance canister refactoring is the main risk. The canister is complex and mixes concerns.

**Why This Order Matters:**
Governance capabilities support VoteOS and institutional decision-making. Important for CivilOS but not on the critical path for the core platform surface.

---

### Capability: record_governance_decision

**Current Status:** Partial
**Business Importance:** Medium
**Technical Complexity:** Medium

**Dependencies:**
- `establish_governance_context` — hard dependency. Decisions are recorded within a governance context.
- `resolve_subject` — must know participant identity
- `authenticate_subject` — must have valid session
- `attest_action` — governance decisions should be attested

**Supporting Canisters / Modules:**
- `governance` — voteOnProposal, executeProposal
- Audit trail with integrity hashing

**Reuse vs New Build:**
Compose from existing + enrich. Voting exists. Needs:
1. **Enriched vote response** — current tally, governance state, outcome (not just confirmation)
2. **Non-repudiation proof** — submission_ref + proof_ref per vote (can leverage attest_action)
3. **Double-vote prevention** — verify ALREADY_DECIDED error exists or add it
4. **State transition reporting** — return governance status after each vote (open, quorum_reached, closed_passed, etc.)

**Required New Abstractions:**
- **Vote response enrichment** — compose tally + state into response
- **Non-repudiation integration** — bind vote recording to attest_action

**Risk Level:** Medium — voting logic exists but enrichment requires careful integration with existing proposal state management.

**Why This Order Matters:**
Cannot record decisions without a governance context. Depends on `establish_governance_context` and benefits from `attest_action` for non-repudiation.

---

## Section 2 — Dependency Graph

```
                    ┌─────────────────┐
                    │ resolve_subject  │  ← ROOT (no dependencies)
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  authenticate_  │
                    │    subject      │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
    ┌─────────▼──────┐ ┌────▼─────────┐    │
    │ resolve_system_│ │  evaluate_   │    │
    │    _state      │ │  legitimacy  │    │
    └────────────────┘ └────┬─────────┘    │
                            │              │
                   ┌────────▼────────┐     │
                   │  attest_action  │     │
                   └────────┬────────┘     │
                            │              │
              ┌─────────────┼──────────────┤
              │             │              │
    ┌─────────▼──────┐ ┌───▼──────┐ ┌─────▼──────────┐
    │explain_decision│ │execute_  │ │register_asset  │
    └────────────────┘ │transfer  │ └────────────────┘
                       └────┬─────┘
                            │
                   ┌────────▼────────────┐
                   │resolve_financial_   │
                   │       state         │
                   └─────────────────────┘

    ┌────────────────────────┐
    │establish_governance_   │  ← depends on resolve_subject +
    │       context          │    authenticate_subject +
    └───────────┬────────────┘    evaluate_legitimacy
                │
    ┌───────────▼────────────┐
    │record_governance_      │  ← depends on establish_governance_context +
    │      decision          │    attest_action
    └────────────────────────┘
```

### Strict Sequential Chains

1. `resolve_subject` → `authenticate_subject` → `evaluate_legitimacy` → `attest_action` → `explain_decision`
2. `establish_governance_context` → `record_governance_decision`

### Parallelizable Work

- `resolve_system_state` can be built in parallel with `evaluate_legitimacy` (both depend only on auth)
- `register_asset` can be built in parallel with `execute_transfer` (both depend on evaluate_legitimacy + attest_action)
- `resolve_financial_state` can be built in parallel with `register_asset`
- `establish_governance_context` can be built in parallel with Phase 4 capabilities

---

## Section 3 — Implementation Phases

### Phase 1 — Platform Foundation

**Focus:** Identity + Authentication + Basic Observability

| Capability | Complexity | Approach |
|-----------|-----------|---------|
| `resolve_subject` | Medium | Compose from user + identity + wallet |
| `authenticate_subject` | Medium | Compose from user + identity session |
| `resolve_system_state` | Low | Compose from namora_ai + canister health |

**What gets built:**
- Subject orchestration module (thin layer over existing triad)
- Unified session service
- System state scope router + sync context provider
- Response normalization for all three capabilities

**Exit criteria:**
- Subjects can be resolved/created via contract endpoint
- Sessions can be established via contract endpoint
- System health is queryable via contract endpoint
- All responses match Part 11 schemas

**Estimated difficulty:** Medium — primitives exist, orchestration is the work

---

### Phase 2 — Legitimacy Core

**Focus:** Unified policy evaluation — the most critical new system

| Capability | Complexity | Approach |
|-----------|-----------|---------|
| `evaluate_legitimacy` | High | New orchestration + policy engine |

**What gets built:**
- Policy evaluation engine (core new module)
- Decision record store (new persistence)
- Policy rule store (structured policy definitions)
- Approval chain resolver
- Integration with admin2 roles + governance authority + identity assurance

**Exit criteria:**
- Any (actor, action, target, context) can be evaluated
- Decisions return proceed / denied / requires_approval
- Every decision is persisted with a decision_ref
- Decision_ref can be passed to subsequent capability calls
- Emergency states in admin2 affect evaluation results

**Estimated difficulty:** High — the most significant new work in the entire system. Requires careful design of the policy composition model.

---

### Phase 3 — Trust & Traceability

**Focus:** Verifiability + Auditability

| Capability | Complexity | Approach |
|-----------|-----------|---------|
| `attest_action` | High | New attestation service |
| `explain_decision` | Medium | New explanation generator (depends on Phase 2 decision store) |

**What gets built:**
- Attestation service (new canister/module)
- Cryptographic signing for actions
- Canonical audit_ref generation
- Attestation storage (immutable)
- Verification endpoint
- Explanation generator from decision records
- Rule reference resolver

**Exit criteria:**
- Any action can be attested with a cryptographic proof
- Attestations produce globally unique audit_ref values
- Attestations include authoritative timestamps
- Attestations can be verified independently
- Decision explanations are available at summary, detailed, and audit_grade levels

**Estimated difficulty:** High for attestation (crypto correctness), medium for explanation

---

### Phase 4 — Action Layer

**Focus:** Financial operations + Asset management

| Capability | Complexity | Approach |
|-----------|-----------|---------|
| `execute_transfer` | High | Compose from 7+ payment canisters |
| `resolve_financial_state` | Medium | Compose from wallet + treasury + payment |
| `register_asset` | Medium | Adapt existing triad-compliant functions |

**What gets built:**
- Transfer orchestration module (type-based router)
- Unified transfer_ref service
- Compliance gate integration
- Financial state aggregator
- Transaction reference resolver
- Asset encumbrance system
- Provenance query endpoint

**Exit criteria:**
- All transfer types (payment, payout, escrow, split, recurring) work through single endpoint
- Financial state queryable by scope (account, department, treasury, transaction)
- Assets can be registered, transferred, and encumbered
- All actions flow through evaluate_legitimacy → act → attest_action

**Estimated difficulty:** High for transfers (7+ canister coordination), medium for financial reads and assets

---

### Phase 5 — Governance Layer

**Focus:** Structured institutional decision-making

| Capability | Complexity | Approach |
|-----------|-----------|---------|
| `establish_governance_context` | Medium | Compose + governance refactor |
| `record_governance_decision` | Medium | Compose + response enrichment |

**What gets built:**
- Governance canister refactoring (separate voting from system admin)
- Eligibility resolver
- Enriched vote response (tally, state, outcome)
- Non-repudiation integration with attest_action
- Double-vote prevention verification

**Exit criteria:**
- Governance contexts can be established with rules, thresholds, and eligibility checks
- Decisions are recorded with tamper-evident proof
- Non-repudiation proofs are generated per decision
- Governance state transitions are reported after each decision

**Estimated difficulty:** Medium — governance primitives exist but canister refactoring is the main work

---

## Section 4 — Parallel vs Sequential Work

### Strictly Sequential (cannot be parallelized)

| Chain | Reason |
|-------|--------|
| `resolve_subject` → `authenticate_subject` | Cannot authenticate without identity |
| `authenticate_subject` → `evaluate_legitimacy` | Cannot evaluate without session |
| `evaluate_legitimacy` → `attest_action` | Attestation references decision_ref |
| `evaluate_legitimacy` → `explain_decision` | Cannot explain without decision store |
| `establish_governance_context` → `record_governance_decision` | Cannot record without context |

### Parallelizable Pairs

| Pair | Why Safe |
|------|----------|
| `resolve_system_state` ∥ `evaluate_legitimacy` | Both depend only on Phase 1; no mutual dependency |
| `execute_transfer` ∥ `register_asset` | Both depend on Phase 2+3; independent domains |
| `resolve_financial_state` ∥ `register_asset` | Both read-oriented or domain-independent |
| `establish_governance_context` ∥ Phase 4 capabilities | Governance and action are independent domains |
| `explain_decision` ∥ `attest_action` | Both depend on Phase 2 but not on each other |

### Within-Phase Parallelism

| Phase | Parallel Work |
|-------|--------------|
| Phase 1 | `resolve_subject` and `resolve_system_state` can start simultaneously; `authenticate_subject` starts after resolve_subject |
| Phase 3 | `attest_action` and `explain_decision` can be built in parallel |
| Phase 4 | All three capabilities can be built in parallel |
| Phase 5 | Both governance capabilities can be built in parallel (if governance canister refactoring is done first) |

---

## Section 5 — Exposure Readiness

### Phase 1 — Safe to Expose

| Capability | External Exposure | Rationale |
|-----------|------------------|-----------|
| `resolve_subject` | Yes — limited to authorized consuming systems | Identity resolution is safe (idempotent, no side effects on resolve) |
| `authenticate_subject` | Yes — public endpoint | Authentication is inherently public-facing |
| `resolve_system_state` | Yes — authenticated operators only | Read-only observability; no risk |

### Phase 2 — Safe to Expose

| Capability | External Exposure | Rationale |
|-----------|------------------|-----------|
| `evaluate_legitimacy` | Yes — to authorized systems | Evaluation is read-only; no action taken |

### Phase 3 — Safe to Expose

| Capability | External Exposure | Rationale |
|-----------|------------------|-----------|
| `attest_action` | Yes — to authenticated subjects | Attestation is safe (adds proof, does not modify the action) |
| `explain_decision` | Yes — to authorized subjects/auditors | Read-only explanation |

### Phase 4 — Expose After Full Composition Pattern Verified

| Capability | External Exposure | Rationale |
|-----------|------------------|-----------|
| `execute_transfer` | Yes — only after evaluate_legitimacy + attest_action are proven | Financial actions must never be exposed without the full composition pattern |
| `resolve_financial_state` | Yes — authenticated | Read-only financial data |
| `register_asset` | Yes — after legitimacy integration proven | Asset mutations require legitimacy gate |

### Phase 5 — Expose After Governance Refactoring

| Capability | External Exposure | Rationale |
|-----------|------------------|-----------|
| `establish_governance_context` | Yes — governance-authorized | Only after governance canister is refactored and eligibility resolver is proven |
| `record_governance_decision` | Yes — eligible participants only | Only after non-repudiation integration with attest_action is proven |

---

## Section 6 — First Deployable Surface

The smallest set of capabilities that forms a **usable platform**:

```
┌─────────────────────────────────────────────────────────┐
│          AxiaSystem v1 Platform Surface                   │
│                                                           │
│  resolve_subject          ← Who is this?                  │
│  authenticate_subject     ← Prove it.                     │
│  evaluate_legitimacy      ← Are they allowed?             │
│  attest_action            ← Prove it happened.            │
│                                                           │
│  + resolve_system_state   ← What is the system condition? │
│  + explain_decision       ← Why was this decided?         │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

**6 capabilities = complete legitimacy execution engine without action capabilities.**

This surface allows:
- Any consuming system to onboard subjects
- Any consuming system to authenticate
- Any consuming system to evaluate whether an action is legitimate
- Any consuming system to attest actions
- Any consuming system to query system health
- Any consuming system to understand authorization decisions

What it does NOT yet allow:
- Financial operations (Phase 4)
- Asset management (Phase 4)
- Governance workflows (Phase 5)

**This is intentional.** The legitimacy engine must be proven before action capabilities are exposed.

**Target designation:** Phases 1 + 2 + 3 = **AxiaSystem v1 Platform Surface**

---

## Section 7 — Deferred Capabilities

### Deferred to Phase 4

| Capability | Reason for Deferral |
|-----------|-------------------|
| `execute_transfer` | Requires legitimacy engine + attestation to be proven first. Financial operations without authorization and traceability are unacceptable. |
| `resolve_financial_state` | Most valuable after transfers exist. Can be partially available earlier for treasury reads, but full utility comes with Phase 4. |
| `register_asset` | Not on the critical path. Triad-compliant foundation is strong, so when it's time, build is faster. |

### Deferred to Phase 5

| Capability | Reason for Deferral |
|-----------|-------------------|
| `establish_governance_context` | Requires governance canister refactoring. Important for VoteOS integration but not blocking the core platform. |
| `record_governance_decision` | Depends on governance context + attestation integration. |

### Deferred Until Unclear Primitives Resolve

| Concern | What's Unclear |
|---------|---------------|
| Compliance canister | Part 12 found compliance report generation but no runtime evaluation. Whether a separate compliance canister is needed or whether `evaluate_legitimacy` absorbs compliance entirely is an open design question. |
| Cross-chain transfer integration | Bridge orchestrator exists but integration into the `execute_transfer` contract path requires bridge state management decisions that may affect the financial orchestration design. |
| Recurring payment lifecycle management | Subscriptions canister handles creation and renewal, but pause/cancel/modify for recurring transfers via `execute_transfer` needs design work. |

---

## Summary

| Phase | Capabilities | New Abstractions | Risk | Deliverable |
|-------|-------------|-----------------|------|-------------|
| **1** | resolve_subject, authenticate_subject, resolve_system_state | Subject orchestrator, session service, state router | Medium | Platform can identify, authenticate, and observe |
| **2** | evaluate_legitimacy | Policy engine, decision store, approval resolver | **High** | Platform can authorize — the core invariant |
| **3** | attest_action, explain_decision | Attestation service, explanation generator | High | Platform can prove and explain — trust established |
| **4** | execute_transfer, resolve_financial_state, register_asset | Transfer orchestrator, financial aggregator, encumbrance system | High | Platform can act on value and assets |
| **5** | establish_governance_context, record_governance_decision | Eligibility resolver, governance refactor | Medium | Platform supports institutional decisions |

> **Phases 1-3 produce the AxiaSystem v1 Platform Surface** — a complete legitimacy execution engine. Phases 4-5 add action and governance capabilities on top of the proven foundation.
