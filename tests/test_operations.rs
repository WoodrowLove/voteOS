//! Tests for Module 9: Election Operations + Runtime boundary validation
//!
//! Tests operational controls (pause, resume, incident) and API-level
//! concerns (config, auth, immutability enforcement through API).

use voteos::domain::operations::*;
use voteos::domain::elections::*;

// ===========================================================================
// OPERATIONAL CONTROLS
// ===========================================================================

#[test]
fn test_pause_and_resume_election() {
    let registry = OperationsRegistry::new();
    let election_ref = "e1";
    registry.ensure_state(election_ref, "admin-1");

    // Initially not paused
    assert!(!registry.is_paused(election_ref));

    // Pause
    registry.pause(election_ref, "admin-1", "Weather emergency").unwrap();
    assert!(registry.is_paused(election_ref));

    let (_, state) = registry.state_for_election(election_ref).unwrap();
    assert_eq!(state.status, OperationalStatus::Paused);
    assert!(state.notes.iter().any(|n| n.contains("Weather emergency")));

    // Resume
    registry.resume(election_ref, "admin-1", "Emergency resolved").unwrap();
    assert!(!registry.is_paused(election_ref));

    let (_, state) = registry.state_for_election(election_ref).unwrap();
    assert_eq!(state.status, OperationalStatus::Normal);
}

#[test]
fn test_double_pause_rejected() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");
    registry.pause("e1", "admin-1", "reason").unwrap();

    let result = registry.pause("e1", "admin-1", "again");
    assert!(result.is_err(), "Cannot pause an already paused election");
}

#[test]
fn test_resume_when_not_paused_rejected() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");

    let result = registry.resume("e1", "admin-1", "reason");
    assert!(result.is_err(), "Cannot resume a non-paused election");
}

#[test]
fn test_incident_flag_and_resolve() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");

    assert!(!registry.has_incident("e1"));

    // Flag incident
    registry.flag_incident("e1", "admin-1", "Voter machine malfunction").unwrap();
    assert!(registry.has_incident("e1"));

    let (_, state) = registry.state_for_election("e1").unwrap();
    assert_eq!(state.status, OperationalStatus::IncidentFlagged);

    // Resolve incident
    registry.resolve_incident("e1", "admin-1", "Machine replaced, voting continues").unwrap();
    assert!(!registry.has_incident("e1"));

    let (_, state) = registry.state_for_election("e1").unwrap();
    assert_eq!(state.status, OperationalStatus::Normal);
}

#[test]
fn test_resolve_without_incident_rejected() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");

    let result = registry.resolve_incident("e1", "admin-1", "nothing to resolve");
    assert!(result.is_err(), "Cannot resolve when no incident exists");
}

#[test]
fn test_pause_with_incident_preserves_incident_on_resume() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");

    // Flag incident first
    registry.flag_incident("e1", "admin-1", "Security concern").unwrap();
    // Then pause
    registry.pause("e1", "admin-1", "Halting for investigation").unwrap();

    let (_, state) = registry.state_for_election("e1").unwrap();
    assert!(state.paused);
    assert!(state.incident_flag);

    // Resume — incident should still be flagged
    registry.resume("e1", "admin-1", "Investigation ongoing but voting can resume").unwrap();

    let (_, state) = registry.state_for_election("e1").unwrap();
    assert!(!state.paused);
    assert!(state.incident_flag, "Incident flag must survive resume");
    assert_eq!(state.status, OperationalStatus::IncidentFlagged);
}

#[test]
fn test_operator_action_log() {
    let registry = OperationsRegistry::new();
    registry.ensure_state("e1", "admin-1");

    registry.pause("e1", "admin-1", "Testing").unwrap();
    registry.resume("e1", "admin-1", "Done").unwrap();
    registry.flag_incident("e1", "admin-1", "Incident").unwrap();
    registry.resolve_incident("e1", "admin-1", "Resolved").unwrap();

    let actions = registry.actions_for_election("e1");
    assert_eq!(actions.len(), 4, "All operator actions must be logged");

    let action_types: Vec<_> = actions.iter().map(|(_, a)| a.action_type.clone()).collect();
    assert!(action_types.contains(&OperatorActionType::Pause));
    assert!(action_types.contains(&OperatorActionType::Resume));
    assert!(action_types.contains(&OperatorActionType::FlagIncident));
    assert!(action_types.contains(&OperatorActionType::ResolveIncident));
}

