//! Tests for Module 4: Vote Recording domain types and registry.
//!
//! Critical tests: double-vote prevention, ballot secrecy, receipt verification.

use voteos::domain::votes::*;
use voteos::domain::ballots::*;

#[test]
fn test_cast_vote_records_correctly() {
    let registry = VoteRegistry::new();

    let vote_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "abc123".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    assert!(vote_ref.starts_with("vote-"));
    let record = registry.records.get(&vote_ref).expect("should exist");
    assert_eq!(record.status, VoteStatus::Recorded);
}

#[test]
fn test_double_vote_prevention() {
    let registry = VoteRegistry::new();

    // First vote
    registry.participation.insert_new(VoterParticipation {
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        voted_at: "2026-03-30T10:00:00Z".into(),
        vote_ref: "vote-001".into(),
    });

    // Check: voter has voted
    assert!(registry.has_voted("voter-001", "elec-001"));

    // Different voter hasn't voted
    assert!(!registry.has_voted("voter-002", "elec-001"));

    // Same voter hasn't voted in different election
    assert!(!registry.has_voted("voter-001", "elec-002"));
}

#[test]
fn test_ballot_secrecy_architecture() {
    let registry = VoteRegistry::new();

    // Create vote record (NO voter_ref field — secrecy by design)
    let vote_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash1".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    // Vote content links to vote_ref but has NO voter identity
    registry.contents.insert_new(VoteContent {
        vote_ref: vote_ref.clone(),
        election_ref: "elec-001".into(),
        selections: vec![VoteSelection {
            ballot_item_ref: "item-001".into(),
            choice_ref: "c-001".into(),
            rank: None,
        }],
    });

    // Participation links voter to vote_ref but NOT to content
    registry.participation.insert_new(VoterParticipation {
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        voted_at: "2026-03-30T10:00:00Z".into(),
        vote_ref: vote_ref.clone(),
    });

    // Verify secrecy: content has no voter_ref field
    let contents = registry.contents.find_all(|c| c.election_ref == "elec-001");
    assert_eq!(contents.len(), 1);
    // VoteContent struct has: vote_ref, election_ref, selections — NO voter_ref

    // Verify secrecy holds
    assert!(registry.verify_ballot_secrecy("elec-001"));
}

#[test]
fn test_vote_receipt_generation_and_verification() {
    let registry = VoteRegistry::new();

    let timestamp = "2026-03-30T10:00:00Z";
    let vote_ref = "vote-001";
    let election_ref = "elec-001";

    let receipt_hash = VoteRegistry::compute_receipt_hash(vote_ref, election_ref, timestamp);
    assert!(!receipt_hash.is_empty());

    // Store receipt
    registry.receipts.insert_new(VotingReceipt {
        voter_ref: "voter-001".into(),
        election_ref: election_ref.into(),
        receipt_hash: receipt_hash.clone(),
        timestamp: timestamp.into(),
        vote_ref: vote_ref.into(),
    });

    // Verify receipt
    let found = registry.get_receipt("voter-001", election_ref);
    assert!(found.is_some());
    let (_, receipt) = found.unwrap();
    assert_eq!(receipt.receipt_hash, receipt_hash);

    // Recompute and verify
    let recomputed = VoteRegistry::compute_receipt_hash(vote_ref, election_ref, timestamp);
    assert_eq!(recomputed, receipt_hash, "Receipt hash must be deterministic");
}

#[test]
fn test_seal_vote() {
    let registry = VoteRegistry::new();

    let vote_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash1".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    // Seal the vote
    let mut record = registry.records.get(&vote_ref).expect("exists");
    record.status = VoteStatus::Sealed;
    record.sealed_at = Some("2026-03-30T11:00:00Z".into());
    registry.records.update(&vote_ref, record);

    let sealed = registry.records.get(&vote_ref).expect("exists");
    assert_eq!(sealed.status, VoteStatus::Sealed);
    assert!(sealed.sealed_at.is_some());
}

#[test]
fn test_spoil_vote() {
    let registry = VoteRegistry::new();

    let vote_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash1".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    // Spoil the vote
    let mut record = registry.records.get(&vote_ref).expect("exists");
    record.status = VoteStatus::Spoiled;
    registry.records.update(&vote_ref, record);

    let spoiled = registry.records.get(&vote_ref).expect("exists");
    assert_eq!(spoiled.status, VoteStatus::Spoiled);

    // Spoiled votes should not count
    assert_eq!(registry.votes_submitted("elec-001"), 0);
}

#[test]
fn test_votes_submitted_count() {
    let registry = VoteRegistry::new();

    for i in 1..=5 {
        registry.records.insert_new(VoteRecord {
            election_ref: "elec-001".into(),
            ballot_issuance_ref: format!("biss-{:03}", i),
            status: VoteStatus::Recorded,
            submitted_at: "2026-03-30T10:00:00Z".into(),
            sealed_at: None,
            receipt_hash: format!("hash-{}", i),
            decision_ref: format!("dec-{:03}", i),
            attestation_ref: None,
        });
    }

    // Add a spoiled vote
    registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-006".into(),
        status: VoteStatus::Spoiled,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash-6".into(),
        decision_ref: "dec-006".into(),
        attestation_ref: None,
    });

    assert_eq!(registry.votes_submitted("elec-001"), 5); // Spoiled excluded
}

