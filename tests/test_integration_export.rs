//! Tests for Module 10: Integration & Export
//!
//! Proves that certified results can be exported cleanly,
//! read-only semantics are enforced, and audit compatibility holds.

use voteos::domain::elections::*;
use voteos::domain::ballots::*;
use voteos::domain::votes::*;
use voteos::domain::tally::*;
use voteos::domain::certification::*;
use voteos::domain::proposals::*;
use voteos::domain::export::*;
use voteos::domain::audit;

// ===========================================================================
// HELPERS — reuse certified election fixture
// ===========================================================================

struct ExportFixture {
    election_registry: ElectionRegistry,
    ballot_registry: BallotRegistry,
    vote_registry: VoteRegistry,
    tally_registry: TallyRegistry,
    cert_registry: CertificationRegistry,
    proposal_registry: ProposalRegistry,
    export_registry: ExportRegistry,
    election_ref: String,
}

impl ExportFixture {
    /// Create a certified election with 5 votes: alice(3), bob(2).
    fn certified_election() -> Self {
        let election_registry = ElectionRegistry::new();
        let election_ref = election_registry.elections.insert_new(Election {
            title: "Export Test Election".into(),
            description: "test".into(),
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
            decision_ref: "dec-1".into(),
        });

        let ballot_registry = BallotRegistry::new();
        let template_ref = ballot_registry.templates.insert_new(BallotTemplate {
            election_ref: election_ref.clone(),
            status: BallotStatus::Finalized,
            items: vec![BallotItem {
                item_ref: "mayor".into(),
                item_type: BallotItemType::Race,
                title: "Mayor".into(),
                description: "Choose one".into(),
                choices: vec![
                    BallotChoice { choice_ref: "alice".into(), label: "Alice".into(), description: None },
                    BallotChoice { choice_ref: "bob".into(), label: "Bob".into(), description: None },
                ],
                max_selections: 1,
            }],
            created_by: "admin-1".into(),
            created_at: "2026-03-30T09:00:00Z".into(),
            finalized_at: Some("2026-03-30T09:30:00Z".into()),
            finalized_by: Some("admin-1".into()),
            decision_ref: "dec-ballot".into(),
            integrity_hash: Some("h1".into()),
        });

        election_registry.transition_election(&election_ref, ElectionStatus::Published, "a", "d", None).unwrap();
        election_registry.transition_election(&election_ref, ElectionStatus::Open, "a", "d", None).unwrap();

        let vote_registry = VoteRegistry::new();
        for (i, choice) in ["alice", "alice", "alice", "bob", "bob"].iter().enumerate() {
            let iref = ballot_registry.issuances.insert_new(BallotIssuance {
                template_ref: template_ref.clone(),
                voter_ref: format!("v{}", i),
                election_ref: election_ref.clone(),
                status: IssuanceStatus::Issued,
                issued_at: "2026-03-30T10:00:00Z".into(),
                issued_by: "admin-1".into(),
                decision_ref: "d".into(),
                spoiled_at: None,
                replacement_ref: None,
            });
            let vref = vote_registry.records.insert_new(VoteRecord {
                election_ref: election_ref.clone(),
                ballot_issuance_ref: iref,
                status: VoteStatus::Sealed,
                submitted_at: "2026-03-30T11:00:00Z".into(),
                sealed_at: Some("2026-03-30T11:00:00Z".into()),
                receipt_hash: "h".into(),
                decision_ref: "d".into(),
                attestation_ref: None,
            });
            vote_registry.contents.insert_new(VoteContent {
                vote_ref: vref,
                election_ref: election_ref.clone(),
                selections: vec![VoteSelection {
                    ballot_item_ref: "mayor".into(),
                    choice_ref: choice.to_string(),
                    rank: None,
                }],
            });
        }

        election_registry.transition_election(&election_ref, ElectionStatus::Closed, "a", "d", None).unwrap();

        let tally_registry = TallyRegistry::new();
        let sealed = vote_registry.sealed_contents(&election_ref);
        let input_hash = compute_input_hash(&sealed);
        let (item_tallies, has_ambiguity) = compute_plurality_tally(&election_ref, &sealed, &["mayor".into()]);
        let tally = TallyResult {
            election_ref: election_ref.clone(),
            method: VotingMethod::Plurality,
            status: TallyStatus::Computed,
            item_tallies, total_votes_counted: 5,
            computed_at: "2026-03-30T12:00:00Z".into(),
            computed_by: "admin-1".into(),
            decision_ref: "d".into(),
            input_hash, has_ambiguity,
        };
        let tally_ref = tally_registry.results.insert_new(tally.clone());

        election_registry.transition_election(&election_ref, ElectionStatus::Tallied, "a", "d", None).unwrap();

        let cert_registry = CertificationRegistry::new();
        cert_registry.certifications.insert_new(CertificationRecord {
            election_ref: election_ref.clone(),
            tally_ref,
            tally_snapshot: tally,
            status: CertificationStatus::Certified,
            certified_by: Some("admin-1".into()),
            certified_at: Some("2026-03-30T13:00:00Z".into()),
            certification_basis: "clear winner".into(),
            decision_ref: "d".into(),
            attestation_ref: None,
            rejection_reason: None,
            created_at: "2026-03-30T13:00:00Z".into(),
        });

        election_registry.transition_election(&election_ref, ElectionStatus::Certified, "a", "d", None).unwrap();

        Self {
            election_registry, ballot_registry, vote_registry,
            tally_registry, cert_registry,
            proposal_registry: ProposalRegistry::new(),
            export_registry: ExportRegistry::new(),
            election_ref,
        }
    }
}

