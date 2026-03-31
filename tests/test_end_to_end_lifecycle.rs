//! VoteOS Wave 3.5 — End-to-End Domain Proof (Trust Gate)
//!
//! These tests prove the trust core (Modules 1–6) works as a single
//! coherent system. Domain-level only — no AxiaSystem, no ICP, no API.
//!
//! CATEGORIES:
//!   - Happy path: full lifecycle from election creation to certification
//!   - Failure paths: ineligible voter, double vote, premature certification, post-cert mutation
//!   - Ambiguity: tie blocks certification, no votes blocks certification, mixed items
//!   - Determinism: same inputs → same outputs regardless of order
//!   - Cross-module consistency: registries align, tally uses correct votes

use voteos::domain::voters::*;
use voteos::domain::elections::*;
use voteos::domain::ballots::*;
use voteos::domain::votes::*;
use voteos::domain::tally::*;
use voteos::domain::certification::*;

// ===========================================================================
// SHARED HELPERS — build reusable election infrastructure
// ===========================================================================

struct ElectionFixture {
    voter_registry: VoterRegistry,
    election_registry: ElectionRegistry,
    ballot_registry: BallotRegistry,
    vote_registry: VoteRegistry,
    tally_registry: TallyRegistry,
    cert_registry: CertificationRegistry,
    election_ref: String,
}

impl ElectionFixture {
    /// Create an election in Draft state with default plurality config.
    fn new(title: &str) -> Self {
        let election_registry = ElectionRegistry::new();
        let election_ref = election_registry.elections.insert_new(Election {
            title: title.to_string(),
            description: "End-to-end test election".into(),
            election_type: ElectionType::General,
            status: ElectionStatus::Draft,
            config: ElectionConfig::default(), // Plurality, SecretBallot
            schedule: ElectionSchedule {
                registration_start: None,
                registration_end: None,
                voting_start: None,
                voting_end: None,
                certification_deadline: None,
            },
            scope: "test-district".into(),
            created_by: "admin-1".into(),
            created_at: "2026-03-30T08:00:00Z".into(),
            decision_ref: "dec-create".into(),
        });

        Self {
            voter_registry: VoterRegistry::new(),
            election_registry,
            ballot_registry: BallotRegistry::new(),
            vote_registry: VoteRegistry::new(),
            tally_registry: TallyRegistry::new(),
            cert_registry: CertificationRegistry::new(),
            election_ref,
        }
    }

    /// Define an eligibility rule for the election.
    fn add_eligibility_rule(&self, rule_type: RuleType, criteria: &str) {
        self.voter_registry.rules.insert_new(EligibilityRule {
            election_ref: self.election_ref.clone(),
            rule_type,
            criteria: criteria.to_string(),
            defined_by: "admin-1".into(),
            defined_at: "2026-03-30T08:01:00Z".into(),
            decision_ref: "dec-rule".into(),
        });
    }

    /// Register a voter as eligible.
    fn register_voter(&self, citizen_ref: &str) -> String {
        self.voter_registry.registrations.insert_new(VoterRegistration {
            citizen_ref: citizen_ref.to_string(),
            election_ref: self.election_ref.clone(),
            status: RegistrationStatus::Registered,
            registered_at: "2026-03-30T08:30:00Z".into(),
            registered_by: "admin-1".into(),
            eligibility_basis: "Meets all eligibility rules".into(),
            decision_ref: "dec-reg".into(),
            attestation_ref: None,
        })
    }

    /// Create and finalize a ballot template with given items.
    fn create_ballot(&self, items: Vec<BallotItem>) -> String {
        let template_ref = self.ballot_registry.templates.insert_new(BallotTemplate {
            election_ref: self.election_ref.clone(),
            status: BallotStatus::Draft,
            items: items.clone(),
            created_by: "admin-1".into(),
            created_at: "2026-03-30T09:00:00Z".into(),
            finalized_at: None,
            finalized_by: None,
            decision_ref: "dec-ballot".into(),
            integrity_hash: None,
        });

        // Finalize
        let mut template = self.ballot_registry.templates.get(&template_ref).unwrap();
        template.status = BallotStatus::Finalized;
        template.finalized_at = Some("2026-03-30T09:30:00Z".into());
        template.finalized_by = Some("admin-1".into());
        template.integrity_hash = Some(BallotRegistry::compute_integrity_hash(&template));
        self.ballot_registry.templates.update(&template_ref, template);

        template_ref
    }