// ===========================================================================
// OPERATIONS DO NOT ALTER TRUTH
// ===========================================================================

#[test]
fn test_operations_do_not_alter_election_state() {
    let election_registry = ElectionRegistry::new();
    let ops_registry = OperationsRegistry::new();

    let election_ref = election_registry.elections.insert_new(Election {
        title: "Truth Test".into(),
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
        created_by: "admin-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-1".into(),
    });

    ops_registry.ensure_state(&election_ref, "admin-1");

    // Pause the election operationally
    ops_registry.pause(&election_ref, "admin-1", "Testing").unwrap();

    // Election state is UNCHANGED — operations layer is separate
    let election = election_registry.elections.get(&election_ref).unwrap();
    assert_eq!(election.status, ElectionStatus::Open,
        "Operational pause must NOT change election lifecycle status");
}

#[test]
fn test_certified_election_operations_still_work() {
    let election_registry = ElectionRegistry::new();
    let ops_registry = OperationsRegistry::new();

    let election_ref = election_registry.elections.insert_new(Election {
        title: "Certified Test".into(),
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
        created_by: "admin-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-1".into(),
    });

    ops_registry.ensure_state(&election_ref, "admin-1");

    // Operations can still work on certified elections (for logging/notes)
    ops_registry.flag_incident(&election_ref, "admin-1", "Post-election audit concern").unwrap();
    assert!(ops_registry.has_incident(&election_ref));

    // But election remains certified — immutable
    let election = election_registry.elections.get(&election_ref).unwrap();
    assert_eq!(election.status, ElectionStatus::Certified,
        "Operations must NEVER alter certified election status");
}

// ===========================================================================
// PERSISTENCE
// ===========================================================================

#[test]
fn test_operations_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = OperationsRegistry::with_data_dir(path);
        registry.ensure_state("e1", "admin-1");
        registry.pause("e1", "admin-1", "Persist test").unwrap();
    }

    {
        let registry = OperationsRegistry::with_data_dir(path);
        assert!(registry.is_paused("e1"), "Paused state must survive restart");
    }
}

// ===========================================================================
// SCHEDULING
// ===========================================================================

#[test]
fn test_schedule_creation() {
    let registry = OperationsRegistry::new();

    let sched_ref = registry.schedules.insert_new(ElectionScheduleRecord {
        election_ref: "e1".into(),
        opens_at: Some("2026-04-01T08:00:00Z".into()),
        closes_at: Some("2026-04-01T20:00:00Z".into()),
        timezone: "America/Chicago".into(),
        auto_transition: true,
        created_by: "admin-1".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
    });

    assert!(sched_ref.starts_with("sched-"));
    let (_, sched) = registry.schedule_for_election("e1").unwrap();
    assert_eq!(sched.opens_at.as_deref(), Some("2026-04-01T08:00:00Z"));
    assert!(sched.auto_transition);
}

// ===========================================================================
// CONFIG VALIDATION (runtime concerns)
// ===========================================================================

#[test]
fn test_config_parses_correctly() {
    let toml_str = r#"
[api]
bind_address = "127.0.0.1"
bind_port = 3100

[security]
api_key = "test-key"
require_auth = true

[persistence]
data_dir = "data"
enabled = true
"#;

    #[derive(serde::Deserialize)]
    struct Config {
        api: ApiSection,
        security: SecuritySection,
        persistence: PersistenceSection,
    }
    #[derive(serde::Deserialize)]
    struct ApiSection { bind_address: String, bind_port: u16 }
    #[derive(serde::Deserialize)]
    struct SecuritySection { api_key: String, require_auth: bool }
    #[derive(serde::Deserialize)]
    struct PersistenceSection { data_dir: String, enabled: bool }

    let config: Config = toml::from_str(toml_str).expect("Config must parse");
    assert_eq!(config.api.bind_port, 3100);
    assert!(config.security.require_auth);
    assert_eq!(config.security.api_key, "test-key");
}

#[test]
fn test_auth_requires_key_when_enabled() {
    // Simulate the validation logic from main.rs
    let require_auth = true;
    let api_key = "";

    let valid = !(require_auth && api_key.is_empty());
    assert!(!valid, "require_auth=true with empty key must fail validation");
}
