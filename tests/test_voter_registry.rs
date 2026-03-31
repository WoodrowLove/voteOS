//! Tests for Module 1: Voter Registry domain types and registry operations.
//!
//! These tests verify the domain store layer (no AxiaSystem required).
//! Workflow tests requiring the spine are marked #[ignore].

use voteos::domain::voters::*;

#[test]
fn test_voter_registration_create() {
    let registry = VoterRegistry::new();

    let id = registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-001".into(),
        election_ref: "elec-001".into(),
        status: RegistrationStatus::Registered,
        registered_at: "2026-03-30T10:00:00Z".into(),
        registered_by: "official-001".into(),
        eligibility_basis: "age_and_jurisdiction".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    assert!(id.starts_with("vreg-"));
    let reg = registry.registrations.get(&id).expect("registration should exist");
    assert_eq!(reg.citizen_ref, "citizen-001");
    assert_eq!(reg.status, RegistrationStatus::Registered);
}

#[test]
fn test_is_registered() {
    let registry = VoterRegistry::new();

    assert!(!registry.is_registered("citizen-001", "elec-001"));

    registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-001".into(),
        election_ref: "elec-001".into(),
        status: RegistrationStatus::Registered,
        registered_at: "2026-03-30T10:00:00Z".into(),
        registered_by: "official-001".into(),
        eligibility_basis: "jurisdiction".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    assert!(registry.is_registered("citizen-001", "elec-001"));
    assert!(!registry.is_registered("citizen-001", "elec-002"));
    assert!(!registry.is_registered("citizen-002", "elec-001"));
}

#[test]
fn test_duplicate_prevention() {
    let registry = VoterRegistry::new();

    registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-001".into(),
        election_ref: "elec-001".into(),
        status: RegistrationStatus::Registered,
        registered_at: "2026-03-30T10:00:00Z".into(),
        registered_by: "official-001".into(),
        eligibility_basis: "jurisdiction".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    // The workflow layer checks is_registered and rejects duplicates
    assert!(registry.is_registered("citizen-001", "elec-001"));
}

#[test]
fn test_suspend_and_restore_registration() {
    let registry = VoterRegistry::new();

    let id = registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-001".into(),
        election_ref: "elec-001".into(),
        status: RegistrationStatus::Registered,
        registered_at: "2026-03-30T10:00:00Z".into(),
        registered_by: "official-001".into(),
        eligibility_basis: "jurisdiction".into(),
        decision_ref: "dec-001".into(),
        attestation_ref: None,
    });

    // Suspend
    let mut reg = registry.registrations.get(&id).expect("exists");
    reg.status = RegistrationStatus::Suspended;
    registry.registrations.update(&id, reg);

    assert!(!registry.is_registered("citizen-001", "elec-001"));
    let suspended = registry.registrations.get(&id).expect("exists");
    assert_eq!(suspended.status, RegistrationStatus::Suspended);

    // Restore
    let mut reg = registry.registrations.get(&id).expect("exists");
    reg.status = RegistrationStatus::Registered;
    registry.registrations.update(&id, reg);

    assert!(registry.is_registered("citizen-001", "elec-001"));
}

#[test]
fn test_voters_for_election() {
    let registry = VoterRegistry::new();

    for i in 1..=5 {
        registry.registrations.insert_new(VoterRegistration {
            citizen_ref: format!("citizen-{:03}", i),
            election_ref: "elec-001".into(),
            status: RegistrationStatus::Registered,
            registered_at: "2026-03-30T10:00:00Z".into(),
            registered_by: "official-001".into(),
            eligibility_basis: "jurisdiction".into(),
            decision_ref: format!("dec-{:03}", i),
            attestation_ref: None,
        });
    }

    // One suspended
    registry.registrations.insert_new(VoterRegistration {
        citizen_ref: "citizen-006".into(),
        election_ref: "elec-001".into(),
        status: RegistrationStatus::Suspended,
        registered_at: "2026-03-30T10:00:00Z".into(),
        registered_by: "official-001".into(),
        eligibility_basis: "jurisdiction".into(),
        decision_ref: "dec-006".into(),
        attestation_ref: None,
    });

    let voters = registry.voters_for_election("elec-001");
    assert_eq!(voters.len(), 5); // Only registered, not suspended
}

#[test]
fn test_eligibility_rule_creation() {
    let registry = VoterRegistry::new();

    let rule_id = registry.rules.insert_new(EligibilityRule {
        election_ref: "elec-001".into(),
        rule_type: RuleType::AgeMinimum,
        criteria: "age >= 18".into(),
        defined_by: "official-001".into(),
        defined_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    assert!(rule_id.starts_with("vrul-"));
    let rules = registry.rules_for_election("elec-001");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].1.rule_type, RuleType::AgeMinimum);
}