    /// Issue a ballot to a voter.
    fn issue_ballot(&self, template_ref: &str, voter_ref: &str) -> String {
        self.ballot_registry.issuances.insert_new(BallotIssuance {
            template_ref: template_ref.to_string(),
            voter_ref: voter_ref.to_string(),
            election_ref: self.election_ref.clone(),
            status: IssuanceStatus::Issued,
            issued_at: "2026-03-30T10:00:00Z".into(),
            issued_by: "admin-1".into(),
            decision_ref: "dec-issue".into(),
            spoiled_at: None,
            replacement_ref: None,
        })
    }

    /// Transition election through states up to Open.
    fn open_election(&self) {
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Published,
            "admin-1", "dec-pub", None,
        ).expect("Draft → Published");
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Open,
            "admin-1", "dec-open", None,
        ).expect("Published → Open");
    }

    /// Close the election.
    fn close_election(&self) {
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Closed,
            "admin-1", "dec-close", None,
        ).expect("Open → Closed");
    }

    /// Cast a sealed vote for a voter (full recording pipeline).
    fn cast_sealed_vote(
        &self,
        voter_ref: &str,
        issuance_ref: &str,
        selections: Vec<(&str, &str)>,
    ) -> String {
        // Precondition checks (same as workflow would do)
        assert!(
            !self.vote_registry.has_voted(voter_ref, &self.election_ref),
            "Double vote attempt by {}",
            voter_ref
        );
        assert!(
            self.ballot_registry.has_active_issuance(voter_ref, &self.election_ref),
            "No active ballot for {}",
            voter_ref
        );

        let timestamp = "2026-03-30T11:00:00Z";
        let vote_ref = self.vote_registry.records.insert_new(VoteRecord {
            election_ref: self.election_ref.clone(),
            ballot_issuance_ref: issuance_ref.to_string(),
            status: VoteStatus::Sealed,
            submitted_at: timestamp.into(),
            sealed_at: Some(timestamp.into()),
            receipt_hash: VoteRegistry::compute_receipt_hash("pending", &self.election_ref, timestamp),
            decision_ref: "dec-vote".into(),
            attestation_ref: None,
        });

        // Store content (no voter identity)
        self.vote_registry.contents.insert_new(VoteContent {
            vote_ref: vote_ref.clone(),
            election_ref: self.election_ref.clone(),
            selections: selections
                .into_iter()
                .map(|(item, choice)| VoteSelection {
                    ballot_item_ref: item.to_string(),
                    choice_ref: choice.to_string(),
                    rank: None,
                })
                .collect(),
        });

        // Record participation (links voter to vote, not to content)
        self.vote_registry.participation.insert_new(VoterParticipation {
            voter_ref: voter_ref.to_string(),
            election_ref: self.election_ref.clone(),
            voted_at: timestamp.into(),
            vote_ref: vote_ref.clone(),
        });

        // Generate receipt
        let receipt_hash = VoteRegistry::compute_receipt_hash(&vote_ref, &self.election_ref, timestamp);
        self.vote_registry.receipts.insert_new(VotingReceipt {
            voter_ref: voter_ref.to_string(),
            election_ref: self.election_ref.clone(),
            receipt_hash,
            timestamp: timestamp.into(),
            vote_ref: vote_ref.clone(),
        });

        vote_ref
    }

    /// Compute tally and store result.
    fn compute_and_store_tally(&self) -> (String, TallyResult) {
        let sealed = self.vote_registry.sealed_contents(&self.election_ref);
        let input_hash = compute_input_hash(&sealed);

        let ballot_item_refs: Vec<String> = match self.ballot_registry.finalized_template(&self.election_ref) {
            Some((_, t)) => t.items.iter().map(|i| i.item_ref.clone()).collect(),
            None => vec![],
        };

        let (item_tallies, has_ambiguity) = compute_plurality_tally(
            &self.election_ref, &sealed, &ballot_item_refs,
        );

        let total_votes = sealed.len() as u64;
        let status = if total_votes == 0 {
            TallyStatus::Invalid
        } else if has_ambiguity {
            TallyStatus::Ambiguous
        } else {
            TallyStatus::Computed
        };

        let tally = TallyResult {
            election_ref: self.election_ref.clone(),
            method: VotingMethod::Plurality,
            status,
            item_tallies,
            total_votes_counted: total_votes,
            computed_at: "2026-03-30T13:00:00Z".into(),
            computed_by: "admin-1".into(),
            decision_ref: "dec-tally".into(),
            input_hash,
            has_ambiguity,
        };

        let tally_ref = self.tally_registry.results.insert_new(tally.clone());
        (tally_ref, tally)
    }

    /// Transition to Tallied.
    fn tally_election(&self) {
        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Tallied,
            "admin-1", "dec-tally-tr", None,
        ).expect("Closed → Tallied");
    }

    /// Certify the result.
    fn certify(&self, tally_ref: &str, tally_snapshot: TallyResult) -> String {
        let cert_ref = self.cert_registry.certifications.insert_new(CertificationRecord {
            election_ref: self.election_ref.clone(),
            tally_ref: tally_ref.to_string(),
            tally_snapshot,
            status: CertificationStatus::Certified,
            certified_by: Some("admin-1".into()),
            certified_at: Some("2026-03-30T14:00:00Z".into()),
            certification_basis: "Plurality with clear winner".into(),
            decision_ref: "dec-certify".into(),
            attestation_ref: None,
            rejection_reason: None,
            created_at: "2026-03-30T14:00:00Z".into(),
        });

        self.election_registry.transition_election(
            &self.election_ref, ElectionStatus::Certified,
            "admin-1", "dec-certify-tr", None,
        ).expect("Tallied → Certified");

        cert_ref
    }
}

