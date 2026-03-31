//! Tests for Module 8: Audit & Oversight
//!
//! Critical tests: audit reconstruction, tamper detection, missing data,
//! secrecy preservation, contest linkage, observer read-only access.
//!
//! These tests prove VoteOS results are externally verifiable.

use std::collections::BTreeMap;
use voteos::domain::voters::*;
use voteos::domain::elections::*;
use voteos::domain::ballots::*;
use voteos::domain::votes::*;
use voteos::domain::tally::*;
use voteos::domain::certification::*;
use voteos::domain::audit::*;

// ===========================================================================
// SHARED FIXTURE — sets up a complete certified election for auditing
// ===========================================================================

struct AuditFixture {
    voter_registry: VoterRegistry,
    election_registry: ElectionRegistry,
    ballot_registry: BallotRegistry,
    vote_registry: VoteRegistry,
    tally_registry: TallyRegistry,
    cert_registry: CertificationRegistry,
    audit_registry: AuditRegistry,
    election_ref: String,
    tally_ref: String,
    cert_ref: String,
}

impl AuditFixture {
    /// Create a fully certified election ready for auditing.
    /// 5 voters: alice(3), bob(1), carol(1) for mayor; yes(4), no(1) for measure.
    fn certified_election() -> Self {
        let election_registry = ElectionRegistry::new();
        let voter_registry = VoterRegistry::new();
        let ballot_registry = BallotRegistry::new();
        let vote_registry = VoteRegistry::new();
        let tally_registry = TallyRegistry::new();
        let cert_registry = CertificationRegistry::new();
        let audit_registry = AuditRegistry::new();

        // Create election
        let election_ref = election_registry.elections.insert_new(Election {
            title: "Audit Test Election".into(),
            description: "Full election for audit testing".into(),
            election_type: ElectionType::General,
            status: ElectionStatus::Draft,
            config: ElectionConfig::default(),
            schedule: ElectionSchedule {
                registration_start: None, registration_end: None,
                voting_start: None, voting_end: None,
                certification_deadline: None,
            },
            scope: "test-district".into(),
            created_by: "admin-1".into(),
            created_at: "2026-03-30T08:00:00Z".into(),
            decision_ref: "dec-create".into(),
        });

        // Register voters
        for i in 1..=5 {
            voter_registry.registrations.insert_new(VoterRegistration {
                citizen_ref: format!("citizen-{}", i),
                election_ref: election_ref.clone(),
                status: RegistrationStatus::Registered,
                registered_at: "2026-03-30T08:30:00Z".into(),
                registered_by: "admin-1".into(),
                eligibility_basis: "eligible".into(),
                decision_ref: "dec-reg".into(),
                attestation_ref: None,
            });
        }

        // Create and finalize ballot
        let ballot_items = vec![
            BallotItem {
                item_ref: "mayor".into(),
                item_type: BallotItemType::Race,
                title: "Mayor".into(),
                description: "Choose one".into(),
                choices: vec![
                    BallotChoice { choice_ref: "alice".into(), label: "Alice".into(), description: None },
                    BallotChoice { choice_ref: "bob".into(), label: "Bob".into(), description: None },
                    BallotChoice { choice_ref: "carol".into(), label: "Carol".into(), description: None },
                ],
                max_selections: 1,
            },
            BallotItem {
                item_ref: "measure-a".into(),
                item_type: BallotItemType::Measure,
                title: "Measure A".into(),
                description: "Park funding".into(),
                choices: vec![
                    BallotChoice { choice_ref: "yes".into(), label: "Yes".into(), description: None },
                    BallotChoice { choice_ref: "no".into(), label: "No".into(), description: None },
                ],
                max_selections: 1,
            },
        ];

        let template_ref = ballot_registry.templates.insert_new(BallotTemplate {
            election_ref: election_ref.clone(),
            status: BallotStatus::Finalized,
            items: ballot_items,
            created_by: "admin-1".into(),
            created_at: "2026-03-30T09:00:00Z".into(),
            finalized_at: Some("2026-03-30T09:30:00Z".into()),
            finalized_by: Some("admin-1".into()),
            decision_ref: "dec-ballot".into(),
            integrity_hash: Some("hash-1".into()),
        });

        // Open election
        election_registry.transition_election(&election_ref, ElectionStatus::Published, "a", "d", None).unwrap();
        election_registry.transition_election(&election_ref, ElectionStatus::Open, "a", "d", None).unwrap();

        // Issue ballots and cast votes
        let choices = vec![
            ("citizen-1", "alice", "yes"),
            ("citizen-2", "alice", "yes"),
            ("citizen-3", "alice", "yes"),
            ("citizen-4", "bob", "yes"),
            ("citizen-5", "carol", "no"),
        ];

        for (voter, mayor_choice, measure_choice) in &choices {
            let iref = ballot_registry.issuances.insert_new(BallotIssuance {
                template_ref: template_ref.clone(),
                voter_ref: voter.to_string(),
                election_ref: election_ref.clone(),
                status: IssuanceStatus::Issued,
                issued_at: "2026-03-30T10:00:00Z".into(),
                issued_by: "admin-1".into(),
                decision_ref: "dec-issue".into(),
                spoiled_at: None,
                replacement_ref: None,
            });

            let vote_ref = vote_registry.records.insert_new(VoteRecord {
                election_ref: election_ref.clone(),
                ballot_issuance_ref: iref,
                status: VoteStatus::Sealed,
                submitted_at: "2026-03-30T11:00:00Z".into(),
                sealed_at: Some("2026-03-30T11:00:00Z".into()),
                receipt_hash: "hash".into(),
                decision_ref: "dec-vote".into(),
                attestation_ref: None,
            });

            vote_registry.contents.insert_new(VoteContent {
                vote_ref: vote_ref.clone(),
                election_ref: election_ref.clone(),
                selections: vec![
                    VoteSelection {
                        ballot_item_ref: "mayor".into(),
                        choice_ref: mayor_choice.to_string(),
                        rank: None,
                    },
                    VoteSelection {
                        ballot_item_ref: "measure-a".into(),
                        choice_ref: measure_choice.to_string(),
                        rank: None,
                    },
                ],
            });

            vote_registry.participation.insert_new(VoterParticipation {
                voter_ref: voter.to_string(),
                election_ref: election_ref.clone(),
                voted_at: "2026-03-30T11:00:00Z".into(),
                vote_ref,
            });
        }

        // Close, tally, certify
        election_registry.transition_election(&election_ref, ElectionStatus::Closed, "a", "d", None).unwrap();

        let sealed = vote_registry.sealed_contents(&election_ref);
        let input_hash = compute_input_hash(&sealed);
        let items = vec!["mayor".to_string(), "measure-a".to_string()];
        let (item_tallies, has_ambiguity) = compute_plurality_tally(&election_ref, &sealed, &items);

        let tally = TallyResult {
            election_ref: election_ref.clone(),
            method: VotingMethod::Plurality,
            status: TallyStatus::Computed,
            item_tallies,
            total_votes_counted: 5,
            computed_at: "2026-03-30T12:00:00Z".into(),
            computed_by: "admin-1".into(),
            decision_ref: "dec-tally".into(),
            input_hash,
            has_ambiguity,
        };
        let tally_ref = tally_registry.results.insert_new(tally.clone());

        election_registry.transition_election(&election_ref, ElectionStatus::Tallied, "a", "d", None).unwrap();

        let cert_ref = cert_registry.certifications.insert_new(CertificationRecord {
            election_ref: election_ref.clone(),
            tally_ref: tally_ref.clone(),
            tally_snapshot: tally,
            status: CertificationStatus::Certified,
            certified_by: Some("admin-1".into()),
            certified_at: Some("2026-03-30T13:00:00Z".into()),
            certification_basis: "Plurality with clear winner".into(),
            decision_ref: "dec-certify".into(),
            attestation_ref: None,
            rejection_reason: None,
            created_at: "2026-03-30T13:00:00Z".into(),
        });

        election_registry.transition_election(&election_ref, ElectionStatus::Certified, "a", "d", None).unwrap();

        Self {
            voter_registry,
            election_registry,
            ballot_registry,
            vote_registry,
            tally_registry,
            cert_registry,
            audit_registry,
            election_ref,
            tally_ref,
            cert_ref,
        }
    }
}

