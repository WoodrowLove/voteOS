//! Tests for Operational Intelligence Layer
//!
//! Validates snapshot aggregation, pilot report generation,
//! empty data handling, and deterministic output.

use std::collections::BTreeMap;
use voteos::domain::elections::*;
use voteos::domain::certification::*;
use voteos::domain::proposals::*;
use voteos::domain::audit::*;
use voteos::domain::operations::*;
use voteos::domain::export::*;
use voteos::domain::tally::*;
use voteos::adoption::*;
use voteos::intelligence::*;

// ===========================================================================
// SYSTEM SNAPSHOT
// ===========================================================================

#[test]
fn test_snapshot_empty_system() {
    let snapshot = build_system_snapshot(
        &ElectionRegistry::new(),
        &CertificationRegistry::new(),
        &ProposalRegistry::new(),
        &AuditRegistry::new(),
        &OperationsRegistry::new(),
        &ExportRegistry::new(),
        true,
        false,
    );

    assert_eq!(snapshot.elections.total, 0);
    assert_eq!(snapshot.proposals.total, 0);
    assert_eq!(snapshot.operations.paused_elections, 0);
    assert_eq!(snapshot.audit.audits_run, 0);
    assert_eq!(snapshot.exports.total_exports, 0);
    assert!(snapshot.runtime.persistence_enabled);
    assert!(!snapshot.runtime.auth_enabled);
    assert_eq!(snapshot.runtime.axia_integration_status, "not_connected");
}

#[test]
fn test_snapshot_with_data() {
    let election_reg = ElectionRegistry::new();
    let cert_reg = CertificationRegistry::new();
    let proposal_reg = ProposalRegistry::new();
    let audit_reg = AuditRegistry::new();
    let ops_reg = OperationsRegistry::new();
    let export_reg = ExportRegistry::new();

    // Create elections in various states
    election_reg.elections.insert_new(Election {
        title: "Draft".into(), description: "t".into(),
        election_type: ElectionType::General, status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "t".into(), created_by: "a".into(),
        created_at: "2026-03-30T08:00:00Z".into(), decision_ref: "d".into(),
    });

    election_reg.elections.insert_new(Election {
        title: "Certified".into(), description: "t".into(),
        election_type: ElectionType::General, status: ElectionStatus::Certified,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "t".into(), created_by: "a".into(),
        created_at: "2026-03-30T08:00:00Z".into(), decision_ref: "d".into(),
    });

    election_reg.elections.insert_new(Election {
        title: "Open".into(), description: "t".into(),
        election_type: ElectionType::General, status: ElectionStatus::Open,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "t".into(), created_by: "a".into(),
        created_at: "2026-03-30T08:00:00Z".into(), decision_ref: "d".into(),
    });

    // Create proposals
    proposal_reg.proposals.insert_new(Proposal {
        title: "Draft Prop".into(), description: "t".into(),
        proposal_type: ProposalType::Measure, jurisdiction_scope: "c".into(),
        status: ProposalStatus::Draft, election_ref: None,
        created_by: "a".into(), created_at: "2026-03-30T08:00:00Z".into(),
        decision_ref: "d".into(),
    });

    proposal_reg.proposals.insert_new(Proposal {
        title: "Certified Prop".into(), description: "t".into(),
        proposal_type: ProposalType::Measure, jurisdiction_scope: "c".into(),
        status: ProposalStatus::Certified, election_ref: None,
        created_by: "a".into(), created_at: "2026-03-30T08:00:00Z".into(),
        decision_ref: "d".into(),
    });

    // Operations
    ops_reg.ensure_state("e1", "admin");
    ops_reg.pause("e1", "admin", "test").unwrap();
    ops_reg.ensure_state("e2", "admin");
    ops_reg.flag_incident("e2", "admin", "test").unwrap();

    // Audit
    audit_reg.records.insert_new(AuditRecord {
        election_ref: "e1".into(), status: AuditStatus::Verified,
        initiated_by: "a".into(), initiated_at: "2026-03-30T15:00:00Z".into(),
        completed_at: None, verification: None, decision_ref: "d".into(),
        contest_ref: None,
    });

    let snapshot = build_system_snapshot(
        &election_reg, &cert_reg, &proposal_reg, &audit_reg, &ops_reg, &export_reg,
        true, true,
    );

    assert_eq!(snapshot.elections.total, 3);
    assert_eq!(snapshot.elections.draft, 1);
    assert_eq!(snapshot.elections.certified, 1);
    assert_eq!(snapshot.elections.open, 1);
    assert_eq!(snapshot.proposals.total, 2);
    assert_eq!(snapshot.proposals.draft, 1);
    assert_eq!(snapshot.proposals.certified, 1);
    assert_eq!(snapshot.operations.paused_elections, 1);
    assert_eq!(snapshot.operations.incidents_open, 1);
    assert_eq!(snapshot.audit.audits_run, 1);
    assert_eq!(snapshot.audit.verified, 1);
    assert!(snapshot.runtime.persistence_enabled);
    assert!(snapshot.runtime.auth_enabled);
}

