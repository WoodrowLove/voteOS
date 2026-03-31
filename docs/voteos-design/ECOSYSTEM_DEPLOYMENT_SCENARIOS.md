# VoteOS Ecosystem Deployment Scenarios

> How VoteOS operates in every real-world deployment configuration.

---

## Scenario 1: VoteOS Alone with AxiaSystem

**Use case:** School board, tribal government, homeowners association, standalone election authority.

```
AxiaSystem (identity/legitimacy)
         │
      VoteOS (elections)
```

| Concern | Source |
|---------|--------|
| Citizen identity | AxiaSystem (VoteOS onboards citizens directly via resolve_subject) |
| Decision truth | VoteOS (elections, tallies, certification) |
| Operational truth | N/A (no CivilOS) |
| Interoperability | None needed — standalone |

**Citizen flow:**
1. Citizen onboarded directly through VoteOS → AxiaSystem
2. Identity established, assurance level set
3. Eligibility verified, voter registered
4. All election operations in VoteOS
5. Certified results published through VoteOS API

---

## Scenario 2: CivilOS Alone with AxiaSystem

**Use case:** City that needs operations but not structured elections yet.

```
AxiaSystem (identity/legitimacy)
         │
      CivilOS (city operations)
```

| Concern | Source |
|---------|--------|
| Citizen identity | AxiaSystem (CivilOS onboards citizens) |
| Decision truth | CivilOS governance module (operational decisions only) |
| Operational truth | CivilOS (permits, DMV, finance, etc.) |
| Interoperability | None needed — standalone |

**Note:** CivilOS's governance module handles operational governance (budget approvals, department decisions) but NOT democratic elections. If the city later needs elections, they deploy VoteOS.

---

## Scenario 3: VoteOS + CivilOS Together with Shared AxiaSystem

**Use case:** Full municipal deployment — city operations AND democratic elections.

```
         AxiaSystem (shared identity/legitimacy)
        /                                      \
    CivilOS                                  VoteOS
    (city operations)                      (elections)
```

| Concern | Source |
|---------|--------|
| Citizen identity | AxiaSystem (onboarded by whichever system the citizen touches first) |
| Decision truth | VoteOS (democratic elections, referenda) |
| Operational truth | CivilOS (permits, DMV, finance, public safety) |
| Interoperability | Via AxiaSystem attested records — no direct coupling |

**Citizen flow (Maria Rodriguez):**
1. Maria onboarded by CivilOS two years ago → AxiaSystem subject_ref `abc123`
2. City deploys VoteOS
3. VoteOS calls resolve_subject for Maria → AxiaSystem says: exists, assurance L0, standing active
4. VoteOS checks eligibility (age 35, jurisdiction Springfield) → eligible
5. Maria registered to vote — no re-onboarding needed
6. Maria votes in city council election
7. VoteOS certifies result → attested in AxiaSystem
8. CivilOS reads certified result → updates governance structure

**Key:** Maria's identity persisted across systems because AxiaSystem is the shared root.

---

## Scenario 4: Legacy City Adopts VoteOS First

**Use case:** City has legacy election system, wants to modernize elections before operations.

```
Legacy Election System ──(migration)──► VoteOS ──► AxiaSystem
```

| Phase | What Happens |
|-------|-------------|
| 1. Assessment | Inventory legacy voter rolls, election configs |
| 2. AxiaSystem setup | Deploy AxiaSystem, onboard citizens from legacy voter data |
| 3. VoteOS deployment | Deploy VoteOS, configure elections |
| 4. Identity reconciliation | Map legacy voters to AxiaSystem identities |
| 5. Shadow run | Run VoteOS alongside legacy for test election |
| 6. Cutover | VoteOS becomes official election system |
| 7. CivilOS later (optional) | If city wants operations, deploy CivilOS — citizens already in AxiaSystem |

**Identity persistence:** Citizens onboarded for VoteOS carry into CivilOS if deployed later.

---

## Scenario 5: Legacy City Adopts CivilOS First, Then VoteOS

**Use case:** City modernizes operations first, then elections.

```
Legacy City Systems ──(migration)──► CivilOS ──► AxiaSystem
                                                       │
                                                    VoteOS (later)
```

| Phase | What Happens |
|-------|-------------|
| 1. CivilOS deployed | Citizens onboarded to AxiaSystem |
| 2. CivilOS operational | Permits, DMV, finance working |
| 3. VoteOS deployed | Election system added |
| 4. Automatic identity | Citizens already in AxiaSystem — no re-onboarding |
| 5. Voter eligibility | VoteOS verifies existing citizens against election rules |
| 6. Elections operational | VoteOS handles elections using shared identity base |

**Identity persistence:** This is the smoothest path. Citizens are already verified and assurance-leveled from CivilOS. VoteOS just checks eligibility.

**Example:**
- Maria was onboarded by CivilOS in 2025
- VoteOS deployed in 2026
- Maria's subject_ref, assurance level, standing all carry over
- VoteOS adds: eligible to vote in Springfield municipal elections
- Maria votes without any additional identity setup

---

## Scenario Comparison Matrix

| Scenario | Identity Source | Citizen Onboarding | Election Authority | Ops Authority | Migration Needed |
|----------|---------------|-------------------|-------------------|---------------|-----------------|
| 1. VoteOS only | AxiaSystem (via VoteOS) | VoteOS | VoteOS | N/A | If replacing legacy elections |
| 2. CivilOS only | AxiaSystem (via CivilOS) | CivilOS | N/A | CivilOS | If replacing legacy city ops |
| 3. Both together | AxiaSystem (shared) | First-touch system | VoteOS | CivilOS | Depends on legacy |
| 4. VoteOS first | AxiaSystem (via VoteOS) | VoteOS | VoteOS | CivilOS (later) | Legacy election migration |
| 5. CivilOS first | AxiaSystem (via CivilOS) | CivilOS | VoteOS (later) | CivilOS | Legacy city ops + later election |

---

## Key Takeaway

**The order of deployment does not matter.**

AxiaSystem is the shared root. Citizens onboarded by any system carry identity into all other systems in the ecosystem. This is the fundamental design principle that makes the ecosystem work.
