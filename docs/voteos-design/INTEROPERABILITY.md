# VoteOS Interoperability Stance

> How VoteOS relates to AxiaSystem, CivilOS, and future systems.

---

## The Shared Truth Model

```
                    AxiaSystem
                   (Source of Truth)
                  /                \
               CivilOS           VoteOS
          (City Operations)  (Election Integrity)
```

AxiaSystem is the canonical source for:
- Identity (who is this person?)
- Authentication (is this person who they claim?)
- Legitimacy (is this person authorized for this action?)
- Attestation (tamper-evident action recording)
- Explanation (audit-ready decision reasoning)

Both CivilOS and VoteOS consume these capabilities. Neither owns them.

---

## Citizen Identity Persistence

### Scenario: City deploys CivilOS first, then VoteOS later

1. **CivilOS onboards citizens** via `resolve_subject`
   - Maria Rodriguez gets subject_ref `abc123-...`
   - Her identity exists in AxiaSystem with assurance level, standing, roles

2. **City later deploys VoteOS**
   - Maria's identity already exists in AxiaSystem
   - VoteOS does NOT re-onboard her
   - VoteOS verifies her eligibility using her existing subject_ref
   - Her assurance level, standing, and demographic data carry over

3. **Maria registers to vote**
   - VoteOS calls `evaluate_legitimacy` with her subject_ref
   - AxiaSystem confirms her identity and standing
   - VoteOS records her as eligible (if she meets election-specific criteria)

4. **Maria votes in a city referendum**
   - VoteOS verifies eligibility (AxiaSystem identity + VoteOS voter roll)
   - Vote is recorded with ballot secrecy
   - Result is certified

5. **CivilOS implements the referendum result**
   - VoteOS publishes certified result
   - CivilOS governance module consumes the result
   - CivilOS implements the policy change
   - VoteOS is not involved in implementation

### Scenario: City deploys VoteOS first

1. **VoteOS deployment triggers citizen onboarding** if citizens don't yet exist
2. Citizens are onboarded to AxiaSystem (same resolve_subject path)
3. If CivilOS is deployed later, those citizens already exist

### Key principle: The order of deployment does not matter. AxiaSystem is the shared root.

---

## Data Boundary Rules

| Data | Owned By | Consumed By |
|------|----------|-------------|
| Citizen identity | AxiaSystem | CivilOS, VoteOS |
| Assurance level | AxiaSystem | CivilOS, VoteOS |
| City department roles | CivilOS | CivilOS only |
| Voter eligibility | VoteOS | VoteOS only |
| Election state | VoteOS | CivilOS (read-only) |
| Certified results | VoteOS | CivilOS, public |
| Individual votes | VoteOS (secret) | Nobody (in secret ballot mode) |
| DMV records | CivilOS | CivilOS only |
| Financial transactions | CivilOS | CivilOS only |

---

## Communication Model

VoteOS and CivilOS do NOT call each other directly.

Instead:
- Both read from AxiaSystem (shared identity truth)
- VoteOS publishes certified results (as attested records)
- CivilOS may read those results when implementing governance decisions
- No real-time coupling between the two systems

This preserves:
- VoteOS neutrality (not influenced by city operations)
- CivilOS independence (not blocked by election timing)
- System resilience (one can be down without affecting the other)

---

## Future Systems

The same pattern extends to any future AxiaSystem consumer:
- **EducationOS**: school operations consuming shared identity
- **HealthOS**: health services consuming shared identity
- **TransitOS**: transportation consuming shared identity

Each system:
- Consumes AxiaSystem identity
- Owns its domain-specific data
- Does not duplicate identity management
- Can interoperate through shared attestation records