#[test]
fn test_snapshot_deterministic() {
    let election_reg = ElectionRegistry::new();
    election_reg.elections.insert_new(Election {
        title: "Test".into(), description: "t".into(),
        election_type: ElectionType::General, status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None, certification_deadline: None,
        },
        scope: "t".into(), created_by: "a".into(),
        created_at: "2026-03-30T08:00:00Z".into(), decision_ref: "d".into(),
    });

    let cert_reg = CertificationRegistry::new();
    let prop_reg = ProposalRegistry::new();
    let audit_reg = AuditRegistry::new();
    let ops_reg = OperationsRegistry::new();
    let export_reg = ExportRegistry::new();

    let s1 = build_system_snapshot(&election_reg, &cert_reg, &prop_reg, &audit_reg, &ops_reg, &export_reg, true, false);
    let s2 = build_system_snapshot(&election_reg, &cert_reg, &prop_reg, &audit_reg, &ops_reg, &export_reg, true, false);

    assert_eq!(s1.elections.total, s2.elections.total);
    assert_eq!(s1.elections.draft, s2.elections.draft);
    assert_eq!(s1.proposals.total, s2.proposals.total);
    assert_eq!(s1.operations.paused_elections, s2.operations.paused_elections);
}

// ===========================================================================
// PILOT REPORT
// ===========================================================================

#[test]
fn test_pilot_report_empty() {
    let audit_reg = AuditRegistry::new();
    let report = build_pilot_report(&[], None, &[], &audit_reg);

    assert_eq!(report.adoption_summary.total_records, 0);
    assert_eq!(report.adoption_summary.normalization_rate, 0.0);
    assert_eq!(report.reconciliation_summary.total, 0);
    assert_eq!(report.shadow_validation_summary.total_validations, 0);
    assert_eq!(report.audit_summary.audits_run, 0);
    assert!(report.key_findings.iter().any(|f| f.contains("No pilot data")));
}

