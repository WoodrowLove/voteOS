---
name: upstream-integrity-check
description: Validate AxiaSystem upstream dependencies BEFORE starting implementation of a capability. Checks canister IDs, known inconsistencies, required setters, expected interfaces, and substrate readiness. Must be run at the start of any implementation session to prevent building on broken foundations.
user-invocable: true
argument-hint: "capability-name"
---

# Upstream Integrity Check

You validate that AxiaSystem substrate is ready and consistent BEFORE implementation begins. You prevent building on broken foundations.

## When to Run

- At the start of every implementation session (after session header, before coding)
- After deploying canisters to a new local replica
- When switching between capabilities that depend on different canisters

## Reference

- [Part 12 — Repo Mapping](../../../docs/repo-mapping/12-AXIA_CAPABILITY_REPO_MAPPING_V1.md)
- [KNOWN_LIMITATIONS.md](../../../docs/implementation/KNOWN_LIMITATIONS.md)
- [docs/agent/FAILURE_PATTERNS.md](../../../docs/agent/FAILURE_PATTERNS.md)

## Checks

### 1. Canister ID Consistency

For every canister the active capability depends on:

| Check | How |
|-------|-----|
| Canister is deployed | `dfx canister id [name]` returns an ID |
| ID matches configuration | Compare deployed ID with configured ID in user canister |
| Setter has been called | Verify `setWalletCanisterId`, `setNamoraAICanisterId`, etc. were called |

**Known issue pattern:** Hardcoded production canister IDs that don't match local replica. All configurable IDs must be set after each `dfx start --clean`.

### 2. System Caller Configuration

For every inter-canister call path:

| Check | How |
|-------|-----|
| User canister is system caller on identity | `dfx canister call identity getSystemCallers` includes user CID |
| User canister is system caller on wallet | Same pattern |
| Wallet is system caller on user (bidirectional) | For wallet→user callbacks |
| Admin2 is system caller on identity (if needed) | For session validation |

### 3. Known Inconsistencies

Check against the failure patterns library:

- Hardcoded canister IDs in substrate code (wallet, admin2, governance)
- `actor` and `system` as Motoko reserved words in record field names
- Session principal mismatch (dfx caller ≠ session identityId)
- `Time.now()` used as IDs instead of stable references

### 4. Expected Interfaces

For each canister the capability will call:

| Check | How |
|-------|-----|
| Expected function exists | `dfx canister call [canister] __get_candid_interface_tmp_hack` or check .did file |
| Function signature matches typed actor reference | Compare actor ref in user/main.mo with actual canister interface |

### 5. Substrate Readiness

| Check | How |
|-------|-----|
| Build passes | `dfx build user` succeeds |
| Local replica is running | `dfx ping` returns healthy |
| Required canisters deployed | All dependency canisters respond |

## Output Format

```
## Upstream Integrity Check: [capability]

### Environment
- Replica: [running/stopped]
- Branch: [current branch]

### Canister IDs
| Canister | Deployed ID | Configured | Match |
|----------|------------|-----------|-------|
| [name] | [id] | [configured id] | YES/NO/NOT SET |

### System Callers
| Path | Status |
|------|--------|
| user → identity | CONFIGURED / MISSING |
| user → wallet | CONFIGURED / MISSING |

### Known Issues
- [any known inconsistencies found]

### Interface Checks
| Canister | Expected Function | Available |
|----------|------------------|-----------|
| [name] | [function] | YES/NO |

### Verdict: READY / BLOCKED

### Blocking Issues:
- [what must be fixed before implementation]
```

## Enforcement

If verdict is BLOCKED → resolve upstream issues before starting implementation. Document any fixes in KNOWN_LIMITATIONS.md.