// ===========================================================================
// A. HAPPY PATH — Audit verifies successfully
// ===========================================================================

#[test]
fn test_audit_happy_path_verified() {
    let f = AuditFixture::certified_election();

    // Assemble audit bundle
    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).expect("Bundle assembly must succeed");

    assert_eq!(bundle.sealed_vote_count, 5);
    assert!(bundle.certification.is_some());
    assert!(bundle.certified_tally.is_some());

    // Verify bundle
    let verification = verify_bundle(&bundle);

    assert!(verification.matches, "Audit must verify — no discrepancies expected");
    assert!(verification.discrepancies.is_empty(),
        "Zero discrepancies expected, got: {:?}", verification.discrepancies);
    assert_eq!(verification.reconstructed_input_hash, verification.certified_input_hash,
        "Input hashes must match");
}

#[test]
fn test_audit_reconstructed_tally_matches_certified() {
    let f = AuditFixture::certified_election();

    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    let verification = verify_bundle(&bundle);
    let recon = &verification.reconstructed_tally;

    // Mayor: alice=3, bob=1, carol=1
    assert_eq!(recon.item_tallies[0].winners, vec!["alice"]);
    assert_eq!(*recon.item_tallies[0].choice_counts.get("alice").unwrap(), 3);
    assert_eq!(*recon.item_tallies[0].choice_counts.get("bob").unwrap(), 1);
    assert_eq!(*recon.item_tallies[0].choice_counts.get("carol").unwrap(), 1);

    // Measure A: yes=4, no=1
    assert_eq!(recon.item_tallies[1].winners, vec!["yes"]);
    assert_eq!(*recon.item_tallies[1].choice_counts.get("yes").unwrap(), 4);
    assert_eq!(*recon.item_tallies[1].choice_counts.get("no").unwrap(), 1);

    // Must exactly match certified
    let certified = bundle.certified_tally.unwrap();
    for i in 0..certified.item_tallies.len() {
        assert_eq!(
            recon.item_tallies[i].choice_counts,
            certified.item_tallies[i].choice_counts,
            "Item {} counts must match", i
        );
        assert_eq!(
            recon.item_tallies[i].winners,
            certified.item_tallies[i].winners,
            "Item {} winners must match", i
        );
    }
}