#[test]
fn test_sealed_contents_for_tallying() {
    let registry = VoteRegistry::new();

    // Create 3 sealed votes
    for i in 1..=3 {
        let vote_ref = registry.records.insert_new(VoteRecord {
            election_ref: "elec-001".into(),
            ballot_issuance_ref: format!("biss-{:03}", i),
            status: VoteStatus::Sealed,
            submitted_at: "2026-03-30T10:00:00Z".into(),
            sealed_at: Some("2026-03-30T11:00:00Z".into()),
            receipt_hash: format!("hash-{}", i),
            decision_ref: format!("dec-{:03}", i),
            attestation_ref: None,
        });

        registry.contents.insert_new(VoteContent {
            vote_ref,
            election_ref: "elec-001".into(),
            selections: vec![VoteSelection {
                ballot_item_ref: "item-001".into(),
                choice_ref: format!("c-{:03}", i),
                rank: None,
            }],
        });
    }

    // Create 1 recorded (not yet sealed)
    let unseal_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-004".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash-4".into(),
        decision_ref: "dec-004".into(),
        attestation_ref: None,
    });

    registry.contents.insert_new(VoteContent {
        vote_ref: unseal_ref,
        election_ref: "elec-001".into(),
        selections: vec![VoteSelection {
            ballot_item_ref: "item-001".into(),
            choice_ref: "c-004".into(),
            rank: None,
        }],
    });

    let sealed = registry.sealed_contents("elec-001");
    assert_eq!(sealed.len(), 3, "Only sealed vote contents should be returned for tallying");
}

#[test]
fn test_multiple_selections_ranked_choice() {
    let registry = VoteRegistry::new();

    let vote_ref = registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash1".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    registry.contents.insert_new(VoteContent {
        vote_ref: vote_ref.clone(),
        election_ref: "elec-001".into(),
        selections: vec![
            VoteSelection { ballot_item_ref: "item-001".into(), choice_ref: "alice".into(), rank: Some(1) },
            VoteSelection { ballot_item_ref: "item-001".into(), choice_ref: "bob".into(), rank: Some(2) },
            VoteSelection { ballot_item_ref: "item-001".into(), choice_ref: "carol".into(), rank: Some(3) },
        ],
    });

    let contents = registry.contents.find_all(|c| c.vote_ref == vote_ref);
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].1.selections.len(), 3);
    assert_eq!(contents[0].1.selections[0].rank, Some(1));
    assert_eq!(contents[0].1.selections[2].rank, Some(3));
}

#[test]
fn test_vote_audit_secrecy() {
    let registry = VoteRegistry::new();

    // Audit entries for votes should NOT contain voter identity
    registry.audit_log.insert_new(VoteAuditEntry {
        action: "cast_vote".into(),
        actor_ref: None, // Secret ballot — no actor identity
        election_ref: "elec-001".into(),
        timestamp: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
        details: "Vote vote-001 recorded".into(),
    });

    let entries = registry.audit_log.find_all(|_| true);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].1.actor_ref.is_none(), "Vote audit should not contain actor identity");
}

#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = VoteRegistry::with_data_dir(path);
        registry.records.insert_new(VoteRecord {
            election_ref: "elec-001".into(),
            ballot_issuance_ref: "biss-001".into(),
            status: VoteStatus::Recorded,
            submitted_at: "2026-03-30T10:00:00Z".into(),
            sealed_at: None,
            receipt_hash: "hash1".into(),
            decision_ref: "dec-001".into(),
            attestation_ref: None,
        });
    }

    {
        let registry = VoteRegistry::with_data_dir(path);
        assert_eq!(registry.records.count(), 1);
    }
}

// ---------------------------------------------------------------------------
// Cross-module integration tests (domain-level, no AxiaSystem)
// ---------------------------------------------------------------------------

#[test]
fn test_vote_requires_ballot_issuance() {
    let ballot_registry = BallotRegistry::new();
    let vote_registry = VoteRegistry::new();

    // No ballot issued — cast_vote would fail the precondition check
    assert!(!ballot_registry.has_active_issuance("voter-001", "elec-001"));

    // Issue a ballot
    ballot_registry.issuances.insert_new(BallotIssuance {
        template_ref: "btpl-001".into(),
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        status: IssuanceStatus::Issued,
        issued_at: "2026-03-30T10:00:00Z".into(),
        issued_by: "official-001".into(),
        decision_ref: "dec-001".into(),
        spoiled_at: None,
        replacement_ref: None,
    });

    assert!(ballot_registry.has_active_issuance("voter-001", "elec-001"));

    // Cast vote
    let vote_ref = vote_registry.records.insert_new(VoteRecord {
        election_ref: "elec-001".into(),
        ballot_issuance_ref: "biss-001".into(),
        status: VoteStatus::Recorded,
        submitted_at: "2026-03-30T10:00:00Z".into(),
        sealed_at: None,
        receipt_hash: "hash1".into(),
        decision_ref: "dec-002".into(),
        attestation_ref: None,
    });

    vote_registry.participation.insert_new(VoterParticipation {
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        voted_at: "2026-03-30T10:00:00Z".into(),
        vote_ref: vote_ref.clone(),
    });

    // Double vote prevented
    assert!(vote_registry.has_voted("voter-001", "elec-001"));
}

// ---------------------------------------------------------------------------
// Workflow tests (require AxiaSystem replica)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_cast_vote_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_double_vote_prevention_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
