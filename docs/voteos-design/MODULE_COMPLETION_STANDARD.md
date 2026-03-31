# VoteOS Module Completion Standard

> Defines what "done" means for a VoteOS module.
> Every module must meet this standard before it can be declared complete.
> Created: 2026-03-29, adapted from CivilOS pattern
> Realigned: 2026-03-30

---

## Required Layers

A module is complete when all 6 layers are satisfied.

### Layer 1: Design

| Artifact | Required |
|----------|----------|
| Module intent | One paragraph describing what the module does and why it exists |
| Module actors | Which actors from ACTOR_MAP participate, with their roles |
| Module capabilities | Complete list of capabilities the module owns |
| Module workflows | Which workflows are dedicated vs engine-driven |
| Ownership boundary | What VoteOS owns vs what AxiaSystem provides |

### Layer 2: Resolution

| Artifact | Required |
|----------|----------|
| Capability classification | Every capability classified: ORCHESTRATION_READY, DOMAIN_EXTENSION_NEEDED, or SYSTEM_PRIMITIVE_NEEDED |
| AxiaSystem gaps identified | Any missing Axia primitives documented |
| Bridge gaps identified | Any missing bridge exposures documented |
| Domain state needs identified | What VoteOS-local state management is required |

### Layer 3: Build

| Artifact | Required |
|----------|----------|
| Dedicated workflows implemented | All capabilities that need custom composition have workflow modules |
| Engine registrations verified | All capabilities that use the generic engine are registered with correct action types |
| Domain registries built | All required in-memory stores exist and are functional |
| Cross-module integration | Any dependencies on other modules are wired |

### Layer 4: Test

| Artifact | Required |
|----------|----------|
| Strict happy-path proof | Every dedicated workflow has a strict Ok test per workflow-proof discipline |
| Failure-path coverage | Every dedicated workflow has at least one separate failure test |
| Domain state verification | Category B workflows verify domain store persistence |
| Engine capability validation | Engine-registered capabilities have valid action types (validated by registry test) |

### Layer 5: Operations

| Artifact | Required |
|----------|----------|
| Module limitations documented | Known constraints specific to this module |
| Deployment requirements | Any module-specific deployment steps (role grants, canister config, etc.) |
| Error codes documented | Module-specific error codes with meaning/cause/fix |
| Recovery procedures | How to handle module-specific failures |

### Layer 6: Review

| Artifact | Required |
|----------|----------|
| Module review summary | Honest assessment of what is proven vs what is not |
| Open risks | Explicitly stated risks that remain |
| Cross-module impact | How this module affects other modules |

---

## Module Completion Verdicts

| Status | Meaning | Criteria |
|--------|---------|----------|
| NOT_STARTED | No work has begun on this module | No design artifacts, no code, no tests |
| IN_DESIGN | Design layer in progress | Module intent defined, capabilities mapped, but no implementation |
| IN_BUILD | Implementation in progress | Some workflows built, domain state partially implemented |
| IN_TEST | Testing in progress | Implementation complete, tests being written and run |
| CONDITIONALLY_COMPLETE | All layers satisfied with documented limitations | Tests pass, limitations documented, but some gaps remain (e.g., environment blockers, missing failure-path depth) |
| COMPLETE | All layers satisfied with no open limitations | Every workflow proven strict, every failure path tested, every operation documented |

### Progression Rules

- A module may not move from IN_DESIGN to IN_BUILD without design layer complete
- A module may not move from IN_BUILD to IN_TEST without build layer complete
- A module may not be declared CONDITIONALLY_COMPLETE without all 6 layers addressed
- COMPLETE requires zero open limitations — if any exist, the status is CONDITIONALLY_COMPLETE

---

## Applying the Standard

### For each module wave:

1. **Start:** Module is at NOT_STARTED or IN_DESIGN
2. **Design pass:** Complete Layer 1 artifacts → move to IN_DESIGN
3. **Resolution pass:** Complete Layer 2 → ready for IN_BUILD
4. **Build pass:** Complete Layer 3 → move to IN_BUILD
5. **Test pass:** Complete Layer 4 → move to IN_TEST
6. **Ops pass:** Complete Layer 5 → ready for review
7. **Review pass:** Complete Layer 6 → CONDITIONALLY_COMPLETE or COMPLETE

### Workflow-proof discipline applies

All testing must follow the codified workflow-proof discipline:
- Strict Ok for happy paths
- Separate failure tests
- Real legitimacy setup
- Honest blocker classification
- No mixed-path tests

---

## Current Module Status

| Module | Status | Evidence |
|--------|--------|---------|
| 1. Voter Registry | DOMAIN_PROVEN | 11 domain tests passing, 5 DomainStore instances, 4 workflow files written |
| 2. Election Management | DOMAIN_PROVEN | 13 domain tests passing, 3 DomainStore instances, state machine proven |
| 3. Ballot Operations | DOMAIN_PROVEN | 12 domain tests passing, SHA-256 integrity hash proven |
| 4. Vote Recording | DOMAIN_PROVEN | 12 domain tests passing, secrecy architecture proven, double-vote prevention proven |
| 5. Tally Engine | DESIGN_COMPLETE | Capability map defined, no implementation |
| 6. Result Certification | DESIGN_COMPLETE | Capability map defined, no implementation |
| 7. Governance Proposals | DESIGN_COMPLETE | Capability map defined, no implementation |
| 8. Audit & Oversight | DESIGN_COMPLETE | Capability map defined, no implementation |
| 9. Election Operations | DESIGN_COMPLETE | Capability map defined, no implementation |
| 10. Integration & Export | DESIGN_COMPLETE | Capability map defined, no implementation |

### Note on DOMAIN_PROVEN vs CONDITIONALLY_COMPLETE
Modules 1-4 were previously labeled CONDITIONALLY_COMPLETE. After ground-truth audit (2026-03-30), they are re-labeled DOMAIN_PROVEN because:
- All tests are domain-level only (in-memory DomainStore, no AxiaSystem)
- No failure-path tests exist
- No integration tests are proven
- CONDITIONALLY_COMPLETE requires all 6 layers addressed; Layer 4 (failure paths) and Layer 5 (operations) are incomplete
