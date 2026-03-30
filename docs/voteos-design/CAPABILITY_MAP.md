# VoteOS Capability Map

> Complete enumeration of VoteOS capabilities grouped by module.
> Each capability is a distinct operation the system must support.

---

## Module 1: Voter Registry (12 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 1 | verify_voter_eligibility | data_access | Check if citizen is eligible for specific election |
| 2 | register_voter | operation | Add eligible citizen to voter roll |
| 3 | suspend_voter_registration | operation | Suspend registration (standing change, dispute) |
| 4 | restore_voter_registration | operation | Restore suspended registration |
| 5 | resolve_voter_record | data_access | Get voter's full registration record |
| 6 | define_eligibility_rule | governance_action | Set criteria for election eligibility |
| 7 | evaluate_eligibility_batch | operation | Batch-check eligibility for voter roll generation |
| 8 | challenge_eligibility | operation | File eligibility dispute |
| 9 | resolve_eligibility_challenge | governance_action | Adjudicate eligibility dispute |
| 10 | generate_voter_roll | operation | Produce finalized voter list for election |
| 11 | export_voter_statistics | data_access | Aggregate voter roll statistics (no PII) |
| 12 | audit_voter_registry | data_access | Retrieve voter registry audit trail |

---

## Module 2: Election Management (14 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 13 | create_election | governance_action | Create new election with type and scope |
| 14 | configure_election | governance_action | Set rules, thresholds, privacy mode |
| 15 | publish_election | governance_action | Make election visible to voters |
| 16 | open_election | governance_action | Begin voting period |
| 17 | close_election | governance_action | End voting period |
| 18 | extend_election | governance_action | Extend voting period (emergency) |
| 19 | cancel_election | governance_action | Cancel before certification |
| 20 | resolve_election_state | data_access | Get current election status |
| 21 | assign_election_official | governance_action | Authorize official for election |
| 22 | revoke_election_official | governance_action | Remove official authorization |
| 23 | configure_voting_method | governance_action | Set counting method (plurality, RCV, etc.) |
| 24 | set_election_schedule | operation | Define timeline/milestones |
| 25 | resolve_election_timeline | data_access | Get schedule details |
| 26 | audit_election_transitions | data_access | Review state change history |

---

## Module 3: Ballot Operations (10 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 27 | create_ballot_template | operation | Design ballot with items/questions |
| 28 | add_ballot_item | operation | Add question/race/measure to ballot |
| 29 | remove_ballot_item | operation | Remove item before publication |
| 30 | finalize_ballot | governance_action | Lock ballot content |
| 31 | issue_ballot | operation | Issue ballot to eligible voter |
| 32 | revoke_ballot | operation | Invalidate issued ballot (spoiled/replaced) |
| 33 | resolve_ballot | data_access | Get ballot content and status |
| 34 | track_ballot_issuance | data_access | Get distribution status |
| 35 | validate_ballot_integrity | data_access | Verify ballot hasn't been tampered |
| 36 | audit_ballot_operations | data_access | Review ballot lifecycle |

---

## Module 4: Vote Recording (11 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 37 | cast_vote | operation | Submit vote for ballot items |
| 38 | validate_vote | operation | Check vote validity before recording |
| 39 | prevent_double_vote | operation | Enforce one-vote-per-voter rule |
| 40 | generate_vote_receipt | operation | Create voter verification receipt |
| 41 | verify_vote_receipt | data_access | Confirm vote was recorded correctly |
| 42 | spoil_ballot | operation | Voter requests fresh ballot |
| 43 | seal_vote | operation | Finalize vote record (immutable after seal) |
| 44 | resolve_vote_status | data_access | Check if voter has voted |
| 45 | enforce_ballot_secrecy | operation | Separate vote content from voter identity |
| 46 | count_votes_submitted | data_access | Get submission count (no content) |
| 47 | audit_vote_recording | data_access | Review vote recording evidence |

---

## Module 5: Tally Engine (9 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 48 | compute_tally | operation | Count votes and produce results |
| 49 | apply_voting_method | operation | Execute specific counting algorithm |
| 50 | evaluate_participation_threshold | data_access | Check minimum participation met |
| 51 | detect_tie | data_access | Identify tie conditions |
| 52 | handle_ambiguity | operation | Classify and record ambiguous outcomes |
| 53 | produce_provisional_result | operation | Generate pre-certification result |
| 54 | verify_tally_integrity | data_access | Independently recompute and compare |
| 55 | export_tally_evidence | data_access | Package tally data for audit |
| 56 | audit_tally_computation | data_access | Review computation steps |

