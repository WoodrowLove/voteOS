---
name: axia-contract-validator
description: Validate any code, module, or integration that calls AxiaSystem platform capabilities against the locked API contracts from Part 11. Use when writing or reviewing code that consumes platform endpoints, when checking request/response schemas, auth levels, error handling, idempotency, or composition pattern compliance. Also use to verify that no code invents unauthorized capabilities or bypasses the legitimacy checkpoint.
user-invocable: true
argument-hint: "file-path-or-module-name"

---

# Axia API Contract Validator

You validate any code that calls AxiaSystem platform capabilities against the **locked API contracts** defined in Part 11. Your job is to catch contract violations before they become runtime bugs or architectural drift.

## Canonical Contract Reference

Read this before every validation:
- [Part 11 — API Contract Design](../../../docs/conceptual/11-AXIA_API_CONTRACTS_V1.md)
- [Part 10 — Production Capabilities](../../../docs/conceptual/10-AXIA_CAPABILITIES_V2.md)

## The 11 Locked Endpoints

| # | Capability | Endpoint | Method | Auth | Mutates | Idempotent |
|---|-----------|----------|--------|------|---------|------------|
| 1 | `resolve_subject` | `POST /v1/resolve_subject` | POST | System-level | Conditional | Yes |
| 2 | `authenticate_subject` | `POST /v1/authenticate_subject` | POST | Public | Yes | No |
| 3 | `evaluate_legitimacy` | `POST /v1/evaluate_legitimacy` | POST | Subject-authenticated | No | Yes* |
| 4 | `explain_decision` | `POST /v1/explain_decision` | POST | Subject-authenticated | No | Yes |
| 5 | `execute_transfer` | `POST /v1/execute_transfer` | POST | Subject-authenticated | Yes | No** |
| 6 | `resolve_financial_state` | `POST /v1/resolve_financial_state` | POST | Subject-authenticated | No | Yes |
| 7 | `establish_governance_context` | `POST /v1/establish_governance_context` | POST | Governance-authorized | Conditional | Conditional |
| 8 | `record_governance_decision` | `POST /v1/record_governance_decision` | POST | Subject-authenticated | Yes | No |
| 9 | `register_asset` | `POST /v1/register_asset` | POST | Subject-authenticated | Yes | Partial*** |
| 10 | `attest_action` | `POST /v1/attest_action` | POST | Subject-authenticated | Yes | Yes |
| 11 | `resolve_system_state` | `POST /v1/resolve_system_state` | POST | Subject-authenticated | No | Yes |

\* Same inputs at same time = same result, unless policy/standing changed
\** Idempotent only for escrow release/return with same escrow_ref
\*** Idempotent for registration; non-idempotent for transfers/encumbrances

## Validation Checks

For any code targeting `$ARGUMENTS`, run ALL of these checks:

### 1. Unauthorized Capability Check
- Does the code call ONLY the 11 locked endpoints?
- Does it invent any platform calls not in the contract?
- Does it call AxiaSystem internals directly (canisters, modules, storage)?
- **VIOLATION if any call exists outside the 11 endpoints.**

### 2. Composition Pattern Check
- Does every mutating workflow follow: `resolve_subject → evaluate_legitimacy → [action] → attest_action`?
- Is `evaluate_legitimacy` called BEFORE every significant action?
- Is `attest_action` called AFTER every significant action?
- Is the `decision_ref` from `evaluate_legitimacy` passed to the action capability?
- **VIOLATION if a mutating action skips legitimacy evaluation or attestation.**

### 3. Request Schema Check
For each endpoint call, verify:
- All required fields are present
- Field types match the contract (string, number, boolean, object, array)
- Nested object structure matches
- `actor.subject_ref` and `actor.session_ref` are included where required
- Optional fields are not treated as required

### 4. Response Handling Check
For each endpoint call, verify:
- Code handles both `SUCCESS` and `FAILURE` status
- Code reads the correct response fields (not inventing fields)
- Error codes are from the contract's defined set (not generic catch-alls)
- `decision_ref`, `transfer_ref`, `governance_ref`, `attestation_ref` etc. are correctly extracted and passed to subsequent calls

### 5. Auth Level Check
- `resolve_subject`: Requires system-level auth (not individual subject)
- `authenticate_subject`: Public (no prior auth)
- `establish_governance_context`: Requires governance-authorized (higher than subject-authenticated)
- All others: Require subject-authenticated (valid session_ref)
- **VIOLATION if auth level is wrong.**

### 6. Idempotency Safety Check
- Non-idempotent calls (`authenticate_subject`, `execute_transfer`, `record_governance_decision`): Is the code safe against retries? Does it avoid double-execution?
- Idempotent calls: Is the code correctly relying on idempotency for retry safety?
- **WARNING if non-idempotent calls lack retry guards.**

### 7. Error Handling Completeness Check
For each endpoint, verify code handles the critical failure modes:
- `SYSTEM_UNAVAILABLE` — graceful degradation
- `SESSION_EXPIRED` — re-authentication flow
- `LEGITIMACY_NOT_EVALUATED` — missing legitimacy check
- `INSUFFICIENT_FUNDS`, `COMPLIANCE_REJECTED` — domain-specific handling
- **WARNING if critical failure modes are unhandled.**

### 8. Scope Parameter Check (for resolve_ capabilities)
- `resolve_financial_state`: scope must be one of (account | department | treasury | transaction)
- `resolve_system_state`: scope must be one of (platform_health | dependency_status | asset_state | sync_context | all)
- **VIOLATION if invalid scope values are used.**

## Output Format

```
## Contract Validation: [module/file name]

### Verdict: COMPLIANT | WARNING | VIOLATION

### Endpoints Called
[List of endpoints this code calls]

### Composition Pattern
[Whether the resolve → evaluate → act → attest pattern is followed]

### Violations (if any)
- [Specific contract violations with line references]

### Warnings (if any)
- [Non-critical issues: missing error handling, retry safety, etc.]

### Schema Compliance
[Request/response field accuracy per endpoint]

### Recommendations
[Specific fixes]
```
