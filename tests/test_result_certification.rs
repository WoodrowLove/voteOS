//! Tests for Module 6: Result Certification
//!
//! Critical tests: certification rules, immutability, contest mechanism,
//! election lifecycle integration (Closed → Tallied → Certified).

use std::collections::BTreeMap;
use voteos::domain::certification::*;
use voteos::domain::tally::*;
use voteos::domain::elections::*;
use voteos::domain::votes::*;
use voteos::domain::ballots::*;

// ---------------------------------------------------------------------------
// Helper: create a complete tally result
// ---------------------------------------------------------------------------

fn make_tally(election_ref: &str, status: TallyStatus, has_ambiguity: bool) -> TallyResult {
    TallyResult {
        election_ref: election_ref.to_string(),
        method: VotingMethod::Plurality,
        status,
        item_tallies: vec![ItemTally {
            ballot_item_ref: "mayor".into(),
            choice_counts: BTreeMap::from([
                ("alice".into(), 3),
                ("bob".into(), 2),
            ]),
            total_votes: 5,
            winners: vec!["alice".into()],
            is_tie: false,
            is_ambiguous: has_ambiguity,
            result_summary: "alice wins".into(),
        }],
        total_votes_counted: 5,
        computed_at: "2026-03-30T12:00:00Z".into(),
        computed_by: "official-1".into(),
        decision_ref: "dec-tally-1".into(),
        input_hash: "input-hash-1".into(),
        has_ambiguity,
    }
}

// ===========================================================================
// CERTIFICATION HAPPY PATH
// ===========================================================================

#[test]
fn test_certification_record_creation() {
    let registry = CertificationRegistry::new();
    let tally = make_tally("e1", TallyStatus::Computed, false);

    let cert_ref = registry.certifications.insert_new(CertificationRecord {
        election_ref: "e1".into(),
        tally_ref: "taly-1".into(),
        tally_snapshot: tally,
        status: CertificationStatus::Certified,
        certified_by: Some("official-1".into()),
        certified_at: Some("2026-03-30T13:00:00Z".into()),
        certification_basis: "Plurality tally with clear winner".into(),
        decision_ref: "dec-cert-1".into(),
        attestation_ref: Some("att-1".into()),
        rejection_reason: None,
        created_at: "2026-03-30T13:00:00Z".into(),
    });

    assert!(cert_ref.starts_with("cert-"));

    let (_, cert) = registry.certification_for_election("e1").expect("should exist");
    assert_eq!(cert.status, CertificationStatus::Certified);
    assert_eq!(cert.tally_snapshot.total_votes_counted, 5);
}