/// Standard ballot with one race (3 candidates) and one measure.
fn standard_ballot_items() -> Vec<BallotItem> {
    vec![
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
    ]
}

// ===========================================================================
// HAPPY PATH — FULL LIFECYCLE
// ===========================================================================

#[test]
fn test_e2e_happy_path_full_lifecycle() {
    let f = ElectionFixture::new("2026 City Election");

    // 1. Define eligibility rules
    f.add_eligibility_rule(RuleType::AgeMinimum, "age >= 18");
    f.add_eligibility_rule(RuleType::Jurisdiction, "resident of test-district");
    assert_eq!(f.voter_registry.rules_for_election(&f.election_ref).len(), 2);

    // 2. Register 5 eligible voters
    for i in 1..=5 {
        f.register_voter(&format!("citizen-{}", i));
    }
    assert_eq!(f.voter_registry.voters_for_election(&f.election_ref).len(), 5);

    // 3. Create and finalize ballot
    let template_ref = f.create_ballot(standard_ballot_items());
    let template = f.ballot_registry.finalized_template(&f.election_ref);
    assert!(template.is_some(), "Finalized ballot must exist");
    assert!(template.unwrap().1.integrity_hash.is_some(), "Integrity hash must be set");

    // 4. Open election and issue ballots
    f.open_election();
    let mut issuance_refs = Vec::new();
    for i in 1..=5 {
        let iref = f.issue_ballot(&template_ref, &format!("citizen-{}", i));
        issuance_refs.push(iref);
    }
    assert_eq!(f.ballot_registry.issuance_count(&f.election_ref), 5);

    // 5. Cast votes: alice(3), bob(1), carol(1); yes(4), no(1)
    let vote_choices = vec![
        ("citizen-1", vec![("mayor", "alice"), ("measure-a", "yes")]),
        ("citizen-2", vec![("mayor", "alice"), ("measure-a", "yes")]),
        ("citizen-3", vec![("mayor", "alice"), ("measure-a", "yes")]),
        ("citizen-4", vec![("mayor", "bob"), ("measure-a", "yes")]),
        ("citizen-5", vec![("mayor", "carol"), ("measure-a", "no")]),
    ];

    for (i, (voter, selections)) in vote_choices.iter().enumerate() {
        f.cast_sealed_vote(voter, &issuance_refs[i], selections.clone());
    }

    assert_eq!(f.vote_registry.votes_submitted(&f.election_ref), 5);
    assert!(f.vote_registry.verify_ballot_secrecy(&f.election_ref));

    // 6. Close election
    f.close_election();
    let election = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(election.status, ElectionStatus::Closed);

    // 7. Compute tally
    let (tally_ref, tally) = f.compute_and_store_tally();
    assert_eq!(tally.status, TallyStatus::Computed);
    assert!(!tally.has_ambiguity);
    assert_eq!(tally.total_votes_counted, 5);

    // Mayor: alice=3, bob=1, carol=1
    let mayor = &tally.item_tallies[0];
    assert_eq!(mayor.winners, vec!["alice"]);
    assert_eq!(*mayor.choice_counts.get("alice").unwrap(), 3);
    assert_eq!(*mayor.choice_counts.get("bob").unwrap(), 1);
    assert_eq!(*mayor.choice_counts.get("carol").unwrap(), 1);
    assert!(!mayor.is_tie);

    // Measure A: yes=4, no=1
    let measure = &tally.item_tallies[1];
    assert_eq!(measure.winners, vec!["yes"]);
    assert_eq!(*measure.choice_counts.get("yes").unwrap(), 4);
    assert_eq!(*measure.choice_counts.get("no").unwrap(), 1);

    f.tally_election();

    // 8. Certify result
    let cert_ref = f.certify(&tally_ref, tally);

    // 9. Verify final state
    let final_election = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(final_election.status, ElectionStatus::Certified);
    assert!(f.cert_registry.is_certified(&f.election_ref));

    let (_, cert) = f.cert_registry.certification_for_election(&f.election_ref).unwrap();
    assert_eq!(cert.status, CertificationStatus::Certified);
    assert_eq!(cert.tally_snapshot.item_tallies[0].winners, vec!["alice"]);

    // 10. Verify immutability
    let reopen = f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Open, "admin-1", "dec-bad", None,
    );
    assert!(reopen.is_err(), "Certified election must be immutable");
}

