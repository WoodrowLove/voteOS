# VoteOS Build Harness

> Adapted from CivilOS Civil System Build Harness v2.1.
> Strengthened for election integrity requirements.

---

## Build Philosophy

VoteOS inherits the proven CivilOS build discipline:
- Harness-driven execution (no ad-hoc implementation)
- Workflow-proof discipline (strict test standards)
- Module-gate discipline (6-layer completion validation)
- Capability-first planning (design before code)
- Test truthfulness (no overclaiming)
- Documentation-first clarity (docs precede implementation)

VoteOS STRENGTHENS this discipline with:
- Election integrity gates (no election without eligibility proof)
- Ballot integrity validation (no tally without evidence)
- Privacy constraint enforcement (no voter-vote linkage in secret ballots)
- Neutrality enforcement (VoteOS never chooses outcomes)

---

## Phases

| Phase | Name | Exit Criteria |
|-------|------|---------------|
| 0 | Foundation Bootstrap | Repo scaffold, harness, skills, design docs |
| 1 | Platform Readiness | AxiaSystem bridge proven, SpineClient created |
| 2 | Domain Design | All modules designed, capabilities mapped |
| 3 | Core Build | Voter Registration + Election Management + Ballot |
| 4 | Vote Recording | Secure vote submission + double-vote prevention |
| 5 | Result Aggregation | Deterministic tallying + threshold evaluation |
| 6 | Audit Layer | Evidence reconstruction + observer access |
| 7 | Integration Testing | End-to-end election lifecycle proven |
| 8 | Pilot Hardening | Security, privacy, API authentication |
| 9 | Pilot Deployment | Controlled election execution |
| 10 | Distribution | Multi-jurisdiction support |

---

## Proof Language

| Term | Meaning |
|------|---------|
| Not Started | No implementation exists |
| Implemented | Code exists but not tested |
| Wired | Connected to AxiaSystem bridge |
| Live | Executes against real canister |
| Proven | Strict test passes, evidence recorded |
| Election-Proven | Full lifecycle tested end-to-end |

---

## Session Protocol

Every work session must:
1. Read current state from SESSION_STATE.md
2. Declare scope and authority level
3. Execute within declared scope
4. Update truth documents honestly
5. Produce handoff summary

---

## VoteOS-Specific Constraints

### Before Any Election Feature Ships:
- [ ] Eligibility verification tested against AxiaSystem
- [ ] Double-vote prevention proven
- [ ] Ballot secrecy enforced (if secret ballot mode)
- [ ] Tally correctness provable from evidence
- [ ] Audit trail complete and reconstructible
- [ ] No hidden authority over election state

### Before Any Result Publication:
- [ ] All votes accounted for
- [ ] Tally independently reproducible
- [ ] Attestation chain complete
- [ ] Ambiguity handled explicitly (not suppressed)
