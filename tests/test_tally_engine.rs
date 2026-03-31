//! Tests for Module 5: Tally Engine
//!
//! Critical tests: determinism, tie detection, ambiguity handling,
//! threshold evaluation, no-vote handling, ordering independence.

use std::collections::BTreeMap;
use voteos::domain::tally::*;
use voteos::domain::votes::*;
use voteos::domain::elections::*;

// ---------------------------------------------------------------------------
// Helper: create vote content for testing
// ---------------------------------------------------------------------------

fn make_vote(id: &str, election: &str, item: &str, choice: &str) -> (String, VoteContent) {
    (
        id.to_string(),
        VoteContent {
            vote_ref: id.to_string(),
            election_ref: election.to_string(),
            selections: vec![VoteSelection {
                ballot_item_ref: item.to_string(),
                choice_ref: choice.to_string(),
                rank: None,
            }],
        },
    )
}

fn make_multi_item_vote(
    id: &str,
    election: &str,
    selections: Vec<(&str, &str)>,
) -> (String, VoteContent) {
    (
        id.to_string(),
        VoteContent {
            vote_ref: id.to_string(),
            election_ref: election.to_string(),
            selections: selections
                .into_iter()
                .map(|(item, choice)| VoteSelection {
                    ballot_item_ref: item.to_string(),
                    choice_ref: choice.to_string(),
                    rank: None,
                })
                .collect(),
        },
    )
}

// ===========================================================================
// TALLY COMPUTATION — HAPPY PATH
// ===========================================================================

#[test]
fn test_plurality_clear_winner() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "alice"),
        make_vote("v3", "e1", "mayor", "alice"),
        make_vote("v4", "e1", "mayor", "bob"),
        make_vote("v5", "e1", "mayor", "bob"),
    ];

    let result = compute_plurality_item("mayor", &contents);

    assert_eq!(result.total_votes, 5);
    assert_eq!(result.winners, vec!["alice"]);
    assert!(!result.is_tie);
    assert!(!result.is_ambiguous);
    assert_eq!(*result.choice_counts.get("alice").unwrap(), 3);
    assert_eq!(*result.choice_counts.get("bob").unwrap(), 2);
}

#[test]
fn test_plurality_multiple_items() {
    let contents = vec![
        make_multi_item_vote("v1", "e1", vec![("mayor", "alice"), ("measure1", "yes")]),
        make_multi_item_vote("v2", "e1", vec![("mayor", "bob"), ("measure1", "yes")]),
        make_multi_item_vote("v3", "e1", vec![("mayor", "alice"), ("measure1", "no")]),
    ];

    let items = vec!["mayor".to_string(), "measure1".to_string()];
    let (tallies, has_ambiguity) = compute_plurality_tally("e1", &contents, &items);

    assert_eq!(tallies.len(), 2);
    assert!(!has_ambiguity);

    // Mayor: alice=2, bob=1
    assert_eq!(tallies[0].winners, vec!["alice"]);
    assert_eq!(tallies[0].total_votes, 3);

    // Measure1: yes=2, no=1
    assert_eq!(tallies[1].winners, vec!["yes"]);
    assert_eq!(tallies[1].total_votes, 3);
}

// ===========================================================================
// DETERMINISM — CRITICAL PROOF
// ===========================================================================

#[test]
fn test_determinism_same_input_same_output() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "alice"),
        make_vote("v4", "e1", "mayor", "carol"),
        make_vote("v5", "e1", "mayor", "bob"),
    ];

    let result1 = compute_plurality_item("mayor", &contents);
    let result2 = compute_plurality_item("mayor", &contents);

    assert_eq!(result1, result2, "Identical input must produce identical output");
}

#[test]
fn test_determinism_different_order_same_result() {
    let order_a = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "alice"),
        make_vote("v4", "e1", "mayor", "carol"),
        make_vote("v5", "e1", "mayor", "bob"),
    ];

    let order_b = vec![
        make_vote("v5", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "alice"),
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v4", "e1", "mayor", "carol"),
        make_vote("v2", "e1", "mayor", "bob"),
    ];

    let result_a = compute_plurality_item("mayor", &order_a);
    let result_b = compute_plurality_item("mayor", &order_b);

    // Core determinism: counts are the same regardless of order
    assert_eq!(result_a.choice_counts, result_b.choice_counts,
        "Vote counts must be identical regardless of input order");
    assert_eq!(result_a.winners, result_b.winners,
        "Winners must be identical regardless of input order");
    assert_eq!(result_a.total_votes, result_b.total_votes);
    assert_eq!(result_a.is_tie, result_b.is_tie);
    assert_eq!(result_a.is_ambiguous, result_b.is_ambiguous);
}

#[test]
fn test_determinism_input_hash_order_independent() {
    let order_a = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
    ];

    let order_b = vec![
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v1", "e1", "mayor", "alice"),
    ];

    let hash_a = compute_input_hash(&order_a);
    let hash_b = compute_input_hash(&order_b);

    assert_eq!(hash_a, hash_b,
        "Input hash must be identical regardless of input order (sorted internally)");
}