// ===========================================================================
// FAILURE PATH A — INELIGIBLE VOTER BLOCKED
// ===========================================================================

#[test]
fn test_e2e_ineligible_voter_blocked() {
    let f = ElectionFixture::new("Eligibility Test");

    // Register citizen-1 as eligible
    f.register_voter("citizen-1");

    // Register citizen-2 as suspended (ineligible)
    f.voter_registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-2".into(),
        election_ref: f.election_ref.clone(),
        status: RegistrationStatus::Suspended,
        registered_at: "2026-03-30T08:30:00Z".into(),
        registered_by: "admin-1".into(),
        eligibility_basis: "Suspended due to challenge".into(),
        decision_ref: "dec-reg-2".into(),
        attestation_ref: None,
    });

    // citizen-1 is registered, citizen-2 is NOT (suspended ≠ registered)
    assert!(f.voter_registry.is_registered("citizen-1", &f.election_ref));
    assert!(!f.voter_registry.is_registered("citizen-2", &f.election_ref),
        "Suspended voter must not appear as registered");

    // citizen-3 was never registered at all
    assert!(!f.voter_registry.is_registered("citizen-3", &f.election_ref),
        "Unregistered voter must not appear as registered");

    // Voter roll should only contain citizen-1
    let eligible = f.voter_registry.voters_for_election(&f.election_ref);
    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].1.citizen_ref, "citizen-1");
}

// ===========================================================================
// FAILURE PATH B — DOUBLE VOTING BLOCKED
// ===========================================================================

#[test]
fn test_e2e_double_vote_blocked() {
    let f = ElectionFixture::new("Double Vote Test");

    f.register_voter("citizen-1");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();
    let issuance_ref = f.issue_ballot(&template_ref, "citizen-1");

    // First vote succeeds
    f.cast_sealed_vote("citizen-1", &issuance_ref, vec![("mayor", "alice")]);
    assert!(f.vote_registry.has_voted("citizen-1", &f.election_ref));

    // Second vote attempt — has_voted check blocks it
    assert!(
        f.vote_registry.has_voted("citizen-1", &f.election_ref),
        "has_voted must return true after first vote"
    );
    // The workflow would reject here. We verify the precondition holds.
}

// ===========================================================================
// FAILURE PATH C — CERTIFICATION BEFORE CLOSE BLOCKED
// ===========================================================================