#[test]
fn test_pilot_report_good_data() {
    let voters = vec![
        normalize_voter(&LegacyVoterRecord {
            legacy_id: "V1".into(), full_name: "Alice".into(),
            date_of_birth: None, jurisdiction: "city".into(),
            legacy_status: "active".into(), metadata: BTreeMap::new(),
        }),
        normalize_voter(&LegacyVoterRecord {
            legacy_id: "V2".into(), full_name: "Bob".into(),
            date_of_birth: None, jurisdiction: "city".into(),
            legacy_status: "registered".into(), metadata: BTreeMap::new(),
        }),
    ];

    let known = BTreeMap::from([
        ("V1".into(), "s1".into()),
        ("V2".into(), "s2".into()),
    ]);
    let recon = reconcile_voters(&voters, &known);

    let shadow = vec![ShadowValidationReport {
        legacy_election_id: "E1".into(),
        overall_result: ShadowComparisonResult::Match,
        item_comparisons: vec![],
        legacy_total_votes: 10,
        voteos_total_votes: 10,
        notes: vec![],
    }];

    let audit_reg = AuditRegistry::new();
    audit_reg.records.insert_new(AuditRecord {
        election_ref: "E1".into(), status: AuditStatus::Verified,
        initiated_by: "a".into(), initiated_at: "2026-03-30T15:00:00Z".into(),
        completed_at: None, verification: None, decision_ref: "d".into(),
        contest_ref: None,
    });

    let report = build_pilot_report(&voters, Some(&recon), &shadow, &audit_reg);

    assert_eq!(report.adoption_summary.total_records, 2);
    assert_eq!(report.adoption_summary.normalized, 2);
    assert!((report.adoption_summary.normalization_rate - 100.0).abs() < 0.01);
    assert_eq!(report.reconciliation_summary.matched, 2);
    assert!((report.reconciliation_summary.match_rate - 100.0).abs() < 0.01);
    assert_eq!(report.shadow_validation_summary.matches, 1);
    assert_eq!(report.shadow_validation_summary.mismatches, 0);
    assert_eq!(report.audit_summary.verified, 1);
    assert!(report.key_findings.iter().any(|f| f.contains("acceptable thresholds")));
}

#[test]
fn test_pilot_report_with_problems() {
    let voters = vec![
        normalize_voter(&LegacyVoterRecord {
            legacy_id: "V1".into(), full_name: "Alice".into(),
            date_of_birth: None, jurisdiction: "city".into(),
            legacy_status: "active".into(), metadata: BTreeMap::new(),
        }),
        normalize_voter(&LegacyVoterRecord {
            legacy_id: "".into(), full_name: "Bad".into(), // Invalid
            date_of_birth: None, jurisdiction: "city".into(),
            legacy_status: "active".into(), metadata: BTreeMap::new(),
        }),
    ];

    let known = BTreeMap::new(); // No matches
    let recon = reconcile_voters(&voters, &known);

    let shadow = vec![ShadowValidationReport {
        legacy_election_id: "E1".into(),
        overall_result: ShadowComparisonResult::TrueMismatch,
        item_comparisons: vec![],
        legacy_total_votes: 10,
        voteos_total_votes: 8,
        notes: vec!["count differs".into()],
    }];

    let audit_reg = AuditRegistry::new();
    audit_reg.records.insert_new(AuditRecord {
        election_ref: "E1".into(), status: AuditStatus::Failed,
        initiated_by: "a".into(), initiated_at: "2026-03-30T15:00:00Z".into(),
        completed_at: None, verification: None, decision_ref: "d".into(),
        contest_ref: None,
    });

    let report = build_pilot_report(&voters, Some(&recon), &shadow, &audit_reg);

    // Should have multiple findings
    assert!(report.key_findings.iter().any(|f| f.contains("invalid")),
        "Must flag invalid records");
    assert!(report.key_findings.iter().any(|f| f.contains("mismatch")),
        "Must flag shadow mismatches");
    assert!(report.key_findings.iter().any(|f| f.contains("FAILED")),
        "Must flag audit failures");
    assert!(report.key_findings.iter().any(|f| f.contains("miss rate")),
        "Must flag high identity miss rate");
    assert_eq!(report.shadow_validation_summary.mismatches, 1);
    assert_eq!(report.audit_summary.failed, 1);
}

#[test]
fn test_pilot_report_deterministic() {
    let voters = vec![
        normalize_voter(&LegacyVoterRecord {
            legacy_id: "V1".into(), full_name: "Alice".into(),
            date_of_birth: None, jurisdiction: "city".into(),
            legacy_status: "active".into(), metadata: BTreeMap::new(),
        }),
    ];
    let audit_reg = AuditRegistry::new();

    let r1 = build_pilot_report(&voters, None, &[], &audit_reg);
    let r2 = build_pilot_report(&voters, None, &[], &audit_reg);

    assert_eq!(r1.adoption_summary.total_records, r2.adoption_summary.total_records);
    assert_eq!(r1.adoption_summary.normalized, r2.adoption_summary.normalized);
    assert_eq!(r1.key_findings.len(), r2.key_findings.len());
}