---

## Module 6: Result Certification (8 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 57 | certify_result | governance_action | Official attestation of election result |
| 58 | contest_result | operation | File formal contest of certified result |
| 59 | resolve_contest | governance_action | Adjudicate contested result |
| 60 | publish_result | operation | Make certified result publicly available |
| 61 | withdraw_certification | governance_action | Revoke certification (extraordinary) |
| 62 | resolve_certification_status | data_access | Get certification state |
| 63 | generate_certification_report | data_access | Produce formal certification document |
| 64 | audit_certification_chain | data_access | Review certification evidence |

---

## Module 7: Governance Proposals (10 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 65 | create_proposal | operation | Submit governance proposal |
| 66 | sponsor_proposal | operation | Add sponsorship to proposal |
| 67 | qualify_proposal | governance_action | Determine proposal meets requirements |
| 68 | link_proposal_to_election | governance_action | Place qualified proposal on ballot |
| 69 | resolve_proposal_status | data_access | Get proposal lifecycle state |
| 70 | collect_signatures | operation | Gather support for initiative |
| 71 | validate_signatures | operation | Verify collected signatures |
| 72 | record_proposal_outcome | operation | Link election result to proposal |
| 73 | archive_proposal | operation | Move decided proposal to archive |
| 74 | audit_proposal_lifecycle | data_access | Review proposal history |

---

## Module 8: Audit & Oversight (10 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 75 | register_observer | operation | Authorize election observer |
| 76 | grant_observer_access | governance_action | Scope observer permissions |
| 77 | initiate_recount | governance_action | Start formal recount |
| 78 | execute_recount | operation | Recompute tally from evidence |
| 79 | reconstruct_evidence | data_access | Rebuild result from raw records |
| 80 | generate_audit_report | data_access | Produce compliance report |
| 81 | verify_chain_of_custody | data_access | Validate attestation chain |
| 82 | file_compliance_report | operation | Submit regulatory compliance |
| 83 | manage_dispute | operation | Track dispute resolution workflow |
| 84 | audit_system_integrity | data_access | Full system integrity check |

---

## Module 9: Election Operations (8 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 85 | schedule_election_event | operation | Create calendar event |
| 86 | assign_poll_worker | operation | Staff polling location/endpoint |
| 87 | report_incident | operation | Log operational incident |
| 88 | resolve_incident | operation | Close incident with resolution |
| 89 | monitor_election_health | data_access | Real-time operational status |
| 90 | manage_polling_location | operation | Configure voting venue/endpoint |
| 91 | generate_operations_report | data_access | Operational summary |
| 92 | escalate_issue | operation | Elevate operational concern |

---

## Module 10: Integration & Export (8 capabilities)

| # | Capability | Action Type | Description |
|---|-----------|-------------|-------------|
| 93 | export_certified_result | operation | Deliver result to external system |
| 94 | register_result_consumer | operation | Configure downstream system |
| 95 | notify_result_published | operation | Send notification to consumers |
| 96 | generate_standard_report | data_access | Produce interoperable report format |
| 97 | export_audit_bundle | data_access | Package full audit evidence |
| 98 | configure_webhook | operation | Set up event notifications |
| 99 | resolve_export_status | data_access | Track delivery status |
| 100 | audit_integrations | data_access | Review export/delivery history |

---

## Summary

| Module | Capabilities | Governance Actions | Operations | Data Access |
|--------|-------------|-------------------|------------|-------------|
| 1. Voter Registry | 12 | 2 | 6 | 4 |
| 2. Election Management | 14 | 8 | 2 | 4 |
| 3. Ballot Operations | 10 | 1 | 5 | 4 |
| 4. Vote Recording | 11 | 0 | 7 | 4 |
| 5. Tally Engine | 9 | 0 | 4 | 5 |
| 6. Result Certification | 8 | 3 | 2 | 3 |
| 7. Governance Proposals | 10 | 2 | 6 | 2 |
| 8. Audit & Oversight | 10 | 2 | 4 | 4 |
| 9. Election Operations | 8 | 0 | 6 | 2 |
| 10. Integration & Export | 8 | 0 | 5 | 3 |
| **Total** | **100** | **18** | **47** | **35** |