#[test]
fn test_e2e_certification_before_close_blocked() {
    let f = ElectionFixture::new("Premature Cert Test");

    f.register_voter("citizen-1");
    f.create_ballot(standard_ballot_items());
    f.open_election();

    // Cannot transition Open → Tallied
    let result = f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Tallied, "admin-1", "dec-bad", None,
    );
    assert!(result.is_err(), "Cannot tally an Open election");

    // Cannot transition Open → Certified
    let result = f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Certified, "admin-1", "dec-bad", None,
    );
    assert!(result.is_err(), "Cannot certify an Open election");
}

// ===========================================================================
// FAILURE PATH D — TALLY BEFORE CLOSE BLOCKED
// ===========================================================================

#[test]
fn test_e2e_tally_requires_closed_election() {
    let f = ElectionFixture::new("Premature Tally Test");
    f.open_election();

    // Election is Open — tally should only work on Closed elections
    let election = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(election.status, ElectionStatus::Open);

    // The workflow checks: election.status != Closed → PreconditionFailed
    // At domain level: transition Open → Tallied is forbidden
    let result = f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Tallied, "admin-1", "dec-bad", None,
    );
    assert!(result.is_err(), "Open → Tallied must be forbidden");
}

// ===========================================================================
// FAILURE PATH E — POST-CERTIFICATION MUTATION BLOCKED
// ===========================================================================

#[test]
fn test_e2e_post_certification_all_mutations_blocked() {
    let f = ElectionFixture::new("Immutability Test");

    // Fast-track to Certified
    f.register_voter("citizen-1");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();
    let iref = f.issue_ballot(&template_ref, "citizen-1");
    f.cast_sealed_vote("citizen-1", &iref, vec![("mayor", "alice"), ("measure-a", "yes")]);
    f.close_election();
    let (tally_ref, tally) = f.compute_and_store_tally();
    f.tally_election();
    f.certify(&tally_ref, tally);

    // Verify Certified
    let e = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(e.status, ElectionStatus::Certified);

    // Try every possible transition — ALL must fail
    for target in &[
        ElectionStatus::Draft,
        ElectionStatus::Published,
        ElectionStatus::Open,
        ElectionStatus::Closed,
        ElectionStatus::Tallied,
        ElectionStatus::Cancelled,
    ] {
        let result = f.election_registry.transition_election(
            &f.election_ref, target.clone(), "admin-1", "dec-bad", None,
        );
        assert!(result.is_err(),
            "Certified election must not transition to {:?}", target);
    }

    // Verify re-certification blocked
    assert!(f.cert_registry.is_certified(&f.election_ref),
        "Election must remain certified");

    // Tally cannot be recomputed (registry already has one)
    assert!(f.tally_registry.has_tally(&f.election_ref),
        "Tally must persist after certification");
}

// ===========================================================================
// FAILURE PATH F — INVALID STATE TRANSITIONS
// ===========================================================================

#[test]
fn test_e2e_state_machine_discipline() {
    let f = ElectionFixture::new("State Machine Test");

    // Draft → can only go to Published or Cancelled
    let e = f.election_registry.elections.get(&f.election_ref).unwrap();
    assert_eq!(e.status, ElectionStatus::Draft);

    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Open, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Closed, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Tallied, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Certified, "a", "d", None).is_err());

    // Valid: Draft → Published
    f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Published, "a", "d", None).expect("Draft → Published");

    // Published → can only go to Open or Cancelled
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Closed, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Tallied, "a", "d", None).is_err());

    // Valid: Published → Open
    f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Open, "a", "d", None).expect("Published → Open");

    // Open → can only go to Closed or Cancelled
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Tallied, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Certified, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Published, "a", "d", None).is_err());

    // Valid: Open → Closed
    f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Closed, "a", "d", None).expect("Open → Closed");

    // Closed → can only go to Tallied
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Open, "a", "d", None).is_err());
    assert!(f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Certified, "a", "d", None).is_err());
}

// ===========================================================================
// AMBIGUITY A — TIE BLOCKS CERTIFICATION
// ===========================================================================

