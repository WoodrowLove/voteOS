# VoteOS Architect Skill

> Validates that proposed features, modules, and capabilities conform to VoteOS architecture.

---

## VoteOS Architecture Layers

| Layer | Responsibility | Example |
|-------|---------------|---------|
| 1. AxiaSystem | Identity, legitimacy, attestation, explanation | resolve_subject, evaluate_legitimacy |
| 2. Rust Bridge | FFI/SDK to AxiaSystem canisters | Candid bindings, service modules |
| 3. SpineClient | Unified access to AxiaSystem services | VoteOS spine client |
| 4. Domain State | Election-specific state (ballots, elections, results) | ElectionRegistry, BallotStore |
| 5. Workflows | Composed election operations | register_voter, cast_vote, certify_result |
| 6. API | HTTP endpoints for election operations | /api/election/*, /api/ballot/* |
| 7. Wrapper | Migration from legacy election systems (if needed) | Legacy adapter, reconciler |

---

## VoteOS Domain Modules (Planned)

| Module | Domain | Key Concepts |
|--------|--------|-------------|
| Voter Registration | Eligibility | Voter rolls, eligibility verification, registration status |
| Election Management | Lifecycle | Election creation, scheduling, opening/closing, certification |
| Ballot Operations | Content | Ballot design, questions/candidates, ballot distribution |
| Vote Recording | Casting | Vote submission, validation, receipt, secrecy preservation |
| Result Aggregation | Tallying | Vote counting, result computation, tie/ambiguity handling |
| Audit & Oversight | Integrity | Audit trail, observer access, recount support, compliance |

---

## Architecture Validation Rules

When a new feature or capability is proposed, validate:

1. **Layer placement**: Does it belong in the correct layer?
2. **Election domain**: Is it an election concern, not a city operations concern?
3. **Legitimacy chain**: Does it follow evaluate → act → attest → explain?
4. **No cross-module imports**: Domain modules must not import from each other
5. **Shared truth**: Does it use AxiaSystem for identity/legitimacy, not its own?
6. **Ballot integrity**: Could this feature compromise ballot secrecy or tally correctness?
7. **Neutrality**: Does VoteOS remain neutral about the outcome?

---

## Anti-Patterns to Reject

- Adding city operations logic (DMV, permits, finance) to VoteOS
- Duplicating identity management that AxiaSystem already provides
- Allowing vote recording without eligibility verification
- Allowing result publication without attestation
- Mixing decision formation with operational execution
- Implementing policy enforcement (that's CivilOS's job)