#[test]
fn test_determinism_repeated_computation() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "carol"),
    ];

    let items = vec!["mayor".to_string()];

    // Run 10 times — must be identical every time
    let (first_tallies, first_ambiguity) = compute_plurality_tally("e1", &contents, &items);
    for _ in 0..10 {
        let (tallies, ambiguity) = compute_plurality_tally("e1", &contents, &items);
        assert_eq!(tallies, first_tallies, "Tally must be identical on repeated computation");
        assert_eq!(ambiguity, first_ambiguity);
    }
}

// ===========================================================================
// TIE DETECTION AND AMBIGUITY
// ===========================================================================

#[test]
fn test_exact_tie_two_candidates() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "alice"),
        make_vote("v4", "e1", "mayor", "bob"),
    ];

    let result = compute_plurality_item("mayor", &contents);

    assert!(result.is_tie, "Equal vote counts must be detected as tie");
    assert!(result.is_ambiguous, "Tie must be flagged as ambiguous");
    assert_eq!(result.winners.len(), 2, "Both tied candidates must appear as winners");
    assert!(result.winners.contains(&"alice".to_string()));
    assert!(result.winners.contains(&"bob".to_string()));
    assert_eq!(result.total_votes, 4);
}

#[test]
fn test_three_way_tie() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "bob"),
        make_vote("v3", "e1", "mayor", "carol"),
    ];

    let result = compute_plurality_item("mayor", &contents);

    assert!(result.is_tie);
    assert!(result.is_ambiguous);
    assert_eq!(result.winners.len(), 3);
}

#[test]
fn test_no_votes_is_ambiguous() {
    let contents: Vec<(String, VoteContent)> = vec![];

    let result = compute_plurality_item("mayor", &contents);

    assert_eq!(result.total_votes, 0);
    assert!(result.is_ambiguous, "No votes must be flagged as ambiguous");
    assert!(result.winners.is_empty(), "No winner when no votes");
    assert!(!result.is_tie, "Zero votes is not a tie — it's absence");
}

#[test]
fn test_full_tally_with_ambiguity_flag() {
    let contents = vec![
        // Mayor has a clear winner
        make_multi_item_vote("v1", "e1", vec![("mayor", "alice"), ("measure1", "yes")]),
        make_multi_item_vote("v2", "e1", vec![("mayor", "alice"), ("measure1", "no")]),
        // Measure1 is a tie
        make_multi_item_vote("v3", "e1", vec![("mayor", "bob"), ("measure1", "yes")]),
    ];

    let items = vec!["mayor".to_string(), "measure1".to_string()];
    let (tallies, has_ambiguity) = compute_plurality_tally("e1", &contents, &items);

    // Mayor is clear
    assert!(!tallies[0].is_ambiguous);
    assert_eq!(tallies[0].winners, vec!["alice"]);

    // Measure1 is tied — but 2 yes vs 1 no, so not tied
    // Actually: yes=2, no=1 → not a tie
    assert!(!tallies[1].is_ambiguous);

    // Overall no ambiguity because no item is ambiguous
    assert!(!has_ambiguity);
}

#[test]
fn test_tally_with_one_item_tied() {
    let contents = vec![
        make_multi_item_vote("v1", "e1", vec![("mayor", "alice"), ("measure1", "yes")]),
        make_multi_item_vote("v2", "e1", vec![("mayor", "alice"), ("measure1", "no")]),
    ];

    let items = vec!["mayor".to_string(), "measure1".to_string()];
    let (tallies, has_ambiguity) = compute_plurality_tally("e1", &contents, &items);

    assert!(!tallies[0].is_ambiguous); // mayor: alice=2
    assert!(tallies[1].is_ambiguous);  // measure1: yes=1, no=1 → TIE
    assert!(has_ambiguity, "Overall tally must flag ambiguity when any item is tied");
}

// ===========================================================================
// THRESHOLD EVALUATION
// ===========================================================================

#[test]
fn test_participation_threshold_met() {
    let (met, pct) = evaluate_participation_threshold(80, 100, 50.0);
    assert!(met);
    assert!((pct - 80.0).abs() < 0.01);
}

#[test]
fn test_participation_threshold_not_met() {
    let (met, pct) = evaluate_participation_threshold(30, 100, 50.0);
    assert!(!met);
    assert!((pct - 30.0).abs() < 0.01);
}

#[test]
fn test_participation_threshold_zero_eligible() {
    let (met, pct) = evaluate_participation_threshold(10, 0, 50.0);
    assert!(!met);
    assert!((pct - 0.0).abs() < 0.01);
}

