//! Tests for Module 7: Governance Proposals
//!
//! Proves that proposals compose correctly with the election engine,
//! produce deterministic outcomes, and respect all trust guarantees.

use voteos::domain::voters::*;
use voteos::domain::elections::*;
use voteos::domain::ballots::*;
use voteos::domain::votes::*;
use voteos::domain::tally::*;
use voteos::domain::certification::*;
use voteos::domain::proposals::*;

// ===========================================================================
// HELPERS
// ===========================================================================

/// Create a complete proposal-driven election:
/// proposal → election → ballot (yes/no) → votes → close → tally → certify
struct ProposalFixture {
    election_registry: ElectionRegistry,
    ballot_registry: BallotRegistry,
    vote_registry: VoteRegistry,
    tally_registry: TallyRegistry,
    cert_registry: CertificationRegistry,
    proposal_registry: ProposalRegistry,
    proposal_ref: String,
    election_ref: String,
}

impl ProposalFixture {
    fn new(title: &str, proposal_type: ProposalType) -> Self {
        let election_registry = ElectionRegistry::new();
        let proposal_registry = ProposalRegistry::new();

        // Create proposal
        let proposal_ref = proposal_registry.proposals.insert_new(Proposal {
            title: title.into(),
            description: format!("Test proposal: {}", title),
            proposal_type,
            jurisdiction_scope: "city".into(),
            status: ProposalStatus::Draft,
            election_ref: None,
            created_by: "admin-1".into(),
            created_at: "2026-03-30T08:00:00Z".into(),
            decision_ref: "dec-prop".into(),
        });

        // Create election linked to proposal (Referendum type for measure)
        let election_ref = election_registry.elections.insert_new(Election {
            title: format!("Election for: {}", title),
            description: "Proposal election".into(),
            election_type: ElectionType::Referendum,
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
            decision_ref: "dec-elec".into(),
        });

        // Link proposal to election
        let mut proposal = proposal_registry.proposals.get(&proposal_ref).unwrap();
        proposal.election_ref = Some(election_ref.clone());
        proposal.status = ProposalStatus::Published;
        proposal_registry.proposals.update(&proposal_ref, proposal);

        Self {
            election_registry,
            ballot_registry: BallotRegistry::new(),
            vote_registry: VoteRegistry::new(),
            tally_registry: TallyRegistry::new(),
            cert_registry: CertificationRegistry::new(),
            proposal_registry,
            proposal_ref,
            election_ref,
        }
    }

    fn create_yes_no_ballot(&self) -> String {
        let template_ref = self.ballot_registry.templates.insert_new(BallotTemplate {
            election_ref: self.election_ref.clone(),
            status: BallotStatus::Finalized,
            items: vec![BallotItem {
                item_ref: "proposal-question".into(),
                item_type: BallotItemType::Referendum,
                title: "Approve proposal?".into(),
                description: "Vote yes or no".into(),
                choices: vec![
                    BallotChoice { choice_ref: "yes".into(), label: "Yes".into(), description: None },
                    BallotChoice { choice_ref: "no".into(), label: "No".into(), description: None },
                ],
                max_selections: 1,
            }],
            created_by: "admin-1".into(),
            created_at: "2026-03-30T09:00:00Z".into(),
            finalized_at: Some("2026-03-30T09:30:00Z".into()),
            finalized_by: Some("admin-1".into()),
            decision_ref: "dec-ballot".into(),
            integrity_hash: Some("hash-1".into()),
        });
        template_ref
    }

