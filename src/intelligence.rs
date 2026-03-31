//! VoteOS Operational Intelligence Layer
//!
//! Read-only observational layer for system behavior.
//! Aggregates metrics from registries, adoption, and audit —
//! never mutates domain state.
//!
//! DATA BOUNDARY:
//! - Election Truth Data: internal (votes, ballots, tally details)
//! - Operational Intelligence: exportable (counts, rates, statuses)
//!
//! NO individual vote content. NO voter linkage. Aggregates only.

use serde::{Deserialize, Serialize};
use crate::domain::elections::{ElectionRegistry, ElectionStatus};
use crate::domain::certification::CertificationRegistry;
use crate::domain::proposals::{ProposalRegistry, ProposalStatus};
use crate::domain::audit::{AuditRegistry, AuditStatus};
use crate::domain::operations::OperationsRegistry;
use crate::domain::export::ExportRegistry;
use crate::adoption::{
    NormalizedVoterCandidate, NormalizationStatus,
    ReconciliationReport, ReconciliationStatus,
    ShadowValidationReport, ShadowComparisonResult,
};

// ===========================================================================
// SYSTEM SNAPSHOT — structured view of current system state
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub version: String,
    pub timestamp: String,

    pub runtime: RuntimeInfo,
    pub elections: ElectionMetrics,
    pub proposals: ProposalMetrics,
    pub operations: OperationsMetrics,
    pub audit: AuditMetrics,
    pub exports: ExportMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub persistence_enabled: bool,
    pub auth_enabled: bool,
    pub axia_integration_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionMetrics {
    pub total: usize,
    pub draft: usize,
    pub published: usize,
    pub open: usize,
    pub closed: usize,
    pub tallied: usize,
    pub certified: usize,
    pub cancelled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMetrics {
    pub total: usize,
    pub draft: usize,
    pub published: usize,
    pub certified: usize,
    pub rejected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsMetrics {
    pub paused_elections: usize,
    pub incidents_open: usize,
    pub total_actions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    pub audits_run: usize,
    pub verified: usize,
    pub failed: usize,
    pub contested: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetrics {
    pub total_exports: usize,
    pub total_events: usize,
}

/// Build a system snapshot from all registries.
/// Pure aggregation — no mutation, no side effects.
pub fn build_system_snapshot(
    election_registry: &ElectionRegistry,
    cert_registry: &CertificationRegistry,
    proposal_registry: &ProposalRegistry,
    audit_registry: &AuditRegistry,
    ops_registry: &OperationsRegistry,
    export_registry: &ExportRegistry,
    persistence_enabled: bool,
    auth_enabled: bool,
) -> SystemSnapshot {
    // Election counts by status
    let all_elections = election_registry.elections.find_all(|_| true);
    let elections = ElectionMetrics {
        total: all_elections.len(),
        draft: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Draft).count(),
        published: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Published).count(),
        open: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Open).count(),
        closed: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Closed).count(),
        tallied: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Tallied).count(),
        certified: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Certified).count(),
        cancelled: all_elections.iter().filter(|(_, e)| e.status == ElectionStatus::Cancelled).count(),
    };

    // Proposal counts
    let all_proposals = proposal_registry.proposals.find_all(|_| true);
    let proposals = ProposalMetrics {
        total: all_proposals.len(),
        draft: all_proposals.iter().filter(|(_, p)| p.status == ProposalStatus::Draft).count(),
        published: all_proposals.iter().filter(|(_, p)| p.status == ProposalStatus::Published).count(),
        certified: all_proposals.iter().filter(|(_, p)| p.status == ProposalStatus::Certified).count(),
        rejected: all_proposals.iter().filter(|(_, p)| p.status == ProposalStatus::Rejected).count(),
    };

    // Operations
    let all_states = ops_registry.states.find_all(|_| true);
    let operations = OperationsMetrics {
        paused_elections: all_states.iter().filter(|(_, s)| s.paused).count(),
        incidents_open: all_states.iter().filter(|(_, s)| s.incident_flag).count(),
        total_actions: ops_registry.actions.count(),
    };

    // Audit
    let all_audits = audit_registry.records.find_all(|_| true);
    let audit = AuditMetrics {
        audits_run: all_audits.len(),
        verified: all_audits.iter().filter(|(_, a)| a.status == AuditStatus::Verified).count(),
        failed: all_audits.iter().filter(|(_, a)| a.status == AuditStatus::Failed).count(),
        contested: all_audits.iter().filter(|(_, a)| a.status == AuditStatus::Contested).count(),
    };

    // Exports
    let exports = ExportMetrics {
        total_exports: export_registry.exports.count(),
        total_events: export_registry.events.count(),
    };

    SystemSnapshot {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        runtime: RuntimeInfo {
            persistence_enabled,
            auth_enabled,
            axia_integration_status: "not_connected".into(),
        },
        elections,
        proposals,
        operations,
        audit,
        exports,
    }
}