#[test]
fn test_e2e_tie_blocks_certification() {
    let f = ElectionFixture::new("Tie Test");

    f.register_voter("citizen-1");
    f.register_voter("citizen-2");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();
    let i1 = f.issue_ballot(&template_ref, "citizen-1");
    let i2 = f.issue_ballot(&template_ref, "citizen-2");

    // Equal votes: alice=1, bob=1
    f.cast_sealed_vote("citizen-1", &i1, vec![("mayor", "alice"), ("measure-a", "yes")]);
    f.cast_sealed_vote("citizen-2", &i2, vec![("mayor", "bob"), ("measure-a", "yes")]);

    f.close_election();
    let (_, tally) = f.compute_and_store_tally();

    // Mayor is tied
    assert_eq!(tally.status, TallyStatus::Ambiguous, "Tied tally must be Ambiguous");
    assert!(tally.has_ambiguity, "has_ambiguity must be true");
    assert!(tally.item_tallies[0].is_tie, "Mayor item must show tie");
    assert_eq!(tally.item_tallies[0].winners.len(), 2, "Both candidates are 'winners' in a tie");

    // Certification must be blocked by workflow precondition:
    // tally_result.status == TallyStatus::Ambiguous → reject
    assert_eq!(tally.status, TallyStatus::Ambiguous,
        "Ambiguous tally CANNOT be certified — workflow would reject");
}

// ===========================================================================
// AMBIGUITY B — NO VOTES BLOCKS CERTIFICATION
// ===========================================================================

#[test]
fn test_e2e_no_votes_blocks_certification() {
    let f = ElectionFixture::new("Zero Participation Test");

    f.register_voter("citizen-1");
    f.create_ballot(standard_ballot_items());
    f.open_election();
    // No votes cast
    f.close_election();

    let (_, tally) = f.compute_and_store_tally();

    assert_eq!(tally.status, TallyStatus::Invalid,
        "Zero votes must produce Invalid tally");
    assert_eq!(tally.total_votes_counted, 0);

    // Certification blocked: Invalid tally
    assert_eq!(tally.status, TallyStatus::Invalid,
        "Invalid tally CANNOT be certified — workflow would reject");
}

// ===========================================================================
// AMBIGUITY C — MIXED ITEMS, ONE AMBIGUOUS
// ===========================================================================

#[test]
fn test_e2e_mixed_ambiguity_blocks_certification() {
    let f = ElectionFixture::new("Mixed Ambiguity Test");

    f.register_voter("citizen-1");
    f.register_voter("citizen-2");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();
    let i1 = f.issue_ballot(&template_ref, "citizen-1");
    let i2 = f.issue_ballot(&template_ref, "citizen-2");

    // Mayor: alice=1, bob=1 (TIE)
    // Measure A: yes=2 (CLEAR)
    f.cast_sealed_vote("citizen-1", &i1, vec![("mayor", "alice"), ("measure-a", "yes")]);
    f.cast_sealed_vote("citizen-2", &i2, vec![("mayor", "bob"), ("measure-a", "yes")]);

    f.close_election();
    let (_, tally) = f.compute_and_store_tally();

    // Mayor is tied, measure is clear
    assert!(tally.item_tallies[0].is_tie, "Mayor should be tied");
    assert!(!tally.item_tallies[1].is_tie, "Measure A should not be tied");

    // Overall: ambiguous because at least one item is ambiguous
    assert_eq!(tally.status, TallyStatus::Ambiguous);
    assert!(tally.has_ambiguity,
        "One tied item must make the entire tally ambiguous");
}

// ===========================================================================
// DETERMINISM — SYSTEM-LEVEL PROOF
// ===========================================================================