    fn open_election(&self) {
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Published, "a", "d", None).unwrap();
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Open, "a", "d", None).unwrap();
    }

    fn cast_vote(&self, voter: &str, template_ref: &str, choice: &str) {
        let iref = self.ballot_registry.issuances.insert_new(BallotIssuance {
            template_ref: template_ref.into(),
            voter_ref: voter.into(),
            election_ref: self.election_ref.clone(),
            status: IssuanceStatus::Issued,
            issued_at: "2026-03-30T10:00:00Z".into(),
            issued_by: "admin-1".into(),
            decision_ref: "dec-iss".into(),
            spoiled_at: None,
            replacement_ref: None,
        });

        let vote_ref = self.vote_registry.records.insert_new(VoteRecord {
            election_ref: self.election_ref.clone(),
            ballot_issuance_ref: iref,
            status: VoteStatus::Sealed,
            submitted_at: "2026-03-30T11:00:00Z".into(),
            sealed_at: Some("2026-03-30T11:00:00Z".into()),
            receipt_hash: "hash".into(),
            decision_ref: "dec-vote".into(),
            attestation_ref: None,
        });

        self.vote_registry.contents.insert_new(VoteContent {
            vote_ref,
            election_ref: self.election_ref.clone(),
            selections: vec![VoteSelection {
                ballot_item_ref: "proposal-question".into(),
                choice_ref: choice.into(),
                rank: None,
            }],
        });
    }

    fn close_tally_certify(&self) -> (TallyResult, String) {
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Closed, "a", "d", None).unwrap();

        let sealed = self.vote_registry.sealed_contents(&self.election_ref);
        let input_hash = compute_input_hash(&sealed);
        let items = vec!["proposal-question".to_string()];
        let (item_tallies, has_ambiguity) = compute_plurality_tally(&self.election_ref, &sealed, &items);

        let tally = TallyResult {
            election_ref: self.election_ref.clone(),
            method: VotingMethod::Plurality,
            status: if has_ambiguity { TallyStatus::Ambiguous } else { TallyStatus::Computed },
            item_tallies,
            total_votes_counted: sealed.len() as u64,
            computed_at: "2026-03-30T12:00:00Z".into(),
            computed_by: "admin-1".into(),
            decision_ref: "dec-tally".into(),
            input_hash,
            has_ambiguity,
        };
        let tally_ref = self.tally_registry.results.insert_new(tally.clone());

        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Tallied, "a", "d", None).unwrap();

        let cert_ref = self.cert_registry.certifications.insert_new(CertificationRecord {
            election_ref: self.election_ref.clone(),
            tally_ref: tally_ref.clone(),
            tally_snapshot: tally.clone(),
            status: CertificationStatus::Certified,
            certified_by: Some("admin-1".into()),
            certified_at: Some("2026-03-30T13:00:00Z".into()),
            certification_basis: "Proposal vote completed".into(),
            decision_ref: "dec-certify".into(),
            attestation_ref: None,
            rejection_reason: None,
            created_at: "2026-03-30T13:00:00Z".into(),
        });

        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Certified, "a", "d", None).unwrap();

        (tally, cert_ref)
    }
}

// ===========================================================================
// A. PROPOSAL LIFECYCLE — APPROVED
// ===========================================================================

#[test]
fn test_proposal_lifecycle_approved() {
    let f = ProposalFixture::new("Park Funding Measure", ProposalType::Measure);
    let tmpl = f.create_yes_no_ballot();
    f.open_election();

    // 7 yes, 3 no → 70% approval
    for i in 1..=7 { f.cast_vote(&format!("v{}", i), &tmpl, "yes"); }
    for i in 8..=10 { f.cast_vote(&format!("v{}", i), &tmpl, "no"); }

    let (tally, _) = f.close_tally_certify();

    // Determine outcome
    let yes = tally.item_tallies[0].choice_counts.get("yes").copied().unwrap_or(0);
    let no = tally.item_tallies[0].choice_counts.get("no").copied().unwrap_or(0);
    let (outcome, summary) = determine_proposal_outcome(yes, no, tally.total_votes_counted, None);

    assert_eq!(outcome, ProposalOutcome::Approved);
    assert!(summary.contains("Approved"));
    assert_eq!(yes, 7);
    assert_eq!(no, 3);
}

// ===========================================================================
// B. PROPOSAL LIFECYCLE — REJECTED
// ===========================================================================

