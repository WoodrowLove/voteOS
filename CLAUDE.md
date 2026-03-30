# VoteOS — Claude Code Project State

## Ecosystem
AxiaSystem path:      ../AxiaSystem
Rust Bridge path:     ../AxiaSystem-Rust-Bridge
CivilOS path:         ../civilOS (sibling system — READ-ONLY reference for patterns)
Target system path:   .

## CRITICAL: Pattern Sources
Before building ANY infrastructure, read these CivilOS files as reference patterns:
- ../civilOS/src/spine/client.rs → SpineClient (COPY this pattern exactly)
- ../civilOS/src/domain/store.rs → DomainStore<T> (COPY this pattern exactly)
- ../civilOS/src/error.rs → WorkflowError (COPY this pattern exactly)
- ../civilOS/src/persistence/fs.rs → Persistence layer (COPY this pattern exactly)
- ../civilOS/src/workflows/citizen_onboarding.rs → Simplest workflow pattern
- ../civilOS/src/lib.rs → Module structure pattern

## CRITICAL: Reuse Rules
1. DO NOT create a new identity system — use AxiaSystem resolve_subject
2. DO NOT create a new legitimacy system — use AxiaSystem evaluate_legitimacy
3. DO NOT create a new attestation system — use AxiaSystem attest_action
4. DO NOT create new AxiaSystem capabilities — compose from the existing 11
5. The Rust Bridge at ../AxiaSystem-Rust-Bridge is SHARED — do not fork it
6. requesting_system in all legitimacy calls must be "voteos" (not "civilos")

## Harness documents
Primary harness:      docs/agent/VOTEOS_BUILD_HARNESS.md
Iteration protocol:   docs/agent/ITERATION_HARNESS.md
Pattern reference:    docs/agent/PATTERN_REFERENCE.md
System intent:        docs/voteos-design/SYSTEM_INTENT.md

## Planning documents
Mission/boundaries:   docs/voteos-design/MISSION_AND_BOUNDARIES.md
Module registry:      docs/voteos-design/MODULE_REGISTRY.md
Capability map:       docs/voteos-design/CAPABILITY_MAP.md
Build sequence:       docs/voteos-design/MODULE_SEQUENCE_PLAN.md
Completion standard:  docs/voteos-design/VOTEOS_COMPLETION_STANDARD.md
Interop hooks:        docs/voteos-design/INTEROPERABILITY_HOOKS.md

## Session documents
Session state:        SESSION_STATE.md
Session index:        docs/sessions/SESSION_INDEX.md

## Truth control documents
Capability status:    docs/implementation/CAPABILITY_STATUS.md (create when building)
Known limitations:    docs/implementation/KNOWN_LIMITATIONS.md (create when building)
Decision log:         docs/implementation/DECISION_LOG.md (create when building)

## Build commands

AxiaSystem:
  build:   cd ../AxiaSystem && dfx build
  test:    cd ../AxiaSystem && bash scripts/golden_path_resolve_subject.sh
  deploy:  cd ../AxiaSystem && dfx deploy

Rust Bridge:
  build:   cd ../AxiaSystem-Rust-Bridge && cargo build
  test:    cd ../AxiaSystem-Rust-Bridge && cargo test

VoteOS:
  build:   cargo build
  test:    cargo test
  run:     cargo run --bin voteos_server -- config/election.example.toml

## Canister IDs (local replica — from CivilOS deployment, SHARED)
Last known IDs (verify with dfx canister id <name> in ../AxiaSystem):
user:       wwifi-ux777-77774-qaana-cai
wallet:     wrjd4-zp777-77774-qaanq-cai
identity:   v27v7-7x777-77774-qaaha-cai
admin2:     u6s2n-gx777-77774-qaaba-cai
asset:      vizcg-th777-77774-qaaea-cai
governance: vt46d-j7777-77774-qaagq-cai

Note: IDs change on every dfx start --clean. Verify before use.

## Build state
Current phase:         VoteOS Phase 1 — Foundation Build (Wave 1)
Last completed gate:   System Planning Wave complete
Current state:         10 modules designed, 100 capabilities mapped, ready for Wave 1

## Module status (update as you build)
| # | Module | Status |
|---|--------|--------|
| 1 | Voter Registry | DESIGN_COMPLETE |
| 2 | Election Management | DESIGN_COMPLETE |
| 3 | Ballot Operations | DESIGN_COMPLETE |
| 4 | Vote Recording | DESIGN_COMPLETE |
| 5 | Tally Engine | DESIGN_COMPLETE |
| 6 | Result Certification | DESIGN_COMPLETE |
| 7 | Governance Proposals | DESIGN_COMPLETE |
| 8 | Audit & Oversight | DESIGN_COMPLETE |
| 9 | Election Operations | DESIGN_COMPLETE |
| 10 | Integration & Export | DESIGN_COMPLETE |

## Wave execution order
Wave 1: Voter Registry + Election Management (foundation)
Wave 2: Ballot Operations + Vote Recording (core action)
Wave 3: Tally Engine + Result Certification (results)
Wave 4: Governance Proposals + Audit (trust layer)
Wave 5: Election Operations + Integration (deployment)

## Architecture stance
- AxiaSystem = source of identity, legitimacy, assurance truth
- CivilOS = city operating system (sovereign sibling, READ-ONLY reference)
- VoteOS = sovereign decision / election / governance legitimacy system
- Citizens onboarded in CivilOS carry identity into VoteOS automatically
- VoteOS certifies decisions; CivilOS/others execute them
- VoteOS is NEUTRAL — it never chooses outcomes

## Open questions
NONE — ready for Wave 1 execution

## Known blockers
NONE — ready for Wave 1 execution

## Iteration protocol
Follow docs/agent/ITERATION_HARNESS.md for the build loop.
After each module: build → test → evaluate → update state → commit → push → next.