// ===========================================================================
// PILOT REPORT — structured summary of adoption/validation activity
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PilotReport {
    pub timestamp: String,

    pub adoption_summary: AdoptionSummary,
    pub reconciliation_summary: ReconciliationSummary,
    pub shadow_validation_summary: ShadowValidationSummary,
    pub audit_summary: PilotAuditSummary,
    pub key_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionSummary {
    pub total_records: usize,
    pub normalized: usize,
    pub incomplete: usize,
    pub invalid: usize,
    pub unsupported: usize,
    pub normalization_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummary {
    pub total: usize,
    pub matched: usize,
    pub ambiguous: usize,
    pub missing: usize,
    pub invalid: usize,
    pub match_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowValidationSummary {
    pub total_validations: usize,
    pub matches: usize,
    pub semantic_equivalent: usize,
    pub mismatches: usize,
    pub incomplete: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PilotAuditSummary {
    pub audits_run: usize,
    pub verified: usize,
    pub failed: usize,
}

/// Build a pilot report from adoption pipeline outputs.
///
/// Takes pre-computed adoption results as input —
/// the intelligence layer does not run the adoption pipeline itself.
pub fn build_pilot_report(
    normalized_voters: &[NormalizedVoterCandidate],
    reconciliation: Option<&ReconciliationReport>,
    shadow_validations: &[ShadowValidationReport],
    audit_registry: &AuditRegistry,
) -> PilotReport {
    // Adoption summary
    let total_records = normalized_voters.len();
    let normalized = normalized_voters.iter()
        .filter(|v| v.normalization_status == NormalizationStatus::Normalized)
        .count();
    let incomplete = normalized_voters.iter()
        .filter(|v| v.normalization_status == NormalizationStatus::Incomplete)
        .count();
    let invalid = normalized_voters.iter()
        .filter(|v| v.normalization_status == NormalizationStatus::Invalid)
        .count();
    let unsupported = normalized_voters.iter()
        .filter(|v| v.normalization_status == NormalizationStatus::Unsupported)
        .count();
    let normalization_rate = if total_records > 0 {
        (normalized as f64 / total_records as f64) * 100.0
    } else {
        0.0
    };

    // Reconciliation summary
    let recon_summary = match reconciliation {
        Some(r) => ReconciliationSummary {
            total: r.total_records,
            matched: r.matched,
            ambiguous: r.ambiguous,
            missing: r.missing,
            invalid: r.invalid,
            match_rate: if r.total_records > 0 {
                (r.matched as f64 / r.total_records as f64) * 100.0
            } else {
                0.0
            },
        },
        None => ReconciliationSummary {
            total: 0, matched: 0, ambiguous: 0, missing: 0, invalid: 0, match_rate: 0.0,
        },
    };

    // Shadow validation summary
    let shadow_summary = ShadowValidationSummary {
        total_validations: shadow_validations.len(),
        matches: shadow_validations.iter()
            .filter(|v| v.overall_result == ShadowComparisonResult::Match)
            .count(),
        semantic_equivalent: shadow_validations.iter()
            .filter(|v| v.overall_result == ShadowComparisonResult::SemanticEquivalent)
            .count(),
        mismatches: shadow_validations.iter()
            .filter(|v| v.overall_result == ShadowComparisonResult::TrueMismatch)
            .count(),
        incomplete: shadow_validations.iter()
            .filter(|v| v.overall_result == ShadowComparisonResult::LegacyDataIncomplete)
            .count(),
    };

    // Audit summary
    let all_audits = audit_registry.records.find_all(|_| true);
    let audit_summary = PilotAuditSummary {
        audits_run: all_audits.len(),
        verified: all_audits.iter().filter(|(_, a)| a.status == AuditStatus::Verified).count(),
        failed: all_audits.iter().filter(|(_, a)| a.status == AuditStatus::Failed).count(),
    };

    // Key findings — deterministic rules
    let mut findings = Vec::new();

    if normalization_rate < 90.0 && total_records > 0 {
        findings.push(format!(
            "Low normalization rate: {:.1}% ({}/{} records) — review legacy data quality",
            normalization_rate, normalized, total_records
        ));
    }

    if invalid > 0 {
        findings.push(format!(
            "{} invalid records detected during normalization — these cannot enter VoteOS",
            invalid
        ));
    }

    if recon_summary.ambiguous > 0 {
        findings.push(format!(
            "{} ambiguous identity reconciliations — require human resolution",
            recon_summary.ambiguous
        ));
    }

    if recon_summary.missing > 0 && recon_summary.total > 0 {
        let missing_pct = (recon_summary.missing as f64 / recon_summary.total as f64) * 100.0;
        if missing_pct > 20.0 {
            findings.push(format!(
                "High identity miss rate: {:.1}% ({}/{}) — legacy voters not found in system",
                missing_pct, recon_summary.missing, recon_summary.total
            ));
        }
    }

    if shadow_summary.mismatches > 0 {
        findings.push(format!(
            "{} shadow validation mismatches — legacy outcomes differ from VoteOS computation",
            shadow_summary.mismatches
        ));
    }

    if audit_summary.failed > 0 {
        findings.push(format!(
            "{} audit verifications FAILED — investigate discrepancies immediately",
            audit_summary.failed
        ));
    }

    if findings.is_empty() && total_records > 0 {
        findings.push("All pilot metrics within acceptable thresholds".into());
    }

    if total_records == 0 && shadow_validations.is_empty() {
        findings.push("No pilot data processed yet — import legacy records to begin".into());
    }

    PilotReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        adoption_summary: AdoptionSummary {
            total_records,
            normalized,
            incomplete,
            invalid,
            unsupported,
            normalization_rate,
        },
        reconciliation_summary: recon_summary,
        shadow_validation_summary: shadow_summary,
        audit_summary,
        key_findings: findings,
    }
}