#[test]
fn test_proposal_lifecycle_rejected() {
    let f = ProposalFixture::new("Tax Increase", ProposalType::Policy);
    let tmpl = f.create_yes_no_ballot();
    f.open_election();

    // 3 yes, 7 no → rejected
    for i in 1..=3 { f.cast_vote(&format!("v{}", i), &tmpl, "yes"); }
    for i in 4..=10 { f.cast_vote(&format!("v{}", i), &tmpl, "no"); }

    let (tally, _) = f.close_tally_certify();

    let yes = tally.item_tallies[0].choice_counts.get("yes").copied().unwrap_or(0);
    let no = tally.item_tallies[0].choice_counts.get("no").copied().unwrap_or(0);
    let (outcome, _) = determine_proposal_outcome(yes, no, tally.total_votes_counted, None);

    assert_eq!(outcome, ProposalOutcome::Rejected);
}

// ===========================================================================
// C. THRESHOLD ENFORCEMENT
// ===========================================================================

#[test]
fn test_proposal_threshold_met() {
    // 60% threshold, 65% yes → Approved
    let (outcome, _) = determine_proposal_outcome(65, 35, 100, Some(60.0));
    assert_eq!(outcome, ProposalOutcome::Approved);
}

#[test]
fn test_proposal_threshold_not_met() {
    // 60% threshold, 55% yes → Rejected (below threshold)
    let (outcome, summary) = determine_proposal_outcome(55, 45, 100, Some(60.0));
    assert_eq!(outcome, ProposalOutcome::Rejected);
    assert!(summary.contains("threshold"));
}

#[test]
fn test_proposal_threshold_exact() {
    // 60% threshold, exactly 60% → Approved
    let (outcome, _) = determine_proposal_outcome(60, 40, 100, Some(60.0));
    assert_eq!(outcome, ProposalOutcome::Approved);
}

// ===========================================================================
// D. AMBIGUITY — TIE PRODUCES AMBIGUOUS OUTCOME
// ===========================================================================

#[test]
fn test_proposal_tie_is_ambiguous() {
    let f = ProposalFixture::new("Controversial Measure", ProposalType::Measure);
    let tmpl = f.create_yes_no_ballot();
    f.open_election();

    // 5 yes, 5 no → tie
    for i in 1..=5 { f.cast_vote(&format!("v{}", i), &tmpl, "yes"); }
    for i in 6..=10 { f.cast_vote(&format!("v{}", i), &tmpl, "no"); }

    let (tally, _) = f.close_tally_certify();

    let yes = tally.item_tallies[0].choice_counts.get("yes").copied().unwrap_or(0);
    let no = tally.item_tallies[0].choice_counts.get("no").copied().unwrap_or(0);
    let (outcome, _) = determine_proposal_outcome(yes, no, tally.total_votes_counted, None);

    assert_eq!(outcome, ProposalOutcome::Ambiguous);
}

#[test]
fn test_proposal_no_votes_is_ambiguous() {
    let (outcome, _) = determine_proposal_outcome(0, 0, 0, None);
    assert_eq!(outcome, ProposalOutcome::Ambiguous);
}

// ===========================================================================
// E. PROPOSAL LINKED TO ELECTION
// ===========================================================================

#[test]
fn test_proposal_election_linkage() {
    let f = ProposalFixture::new("Zoning Change", ProposalType::Referendum);

    // Verify proposal is linked to election
    let proposal = f.proposal_registry.proposals.get(&f.proposal_ref).unwrap();
    assert_eq!(proposal.election_ref.as_deref(), Some(f.election_ref.as_str()));
    assert_eq!(proposal.status, ProposalStatus::Published);

    // Verify lookup works
    let found = f.proposal_registry.proposal_for_election(&f.election_ref);
    assert!(found.is_some());
    assert_eq!(found.unwrap().0, f.proposal_ref);
}