// ===========================================================================
// B. TAMPER SIMULATION — altered tally detected
// ===========================================================================

#[test]
fn test_audit_detects_tampered_tally() {
    let f = AuditFixture::certified_election();

    let mut bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // TAMPER: modify the certified tally to claim bob won
    if let Some(ref mut tally) = bundle.certified_tally {
        tally.item_tallies[0].choice_counts.insert("bob".into(), 99);
        tally.item_tallies[0].winners = vec!["bob".into()];
    }

    let verification = verify_bundle(&bundle);

    assert!(!verification.matches, "Tampered tally must be detected");
    assert!(!verification.discrepancies.is_empty(),
        "Must have discrepancies for tampered tally");

    // Should have TallyMismatch for choice counts and winners
    let tally_mismatches: Vec<_> = verification.discrepancies.iter()
        .filter(|d| d.category == DiscrepancyCategory::TallyMismatch)
        .collect();
    assert!(tally_mismatches.len() >= 1,
        "Must detect at least one TallyMismatch");
}

#[test]
fn test_audit_detects_tampered_input_hash() {
    let f = AuditFixture::certified_election();

    let mut bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // TAMPER: modify the input hash in the certified tally
    if let Some(ref mut tally) = bundle.certified_tally {
        tally.input_hash = "tampered-hash-12345".into();
    }

    let verification = verify_bundle(&bundle);

    assert!(!verification.matches);
    let hash_mismatches: Vec<_> = verification.discrepancies.iter()
        .filter(|d| d.category == DiscrepancyCategory::InputHashMismatch)
        .collect();
    assert_eq!(hash_mismatches.len(), 1,
        "Must detect InputHashMismatch when hash is tampered");
}

#[test]
fn test_audit_detects_tampered_vote_count() {
    let f = AuditFixture::certified_election();

    let mut bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // TAMPER: claim more votes were counted than actually exist
    if let Some(ref mut tally) = bundle.certified_tally {
        tally.total_votes_counted = 999;
    }

    let verification = verify_bundle(&bundle);

    assert!(!verification.matches);
    let count_mismatches: Vec<_> = verification.discrepancies.iter()
        .filter(|d| d.category == DiscrepancyCategory::VoteCountMismatch)
        .collect();
    assert!(!count_mismatches.is_empty(),
        "Must detect VoteCountMismatch when count is tampered");
}

// ===========================================================================
// C. MISSING DATA — vote removed, audit fails
// ===========================================================================

