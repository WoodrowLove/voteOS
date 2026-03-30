# VoteOS — Claude Code Project State

## Ecosystem
AxiaSystem path:      ../AxiaSystem
Rust Bridge path:     ../AxiaSystem-Rust-Bridge
CivilOS path:         ../civilOS
Target system path:   .

## Harness documents
Primary harness:      docs/agent/VOTEOS_BUILD_HARNESS.md
System intent:        docs/voteos-design/SYSTEM_INTENT.md

## Session documents
Session state:        SESSION_STATE.md
Session index:        docs/sessions/SESSION_INDEX.md
Handoff location:     docs/sessions/

## VoteOS design documents
System intent:        docs/voteos-design/SYSTEM_INTENT.md
Actor map:            docs/voteos-design/ACTOR_MAP.md
Capability surface:   docs/voteos-design/CAPABILITY_SURFACE.md (pending)
Domain breakdown:     docs/voteos-design/DOMAIN_BREAKDOWN.md

## Truth control documents
Capability status:    docs/implementation/CAPABILITY_STATUS.md
Known limitations:    docs/implementation/KNOWN_LIMITATIONS.md
Decision log:         docs/implementation/DECISION_LOG.md

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

## Build state
Current phase:         VoteOS Phase 0 — Foundation Bootstrap
Last completed gate:   Repository scaffold + harness adaptation
Current state:         Foundation docs + skills created, ready for planning wave

## Architecture stance
- AxiaSystem = source of identity, legitimacy, assurance truth
- CivilOS = city operating system (sovereign sibling)
- VoteOS = sovereign decision / election / governance legitimacy system
- When deployed together, CivilOS and VoteOS share AxiaSystem truth
- Citizens onboarded in CivilOS carry identity into VoteOS automatically
- VoteOS remains neutral — it does not execute policy, it certifies decisions

## Open questions
- VoteOS capability set (pending planning wave)
- Module structure (pending domain breakdown)
- Ballot privacy model details (pending design)

## Known blockers
NONE — ready for planning wave
