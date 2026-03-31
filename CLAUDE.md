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
Build sequence:       docs/voteos-design/NEXT_BUILD_ORDER.md (corrected 2026-03-30)
Original sequence:    docs/voteos-design/MODULE_SEQUENCE_PLAN.md (original, superseded by above)
Completion standard:  docs/voteos-design/VOTEOS_COMPLETION_STANDARD.md
Interop hooks:        docs/voteos-design/INTEROPERABILITY_HOOKS.md

## Realignment documents (2026-03-30)
Ground truth:         docs/voteos-design/GROUND_TRUTH_STATUS.md
Trust core:           docs/voteos-design/TRUST_CORE_DEFINITION.md
Corrected build plan: docs/voteos-design/NEXT_BUILD_ORDER.md
Adoption stance:      docs/voteos-design/ADOPTION_AND_MIGRATION_STANCE.md

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
  run:     cargo run -- config/voteos.toml

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
Current phase:         PILOT READY — All waves through Wave 9 complete
Last completed gate:   Wave 9 (Pilot Preparation — deployment, runbook, quickstart)
Current state:         10 modules + adoption layer BUILD_COMPLETE, 187 tests, Dockerfile, operator docs, pilot scenario
Ground truth:          docs/voteos-design/GROUND_TRUTH_STATUS.md (2026-03-30 audit)

## Module status (update as you build)
| # | Module | Status |
|---|--------|--------|
| 1 | Voter Registry | BUILD_COMPLETE |
| 2 | Election Management | BUILD_COMPLETE |
| 3 | Ballot Operations | BUILD_COMPLETE |
| 4 | Vote Recording | BUILD_COMPLETE |
| 5 | Tally Engine | BUILD_COMPLETE |
| 6 | Result Certification | BUILD_COMPLETE |
| 7 | Governance Proposals | BUILD_COMPLETE |
| 8 | Audit & Oversight | BUILD_COMPLETE |
| 9 | Election Operations | BUILD_COMPLETE |
| 10 | Integration & Export | BUILD_COMPLETE |

## Wave execution order (corrected 2026-03-30)
Wave 1: Voter Registry + Election Management (foundation) — BUILD_COMPLETE
Wave 2: Ballot Operations + Vote Recording (core action) — BUILD_COMPLETE
Wave 3: Tally Engine + Result Certification (results) — BUILD_COMPLETE
Wave 3.5: End-to-end domain proof (trust gate) — PASSED
Wave 4: Audit & Oversight (trust verification — promoted to solo wave) — BUILD_COMPLETE
Wave 5: Governance Proposals + Integration & Export (extensions) — BUILD_COMPLETE
Wave 6: Election Operations + API + Deployment (runtime boundary) — BUILD_COMPLETE

## Architecture stance
- AxiaSystem = source of identity, legitimacy, assurance truth
- CivilOS = city operating system (sovereign sibling, READ-ONLY reference)
- VoteOS = sovereign decision / election / governance legitimacy system
- Citizens onboarded in CivilOS carry identity into VoteOS automatically
- VoteOS certifies decisions; CivilOS/others execute them
- VoteOS is NEUTRAL — it never chooses outcomes

## Open questions

- AxiaSystem live integration: workflows compile but are untested against real canister

## Known blockers

- No AxiaSystem integration tests — requires local ICP replica
- 17 workflow test stubs are empty bodies, not blocked tests with real logic
- Only plurality voting method implemented

## Iteration protocol
Follow docs/agent/ITERATION_HARNESS.md for the build loop.
After each module: build → test → evaluate → update state → commit → push → next.
