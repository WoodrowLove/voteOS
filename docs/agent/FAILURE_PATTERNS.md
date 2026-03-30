# VoteOS Known Failure Patterns

> Common failures the agent will encounter and how to handle them.
> Adapted from CivilOS proven experience + election-specific additions.

---

## Infrastructure Failures

### 1. "Cannot read PEM file"
**Cause:** identity.pem_path in config points to wrong location
**Fix:** Verify `../AxiaSystem-Rust-Bridge/identity.pem` exists. Use relative path.

### 2. "Agent call failed: Connection refused"
**Cause:** ICP replica not running
**Fix:** `cd ../AxiaSystem && dfx start --background`

### 3. "Canister ID not found" / "invalid canister ID"
**Cause:** Canister IDs change on every `dfx start --clean`
**Fix:** Run `dfx canister id user` in ../AxiaSystem to get current IDs. Update config.

### 4. "blob_of_principal: invalid principal"
**Cause:** Passing a non-principal string where an ICP principal is expected
**Fix:** Use real subject_refs from resolve_subject, not arbitrary strings.

### 5. Cargo build fails with bridge type errors
**Cause:** axia_system_rust_bridge crate changed or path wrong
**Fix:** `cd ../AxiaSystem-Rust-Bridge && cargo build` first. Verify path in Cargo.toml.

---

## AxiaSystem Failures

### 6. "User not found" on evaluate_legitimacy
**Cause:** Subject hasn't been onboarded via resolve_subject yet
**Fix:** Onboard the subject first. VoteOS uses the same AxiaSystem path as CivilOS.

### 7. "insufficient assurance: has level 0 but requires 1"
**Cause:** governance_action requires L1 but subject is at L0
**Fix:** Elevate assurance: call identity canister's setAssuranceLevel (L0 → L1).

### 8. "already_bootstrapped" during admin2.bootstrap
**Cause:** Bootstrap already ran (from CivilOS deployment)
**Fix:** This is NOT an error. Ignore it. The bootstrap state is shared.

### 9. "Biographical type requires material.username"
**Cause:** resolve_subject needs username field, not just email
**Fix:** Always provide username in IdentificationMaterial.

---

## VoteOS-Specific Failures

### 10. Double-vote detected but no prevention
**Cause:** Vote recording logic doesn't check existing votes
**Fix:** Before recording, query VoteRegistry for existing vote by (voter_ref, election_ref).

### 11. Election state transition rejected
**Cause:** Attempting invalid transition (e.g., OPEN → DRAFT)
**Fix:** Check state machine rules. Only forward transitions allowed (with rollback exceptions).

### 12. Ballot secrecy violated in test
**Cause:** VoteRecord links voter_ref to vote content in secret ballot mode
**Fix:** In secret ballot mode, separate VoteRecord (who voted) from VoteContent (what they chose).

---

## Test Failures

### 13. Test uses match Ok/Err instead of .expect()
**Cause:** Mixed-path test — violates workflow-proof discipline
**Fix:** Split into strict happy-path test (.expect()) and separate denial test.

### 14. Integration test fails with "requires local ICP replica"
**Cause:** Test marked #[ignore] but ran anyway, or replica not running
**Fix:** Integration tests should be #[ignore = "requires local ICP replica"].
Run with `cargo test -- --ignored` when replica is available.

### 15. Test claims PROVEN but assertion is weak
**Cause:** Test doesn't verify all workflow artifacts
**Fix:** Verify decision_ref, attestation_ref, all output fields. Not just "it didn't crash."
