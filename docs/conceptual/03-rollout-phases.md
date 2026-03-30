# CivilOS Conceptual Breakdown — Part 3

## Rollout Phases

> This document defines how CivilOS should be built and rolled out in phases — realistic, verifiable, and aligned with the platform relationship defined in Parts 1 and 2. It establishes what must exist first, what comes next, and how to avoid false "completion" signals.

---

## 1. Core Principle of Rollout

CivilOS must be built in phases that are **verifiable at runtime**, not just in code structure.

Each phase must answer:

- What exists **in reality** (not just in code)
- What is still **local-only or simulated**
- What is **externally connected and proven**
- What is **safe to build on top of**

> **Do not treat passing tests or compiled code as completion. Completion requires runtime truth.**

---

## 2. Phase Overview

| Phase | Name | Owner | Nature |
|-------|------|-------|--------|
| 0 | Platform Readiness | AxiaSystem | Dependency — not CivilOS-built |
| 1 | CivilOS Foundation | CivilOS | City structure |
| 2 | Observability Layer | CivilOS | Read-only system truth |
| 3 | Platform Connectivity | CivilOS + Platform | First real external edge |
| 4 | Additional External Composition | CivilOS + Platform | Bounded external edges |
| 5 | Controlled Composition | CivilOS | Cross-system view |
| 6 | First Real Capability Consumption | CivilOS + Platform | High-level calls |
| 7 | Institutional Workflows | CivilOS | Department operations |
| 8 | Policy and Enforcement Integration | CivilOS + Platform | Real enforcement |
| 9 | City Deployment Model | CivilOS | Multi-city readiness |
| 10 | Distribution and Packaging | CivilOS | Installable product |

```
Phase 0   Platform Readiness          ── prerequisite, not CivilOS-built
Phase 1   CivilOS Foundation          ── city exists structurally
Phase 2   Observability               ── truth before action
Phase 3   Platform Connectivity       ── first real external edge
Phase 4   Additional Composition      ── bounded external edges
Phase 5   Controlled Composition      ── cross-system view
Phase 6   Capability Consumption      ── real platform usage
Phase 7   Institutional Workflows     ── departments operate
Phase 8   Policy & Enforcement        ── governance becomes real
Phase 9   City Deployment Model       ── multi-city readiness
Phase 10  Distribution & Packaging    ── installable product
```

---

## Phase 0 — Platform Readiness

**Owner:** AxiaSystem (capability provider)

Before CivilOS can function, the underlying platform must expose usable capabilities.

**This phase ensures:**

- Identity / triad works as a callable capability
- Wallet/account context is accessible
- Governance primitives are callable
- Payment/treasury primitives are callable
- Compliance/legitimacy primitives exist
- Bridge access (Rust/Swift/HTTP) is functional

**Constraints:**

- CivilOS does **not** build these
- CivilOS only **depends** on them
- These must be exposed in a consumable way (API/capability level, not raw canister internals)

**Exit criteria:**

- Platform is callable
- Platform capabilities are identifiable
- Platform is usable by an external system

---

## Phase 1 — CivilOS Foundation

**Owner:** CivilOS

The first real CivilOS phase. Defines the city as an operational system.

**Includes:**

- City registration
- Department definitions
- Agency structures
- Role definitions
- User-to-role assignment (city context)
- Access control (city-scoped)
- Partition rules
- Basic admin controls

**Constraints:**

- No complex workflows yet
- No orchestration
- No external system dependency required for correctness
- Platform usage is minimal and controlled

**Exit criteria:**

- A city can exist inside CivilOS
- Users can belong to departments
- Roles and permissions are defined
- Access is partitioned correctly

---

## Phase 2 — Observability Layer

**Owner:** CivilOS

> **Truth before action.** Before adding complex behavior, the system must observe itself truthfully.

**Introduces:**

- Platform probe (Axia connectivity)
- Namora Host probe (if applicable)
- Aegis probe (if applicable)
- System snapshot (raw signals)
- System interpretation (derived meaning)
- System posture (overall condition)

**Constraints:**

- **Read-only only** — no writes
- No orchestration
- No decision-making automation
- No hidden logic

**Exit criteria:**

- Operators can see system state
- System dependencies are visible
- External connectivity is provable
- No false claims of integration

---

## Phase 3 — Platform Connectivity

**Owner:** CivilOS + Platform

The first real external platform interaction, introduced in a controlled way.

**Includes:**

- Selectable backend (local vs real)
- First real platform call path (e.g., probe)
- Verified external connectivity (CONNECTED state)
- Truthful verification layer

**Constraints:**

- Only one bounded call path at first
- Read-only where possible
- No broad integration claims
- No orchestration

**Exit criteria:**

