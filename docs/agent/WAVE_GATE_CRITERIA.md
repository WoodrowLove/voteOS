# VoteOS Wave Gate Criteria

> Exact checklist the agent evaluates after each wave.
> The agent MUST check every item and record pass/fail.

---

## Wave 1 Gate: Foundation (Voter Registry + Election Management)

### Module 1: Voter Registry
- [ ] `src/domain/voter_registry.rs` exists with VoterRegistration + EligibilityRule types
- [ ] VoterRegistry struct with DomainStore backing
- [ ] `new()` and `with_data_dir()` constructors
- [ ] `verify_voter_eligibility` workflow exists (uses evaluate_legitimacy)
- [ ] `register_voter` workflow exists (writes to registry + attests)
- [ ] `resolve_voter_record` workflow exists (data_access)
- [ ] `generate_voter_roll` workflow exists
- [ ] At least 4 capabilities have dedicated workflows
- [ ] Tests exist: at least 1 strict happy path + 1 denial test
- [ ] `cargo build` passes
- [ ] `cargo test` reports results (may have ignored integration tests)

### Module 2: Election Management
- [ ] `src/domain/elections.rs` exists with Election + ElectionConfig types
- [ ] ElectionStatus enum: Draft, Published, Open, Closed, Tallied, Certified, Archived, Contested
- [ ] ElectionRegistry with DomainStore backing
- [ ] `create_election` workflow exists (governance_action)
- [ ] `configure_election` workflow exists
- [ ] `publish_election` / `open_election` / `close_election` workflows exist
- [ ] State transition enforcement (no invalid transitions)
- [ ] At least 4 capabilities have dedicated workflows
- [ ] Tests exist: at least 1 lifecycle test + 1 invalid transition test
- [ ] `cargo build` passes

### Wave 1 Composite
- [ ] `src/lib.rs` exports: spine, domain, workflows, error, persistence
- [ ] SpineClient created (copied from CivilOS pattern)
- [ ] DomainStore<T> created (copied from CivilOS pattern)
- [ ] WorkflowError created (copied from CivilOS pattern)
- [ ] No cross-module imports between voter_registry and elections
- [ ] All compilation warnings are acceptable (no errors)

---

## Wave 2 Gate: Core Action (Ballot Operations + Vote Recording)

### Module 3: Ballot Operations
- [ ] `src/domain/ballots.rs` exists with BallotTemplate + BallotItem types
- [ ] BallotRegistry with DomainStore backing
- [ ] `create_ballot_template` workflow
- [ ] `add_ballot_item` workflow
- [ ] `finalize_ballot` workflow (governance_action — locks content)
- [ ] `issue_ballot` workflow (requires eligible voter)
- [ ] Tests: ballot creation + finalization + issuance

### Module 4: Vote Recording
- [ ] `src/domain/votes.rs` exists with VoteRecord + VotingReceipt types
- [ ] VoteRegistry with DomainStore backing
- [ ] `cast_vote` workflow (the core action)
- [ ] `validate_vote` logic (eligible voter, valid ballot, election open)
- [ ] `prevent_double_vote` logic (one vote per voter per election)
- [ ] `generate_vote_receipt` workflow
- [ ] In secret ballot mode: VoteRecord does NOT contain voter_ref for content
- [ ] Tests: vote casting + double-vote prevention + receipt generation

### Wave 2 Composite
- [ ] Module 4 depends on Module 1 (eligibility check before voting)
- [ ] Module 4 depends on Module 2 (election must be OPEN)
- [ ] Module 4 depends on Module 3 (ballot must be issued)
- [ ] End-to-end test: create election → register voter → issue ballot → cast vote
- [ ] No cross-module imports (dependencies through workflow parameters, not imports)

---

## How the Agent Evaluates

After completing both modules in a wave:

1. Open this file
2. Check each item
3. Record results in SESSION_STATE.md:
   ```
   Wave N Gate: [PASS / CONDITIONAL / FAIL]
   Missing: [list of unchecked items]
   ```
4. If PASS or CONDITIONAL → advance to next wave
5. If FAIL → fix failures before advancing