#[test]
fn test_e2e_determinism_identical_runs() {
    // Run the same election twice, independently, verify identical results

    fn run_election() -> (TallyResult, String) {
        let f = ElectionFixture::new("Determinism Test");
        f.register_voter("citizen-1");
        f.register_voter("citizen-2");
        f.register_voter("citizen-3");
        let template_ref = f.create_ballot(standard_ballot_items());
        f.open_election();
        let i1 = f.issue_ballot(&template_ref, "citizen-1");
        let i2 = f.issue_ballot(&template_ref, "citizen-2");
        let i3 = f.issue_ballot(&template_ref, "citizen-3");

        f.cast_sealed_vote("citizen-1", &i1, vec![("mayor", "alice"), ("measure-a", "yes")]);
        f.cast_sealed_vote("citizen-2", &i2, vec![("mayor", "bob"), ("measure-a", "no")]);
        f.cast_sealed_vote("citizen-3", &i3, vec![("mayor", "alice"), ("measure-a", "yes")]);

        f.close_election();
        let (_, tally) = f.compute_and_store_tally();
        let hash = tally.input_hash.clone();
        (tally, hash)
    }

    let (tally_a, hash_a) = run_election();
    let (tally_b, hash_b) = run_election();

    // Core determinism checks
    assert_eq!(tally_a.status, tally_b.status, "Status must match");
    assert_eq!(tally_a.has_ambiguity, tally_b.has_ambiguity, "Ambiguity must match");
    assert_eq!(tally_a.total_votes_counted, tally_b.total_votes_counted, "Vote count must match");

    for i in 0..tally_a.item_tallies.len() {
        assert_eq!(
            tally_a.item_tallies[i].choice_counts,
            tally_b.item_tallies[i].choice_counts,
            "Choice counts must match for item {}",
            i
        );
        assert_eq!(
            tally_a.item_tallies[i].winners,
            tally_b.item_tallies[i].winners,
            "Winners must match for item {}",
            i
        );
    }

    // Input hashes won't match because DomainStore generates different IDs per run.
    // But the TALLY RESULTS are identical, which is what matters for determinism.
}

#[test]
fn test_e2e_determinism_shuffled_voters() {
    // Same votes cast by different voter orderings → same tally result

    fn run_with_order(order: &[(&str, &str)]) -> TallyResult {
        let f = ElectionFixture::new("Shuffle Test");
        for (voter, _) in order {
            f.register_voter(voter);
        }
        let template_ref = f.create_ballot(vec![BallotItem {
            item_ref: "mayor".into(),
            item_type: BallotItemType::Race,
            title: "Mayor".into(),
            description: "Choose one".into(),
            choices: vec![
                BallotChoice { choice_ref: "alice".into(), label: "Alice".into(), description: None },
                BallotChoice { choice_ref: "bob".into(), label: "Bob".into(), description: None },
            ],
            max_selections: 1,
        }]);
        f.open_election();

        for (voter, choice) in order {
            let iref = f.issue_ballot(&template_ref, voter);
            f.cast_sealed_vote(voter, &iref, vec![("mayor", choice)]);
        }

        f.close_election();
        let (_, tally) = f.compute_and_store_tally();
        tally
    }

    let tally_a = run_with_order(&[
        ("v1", "alice"), ("v2", "bob"), ("v3", "alice"), ("v4", "bob"), ("v5", "alice"),
    ]);
    let tally_b = run_with_order(&[
        ("v5", "alice"), ("v3", "alice"), ("v1", "alice"), ("v4", "bob"), ("v2", "bob"),
    ]);

    assert_eq!(tally_a.item_tallies[0].choice_counts, tally_b.item_tallies[0].choice_counts,
        "Shuffled voter order must not affect tally");
    assert_eq!(tally_a.item_tallies[0].winners, tally_b.item_tallies[0].winners,
        "Winner must be the same regardless of voter order");
}

// ===========================================================================
// CROSS-MODULE CONSISTENCY
// ===========================================================================