#[test]
fn test_margin_threshold_clear_margin() {
    let item = ItemTally {
        ballot_item_ref: "mayor".to_string(),
        choice_counts: BTreeMap::from([
            ("alice".to_string(), 70),
            ("bob".to_string(), 30),
        ]),
        total_votes: 100,
        winners: vec!["alice".to_string()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: String::new(),
    };

    let (met, margin) = evaluate_margin_threshold(&item, 10.0);
    assert!(met);
    assert!((margin - 40.0).abs() < 0.01); // (70-30)/100 = 40%
}

#[test]
fn test_margin_threshold_narrow() {
    let item = ItemTally {
        ballot_item_ref: "mayor".to_string(),
        choice_counts: BTreeMap::from([
            ("alice".to_string(), 51),
            ("bob".to_string(), 49),
        ]),
        total_votes: 100,
        winners: vec!["alice".to_string()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: String::new(),
    };

    let (met, margin) = evaluate_margin_threshold(&item, 5.0);
    assert!(!met); // 2% < 5%
    assert!((margin - 2.0).abs() < 0.01);
}

#[test]
fn test_margin_threshold_tie() {
    let item = ItemTally {
        ballot_item_ref: "mayor".to_string(),
        choice_counts: BTreeMap::from([
            ("alice".to_string(), 50),
            ("bob".to_string(), 50),
        ]),
        total_votes: 100,
        winners: vec!["alice".to_string(), "bob".to_string()],
        is_tie: true,
        is_ambiguous: true,
        result_summary: String::new(),
    };

    let (met, margin) = evaluate_margin_threshold(&item, 1.0);
    assert!(!met); // Tie → margin = 0
}

// ===========================================================================
// TALLY REGISTRY
// ===========================================================================

#[test]
fn test_tally_registry_store_and_retrieve() {
    let registry = TallyRegistry::new();

    let tally = TallyResult {
        election_ref: "e1".into(),
        method: VotingMethod::Plurality,
        status: TallyStatus::Computed,
        item_tallies: vec![ItemTally {
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
        }],
        total_votes_counted: 5,
        computed_at: "2026-03-30T10:00:00Z".into(),
        computed_by: "official-1".into(),
        decision_ref: "dec-1".into(),
        input_hash: "abc123".into(),
        has_ambiguity: false,
    };

    let tally_ref = registry.results.insert_new(tally);
    assert!(tally_ref.starts_with("taly-"));

    let (_, retrieved) = registry.result_for_election("e1").expect("should exist");
    assert_eq!(retrieved.status, TallyStatus::Computed);
    assert_eq!(retrieved.total_votes_counted, 5);
    assert!(!retrieved.has_ambiguity);
}

#[test]
fn test_tally_registry_has_tally() {
    let registry = TallyRegistry::new();
    assert!(!registry.has_tally("e1"));

    registry.results.insert_new(TallyResult {
        election_ref: "e1".into(),
        method: VotingMethod::Plurality,
        status: TallyStatus::Computed,
        item_tallies: vec![],
        total_votes_counted: 0,
        computed_at: "2026-03-30T10:00:00Z".into(),
        computed_by: "official-1".into(),
        decision_ref: "dec-1".into(),
        input_hash: "abc".into(),
        has_ambiguity: false,
    });

    assert!(registry.has_tally("e1"));
    assert!(!registry.has_tally("e2"));
}

#[test]
fn test_tally_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = TallyRegistry::with_data_dir(path);
        registry.results.insert_new(TallyResult {
            election_ref: "e1".into(),
            method: VotingMethod::Plurality,
            status: TallyStatus::Computed,
            item_tallies: vec![],
            total_votes_counted: 42,
            computed_at: "2026-03-30T10:00:00Z".into(),
            computed_by: "official-1".into(),
            decision_ref: "dec-1".into(),
            input_hash: "hash1".into(),
            has_ambiguity: false,
        });
    }

    {
        let registry = TallyRegistry::with_data_dir(path);
        assert_eq!(registry.results.count(), 1);
        let (_, r) = registry.result_for_election("e1").expect("should survive roundtrip");
        assert_eq!(r.total_votes_counted, 42);
    }
}

// ===========================================================================
// SINGLE CANDIDATE / EDGE CASES
// ===========================================================================

#[test]
fn test_single_candidate_wins() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
        make_vote("v2", "e1", "mayor", "alice"),
    ];

    let result = compute_plurality_item("mayor", &contents);

    assert_eq!(result.winners, vec!["alice"]);
    assert!(!result.is_tie);
    assert!(!result.is_ambiguous);
    assert_eq!(result.total_votes, 2);
}

#[test]
fn test_single_vote_single_candidate() {
    let contents = vec![
        make_vote("v1", "e1", "mayor", "alice"),
    ];

    let result = compute_plurality_item("mayor", &contents);
    assert_eq!(result.winners, vec!["alice"]);
    assert!(!result.is_tie);
    assert_eq!(result.total_votes, 1);
}

// ===========================================================================
// WORKFLOW TESTS (require ICP replica)
// ===========================================================================

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_compute_tally_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_compute_tally_blocked_when_not_closed() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
