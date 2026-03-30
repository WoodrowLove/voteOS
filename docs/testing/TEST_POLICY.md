# VoteOS Test Policy

> Testing standards for election-critical software.
> Adapted from CivilOS workflow-proof discipline, strengthened for voting integrity.

---

## Test Philosophy

VoteOS tests must be **stricter than general software tests** because:
- Incorrect election results have democratic consequences
- Voter privacy violations are irreversible
- Audit failures undermine public trust
- Ambiguity suppression is a form of corruption

---

## Proof Classifications

| Classification | Meaning | Required For |
|---------------|---------|-------------|
| STRICT_HAPPY_PATH_PROVEN | Ok result with real AxiaSystem setup | Any capability claim |
| DENIAL_PROVEN | Rejection correctly enforced | Security/legitimacy claims |
| LIFECYCLE_PROVEN | Full state machine traversal | Election/ballot modules |
| TALLY_PROVEN | Deterministic result from known votes | Result aggregation |
| PRIVACY_PROVEN | Vote not linkable to voter | Secret ballot mode |
| AUDIT_PROVEN | Result reconstructible from evidence | Audit module |
| INTEGRITY_BLOCKED | Cannot be proven — blocker exists | Honest status |

---

## Test Requirements by Module

### Voter Registration
- [ ] Eligible voter registers successfully
- [ ] Ineligible voter rejected with reason
- [ ] Already-registered voter not duplicated
- [ ] Registration persists across restart

### Election Management
- [ ] Full lifecycle: DRAFT → PUBLISHED → OPEN → CLOSED → TALLIED → CERTIFIED
- [ ] Forbidden transitions rejected
- [ ] Only authorized officials can transition state
- [ ] All transitions attested and explained

### Vote Recording
- [ ] Eligible voter's vote accepted
- [ ] Ineligible voter's vote rejected
- [ ] Double-vote prevented
- [ ] Vote receipt generated
- [ ] In secret ballot mode: vote content not linked to voter identity

### Result Aggregation
- [ ] Tally matches manual count of known votes
- [ ] Tie correctly detected
- [ ] Participation threshold enforced
- [ ] Ambiguous result declared (not hidden)
- [ ] Same votes always produce same result (determinism)

### Audit
- [ ] Result independently reproducible from evidence
- [ ] Observer can verify without accessing individual votes
- [ ] Recount produces same result
- [ ] Contest pathway works

---

## Forbidden Test Practices

1. **No mock legitimacy** — tests must use real AxiaSystem evaluation
2. **No assumed eligibility** — voter must be actually verified
3. **No hardcoded results** — tallies must be computed, not asserted
4. **No suppressed failures** — all test failures must be visible
5. **No privacy violations in tests** — test data must not link voter to vote in secret mode
