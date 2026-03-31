# VoteOS Pilot Scenario — Shadow Election Validation

> Defines the recommended first pilot deployment of VoteOS.

---

## Pilot Type: Shadow Election

The recommended first pilot is a **shadow election** — running VoteOS alongside a legacy system to compare outcomes without replacing anything.

This is the safest way to build confidence:
- No risk to real election operations
- Direct comparison with legacy results
- Audit reconstruction tested against real data

---

## Pilot Flow

### Phase 1: Setup (Day 1)

1. Deploy VoteOS using pilot config (`config/templates/voteos.pilot.toml`)
2. Verify health/ready/status endpoints
3. Run deployment checklist (see DEPLOYMENT_CHECKLIST.md)

### Phase 2: Data Import (Day 1-2)

4. Collect legacy election data:
   - Voter roll (JSON format)
   - Election configuration
   - Reported outcome (winner, vote counts)

5. Run adoption layer normalization:
   - Load legacy voter records via JSON adapter
   - Normalize records (check for Invalid/Incomplete)
   - Run identity reconciliation against known subjects

6. Review normalization report:
   - How many voters normalized successfully?
   - How many incomplete/invalid?
   - How many identities matched/missing/ambiguous?

### Phase 3: Shadow Election (Day 2-3)

7. Create VoteOS election matching the legacy configuration
8. Register normalized voters
9. Create ballot matching legacy ballot items
10. Enter vote data (replaying legacy votes through VoteOS recording)
11. Close election
12. Compute tally

### Phase 4: Comparison (Day 3)

13. Run shadow validation:
    - Compare legacy reported outcome vs VoteOS computed tally
    - Check per-item: winners match? counts match?

14. Classify result:
    - **Match**: VoteOS produces identical outcome
    - **Semantic Equivalent**: Same winners, minor count differences
    - **True Mismatch**: Different winners — investigate

15. Run audit verification:
    - Assemble audit bundle
    - Verify reconstruction matches certification

### Phase 5: Report (Day 4)

16. Produce pilot report:
    - Normalization success rate
    - Reconciliation coverage
    - Shadow validation result
    - Audit verification result
    - Any discrepancies found

---

## Success Criteria

| Criterion | Threshold |
|-----------|-----------|
| Normalization success rate | > 90% of records normalized |
| Identity reconciliation | > 80% matched |
| Shadow validation outcome | Match or Semantic Equivalent |
| Audit reconstruction | Zero discrepancies |
| System uptime during pilot | No crashes or data loss |

## Failure Scenarios

| Scenario | Action |
|----------|--------|
| True Mismatch in shadow validation | Investigate legacy data quality vs VoteOS logic |
| > 20% invalid legacy records | Legacy data format needs adapter adjustment |
| Audit reconstruction fails | Critical bug — investigate before continuing |
| System crash during pilot | Investigate, fix, restart from persisted state |

## What Counts as Acceptable Mismatch

- **Acceptable**: Semantic Equivalent (same winner, counts differ by < 1% due to provisional ballot handling differences)
- **Acceptable**: Legacy has extra ballot items not in VoteOS scope
- **NOT acceptable**: Different winner on any shared ballot item
- **NOT acceptable**: Audit reconstruction discrepancy

---

## What This Pilot Does NOT Cover

- Live AxiaSystem identity resolution (uses lookup tables)
- Real-time voter interaction (votes are replayed, not cast live)
- Parallel mode (both systems active simultaneously)
- Cutover (replacing legacy system)

These are future phases after shadow validation succeeds.