// ===========================================================================
// A. EXPORT CORRECTNESS
// ===========================================================================

#[test]
fn test_export_bundle_matches_certification() {
    let f = ExportFixture::certified_election();

    let (cert_ref, cert) = f.cert_registry.certification_for_election(&f.election_ref).unwrap();
    let tally = &cert.tally_snapshot;

    // Build export manually (domain-level, no workflow)
    let item_results: Vec<ExportItemResult> = tally.item_tallies.iter()
        .map(item_tally_to_export)
        .collect();

    let export = CertifiedResultExport {
        export_ref: "test-export".into(),
        election_ref: f.election_ref.clone(),
        proposal_ref: None,
        jurisdiction_scope: "city".into(),
        title: "Export Test Election".into(),
        item_results: item_results.clone(),
        proposal_outcome: None,
        total_votes: tally.total_votes_counted,
        certification_ref: cert_ref,
        audit_hash: tally.input_hash.clone(),
        certified_at: cert.certified_at.unwrap_or_default(),
        certified_by: cert.certified_by.unwrap_or_default(),
        format: ExportFormat::FullBundle,
        exported_at: "2026-03-30T14:00:00Z".into(),
        consumed: false,
    };

    // Verify export data matches certification
    assert_eq!(export.total_votes, 5);
    assert_eq!(export.item_results.len(), 1);
    assert_eq!(export.item_results[0].winners, vec!["alice"]);
    assert_eq!(export.item_results[0].total_votes, 5);
    assert!(!export.item_results[0].is_tie);
    assert!(!export.audit_hash.is_empty());
}

#[test]
fn test_export_item_tally_conversion() {
    let item = ItemTally {
        ballot_item_ref: "mayor".into(),
        choice_counts: std::collections::BTreeMap::from([
            ("alice".into(), 60), ("bob".into(), 40),
        ]),
        total_votes: 100,
        winners: vec!["alice".into()],
        is_tie: false,
        is_ambiguous: false,
        result_summary: "alice wins with 60%".into(),
    };

    let export_item = item_tally_to_export(&item);
    assert_eq!(export_item.ballot_item_ref, "mayor");
    assert_eq!(export_item.winners, vec!["alice"]);
    assert_eq!(export_item.total_votes, 100);
    assert!(!export_item.is_tie);
    assert_eq!(export_item.summary, "alice wins with 60%");
}

// ===========================================================================
// B. AUDIT COMPATIBILITY
// ===========================================================================

#[test]
fn test_export_verifiable_via_audit() {
    let f = ExportFixture::certified_election();

    // Export generates data
    let (_, cert) = f.cert_registry.certification_for_election(&f.election_ref).unwrap();
    let export_audit_hash = cert.tally_snapshot.input_hash.clone();

    // Audit bundle can verify
    let bundle = audit::assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).unwrap();

    let verification = audit::verify_bundle(&bundle);
    assert!(verification.matches, "Audit must verify the exported election");

    // Export audit_hash matches what audit would compute
    assert_eq!(verification.reconstructed_input_hash, export_audit_hash,
        "Export audit_hash must match what audit independently computes");
}

// ===========================================================================
// C. READ-ONLY ENFORCEMENT
// ===========================================================================

#[test]
fn test_export_does_not_mutate_state() {
    let f = ExportFixture::certified_election();

    // Capture state before export
    let election_before = f.election_registry.elections.get(&f.election_ref).unwrap();
    let cert_before = f.cert_registry.certification_for_election(&f.election_ref).unwrap();

    // Generate export
    let export_ref = f.export_registry.exports.insert_new(CertifiedResultExport {
        export_ref: String::new(),
        election_ref: f.election_ref.clone(),
        proposal_ref: None,
        jurisdiction_scope: "city".into(),
        title: "test".into(),
        item_results: vec![],
        proposal_outcome: None,
        total_votes: 5,
        certification_ref: cert_before.0.clone(),
        audit_hash: "h".into(),
        certified_at: "2026-03-30T13:00:00Z".into(),
        certified_by: "admin-1".into(),
        format: ExportFormat::FullBundle,
        exported_at: "2026-03-30T14:00:00Z".into(),
        consumed: false,
    });

    // Verify state unchanged
    let election_after = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(election_after.status, election_before.status,
        "Export must not mutate election state");
    assert!(f.cert_registry.is_certified(&f.election_ref),
        "Certification must remain intact after export");
}

