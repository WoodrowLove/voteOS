# VoteOS Known Limitations

> Honest accounting of what VoteOS cannot yet do.
> Updated: 2026-03-31

---

## AxiaSystem Integration

- **Live AxiaSystem calls are untested.** All 88 AxiaSystem calls across 15 workflows are correctly structured and compile, but they have never executed against a real ICP canister. This requires a running ICP replica with deployed AxiaSystem canisters.
- **17 workflow integration tests are empty stubs.** They exist to mark where live tests belong but contain no test logic.

## Voting Methods

- **Only plurality voting is implemented.** Ranked choice, approval, and score voting are declared in the `VotingMethod` enum but return errors if selected. Adding new methods is a code change, not a config change.

## Authentication & Authorization

- **Basic API key only.** No role-based access control (RBAC). Any valid API key grants full access to all endpoints. Separate operator/observer/voter roles are not enforced at the API level.

## Adoption Layer

- **JSON file adapter only.** Legacy data must be provided as JSON files. No database adapters (SQL, CSV, etc.) exist.
- **Identity reconciliation uses lookup tables.** In production, this would call AxiaSystem `resolve_subject`. Currently uses an in-memory map for domain-level proof.
- **No parallel or cutover mode.** Shadow validation compares outcomes but cannot run both systems simultaneously or switch authority.

## Runtime

- **No TLS/HTTPS.** The API server runs plain HTTP. For production, place behind a reverse proxy (nginx, caddy) with TLS termination.
- **No rate limiting.** Config declares rate limit fields but they are not enforced in the current runtime.
- **Single-process only.** No clustering, load balancing, or horizontal scaling.

## Data

- **JSON file persistence only.** No database backend. Suitable for pilot/small-scale use. Large-scale deployments would need database-backed persistence.
- **No encryption at rest.** Data files are plaintext JSON. Sensitive deployments need filesystem-level encryption.
- **No backup/rotation.** Operator must implement backup strategy externally.

## Deployment

- **Dockerfile references sibling directory for Rust Bridge.** Production deployment would need the bridge published as a crate or vendored.
- **No systemd service file.** Manual process management required outside Docker.

## Election Features

- **No write-in candidate support.**
- **No provisional ballot handling.**
- **No absentee/mail-in ballot workflow.**
- **No multi-language ballot support.**
- **No accessibility features.**

---

These limitations are known and intentional for the current development phase. They represent work to be done, not bugs to be fixed.