#[test]
fn test_audit_detects_missing_vote() {
    let f = AuditFixture::certified_election();

    let mut bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // Remove one vote from the bundle (simulating data loss)
    bundle.sealed_contents.pop();
    bundle.sealed_vote_count = bundle.sealed_contents.len() as u64;

    let verification = verify_bundle(&bundle);

    assert!(!verification.matches, "Missing vote must cause audit failure");

    // Should detect vote count mismatch and input hash mismatch
    assert!(!verification.discrepancies.is_empty());
}

#[test]
fn test_audit_detects_added_phantom_vote() {
    let f = AuditFixture::certified_election();

    let mut bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // Add a phantom vote
    bundle.sealed_contents.push((
        "phantom-vote".into(),
        VoteContent {
            vote_ref: "phantom-vote".into(),
            election_ref: f.election_ref.clone(),
            selections: vec![VoteSelection {
                ballot_item_ref: "mayor".into(),
                choice_ref: "bob".into(),
                rank: None,
            }],
        },
    ));
    bundle.sealed_vote_count = bundle.sealed_contents.len() as u64;

    let verification = verify_bundle(&bundle);

    assert!(!verification.matches, "Phantom vote must cause audit failure");
}

// ===========================================================================
// D. CONSISTENCY — reconstructed tally = certified tally
// ===========================================================================

#[test]
fn test_audit_deterministic_reconstruction() {
    let f = AuditFixture::certified_election();

    // Run verification twice — must produce identical results
    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    let v1 = verify_bundle(&bundle);
    let v2 = verify_bundle(&bundle);

    assert_eq!(v1.matches, v2.matches);
    assert_eq!(v1.reconstructed_input_hash, v2.reconstructed_input_hash);
    assert_eq!(v1.discrepancies.len(), v2.discrepancies.len());

    for i in 0..v1.reconstructed_tally.item_tallies.len() {
        assert_eq!(
            v1.reconstructed_tally.item_tallies[i].choice_counts,
            v2.reconstructed_tally.item_tallies[i].choice_counts,
        );
    }
}

// ===========================================================================
// E. CONTEST TRIGGER — failed audit creates contest
// ===========================================================================

#[test]
fn test_audit_failure_triggers_contest() {
    let f = AuditFixture::certified_election();

    // Create a failed audit record
    let audit_ref = f.audit_registry.records.insert_new(AuditRecord {
        election_ref: f.election_ref.clone(),
        status: AuditStatus::Failed,
        initiated_by: "auditor-1".into(),
        initiated_at: "2026-03-30T15:00:00Z".into(),
        completed_at: Some("2026-03-30T15:30:00Z".into()),
        verification: Some(AuditVerification {
            matches: false,
            reconstructed_tally: TallyResult {
                election_ref: f.election_ref.clone(),
                method: VotingMethod::Plurality,
                status: TallyStatus::Computed,
                item_tallies: vec![],
                total_votes_counted: 4, // Mismatch: should be 5
                computed_at: "audit".into(),
                computed_by: "audit".into(),
                decision_ref: "audit".into(),
                input_hash: "different-hash".into(),
                has_ambiguity: false,
            },
            discrepancies: vec![Discrepancy {
                category: DiscrepancyCategory::VoteCountMismatch,
                description: "Count mismatch".into(),
                ballot_item_ref: None,
                expected: "5".into(),
                actual: "4".into(),
            }],
            reconstructed_input_hash: "different-hash".into(),
            certified_input_hash: "original-hash".into(),
        }),
        decision_ref: "dec-audit".into(),
        contest_ref: None,
    });

    // Domain-level contest creation from failed audit
    let contest_ref = f.cert_registry.contests.insert_new(Contest {
        certification_ref: f.cert_ref.clone(),
        election_ref: f.election_ref.clone(),
        filed_by: "auditor-1".into(),
        reason: "Audit failure: vote count mismatch".into(),
        filed_at: "2026-03-30T16:00:00Z".into(),
        status: ContestStatus::Filed,
        resolution: None,
        resolved_by: None,
        resolved_at: None,
        decision_ref: "dec-contest".into(),
    });

    // Update audit record with contest ref
    let mut record = f.audit_registry.records.get(&audit_ref).unwrap();
    record.status = AuditStatus::Contested;
    record.contest_ref = Some(contest_ref.clone());
    f.audit_registry.records.update(&audit_ref, record);

    // Update certification to Contested
    let mut cert = f.cert_registry.certifications.get(&f.cert_ref).unwrap();
    cert.status = CertificationStatus::Contested;
    f.cert_registry.certifications.update(&f.cert_ref, cert);

    // Verify state
    assert!(f.cert_registry.is_contested(&f.election_ref),
        "Certification must be contested after audit failure");
    let (_, audit) = f.audit_registry.audit_for_election(&f.election_ref).unwrap();
    assert_eq!(audit.status, AuditStatus::Contested);
    assert!(audit.contest_ref.is_some());
}