- CivilOS can **prove** it can reach the platform
- Real vs local behavior is distinguishable
- Integration is no longer hypothetical

---

## Phase 4 — Additional External Composition

**Owner:** CivilOS + Platform

Adds additional bounded external edges.

**Examples:**

- Namora Host health probe
- Aegis health probe

**Constraints — each edge must be:**

- Bounded
- Read-only
- Independently verifiable
- No enforcement logic yet
- No orchestration

**Exit criteria:**

- Multiple external dependencies are visible
- System can prove reachability to each
- System remains LOCAL_ONLY in behavior

---

## Phase 5 — Controlled Composition

**Owner:** CivilOS

Introduces composition of multiple signals into a unified view.

**Examples:**

- System snapshot endpoint (aggregating Axia, Namora, Aegis)
- Interpretation layer (deriving system state)
- Unified observability surface

**Constraints:**

- Still **read-only**
- No execution or orchestration
- No mutation
- No policy enforcement

**Exit criteria:**

- Operators can see a unified system view
- System state is understandable
- Multiple dependencies are contextualized together

---

## Phase 6 — First Real Capability Consumption

**Owner:** CivilOS + Platform

CivilOS begins to **use platform capabilities for real work**.

**Examples:**

- Create triad (user + identity + wallet)
- Attach user to city role
- Register city-level entity with platform backing
- Initiate governance action via platform

**Constraints — each call must be:**

- Explicit
- Auditable
- Verifiable
- Uses high-level capability calls, not raw platform internals
- No hidden multi-step orchestration yet
- No silent side effects

**Exit criteria:**

- CivilOS performs real actions using the platform
- Actions are traceable and auditable
- System begins to move beyond observation

---

## Phase 7 — Institutional Workflows

**Owner:** CivilOS

Real departmental workflows become operational.

**Examples:**

- Permit approval flow
- Licensing workflow
- Voting workflows (if applicable)
- Financial approvals
- Multi-step department processes

**Constraints — workflows must remain:**

- Role-aware
- Permission-aware
- Audit-aware
- Platform calls must remain explicit
- No hidden automation chains

**Exit criteria:**

- Departments can operate real workflows
- CivilOS becomes **operationally useful**

---

## Phase 8 — Policy and Enforcement Integration

**Owner:** CivilOS + Platform

Real enforcement and policy evaluation enter the system.

**Examples:**

- Aegis enforcement in decision paths
- Compliance checks before actions
- Governance policy enforcement
- Denial and escalation handling

**Constraints:**

- No silent enforcement
- All enforcement must be **visible, explainable, and auditable**
- Fallback behavior must be defined

**Exit criteria:**

- System decisions are policy-aware
- Enforcement becomes real and reliable

---

## Phase 9 — City Deployment Model

**Owner:** CivilOS

Ensures the system is deployable to **multiple cities**.

**Includes:**

- Per-city configuration
- Partition isolation between cities
- Separate or shared canister strategy (decided explicitly)
- Deployment templates
- Onboarding flows for new cities
- Monitoring and maintenance setup
- Cycle management strategy

**Constraints:**

- No shared data leakage across cities
- Clear ownership model (company vs city)
- Reproducible deployment

**Exit criteria:**

- A new city can be onboarded with a defined process
- System is reproducible and scalable

---

## Phase 10 — Distribution and Packaging

**Owner:** CivilOS

Fulfills the **core business requirement** — an installable product.

**Includes:**

- Installable system package
- Bootstrap scripts
- Environment configuration templates
- Manifest-driven setup
- Verification entrypoints
- Documentation for operators and developers
- Agent-friendly interface definitions

**Constraints:**

- System must be installable from a link or package
- System must be inspectable
- System must not rely on hidden setup steps
- System must include verification tooling

**Exit criteria:**

- A city can install CivilOS
- System boots and verifies itself
- Dependencies are visible
- System is **production-ready**

---

## 3. Critical Rollout Rule

> **Never skip phases.**

| Anti-pattern | Why it fails |
|---|---|
| Building workflows before observability | No way to verify system state supports the workflow |
| Claiming integration before real connectivity | False completion signals, brittle at runtime |
| Adding enforcement before visibility | Enforcement becomes opaque and unauditable |
| Packaging before system truth is established | Distributes unverified assumptions |

**Each phase builds on verified reality, not assumptions.**

---

## 4. Summary

CivilOS must be built from:

1. Platform readiness
2. City structure
3. System observability
4. Verified external connectivity
5. Composed system understanding
6. Real capability consumption
7. Operational workflows
8. Policy enforcement
9. Multi-city deployment
10. Full packaging and distribution

> This phased approach ensures **no false completion claims**, **no hidden integration gaps**, **no fragile architecture** — a reproducible, installable system aligned with the long-term vision.