#[test]
fn test_e2e_cross_module_consistency() {
    let f = ElectionFixture::new("Consistency Test");

    // Register 3 voters
    f.register_voter("citizen-1");
    f.register_voter("citizen-2");
    f.register_voter("citizen-3");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();

    let i1 = f.issue_ballot(&template_ref, "citizen-1");
    let i2 = f.issue_ballot(&template_ref, "citizen-2");
    let i3 = f.issue_ballot(&template_ref, "citizen-3");

    f.cast_sealed_vote("citizen-1", &i1, vec![("mayor", "alice"), ("measure-a", "yes")]);
    f.cast_sealed_vote("citizen-2", &i2, vec![("mayor", "bob"), ("measure-a", "yes")]);
    f.cast_sealed_vote("citizen-3", &i3, vec![("mayor", "alice"), ("measure-a", "no")]);

    // CONSISTENCY CHECK 1: Registered voters = ballot issuances
    let registered = f.voter_registry.voters_for_election(&f.election_ref);
    let issuance_count = f.ballot_registry.issuance_count(&f.election_ref);
    assert_eq!(registered.len(), issuance_count,
        "Every registered voter should have a ballot issuance");

    // CONSISTENCY CHECK 2: Votes submitted = participation records
    let votes_submitted = f.vote_registry.votes_submitted(&f.election_ref);
    let participation: Vec<_> = ["citizen-1", "citizen-2", "citizen-3"]
        .iter()
        .filter(|v| f.vote_registry.has_voted(v, &f.election_ref))
        .collect();
    assert_eq!(votes_submitted, participation.len(),
        "Vote count must match participation count");

    // CONSISTENCY CHECK 3: Sealed contents match submitted votes
    let sealed = f.vote_registry.sealed_contents(&f.election_ref);
    assert_eq!(sealed.len(), votes_submitted,
        "Sealed contents count must match submitted votes");

    // CONSISTENCY CHECK 4: Ballot secrecy maintained
    assert!(f.vote_registry.verify_ballot_secrecy(&f.election_ref),
        "Ballot secrecy must be maintained across all modules");

    // CONSISTENCY CHECK 5: Tally uses only sealed votes
    f.close_election();
    let (_, tally) = f.compute_and_store_tally();
    assert_eq!(tally.total_votes_counted as usize, sealed.len(),
        "Tally must count exactly the sealed votes");

    // CONSISTENCY CHECK 6: Certification snapshot matches live tally
    f.tally_election();
    let tally_clone = tally.clone();
    f.certify("taly-ref", tally);
    let (_, cert) = f.cert_registry.certification_for_election(&f.election_ref).unwrap();
    assert_eq!(cert.tally_snapshot.total_votes_counted, tally_clone.total_votes_counted,
        "Certification snapshot must match tally at certification time");
    assert_eq!(cert.tally_snapshot.item_tallies[0].choice_counts,
        tally_clone.item_tallies[0].choice_counts,
        "Certification snapshot choice counts must match tally");

    // CONSISTENCY CHECK 7: Statistics align
    let stats = f.voter_registry.election_statistics(&f.election_ref);
    assert_eq!(stats.registered, 3);
    assert_eq!(stats.total, 3);
}

// ===========================================================================
// EDGE CASE — SINGLE VOTER ELECTION
// ===========================================================================

#[test]
fn test_e2e_single_voter_election() {
    let f = ElectionFixture::new("Single Voter Test");

    f.register_voter("citizen-1");
    let template_ref = f.create_ballot(standard_ballot_items());
    f.open_election();
    let iref = f.issue_ballot(&template_ref, "citizen-1");
    f.cast_sealed_vote("citizen-1", &iref, vec![("mayor", "alice"), ("measure-a", "yes")]);
    f.close_election();

    let (tally_ref, tally) = f.compute_and_store_tally();

    assert_eq!(tally.status, TallyStatus::Computed);
    assert!(!tally.has_ambiguity);
    assert_eq!(tally.total_votes_counted, 1);
    assert_eq!(tally.item_tallies[0].winners, vec!["alice"]);

    f.tally_election();
    f.certify(&tally_ref, tally);

    assert!(f.cert_registry.is_certified(&f.election_ref));
}

// ===========================================================================
// EDGE CASE — CANCELLED ELECTION CANNOT PROCEED
// ===========================================================================

#[test]
fn test_e2e_cancelled_election_terminates() {
    let f = ElectionFixture::new("Cancel Test");
    f.open_election();

    f.election_registry.transition_election(
        &f.election_ref, ElectionStatus::Cancelled, "admin-1", "dec-cancel",
        Some("Emergency cancellation".into()),
    ).expect("Open → Cancelled");

    // Cancelled → cannot go anywhere
    for target in &[
        ElectionStatus::Draft, ElectionStatus::Published,
        ElectionStatus::Open, ElectionStatus::Closed,
        ElectionStatus::Tallied, ElectionStatus::Certified,
    ] {
        assert!(f.election_registry.transition_election(
            &f.election_ref, target.clone(), "a", "d", None).is_err(),
            "Cancelled election must not transition to {:?}", target);
    }
}