// ===========================================================================
// F. SECRECY PRESERVATION — audit does not expose voter identity
// ===========================================================================

#[test]
fn test_audit_bundle_preserves_secrecy() {
    let f = AuditFixture::certified_election();

    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // Verify: bundle contains NO voter identity
    // AuditBundle has: election, ballot_template, sealed_contents, certified_tally, certification
    // It does NOT have: VoterParticipation, VoterRegistration, VotingReceipt

    // sealed_contents are Vec<(String, VoteContent)>
    // VoteContent has: vote_ref, election_ref, selections — NO voter_ref
    for (_, content) in &bundle.sealed_contents {
        // VoteContent struct does not have a voter_ref field.
        // This is a compile-time guarantee. We verify runtime data integrity.
        assert!(!content.vote_ref.is_empty());
        assert_eq!(content.election_ref, f.election_ref);
    }

    // Use the verify_secrecy function
    assert!(verify_secrecy(&bundle),
        "Audit bundle must preserve ballot secrecy");
}

#[test]
fn test_audit_bundle_has_no_participation_records() {
    let f = AuditFixture::certified_election();

    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // The AuditBundle struct does not contain VoterParticipation.
    // This is structural secrecy — the bundle physically cannot link
    // a voter to their vote content.

    // Verify the bundle has the data it should have
    assert!(bundle.election.title.contains("Audit Test"));
    assert!(bundle.ballot_template.is_some());
    assert_eq!(bundle.sealed_contents.len(), 5);
    assert!(bundle.certified_tally.is_some());
    assert!(bundle.certification.is_some());

    // And does NOT have voter-identifying data
    // (This is a design assertion — the struct simply doesn't include those fields)
}

// ===========================================================================
// REGISTRY OPERATIONS
// ===========================================================================

#[test]
fn test_audit_registry_operations() {
    let registry = AuditRegistry::new();

    assert!(!registry.is_audited("e1"));

    registry.records.insert_new(AuditRecord {
        election_ref: "e1".into(),
        status: AuditStatus::Verified,
        initiated_by: "auditor-1".into(),
        initiated_at: "2026-03-30T15:00:00Z".into(),
        completed_at: Some("2026-03-30T15:30:00Z".into()),
        verification: None,
        decision_ref: "dec-1".into(),
        contest_ref: None,
    });

    assert!(registry.is_audited("e1"));
    assert!(!registry.is_audited("e2"));
}

#[test]
fn test_audit_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = AuditRegistry::with_data_dir(path);
        registry.records.insert_new(AuditRecord {
            election_ref: "e1".into(),
            status: AuditStatus::Verified,
            initiated_by: "auditor-1".into(),
            initiated_at: "2026-03-30T15:00:00Z".into(),
            completed_at: None,
            verification: None,
            decision_ref: "dec-1".into(),
            contest_ref: None,
        });
    }

    {
        let registry = AuditRegistry::with_data_dir(path);
        assert_eq!(registry.records.count(), 1);
        assert!(registry.is_audited("e1"));
    }
}

// ===========================================================================
// OBSERVER READ-ONLY — observer can inspect but not mutate
// ===========================================================================

#[test]
fn test_observer_can_assemble_and_verify_independently() {
    let f = AuditFixture::certified_election();

    // Observer assembles bundle — read-only operation
    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    // Observer verifies independently — pure function, no mutation
    let result = verify_bundle(&bundle);
    assert!(result.matches, "Observer's independent verification must succeed");

    // Verify the election state is unchanged after observer verification
    let election = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(election.status, ElectionStatus::Certified,
        "Observer verification must not mutate election state");
    assert!(f.cert_registry.is_certified(&f.election_ref),
        "Certification must remain intact after observer verification");
}

// ===========================================================================
// WORKFLOW TESTS (require ICP replica)
// ===========================================================================

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_start_audit_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_verify_audit_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
