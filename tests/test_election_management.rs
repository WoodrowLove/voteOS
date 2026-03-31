//! Tests for Module 2: Election Management domain types and registry operations.

use voteos::domain::elections::*;

#[test]
fn test_create_election() {
    let registry = ElectionRegistry::new();

    let id = registry.elections.insert_new(Election {
        title: "City Council General Election 2026".into(),
        description: "Annual election for city council seats".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: Some("2026-09-01T00:00:00Z".into()),
            registration_end: Some("2026-10-15T23:59:59Z".into()),
            voting_start: Some("2026-11-01T06:00:00Z".into()),
            voting_end: Some("2026-11-01T20:00:00Z".into()),
            certification_deadline: Some("2026-11-15T17:00:00Z".into()),
        },
        scope: "city_council".into(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    assert!(id.starts_with("elec-"));
    let election = registry.elections.get(&id).expect("should exist");
    assert_eq!(election.status, ElectionStatus::Draft);
    assert_eq!(election.election_type, ElectionType::General);
}

#[test]
fn test_election_state_machine_valid_transitions() {
    // Draft → Published → Open → Closed → Tallied → Certified
    assert!(ElectionStatus::Draft.can_transition_to(&ElectionStatus::Published));
    assert!(ElectionStatus::Published.can_transition_to(&ElectionStatus::Open));
    assert!(ElectionStatus::Open.can_transition_to(&ElectionStatus::Closed));
    assert!(ElectionStatus::Closed.can_transition_to(&ElectionStatus::Tallied));
    assert!(ElectionStatus::Tallied.can_transition_to(&ElectionStatus::Certified));

    // Cancellation from multiple states
    assert!(ElectionStatus::Draft.can_transition_to(&ElectionStatus::Cancelled));
    assert!(ElectionStatus::Published.can_transition_to(&ElectionStatus::Cancelled));
    assert!(ElectionStatus::Open.can_transition_to(&ElectionStatus::Cancelled));
}

#[test]
fn test_election_state_machine_invalid_transitions() {
    assert!(!ElectionStatus::Draft.can_transition_to(&ElectionStatus::Open));
    assert!(!ElectionStatus::Draft.can_transition_to(&ElectionStatus::Closed));
    assert!(!ElectionStatus::Open.can_transition_to(&ElectionStatus::Published));
    assert!(!ElectionStatus::Closed.can_transition_to(&ElectionStatus::Open));
    assert!(!ElectionStatus::Certified.can_transition_to(&ElectionStatus::Open));
    assert!(!ElectionStatus::Cancelled.can_transition_to(&ElectionStatus::Draft));
    assert!(!ElectionStatus::Cancelled.can_transition_to(&ElectionStatus::Open));
}

#[test]
fn test_full_lifecycle_transition() {
    let registry = ElectionRegistry::new();

    let id = registry.elections.insert_new(Election {
        title: "Test Election".into(),
        description: "Test".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    // Draft → Published
    registry.transition_election(&id, ElectionStatus::Published, "official-001", "dec-002", None)
        .expect("Draft → Published should succeed");
    assert_eq!(registry.elections.get(&id).unwrap().status, ElectionStatus::Published);

    // Published → Open
    registry.transition_election(&id, ElectionStatus::Open, "official-001", "dec-003", None)
        .expect("Published → Open should succeed");
    assert_eq!(registry.elections.get(&id).unwrap().status, ElectionStatus::Open);

    // Open → Closed
    registry.transition_election(&id, ElectionStatus::Closed, "official-001", "dec-004", None)
        .expect("Open → Closed should succeed");
    assert_eq!(registry.elections.get(&id).unwrap().status, ElectionStatus::Closed);

    // Closed → Tallied
    registry.transition_election(&id, ElectionStatus::Tallied, "official-001", "dec-005", None)
        .expect("Closed → Tallied should succeed");
    assert_eq!(registry.elections.get(&id).unwrap().status, ElectionStatus::Tallied);

    // Tallied → Certified
    registry.transition_election(&id, ElectionStatus::Certified, "official-001", "dec-006", None)
        .expect("Tallied → Certified should succeed");
    assert_eq!(registry.elections.get(&id).unwrap().status, ElectionStatus::Certified);
}

#[test]
fn test_invalid_transition_rejected() {
    let registry = ElectionRegistry::new();

    let id = registry.elections.insert_new(Election {
        title: "Test Election".into(),
        description: "Test".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    // Draft → Open (should fail — must go through Published first)
    let result = registry.transition_election(&id, ElectionStatus::Open, "official-001", "dec-002", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid transition"));
}

#[test]
fn test_transition_history() {
    let registry = ElectionRegistry::new();

    let id = registry.elections.insert_new(Election {
        title: "Test".into(),
        description: "Test".into(),
        election_type: ElectionType::Referendum,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    registry.transition_election(&id, ElectionStatus::Published, "official-001", "dec-002", None).unwrap();
    registry.transition_election(&id, ElectionStatus::Open, "official-001", "dec-003", None).unwrap();

    let history = registry.transition_history(&id);
    assert_eq!(history.len(), 2);
}

#[test]
fn test_election_officials() {
    let registry = ElectionRegistry::new();

    let elec_id = "elec-001";

    // Assign two officials
    registry.officials.insert_new(ElectionOfficial {
        subject_ref: "official-001".into(),
        election_ref: elec_id.into(),
        role: "commissioner".into(),
        assigned_by: "admin-001".into(),
        assigned_at: "2026-03-30T10:00:00Z".into(),
        revoked: false,
        decision_ref: "dec-001".into(),
    });

    registry.officials.insert_new(ElectionOfficial {
        subject_ref: "official-002".into(),
        election_ref: elec_id.into(),
        role: "clerk".into(),
        assigned_by: "admin-001".into(),
        assigned_at: "2026-03-30T10:00:00Z".into(),
        revoked: false,
        decision_ref: "dec-002".into(),
    });

    assert!(registry.is_official("official-001", elec_id));
    assert!(registry.is_official("official-002", elec_id));
    assert!(!registry.is_official("random-person", elec_id));

    let officials = registry.officials_for_election(elec_id);
    assert_eq!(officials.len(), 2);
}

#[test]
fn test_revoke_official() {
    let registry = ElectionRegistry::new();

    let official_id = registry.officials.insert_new(ElectionOfficial {
        subject_ref: "official-001".into(),
        election_ref: "elec-001".into(),
        role: "commissioner".into(),
        assigned_by: "admin-001".into(),
        assigned_at: "2026-03-30T10:00:00Z".into(),
        revoked: false,
        decision_ref: "dec-001".into(),
    });

    assert!(registry.is_official("official-001", "elec-001"));

    // Revoke
    let mut official = registry.officials.get(&official_id).expect("exists");
    official.revoked = true;
    registry.officials.update(&official_id, official);

    assert!(!registry.is_official("official-001", "elec-001"));
}

#[test]
fn test_election_config_defaults() {
    let config = ElectionConfig::default();
    assert_eq!(config.privacy_mode, PrivacyMode::SecretBallot);
    assert_eq!(config.voting_method, VotingMethod::Plurality);
    assert!(config.participation_threshold.is_none());
    assert_eq!(config.max_choices_per_item, Some(1));
}

#[test]
fn test_configure_election() {
    let registry = ElectionRegistry::new();

    let id = registry.elections.insert_new(Election {
        title: "Ranked Choice Test".into(),
        description: "Test RCV".into(),
        election_type: ElectionType::General,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
    });

    // Update config
    let mut election = registry.elections.get(&id).expect("exists");
    election.config.voting_method = VotingMethod::RankedChoice;
    election.config.participation_threshold = Some(0.5);
    registry.elections.update(&id, election);

    let updated = registry.elections.get(&id).expect("exists");
    assert_eq!(updated.config.voting_method, VotingMethod::RankedChoice);
    assert_eq!(updated.config.participation_threshold, Some(0.5));
}

#[test]
fn test_multiple_election_types() {
    let registry = ElectionRegistry::new();

    let types = vec![
        ElectionType::General,
        ElectionType::Primary,
        ElectionType::Special,
        ElectionType::Referendum,
        ElectionType::Recall,
        ElectionType::Initiative,
    ];

    for (i, etype) in types.into_iter().enumerate() {
        registry.elections.insert_new(Election {
            title: format!("Election {}", i),
            description: "Test".into(),
            election_type: etype,
            status: ElectionStatus::Draft,
            config: ElectionConfig::default(),
            schedule: ElectionSchedule {
                registration_start: None, registration_end: None,
                voting_start: None, voting_end: None, certification_deadline: None,
            },
            scope: "test".into(),
            created_by: "official-001".into(),
            created_at: "2026-03-30T10:00:00Z".into(),
            decision_ref: format!("dec-{:03}", i),
        });
    }

    assert_eq!(registry.elections.count(), 6);
}

#[test]
fn test_cancel_from_various_states() {
    let registry = ElectionRegistry::new();

    // Cancel from Draft
    let id1 = registry.elections.insert_new(Election {
        title: "Cancel Test 1".into(), description: "".into(),
        election_type: ElectionType::General, status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(), created_by: "o".into(), created_at: "t".into(), decision_ref: "d".into(),
    });
    registry.transition_election(&id1, ElectionStatus::Cancelled, "o", "d", Some("test".into())).expect("Draft → Cancelled");

    // Cancel from Published
    let id2 = registry.elections.insert_new(Election {
        title: "Cancel Test 2".into(), description: "".into(),
        election_type: ElectionType::General, status: ElectionStatus::Published,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(), created_by: "o".into(), created_at: "t".into(), decision_ref: "d".into(),
    });
    registry.transition_election(&id2, ElectionStatus::Cancelled, "o", "d", Some("test".into())).expect("Published → Cancelled");

    // Cancel from Open
    let id3 = registry.elections.insert_new(Election {
        title: "Cancel Test 3".into(), description: "".into(),
        election_type: ElectionType::General, status: ElectionStatus::Open,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(), created_by: "o".into(), created_at: "t".into(), decision_ref: "d".into(),
    });
    registry.transition_election(&id3, ElectionStatus::Cancelled, "o", "d", Some("test".into())).expect("Open → Cancelled");

    // Cannot cancel from Certified
    let id4 = registry.elections.insert_new(Election {
        title: "Cancel Test 4".into(), description: "".into(),
        election_type: ElectionType::General, status: ElectionStatus::Certified,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "test".into(), created_by: "o".into(), created_at: "t".into(), decision_ref: "d".into(),
    });
    let result = registry.transition_election(&id4, ElectionStatus::Cancelled, "o", "d", Some("test".into()));
    assert!(result.is_err());
}

#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = ElectionRegistry::with_data_dir(path);
        registry.elections.insert_new(Election {
            title: "Persisted Election".into(),
            description: "Test persistence".into(),
            election_type: ElectionType::General,
            status: ElectionStatus::Draft,
            config: ElectionConfig::default(),
            schedule: ElectionSchedule {
                registration_start: None, registration_end: None,
                voting_start: None, voting_end: None, certification_deadline: None,
            },
            scope: "test".into(),
            created_by: "official-001".into(),
            created_at: "2026-03-30T10:00:00Z".into(),
            decision_ref: "dec-001".into(),
        });
    }

    {
        let registry = ElectionRegistry::with_data_dir(path);
        assert_eq!(registry.elections.count(), 1);
        let all = registry.elections.find_all(|_| true);
        assert_eq!(all[0].1.title, "Persisted Election");
    }
}

// ---------------------------------------------------------------------------
// Workflow tests (require AxiaSystem replica)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_create_election_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_election_lifecycle_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
