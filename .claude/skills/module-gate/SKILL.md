---
name: module-gate
description: Validate module completion against the 6-layer MODULE_COMPLETION_STANDARD. Use before declaring any module complete or conditionally complete. Checks design, resolution, build, test, operations, and review layers.
user-invocable: true
argument-hint: "check <module_name> | status"
---

# Module Gate Validator

You validate whether a CivilOS module meets the MODULE_COMPLETION_STANDARD.

## Reference Documents

- [MODULE_COMPLETION_STANDARD.md](../../../docs/civilos-design/MODULE_COMPLETION_STANDARD.md)
- [MODULE_REGISTRY.md](../../../docs/civilos-design/MODULE_REGISTRY.md)
- [TEST_REGISTRY.md](../../../docs/testing/TEST_REGISTRY.md)

## Modes

### `/module-gate check <module_name>`

For the named module, verify each of the 6 layers:

**Layer 1 — Design:**
- Module intent exists? (docs/modules/<MODULE>.md)
- Module actors listed?
- Module capabilities mapped?
- Module workflows identified?
- Ownership boundary defined?

**Layer 2 — Resolution:**
- All capabilities classified (ORCHESTRATION_READY / DOMAIN_EXTENSION / SYSTEM_PRIMITIVE)?
- AxiaSystem gaps documented?
- Bridge gaps documented?
- Domain state needs identified?

**Layer 3 — Build:**
- Dedicated workflows exist for capabilities needing custom composition?
- Engine registrations correct for remaining capabilities?
- Domain registries built?
- Cross-module integration wired?

**Layer 4 — Test:**
- Every dedicated workflow has strict happy-path test?
- Every dedicated workflow has separate failure test?
- Domain state verified in workflow context?
- Engine capabilities have valid action types?

**Layer 5 — Operations:**
- Module limitations documented?
- Deployment requirements documented?
- Error codes documented?
- Recovery procedures documented?

**Layer 6 — Review:**
- Module review summary exists?
- Open risks stated?
- Cross-module impact documented?

Output:
```
Module: <name>
Layer 1 (Design):      PASS / PARTIAL / FAIL — [details]
Layer 2 (Resolution):  PASS / PARTIAL / FAIL — [details]
Layer 3 (Build):       PASS / PARTIAL / FAIL — [details]
Layer 4 (Test):        PASS / PARTIAL / FAIL — [details]
Layer 5 (Operations):  PASS / PARTIAL / FAIL — [details]
Layer 6 (Review):      PASS / PARTIAL / FAIL — [details]

Verdict: NOT_STARTED / IN_DESIGN / IN_BUILD / IN_TEST / CONDITIONALLY_COMPLETE / COMPLETE
Blockers: [list or none]
```

### `/module-gate status`

Show current status of all 8 modules from MODULE_COMPLETION_STANDARD.md.

## Validation Rules

- A module cannot be CONDITIONALLY_COMPLETE unless all 6 layers are at least PARTIAL
- A module cannot be COMPLETE unless all 6 layers are PASS
- The test layer MUST use workflow-proof discipline classifications
- No layer may be marked PASS based on assumptions — file-level evidence required