#[test]
fn test_proposal_result_stored() {
    let f = ProposalFixture::new("Budget Amendment", ProposalType::Measure);
    let tmpl = f.create_yes_no_ballot();
    f.open_election();
    for i in 1..=8 { f.cast_vote(&format!("v{}", i), &tmpl, "yes"); }
    for i in 9..=10 { f.cast_vote(&format!("v{}", i), &tmpl, "no"); }
    let (tally, _) = f.close_tally_certify();

    let yes = tally.item_tallies[0].choice_counts.get("yes").copied().unwrap_or(0);
    let no = tally.item_tallies[0].choice_counts.get("no").copied().unwrap_or(0);
    let (outcome, vote_summary) = determine_proposal_outcome(yes, no, tally.total_votes_counted, None);

    let (cert_ref, _) = f.cert_registry.certification_for_election(&f.election_ref).unwrap();

    let result_ref = f.proposal_registry.results.insert_new(ProposalResult {
        proposal_ref: f.proposal_ref.clone(),
        election_ref: f.election_ref.clone(),
        outcome: outcome.clone(),
        vote_summary: vote_summary.clone(),
        certification_ref: cert_ref,
        certified_at: "2026-03-30T13:00:00Z".into(),
        threshold_applied: None,
        threshold_met: None,
    });

    // Verify retrieval
    let (_, result) = f.proposal_registry.result_for_proposal(&f.proposal_ref).unwrap();
    assert_eq!(result.outcome, ProposalOutcome::Approved);
    assert!(result.vote_summary.contains("Approved"));
}

// ===========================================================================
// F. DETERMINISM
// ===========================================================================

#[test]
fn test_proposal_outcome_deterministic() {
    // Same inputs must always produce same outcome
    let (o1, s1) = determine_proposal_outcome(65, 35, 100, Some(60.0));
    let (o2, s2) = determine_proposal_outcome(65, 35, 100, Some(60.0));
    assert_eq!(o1, o2);
    assert_eq!(s1, s2);
}

// ===========================================================================
// G. AUDIT COMPATIBILITY
// ===========================================================================

#[test]
fn test_proposal_audit_compatible() {
    let f = ProposalFixture::new("Audit Test Proposal", ProposalType::Measure);
    let tmpl = f.create_yes_no_ballot();
    f.open_election();
    for i in 1..=6 { f.cast_vote(&format!("v{}", i), &tmpl, "yes"); }
    for i in 7..=10 { f.cast_vote(&format!("v{}", i), &tmpl, "no"); }
    let (_, _) = f.close_tally_certify();

    // Audit bundle can be assembled for proposal election
    use voteos::domain::audit::*;
    let bundle = assemble_audit_bundle(
        &f.election_ref,
        &f.election_registry,
        &f.ballot_registry,
        &f.vote_registry,
        &f.tally_registry,
        &f.cert_registry,
    ).expect("Bundle must assemble for proposal election");

    let verification = verify_bundle(&bundle);
    assert!(verification.matches, "Proposal election must be auditable");
    assert!(verify_secrecy(&bundle), "Secrecy must hold for proposal votes");
}

// ===========================================================================
// H. PERSISTENCE
// ===========================================================================

#[test]
fn test_proposal_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = ProposalRegistry::with_data_dir(path);
        registry.proposals.insert_new(Proposal {
            title: "Persist Test".into(),
            description: "test".into(),
            proposal_type: ProposalType::Measure,
            jurisdiction_scope: "city".into(),
            status: ProposalStatus::Draft,
            election_ref: None,
            created_by: "admin-1".into(),
            created_at: "2026-03-30T08:00:00Z".into(),
            decision_ref: "dec-1".into(),
        });
    }

    {
        let registry = ProposalRegistry::with_data_dir(path);
        assert_eq!(registry.proposals.count(), 1);
    }
}

// ===========================================================================
// WORKFLOW TESTS (require ICP replica)
// ===========================================================================

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_create_proposal_workflow_strict() {}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_certify_proposal_result_workflow_strict() {}