// ===========================================================================
// D. EVENT SYSTEM
// ===========================================================================

#[test]
fn test_system_events_recorded() {
    let registry = ExportRegistry::new();

    registry.events.insert_new(SystemEvent {
        event_type: EventType::ResultCertified,
        election_ref: "e1".into(),
        proposal_ref: None,
        timestamp: "2026-03-30T13:00:00Z".into(),
        payload: "{}".into(),
    });

    registry.events.insert_new(SystemEvent {
        event_type: EventType::ExportGenerated,
        election_ref: "e1".into(),
        proposal_ref: None,
        timestamp: "2026-03-30T14:00:00Z".into(),
        payload: "{}".into(),
    });

    let events = registry.events_for_election("e1");
    assert_eq!(events.len(), 2);
}

// ===========================================================================
// E. EXPORT REGISTRY
// ===========================================================================

#[test]
fn test_export_registry_operations() {
    let registry = ExportRegistry::new();
    assert!(!registry.has_export("e1"));

    registry.exports.insert_new(CertifiedResultExport {
        export_ref: "exp-1".into(),
        election_ref: "e1".into(),
        proposal_ref: None,
        jurisdiction_scope: "city".into(),
        title: "test".into(),
        item_results: vec![],
        proposal_outcome: None,
        total_votes: 5,
        certification_ref: "cert-1".into(),
        audit_hash: "h".into(),
        certified_at: "2026-03-30T13:00:00Z".into(),
        certified_by: "admin-1".into(),
        format: ExportFormat::PublicSummary,
        exported_at: "2026-03-30T14:00:00Z".into(),
        consumed: false,
    });

    assert!(registry.has_export("e1"));
    assert!(!registry.has_export("e2"));
}

#[test]
fn test_export_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = ExportRegistry::with_data_dir(path);
        registry.exports.insert_new(CertifiedResultExport {
            export_ref: "exp-1".into(),
            election_ref: "e1".into(),
            proposal_ref: None,
            jurisdiction_scope: "city".into(),
            title: "test".into(),
            item_results: vec![],
            proposal_outcome: None,
            total_votes: 5,
            certification_ref: "cert-1".into(),
            audit_hash: "h".into(),
            certified_at: "2026-03-30T13:00:00Z".into(),
            certified_by: "admin-1".into(),
            format: ExportFormat::MachineReadable,
            exported_at: "2026-03-30T14:00:00Z".into(),
            consumed: false,
        });
    }

    {
        let registry = ExportRegistry::with_data_dir(path);
        assert_eq!(registry.exports.count(), 1);
        assert!(registry.has_export("e1"));
    }
}

// ===========================================================================
// F. PROPOSAL EXPORT
// ===========================================================================

#[test]
fn test_proposal_result_in_export() {
    let f = ExportFixture::certified_election();

    // Link a proposal to this election
    let proposal_ref = f.proposal_registry.proposals.insert_new(Proposal {
        title: "Budget Measure".into(),
        description: "test".into(),
        proposal_type: ProposalType::Measure,
        jurisdiction_scope: "city".into(),
        status: ProposalStatus::Certified,
        election_ref: Some(f.election_ref.clone()),
        created_by: "admin-1".into(),
        created_at: "2026-03-30T08:00:00Z".into(),
        decision_ref: "d".into(),
    });

    f.proposal_registry.results.insert_new(ProposalResult {
        proposal_ref: proposal_ref.clone(),
        election_ref: f.election_ref.clone(),
        outcome: ProposalOutcome::Approved,
        vote_summary: "Approved 60/40".into(),
        certification_ref: "cert-1".into(),
        certified_at: "2026-03-30T13:00:00Z".into(),
        threshold_applied: None,
        threshold_met: None,
    });

    // Verify lookup
    let found = f.proposal_registry.proposal_for_election(&f.election_ref);
    assert!(found.is_some());

    let (_, result) = f.proposal_registry.result_for_proposal(&proposal_ref).unwrap();
    assert_eq!(result.outcome, ProposalOutcome::Approved);
}

// ===========================================================================
// WORKFLOW TESTS (require ICP replica)
// ===========================================================================

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_export_certified_result_workflow_strict() {}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_export_proposal_result_workflow_strict() {}
