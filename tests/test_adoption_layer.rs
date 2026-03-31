//! Tests for Wave 7: Adoption Layer — Legacy Migration & Shadow Validation
//!
//! Proves that VoteOS can:
//! - ingest legacy election data honestly
//! - normalize without silent data loss
//! - reconcile identities preserving ambiguity
//! - shadow-validate against VoteOS recomputation
//! - detect real mismatches between legacy and VoteOS outcomes

use std::collections::BTreeMap;
use voteos::adoption::*;
use voteos::domain::tally::*;
use voteos::domain::votes::*;

// ===========================================================================
// HELPERS
// ===========================================================================

fn make_voter(id: &str, name: &str, jurisdiction: &str, status: &str) -> LegacyVoterRecord {
    LegacyVoterRecord {
        legacy_id: id.into(),
        full_name: name.into(),
        date_of_birth: Some("1990-01-01".into()),
        jurisdiction: jurisdiction.into(),
        legacy_status: status.into(),
        metadata: BTreeMap::new(),
    }
}

fn make_election(
    id: &str,
    title: &str,
    items: Vec<LegacyBallotItem>,
    outcome: Option<LegacyOutcome>,
) -> LegacyElectionRecord {
    LegacyElectionRecord {
        legacy_id: id.into(),
        title: title.into(),
        election_type: "general".into(),
        jurisdiction: "city".into(),
        voting_method: "plurality".into(),
        ballot_items: items,
        reported_outcome: outcome,
    }
}

fn make_ballot_item(id: &str, title: &str, choices: Vec<&str>) -> LegacyBallotItem {
    LegacyBallotItem {
        item_id: id.into(),
        title: title.into(),
        choices: choices.into_iter().map(String::from).collect(),
    }
}

// ===========================================================================
// A. VOTER NORMALIZATION
// ===========================================================================

#[test]
fn test_normalize_valid_voter() {
    let voter = make_voter("V001", "Alice Smith", "city", "active");
    let normalized = normalize_voter(&voter);

    assert_eq!(normalized.normalization_status, NormalizationStatus::Normalized);
    assert!(normalized.normalization_notes.is_empty());
    assert_eq!(normalized.legacy_id, "V001");
    assert_eq!(normalized.full_name, "Alice Smith");
}

#[test]
fn test_normalize_voter_missing_name() {
    let voter = make_voter("V001", "", "city", "active");
    let normalized = normalize_voter(&voter);

    assert_eq!(normalized.normalization_status, NormalizationStatus::Invalid);
    assert!(normalized.normalization_notes.iter().any(|n| n.contains("full_name")));
}

#[test]
fn test_normalize_voter_missing_id() {
    let voter = make_voter("", "Alice", "city", "active");
    let normalized = normalize_voter(&voter);

    assert_eq!(normalized.normalization_status, NormalizationStatus::Invalid);
}

#[test]
fn test_normalize_voter_unknown_status() {
    let voter = make_voter("V001", "Alice", "city", "weird_status");
    let normalized = normalize_voter(&voter);

    assert_eq!(normalized.normalization_status, NormalizationStatus::Incomplete);
    assert!(normalized.normalization_notes.iter().any(|n| n.contains("Unknown legacy status")));
}

#[test]
fn test_normalize_voter_batch() {
    let voters = vec![
        make_voter("V001", "Alice", "city", "active"),
        make_voter("V002", "Bob", "city", "registered"),
        make_voter("", "No ID", "city", "active"),      // Invalid
        make_voter("V004", "", "city", "active"),        // Invalid
        make_voter("V005", "Eve", "city", "unknown"),    // Incomplete
    ];

    let (normalized, rejected) = normalize_voter_batch(&voters);
    assert_eq!(normalized.len(), 5);
    assert_eq!(rejected, 2, "2 invalid records should be counted as rejected");

    let valid_count = normalized.iter()
        .filter(|n| n.normalization_status == NormalizationStatus::Normalized)
        .count();
    assert_eq!(valid_count, 2);
}

// ===========================================================================
// B. ELECTION NORMALIZATION
// ===========================================================================

#[test]
fn test_normalize_valid_election() {
    let election = make_election(
        "E001",
        "2026 City Election",
        vec![make_ballot_item("mayor", "Mayor", vec!["alice", "bob"])],
        None,
    );

    let normalized = normalize_election(&election);
    assert_eq!(normalized.normalization_status, NormalizationStatus::Normalized);
    assert_eq!(normalized.ballot_items.len(), 1);
}

#[test]
fn test_normalize_election_unsupported_method() {
    let mut election = make_election(
        "E001", "Test", vec![make_ballot_item("m", "M", vec!["a"])], None,
    );
    election.voting_method = "ranked_choice".into();

    let normalized = normalize_election(&election);
    assert_eq!(normalized.normalization_status, NormalizationStatus::Unsupported);
    assert!(normalized.normalization_notes.iter().any(|n| n.contains("Unsupported")));
}