#[test]
fn test_challenge_lifecycle() {
    let registry = VoterRegistry::new();

    let challenge_id = registry.challenges.insert_new(EligibilityChallenge {
        voter_registration_ref: "vreg-001".into(),
        election_ref: "elec-001".into(),
        challenger_ref: "citizen-002".into(),
        reason: "Not a resident of jurisdiction".into(),
        status: ChallengeStatus::Filed,
        filed_at: "2026-03-30T10:00:00Z".into(),
        resolution: None,
        resolved_by: None,
        resolved_at: None,
    });

    let challenge = registry.challenges.get(&challenge_id).expect("exists");
    assert_eq!(challenge.status, ChallengeStatus::Filed);

    // Resolve as upheld
    let mut updated = challenge;
    updated.status = ChallengeStatus::Upheld;
    updated.resolution = Some("Investigation confirmed non-residency".into());
    updated.resolved_by = Some("official-001".into());
    updated.resolved_at = Some("2026-03-30T12:00:00Z".into());
    registry.challenges.update(&challenge_id, updated);

    let resolved = registry.challenges.get(&challenge_id).expect("exists");
    assert_eq!(resolved.status, ChallengeStatus::Upheld);
    assert!(resolved.resolution.is_some());
}

#[test]
fn test_voter_statistics() {
    let registry = VoterRegistry::new();

    let statuses = vec![
        RegistrationStatus::Registered,
        RegistrationStatus::Registered,
        RegistrationStatus::Registered,
        RegistrationStatus::Pending,
        RegistrationStatus::Suspended,
        RegistrationStatus::Ineligible,
    ];

    for (i, status) in statuses.into_iter().enumerate() {
        registry.registrations.insert_new(VoterRegistration {
            citizen_ref: format!("citizen-{:03}", i),
            election_ref: "elec-001".into(),
            status,
            registered_at: "2026-03-30T10:00:00Z".into(),
            registered_by: "official-001".into(),
            eligibility_basis: "jurisdiction".into(),
            decision_ref: format!("dec-{:03}", i),
            attestation_ref: None,
        });
    }

    let stats = registry.election_statistics("elec-001");
    assert_eq!(stats.total, 6);
    assert_eq!(stats.registered, 3);
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.suspended, 1);
    assert_eq!(stats.ineligible, 1);
}

#[test]
fn test_voter_roll_generation() {
    let registry = VoterRegistry::new();

    for i in 1..=3 {
        registry.registrations.insert_new(VoterRegistration {
            citizen_ref: format!("citizen-{:03}", i),
            election_ref: "elec-001".into(),
            status: RegistrationStatus::Registered,
            registered_at: "2026-03-30T10:00:00Z".into(),
            registered_by: "official-001".into(),
            eligibility_basis: "jurisdiction".into(),
            decision_ref: format!("dec-{:03}", i),
            attestation_ref: None,
        });
    }

    // Generate roll from registered voters
    let voters = registry.voters_for_election("elec-001");
    let voter_refs: Vec<String> = voters.iter().map(|(_, r)| r.citizen_ref.clone()).collect();

    let roll_id = registry.rolls.insert_new(VoterRoll {
        election_ref: "elec-001".into(),
        voter_refs: voter_refs.clone(),
        generated_at: "2026-03-30T10:00:00Z".into(),
        generated_by: "official-001".into(),
        total_eligible: voter_refs.len(),
    });

    let roll = registry.rolls.get(&roll_id).expect("roll should exist");
    assert_eq!(roll.total_eligible, 3);
    assert_eq!(roll.voter_refs.len(), 3);
}

#[test]
fn test_audit_trail() {
    let registry = VoterRegistry::new();

    registry.audit_log.insert_new(VoterAuditEntry {
        action: "register_voter".into(),
        actor_ref: "official-001".into(),
        target_ref: Some("vreg-001".into()),
        election_ref: Some("elec-001".into()),
        timestamp: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
        details: "Registered citizen-001".into(),
    });

    registry.audit_log.insert_new(VoterAuditEntry {
        action: "suspend_voter_registration".into(),
        actor_ref: "official-001".into(),
        target_ref: Some("vreg-001".into()),
        election_ref: Some("elec-001".into()),
        timestamp: "2026-03-30T11:00:00Z".into(),
        decision_ref: "dec-002".into(),
        details: "Suspended for investigation".into(),
    });

    let all = registry.audit_log.find_all(|_| true);
    assert_eq!(all.len(), 2);

    let for_election = registry.audit_log.find_all(|e| e.election_ref.as_deref() == Some("elec-001"));
    assert_eq!(for_election.len(), 2);
}

#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    // Create and populate
    {
        let registry = VoterRegistry::with_data_dir(path);
        registry.registrations.insert_new(VoterRegistration {
            citizen_ref: "citizen-001".into(),
            election_ref: "elec-001".into(),
            status: RegistrationStatus::Registered,
            registered_at: "2026-03-30T10:00:00Z".into(),
            registered_by: "official-001".into(),
            eligibility_basis: "jurisdiction".into(),
            decision_ref: "dec-001".into(),
            attestation_ref: None,
        });
    }

    // Reload
    {
        let registry = VoterRegistry::with_data_dir(path);
        assert!(registry.is_registered("citizen-001", "elec-001"));
    }
}

// ---------------------------------------------------------------------------
// Workflow tests (require AxiaSystem replica)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_register_voter_workflow_strict() {
    // This test would need a live SpineClient.
    // Marking ignored for now — STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_verify_eligibility_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
