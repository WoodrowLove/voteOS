# VoteOS Module Completion Standard

> Adapted from CivilOS 6-layer standard, strengthened for election integrity.
> A module is COMPLETE only when all applicable layers are satisfied.

---

## Layer 1: Design

- [ ] Module responsibility documented
- [ ] Capability list defined
- [ ] Domain state types defined
- [ ] Actor-action matrix documented
- [ ] Privacy implications assessed
- [ ] Dependencies declared

## Layer 2: Resolution

- [ ] Each capability mapped to AxiaSystem primitive composition
- [ ] Legitimacy chain defined (evaluate → act → attest → explain)
- [ ] Action types classified (operation, data_access, governance_action)
- [ ] Privacy mode behavior specified per capability

## Layer 3: Build

- [ ] Domain state types implemented with persistence
- [ ] Workflows implemented composing AxiaSystem capabilities
- [ ] API endpoints created
- [ ] Module boundary enforced (no cross-module imports)

## Layer 4: Test

- [ ] Strict happy path proven for each capability
- [ ] Denial proven for unauthorized actors
- [ ] Election-specific integrity tests:
  - [ ] Eligibility enforcement proven (Module 1+)
  - [ ] Double-vote prevention proven (Module 4)
  - [ ] Ballot secrecy proven (Module 4, secret ballot mode)
  - [ ] Tally determinism proven (Module 5)
  - [ ] Evidence reconstruction proven (Module 8)
- [ ] Lifecycle tests proven (full state machine traversal)

## Layer 5: Operations

- [ ] Health/readiness checks implemented
- [ ] Operational documentation complete
- [ ] Error messages actionable for operators
- [ ] Backup/recovery documented

## Layer 6: Review

- [ ] Module coherence review passed
- [ ] No cross-module contamination
- [ ] Privacy model validated
- [ ] Audit trail complete for all state changes
- [ ] Interoperability hooks documented

---

## Election-Specific Proof Requirements

Beyond the 6 layers, election modules must satisfy:

| Proof | Required For | What It Proves |
|-------|-------------|----------------|
| ELIGIBILITY_PROVEN | Modules 1, 4 | Ineligible voters cannot vote |
| BALLOT_INTEGRITY_PROVEN | Module 3, 4 | Ballot content unchanged from design to recording |
| DOUBLE_VOTE_PREVENTION_PROVEN | Module 4 | No voter can vote twice |
| SECRECY_PROVEN | Module 4 | Vote content not linkable to voter (secret ballot) |
| TALLY_DETERMINISM_PROVEN | Module 5 | Same votes → same result, always |
| CERTIFICATION_CHAIN_PROVEN | Module 6 | Result traceable from votes through attestation |
| AUDIT_RECONSTRUCTION_PROVEN | Module 8 | Result independently reproducible from evidence |

---

## Module Status Classifications

| Status | Meaning |
|--------|---------|
| NOT_STARTED | No work done |
| DESIGN_COMPLETE | Layer 1 done |
| RESOLUTION_COMPLETE | Layers 1-2 done |
| BUILD_COMPLETE | Layers 1-3 done |
| TEST_COMPLETE | Layers 1-4 done |
| OPERATIONS_COMPLETE | Layers 1-5 done |
| **MODULE_COMPLETE** | **All 6 layers + election proofs** |
| CONDITIONALLY_COMPLETE | Layers done with documented caveats |