#[test]
fn test_normalize_election_no_items() {
    let election = make_election("E001", "Empty", vec![], None);
    let normalized = normalize_election(&election);
    assert_eq!(normalized.normalization_status, NormalizationStatus::Incomplete);
}

#[test]
fn test_normalize_election_missing_title() {
    let election = make_election("E001", "", vec![make_ballot_item("m", "M", vec!["a"])], None);
    let normalized = normalize_election(&election);
    assert_eq!(normalized.normalization_status, NormalizationStatus::Invalid);
}

// ===========================================================================
// C. IDENTITY RECONCILIATION
// ===========================================================================

#[test]
fn test_reconcile_all_matched() {
    let voters = vec![
        normalize_voter(&make_voter("V001", "Alice", "city", "active")),
        normalize_voter(&make_voter("V002", "Bob", "city", "active")),
    ];

    let known = BTreeMap::from([
        ("V001".into(), "subject-001".into()),
        ("V002".into(), "subject-002".into()),
    ]);

    let report = reconcile_voters(&voters, &known);

    assert_eq!(report.total_records, 2);
    assert_eq!(report.matched, 2);
    assert_eq!(report.missing, 0);
    assert_eq!(report.ambiguous, 0);

    assert_eq!(report.entries[0].status, ReconciliationStatus::Matched);
    assert_eq!(report.entries[0].matched_subject_ref, Some("subject-001".into()));
}

#[test]
fn test_reconcile_with_missing() {
    let voters = vec![
        normalize_voter(&make_voter("V001", "Alice", "city", "active")),
        normalize_voter(&make_voter("V002", "Bob", "city", "active")),
        normalize_voter(&make_voter("V003", "Carol", "city", "active")),
    ];

    // Only V001 and V002 are known
    let known = BTreeMap::from([
        ("V001".into(), "subject-001".into()),
        ("V002".into(), "subject-002".into()),
    ]);

    let report = reconcile_voters(&voters, &known);

    assert_eq!(report.matched, 2);
    assert_eq!(report.missing, 1);
    assert_eq!(report.entries[2].status, ReconciliationStatus::Missing);
}

#[test]
fn test_reconcile_with_invalid_normalization() {
    let voters = vec![
        normalize_voter(&make_voter("", "No ID", "city", "active")), // Invalid
        normalize_voter(&make_voter("V002", "Bob", "city", "active")),
    ];

    let known = BTreeMap::from([
        ("V002".into(), "subject-002".into()),
    ]);

    let report = reconcile_voters(&voters, &known);

    assert_eq!(report.invalid, 1);
    assert_eq!(report.matched, 1);
    assert_eq!(report.entries[0].status, ReconciliationStatus::Invalid);
    assert!(report.entries[0].notes.contains("Invalid"));
}

#[test]
fn test_ambiguous_reconciliation_explicit() {
    // Ambiguity must be represented explicitly, never auto-resolved
    let entry = create_ambiguous_entry(
        "V001",
        "John Smith",
        vec!["subject-001".into(), "subject-007".into()],
    );

    assert_eq!(entry.status, ReconciliationStatus::Ambiguous);
    assert_eq!(entry.candidates.len(), 2);
    assert!(entry.matched_subject_ref.is_none(),
        "Ambiguous entry must NOT have a matched ref — no silent resolution");
}

// ===========================================================================
// D. SHADOW VALIDATION — MATCH
// ===========================================================================

