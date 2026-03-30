---
name: phase0-gate-validator
description: Validate CivilOS Phase 0 documents against the harness gate checklist. Checks SYSTEM_INTENT passes read-back test, ACTOR_MAP covers all actors mentioned in workflows, CAPABILITY_SURFACE has no untraceable capabilities, CONSTRAINT_REGISTER cross-checks against SYSTEM_INTENT, and GLOSSARY covers all domain terms. Use before declaring Phase 0 complete.
user-invocable: true
argument-hint: "document-name or all"
---

# Phase 0 Gate Validator

You validate CivilOS Phase 0 design documents against the harness gate checklist from Section 4 of CIVIL_SYSTEM_BUILD_HARNESS_v2.1.md.

## Phase 0 Documents

All located in `docs/civilos-design/`:

- SYSTEM_INTENT.md
- ACTOR_MAP.md
- CAPABILITY_SURFACE.md
- CONSTRAINT_REGISTER.md (if produced)
- GLOSSARY.md (if produced)
- WORKFLOW_MAP.md (extension — not in base harness but required for CivilOS)

## Gate Checklist (from harness Section 4)

### SYSTEM_INTENT.md — Read-Back Test

Read SYSTEM_INTENT.md in full. Then answer:

> If someone read ONLY this document, would they understand what CivilOS is, what it does, and what it must never do?

Check for:
- [ ] System definition is clear and specific
- [ ] Purpose is stated explicitly
- [ ] Core function enumerated (not vague)
- [ ] System boundaries defined (IS / IS NOT)
- [ ] Core invariants are non-negotiable and testable
- [ ] Prohibited states are concrete, not abstract
- [ ] Actor model is identified (types, not just "users")
- [ ] Architecture model relates CivilOS to AxiaSystem and Bridge
- [ ] Execution model is stated (resolve → authenticate → evaluate → execute → attest → explain)
- [ ] Deployment model addresses multi-city
- [ ] System constraints are honest (ICP limitations, partial failure, etc.)

**If any check fails:** document what's missing. Do not pass the gate.

### ACTOR_MAP.md — Coverage Test

Read ACTOR_MAP.md. Then cross-reference against:
- Every actor mentioned in CAPABILITY_SURFACE.md
- Every actor mentioned in WORKFLOW_MAP.md
- Every role implied by SYSTEM_INTENT.md

Check for:
- [ ] Every actor type has: identity model, role set, authority scope
- [ ] Citizens, workers, enforcement, executives, governance, institutions, oversight, system actors all present
- [ ] Actor relationships documented (citizen↔department, worker↔supervisor, etc.)
- [ ] No actor mentioned in workflows is missing from the map

**If any actor is missing:** list which actor and where it was referenced.

### CAPABILITY_SURFACE.md — Traceability Test

Read CAPABILITY_SURFACE.md. For each capability, verify:
- [ ] It traces to a stated system need in SYSTEM_INTENT.md
- [ ] It maps to at least one actor in ACTOR_MAP.md
- [ ] It appears in at least one workflow in WORKFLOW_MAP.md
- [ ] It is not redundant with another capability (no duplicates with different names)

**If any capability is untraceable:** flag it with the reason.

### WORKFLOW_MAP.md — Composition Test

Read WORKFLOW_MAP.md. For each workflow, verify:
- [ ] Every step maps to a CivilOS capability in CAPABILITY_SURFACE.md
- [ ] Every CivilOS capability maps to one or more Axia platform capabilities
- [ ] The composition pattern (resolve → evaluate → execute → attest → explain) is followed
- [ ] Actors in each workflow are in ACTOR_MAP.md

### Cross-Document Consistency

- [ ] No term used in one document is undefined in another
- [ ] No invariant in SYSTEM_INTENT.md is contradicted by any capability or workflow
- [ ] No actor scope in ACTOR_MAP.md exceeds what SYSTEM_INTENT.md allows

## Output Format

```
## Phase 0 Gate Validation — CivilOS

### SYSTEM_INTENT.md
Read-back test: PASS / FAIL
Missing items: [list or NONE]

### ACTOR_MAP.md
Coverage test: PASS / FAIL
Missing actors: [list or NONE]
Uncovered relationships: [list or NONE]

### CAPABILITY_SURFACE.md
Traceability test: PASS / FAIL
Untraceable capabilities: [list or NONE]
Redundancies: [list or NONE]

### WORKFLOW_MAP.md
Composition test: PASS / FAIL
Broken mappings: [list or NONE]
Missing actors: [list or NONE]

### Cross-Document Consistency
Consistent: YES / NO
Issues: [list or NONE]

### Gate Verdict: OPEN / BLOCKED
Blocking items: [list or NONE]
```