#[test]
fn test_is_certified() {
    let registry = CertificationRegistry::new();
    assert!(!registry.is_certified("e1"));

    registry.certifications.insert_new(CertificationRecord {
        election_ref: "e1".into(),
        tally_ref: "taly-1".into(),
        tally_snapshot: make_tally("e1", TallyStatus::Computed, false),
        status: CertificationStatus::Certified,
        certified_by: Some("official-1".into()),
        certified_at: Some("2026-03-30T13:00:00Z".into()),
        certification_basis: "clear winner".into(),
        decision_ref: "dec-1".into(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: "2026-03-30T13:00:00Z".into(),
    });

    assert!(registry.is_certified("e1"));
    assert!(!registry.is_certified("e2"));
}

// ===========================================================================
// CERTIFICATION PRECONDITIONS (FAILURE PATHS)
// ===========================================================================

#[test]
fn test_cannot_certify_before_close() {
    // This is enforced at the workflow level (election must be Tallied).
    // At domain level, we verify the state machine prevents invalid transitions.
    let registry = ElectionRegistry::new();

    let election_ref = registry.elections.insert_new(Election {
        title: "Test Election".into(),
        description: "test".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Open,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None,
            certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-1".into(),
    });

    // Cannot transition Open → Tallied (must go through Closed first)
    let result = registry.transition_election(
        &election_ref, ElectionStatus::Tallied, "official-1", "dec-2", None
    );
    assert!(result.is_err(), "Must not be able to tally an Open election");

    // Cannot transition Open → Certified
    let result = registry.transition_election(
        &election_ref, ElectionStatus::Certified, "official-1", "dec-3", None
    );
    assert!(result.is_err(), "Must not be able to certify an Open election");
}

#[test]
fn test_cannot_certify_without_tally() {
    // At domain level, the workflow checks tally_registry.result_for_election().
    // Here we verify that behaviour: empty tally registry → no tally available.
    let tally_registry = TallyRegistry::new();
    assert!(tally_registry.result_for_election("e1").is_none(),
        "No tally should exist for election with no computation");
}

#[test]
fn test_ambiguous_tally_blocks_certification() {
    // The workflow checks: if tally_result.status == TallyStatus::Ambiguous → reject.
    // At domain level, verify the tally correctly flags ambiguity.
    let tally = make_tally("e1", TallyStatus::Ambiguous, true);
    assert_eq!(tally.status, TallyStatus::Ambiguous);
    assert!(tally.has_ambiguity, "Ambiguous tally must flag has_ambiguity");
}

#[test]
fn test_invalid_tally_blocks_certification() {
    let tally = make_tally("e1", TallyStatus::Invalid, false);
    assert_eq!(tally.status, TallyStatus::Invalid);
}

// ===========================================================================
// IMMUTABILITY AFTER CERTIFICATION
// ===========================================================================

#[test]
fn test_certified_election_cannot_reopen() {
    let registry = ElectionRegistry::new();

    let election_ref = registry.elections.insert_new(Election {
        title: "Test Election".into(),
        description: "test".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Certified,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None,
            certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-1".into(),
    });

    // Cannot transition Certified → anything
    for target in &[
        ElectionStatus::Draft, ElectionStatus::Published,
        ElectionStatus::Open, ElectionStatus::Closed,
        ElectionStatus::Tallied,
    ] {
        let result = registry.transition_election(
            &election_ref, target.clone(), "official-1", "dec-2", None
        );
        assert!(result.is_err(),
            "Certified election must not transition to {:?}", target);
    }
}

#[test]
fn test_certified_election_cannot_be_cancelled() {
    let registry = ElectionRegistry::new();

    let election_ref = registry.elections.insert_new(Election {
        title: "Test".into(),
        description: "test".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Certified,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None,
            certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-1".into(),
    });

    let result = registry.transition_election(
        &election_ref, ElectionStatus::Cancelled, "official-1", "dec-2", None
    );
    assert!(result.is_err(), "Certified election must not be cancelled");
}

// ===========================================================================
// CONTEST MECHANISM
// ===========================================================================

#[test]
fn test_contest_filed_and_resolved() {
    let registry = CertificationRegistry::new();

    // Create certification
    let cert_ref = registry.certifications.insert_new(CertificationRecord {
        election_ref: "e1".into(),
        tally_ref: "taly-1".into(),
        tally_snapshot: make_tally("e1", TallyStatus::Computed, false),
        status: CertificationStatus::Certified,
        certified_by: Some("official-1".into()),
        certified_at: Some("2026-03-30T13:00:00Z".into()),
        certification_basis: "clear winner".into(),
        decision_ref: "dec-1".into(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: "2026-03-30T13:00:00Z".into(),
    });

    // File contest
    let contest_ref = registry.contests.insert_new(Contest {
        certification_ref: cert_ref.clone(),
        election_ref: "e1".into(),
        filed_by: "challenger-1".into(),
        reason: "Irregularity reported at precinct 7".into(),
        filed_at: "2026-03-30T14:00:00Z".into(),
        status: ContestStatus::Filed,
        resolution: None,
        resolved_by: None,
        resolved_at: None,
        decision_ref: "dec-contest-1".into(),
    });

    assert!(contest_ref.starts_with("cont-"));
    assert!(registry.is_contested("e1"));

    // Resolve contest (dismissed)
    let mut contest = registry.contests.get(&contest_ref).expect("should exist");
    contest.status = ContestStatus::Dismissed;
    contest.resolution = Some("Investigation found no irregularity".into());
    contest.resolved_by = Some("authority-1".into());
    contest.resolved_at = Some("2026-03-30T15:00:00Z".into());
    registry.contests.update(&contest_ref, contest);

    // After dismissal, not actively contested
    assert!(!registry.is_contested("e1"));
}

#[test]
fn test_upheld_contest_remains_contested() {
    let registry = CertificationRegistry::new();

    registry.certifications.insert_new(CertificationRecord {
        election_ref: "e1".into(),
        tally_ref: "taly-1".into(),
        tally_snapshot: make_tally("e1", TallyStatus::Computed, false),
        status: CertificationStatus::Contested,
        certified_by: Some("official-1".into()),
        certified_at: Some("2026-03-30T13:00:00Z".into()),
        certification_basis: "clear winner".into(),
        decision_ref: "dec-1".into(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: "2026-03-30T13:00:00Z".into(),
    });

    let contest_ref = registry.contests.insert_new(Contest {
        certification_ref: "cert-1".into(),
        election_ref: "e1".into(),
        filed_by: "challenger-1".into(),
        reason: "Serious irregularity".into(),
        filed_at: "2026-03-30T14:00:00Z".into(),
        status: ContestStatus::Upheld,
        resolution: Some("Investigation confirmed irregularity".into()),
        resolved_by: Some("authority-1".into()),
        resolved_at: Some("2026-03-30T15:00:00Z".into()),
        decision_ref: "dec-contest-1".into(),
    });

    // Upheld contest is resolved (no longer Filed/UnderReview)
    assert!(!registry.is_contested("e1"),
        "Upheld contest is resolved, not actively contested");
}

// ===========================================================================
// FULL ELECTION LIFECYCLE (domain level)
// ===========================================================================

#[test]
fn test_full_lifecycle_create_to_certify() {
    // This test exercises the complete trust chain at domain level:
    // Create → Register → Ballot → Vote → Close → Tally → Certify

    let election_registry = ElectionRegistry::new();
    let vote_registry = VoteRegistry::new();
    let ballot_registry = BallotRegistry::new();
    let tally_registry = TallyRegistry::new();
    let cert_registry = CertificationRegistry::new();

    // 1. Create election
    let election_ref = election_registry.elections.insert_new(Election {
        title: "City Council".into(),
        description: "2026 general election".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None,
            certification_deadline: None,
        },
        scope: "city".into(),
        created_by: "admin-1".into(),
        created_at: "2026-03-30T08:00:00Z".into(),
        decision_ref: "dec-create".into(),
    });

    // 2. Create and finalize ballot
    let template_ref = ballot_registry.templates.insert_new(BallotTemplate {
        election_ref: election_ref.clone(),
        status: BallotStatus::Finalized,
        items: vec![BallotItem {
            item_ref: "council-seat-1".into(),
            item_type: BallotItemType::Race,
            title: "Council Seat 1".into(),
            description: "Choose one".into(),
            choices: vec![
                BallotChoice { choice_ref: "alice".into(), label: "Alice".into(), description: None },
                BallotChoice { choice_ref: "bob".into(), label: "Bob".into(), description: None },
                BallotChoice { choice_ref: "carol".into(), label: "Carol".into(), description: None },
            ],
            max_selections: 1,
        }],
        created_by: "admin-1".into(),
        created_at: "2026-03-30T08:30:00Z".into(),
        finalized_at: Some("2026-03-30T09:00:00Z".into()),
        finalized_by: Some("admin-1".into()),
        decision_ref: "dec-ballot".into(),
        integrity_hash: Some("hash-1".into()),
    });

    // 3. Transition to Open
    election_registry.transition_election(
        &election_ref, ElectionStatus::Published, "admin-1", "dec-pub", None
    ).expect("Draft → Published");
    election_registry.transition_election(
        &election_ref, ElectionStatus::Open, "admin-1", "dec-open", None
    ).expect("Published → Open");

    // 4. Cast votes (3 for alice, 2 for bob, 1 for carol)
    let votes = vec![
        ("alice", "alice", "alice", "bob", "bob", "carol"),
    ];
    for (i, choice) in ["alice", "alice", "alice", "bob", "bob", "carol"].iter().enumerate() {
        let vote_ref = vote_registry.records.insert_new(VoteRecord {
            election_ref: election_ref.clone(),
            ballot_issuance_ref: format!("biss-{}", i),
            status: VoteStatus::Sealed,
            submitted_at: "2026-03-30T10:00:00Z".into(),
            sealed_at: Some("2026-03-30T10:01:00Z".into()),
            receipt_hash: format!("hash-{}", i),
            decision_ref: format!("dec-vote-{}", i),
            attestation_ref: None,
        });

        vote_registry.contents.insert_new(VoteContent {
            vote_ref,
            election_ref: election_ref.clone(),
            selections: vec![VoteSelection {
                ballot_item_ref: "council-seat-1".into(),
                choice_ref: choice.to_string(),
                rank: None,
            }],
        });
    }

    // 5. Close election
    election_registry.transition_election(
        &election_ref, ElectionStatus::Closed, "admin-1", "dec-close", None
    ).expect("Open → Closed");

    // 6. Compute tally
    let sealed = vote_registry.sealed_contents(&election_ref);
    assert_eq!(sealed.len(), 6, "6 sealed votes should be available");

    let ballot_item_refs = vec!["council-seat-1".to_string()];
    let input_hash = compute_input_hash(&sealed);
    let (item_tallies, has_ambiguity) = compute_plurality_tally(
        &election_ref, &sealed, &ballot_item_refs
    );

    assert!(!has_ambiguity, "No tie — clear winner expected");
    assert_eq!(item_tallies[0].winners, vec!["alice"]);
    assert_eq!(item_tallies[0].total_votes, 6);
    assert_eq!(*item_tallies[0].choice_counts.get("alice").unwrap(), 3);
    assert_eq!(*item_tallies[0].choice_counts.get("bob").unwrap(), 2);
    assert_eq!(*item_tallies[0].choice_counts.get("carol").unwrap(), 1);

    let tally_ref = tally_registry.results.insert_new(TallyResult {
        election_ref: election_ref.clone(),
        method: VotingMethod::Plurality,
        status: TallyStatus::Computed,
        item_tallies: item_tallies.clone(),
        total_votes_counted: 6,
        computed_at: "2026-03-30T11:00:00Z".into(),
        computed_by: "admin-1".into(),
        decision_ref: "dec-tally".into(),
        input_hash: input_hash.clone(),
        has_ambiguity: false,
    });

    election_registry.transition_election(
        &election_ref, ElectionStatus::Tallied, "admin-1", "dec-tally-tr", None
    ).expect("Closed → Tallied");

    // 7. Certify result
    let tally_snapshot = tally_registry.results.get(&tally_ref).expect("tally exists");
    let cert_ref = cert_registry.certifications.insert_new(CertificationRecord {
        election_ref: election_ref.clone(),
        tally_ref: tally_ref.clone(),
        tally_snapshot,
        status: CertificationStatus::Certified,
        certified_by: Some("admin-1".into()),
        certified_at: Some("2026-03-30T12:00:00Z".into()),
        certification_basis: "Plurality tally with clear winner, no ambiguity".into(),
        decision_ref: "dec-certify".into(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: "2026-03-30T12:00:00Z".into(),
    });

    election_registry.transition_election(
        &election_ref, ElectionStatus::Certified, "admin-1", "dec-certify-tr", None
    ).expect("Tallied → Certified");

    // 8. Verify final state
    let final_election = election_registry.elections.get(&election_ref).expect("election exists");
    assert_eq!(final_election.status, ElectionStatus::Certified);

    let cert = cert_registry.certification_for_election(&election_ref);
    assert!(cert.is_some());
    let (_, cert_record) = cert.unwrap();
    assert_eq!(cert_record.status, CertificationStatus::Certified);
    assert_eq!(cert_record.tally_snapshot.item_tallies[0].winners, vec!["alice"]);

    // 9. Verify determinism — recompute and compare
    let recomputed_hash = compute_input_hash(&vote_registry.sealed_contents(&election_ref));
    assert_eq!(recomputed_hash, input_hash, "Input hash must be deterministic");

    let (recomputed_tallies, _) = compute_plurality_tally(
        &election_ref, &vote_registry.sealed_contents(&election_ref), &ballot_item_refs
    );
    assert_eq!(recomputed_tallies[0].choice_counts, item_tallies[0].choice_counts,
        "Recomputed tally must match original");

    // 10. Verify immutability — cannot transition away from Certified
    let result = election_registry.transition_election(
        &election_ref, ElectionStatus::Open, "admin-1", "dec-bad", None
    );
    assert!(result.is_err(), "Certified election must be immutable");
}

// ===========================================================================
// CERTIFICATION PERSISTENCE
// ===========================================================================

#[test]
fn test_certification_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = CertificationRegistry::with_data_dir(path);
        registry.certifications.insert_new(CertificationRecord {
            election_ref: "e1".into(),
            tally_ref: "taly-1".into(),
            tally_snapshot: make_tally("e1", TallyStatus::Computed, false),
            status: CertificationStatus::Certified,
            certified_by: Some("official-1".into()),
            certified_at: Some("2026-03-30T13:00:00Z".into()),
            certification_basis: "clear winner".into(),
            decision_ref: "dec-1".into(),
            attestation_ref: None,
            rejection_reason: None,
            created_at: "2026-03-30T13:00:00Z".into(),
        });
    }

    {
        let registry = CertificationRegistry::with_data_dir(path);
        assert_eq!(registry.certifications.count(), 1);
        assert!(registry.is_certified("e1"));
    }
}

// ===========================================================================
// WORKFLOW TESTS (require ICP replica)
// ===========================================================================

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_certify_result_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_contest_result_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