#[test]
fn test_shadow_validate_exact_match() {
    // Legacy outcome
    let legacy = LegacyOutcome {
        item_results: vec![LegacyItemResult {
            item_id: "mayor".into(),
            winner: Some("alice".into()),
            vote_counts: BTreeMap::from([
                ("alice".into(), 3),
                ("bob".into(), 2),
            ]),
            total_votes: 5,
        }],
        total_votes_reported: 5,
        certified: true,
    };

    // VoteOS tally (same data)
    let voteos_tallies = vec![ItemTally {
        ballot_item_ref: "mayor".into(),
        choice_counts: BTreeMap::from([
            ("alice".into(), 3),
            ("bob".into(), 2),
        ]),
        total_votes: 5,
        winners: vec!["alice".into()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: "alice wins".into(),
    }];

    let report = shadow_validate(&legacy, &voteos_tallies, "E001");

    assert_eq!(report.overall_result, ShadowComparisonResult::Match);
    assert_eq!(report.item_comparisons.len(), 1);
    assert_eq!(report.item_comparisons[0].result, ShadowComparisonResult::Match);
}

// ===========================================================================
// E. SHADOW VALIDATION — TRUE MISMATCH
// ===========================================================================

#[test]
fn test_shadow_validate_true_mismatch() {
    // Legacy claims bob won
    let legacy = LegacyOutcome {
        item_results: vec![LegacyItemResult {
            item_id: "mayor".into(),
            winner: Some("bob".into()),
            vote_counts: BTreeMap::from([
                ("alice".into(), 2),
                ("bob".into(), 3),
            ]),
            total_votes: 5,
        }],
        total_votes_reported: 5,
        certified: true,
    };

    // VoteOS independently computed: alice won
    let voteos_tallies = vec![ItemTally {
        ballot_item_ref: "mayor".into(),
        choice_counts: BTreeMap::from([
            ("alice".into(), 3),
            ("bob".into(), 2),
        ]),
        total_votes: 5,
        winners: vec!["alice".into()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: "alice wins".into(),
    }];

    let report = shadow_validate(&legacy, &voteos_tallies, "E001");

    assert_eq!(report.overall_result, ShadowComparisonResult::TrueMismatch,
        "Different winners must be a true mismatch");
    assert!(report.item_comparisons[0].notes.contains("MISMATCH"));
}

// ===========================================================================
// F. SHADOW VALIDATION — SEMANTIC EQUIVALENT
// ===========================================================================

#[test]
fn test_shadow_validate_semantic_equivalent() {
    // Same winner, slightly different counts (maybe legacy rounded or had provisional)
    let legacy = LegacyOutcome {
        item_results: vec![LegacyItemResult {
            item_id: "mayor".into(),
            winner: Some("alice".into()),
            vote_counts: BTreeMap::from([
                ("alice".into(), 31),
                ("bob".into(), 20),
            ]),
            total_votes: 51,
        }],
        total_votes_reported: 51,
        certified: true,
    };

    let voteos_tallies = vec![ItemTally {
        ballot_item_ref: "mayor".into(),
        choice_counts: BTreeMap::from([
            ("alice".into(), 30),
            ("bob".into(), 20),
        ]),
        total_votes: 50,
        winners: vec!["alice".into()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: "alice wins".into(),
    }];

    let report = shadow_validate(&legacy, &voteos_tallies, "E001");

    assert_eq!(report.overall_result, ShadowComparisonResult::SemanticEquivalent,
        "Same winner but different counts = semantic equivalent");
}

// ===========================================================================
// G. SHADOW VALIDATION — LEGACY DATA INCOMPLETE
// ===========================================================================

#[test]
fn test_shadow_validate_legacy_incomplete() {
    let legacy = LegacyOutcome {
        item_results: vec![],
        total_votes_reported: 0,
        certified: false,
    };

    let voteos_tallies = vec![ItemTally {
        ballot_item_ref: "mayor".into(),
        choice_counts: BTreeMap::from([("alice".into(), 3)]),
        total_votes: 3,
        winners: vec!["alice".into()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: "alice wins".into(),
    }];

    let report = shadow_validate(&legacy, &voteos_tallies, "E001");

    assert_eq!(report.overall_result, ShadowComparisonResult::LegacyDataIncomplete);
}

// ===========================================================================
// H. JSON ADAPTER
// ===========================================================================

#[test]
fn test_json_adapter_voters() {
    let json = r#"[
        {"legacy_id": "V001", "full_name": "Alice Smith", "date_of_birth": "1990-01-01", "jurisdiction": "city", "legacy_status": "active", "metadata": {}},
        {"legacy_id": "V002", "full_name": "Bob Jones", "date_of_birth": null, "jurisdiction": "city", "legacy_status": "registered", "metadata": {}}
    ]"#;

    let voters = load_voters_from_json(json).expect("Should parse");
    assert_eq!(voters.len(), 2);
    assert_eq!(voters[0].legacy_id, "V001");
    assert_eq!(voters[1].full_name, "Bob Jones");
}

#[test]
fn test_json_adapter_election() {
    let json = r#"{
        "legacy_id": "E001",
        "title": "2026 City Election",
        "election_type": "general",
        "jurisdiction": "city",
        "voting_method": "plurality",
        "ballot_items": [
            {"item_id": "mayor", "title": "Mayor", "choices": ["alice", "bob", "carol"]}
        ],
        "reported_outcome": {
            "item_results": [
                {"item_id": "mayor", "winner": "alice", "vote_counts": {"alice": 3, "bob": 2, "carol": 1}, "total_votes": 6}
            ],
            "total_votes_reported": 6,
            "certified": true
        }
    }"#;

    let election = load_election_from_json(json).expect("Should parse");
    assert_eq!(election.legacy_id, "E001");
    assert_eq!(election.ballot_items.len(), 1);

    let outcome = election.reported_outcome.as_ref().unwrap();
    assert_eq!(outcome.total_votes_reported, 6);
    assert!(outcome.certified);
}

#[test]
fn test_json_adapter_invalid_json() {
    let result = load_voters_from_json("not json");
    assert!(result.is_err());
}

// ===========================================================================
// I. END-TO-END: IMPORT → NORMALIZE → RECONCILE → SHADOW VALIDATE
// ===========================================================================

#[test]
fn test_end_to_end_adoption_pipeline() {
    // 1. Load legacy data
    let legacy_voters = vec![
        make_voter("V001", "Alice Smith", "city", "active"),
        make_voter("V002", "Bob Jones", "city", "registered"),
        make_voter("V003", "Carol Williams", "city", "active"),
    ];

    let legacy_election = make_election(
        "E001",
        "2026 City Mayor",
        vec![make_ballot_item("mayor", "Mayor", vec!["alice", "bob", "carol"])],
        Some(LegacyOutcome {
            item_results: vec![LegacyItemResult {
                item_id: "mayor".into(),
                winner: Some("alice".into()),
                vote_counts: BTreeMap::from([
                    ("alice".into(), 2),
                    ("bob".into(), 1),
                ]),
                total_votes: 3,
            }],
            total_votes_reported: 3,
            certified: true,
        }),
    );

    // 2. Normalize
    let (normalized_voters, rejected) = normalize_voter_batch(&legacy_voters);
    assert_eq!(rejected, 0);
    assert!(normalized_voters.iter().all(|v| v.normalization_status == NormalizationStatus::Normalized));

    let normalized_election = normalize_election(&legacy_election);
    assert_eq!(normalized_election.normalization_status, NormalizationStatus::Normalized);

    // 3. Reconcile identities
    let known_subjects = BTreeMap::from([
        ("V001".into(), "subject-alice".into()),
        ("V002".into(), "subject-bob".into()),
        // V003 is missing — Carol not in system yet
    ]);

    let recon_report = reconcile_voters(&normalized_voters, &known_subjects);
    assert_eq!(recon_report.matched, 2);
    assert_eq!(recon_report.missing, 1, "Carol should be missing from system");

    // 4. Simulate VoteOS election with same votes
    let vote_contents = vec![
        ("v1".into(), VoteContent {
            vote_ref: "v1".into(),
            election_ref: "E001".into(),
            selections: vec![VoteSelection {
                ballot_item_ref: "mayor".into(),
                choice_ref: "alice".into(),
                rank: None,
            }],
        }),
        ("v2".into(), VoteContent {
            vote_ref: "v2".into(),
            election_ref: "E001".into(),
            selections: vec![VoteSelection {
                ballot_item_ref: "mayor".into(),
                choice_ref: "alice".into(),
                rank: None,
            }],
        }),
        ("v3".into(), VoteContent {
            vote_ref: "v3".into(),
            election_ref: "E001".into(),
            selections: vec![VoteSelection {
                ballot_item_ref: "mayor".into(),
                choice_ref: "bob".into(),
                rank: None,
            }],
        }),
    ];

    let items = vec!["mayor".to_string()];
    let (voteos_tallies, _) = compute_plurality_tally("E001", &vote_contents, &items);

    // 5. Shadow validate
    let legacy_outcome = legacy_election.reported_outcome.as_ref().unwrap();
    let shadow_report = shadow_validate(legacy_outcome, &voteos_tallies, "E001");

    assert_eq!(shadow_report.overall_result, ShadowComparisonResult::Match,
        "Same data should produce matching shadow validation");
    assert_eq!(shadow_report.item_comparisons[0].result, ShadowComparisonResult::Match);
}

// ===========================================================================
// J. DETERMINISM
// ===========================================================================

#[test]
fn test_normalization_deterministic() {
    let voter = make_voter("V001", "Alice", "city", "active");
    let n1 = normalize_voter(&voter);
    let n2 = normalize_voter(&voter);
    assert_eq!(n1.normalization_status, n2.normalization_status);
    assert_eq!(n1.normalization_notes, n2.normalization_notes);
}

#[test]
fn test_shadow_validation_deterministic() {
    let legacy = LegacyOutcome {
        item_results: vec![LegacyItemResult {
            item_id: "m".into(), winner: Some("a".into()),
            vote_counts: BTreeMap::from([("a".into(), 3), ("b".into(), 2)]),
            total_votes: 5,
        }],
        total_votes_reported: 5, certified: true,
    };

    let tallies = vec![ItemTally {
        ballot_item_ref: "m".into(),
        choice_counts: BTreeMap::from([("a".into(), 3), ("b".into(), 2)]),
        total_votes: 5, winners: vec!["a".into()],
        is_tie: false, is_ambiguous: false, result_summary: "a wins".into(),
    }];

    let r1 = shadow_validate(&legacy, &tallies, "E1");
    let r2 = shadow_validate(&legacy, &tallies, "E1");
    assert_eq!(r1.overall_result, r2.overall_result);
}
