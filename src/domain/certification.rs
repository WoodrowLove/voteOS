//! Result Certification domain types and registry.
//!
//! Module 6: Authority and finality for election outcomes.
//! Certification is separate from computation. Tally Engine computes;
//! Result Certification attests, finalizes, and declares.
//!
//! Once certified, a result is IMMUTABLE. No modification, no reopening.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;
use crate::domain::tally::TallyResult;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Status of a certification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificationStatus {
    /// Tally computed, awaiting certification decision.
    Pending,
    /// Result certified — IMMUTABLE from this point.
    Certified,
    /// Certification rejected (procedural issue, not ambiguity).
    Rejected,
    /// Ambiguity exists — requires resolution before certification.
    RequiresResolution,
    /// A contest (challenge) has been filed against a certified result.
    Contested,
}

/// Immutable certification record for an election result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationRecord {
    /// Election this certification applies to.
    pub election_ref: String,
    /// Reference to the tally result being certified.
    pub tally_ref: String,
    /// Snapshot of the tally result at certification time.
    pub tally_snapshot: TallyResult,
    /// Current certification status.
    pub status: CertificationStatus,
    /// Who certified (election official).
    pub certified_by: Option<String>,
    /// When certification occurred.
    pub certified_at: Option<String>,
    /// Basis for certification (method, rules applied).
    pub certification_basis: String,
    /// Decision ref from AxiaSystem legitimacy evaluation.
    pub decision_ref: String,
    /// Attestation ref from AxiaSystem attestation.
    pub attestation_ref: Option<String>,
    /// Rejection reason (if rejected).
    pub rejection_reason: Option<String>,
    /// When the record was created.
    pub created_at: String,
}

/// A contest (challenge) filed against a certified result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contest {
    /// The certification being contested.
    pub certification_ref: String,
    /// Election reference.
    pub election_ref: String,
    /// Who filed the contest.
    pub filed_by: String,
    /// Reason for the contest.
    pub reason: String,
    /// When filed.
    pub filed_at: String,
    /// Current resolution status.
    pub status: ContestStatus,
    /// Resolution details (if resolved).
    pub resolution: Option<String>,
    /// Who resolved the contest.
    pub resolved_by: Option<String>,
    /// When resolved.
    pub resolved_at: Option<String>,
    /// Decision ref from legitimacy evaluation.
    pub decision_ref: String,
}

/// Status of a contest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContestStatus {
    Filed,
    UnderReview,
    Upheld,
    Dismissed,
}

/// Audit entry for certification operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationAuditEntry {
    pub action: String,
    pub actor_ref: String,
    pub election_ref: String,
    pub timestamp: String,
    pub decision_ref: String,
    pub details: String,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct CertificationRegistry {
    pub certifications: DomainStore<CertificationRecord>,
    pub contests: DomainStore<Contest>,
    pub audit_log: DomainStore<CertificationAuditEntry>,
}

impl CertificationRegistry {
    pub fn new() -> Self {
        Self {
            certifications: DomainStore::new("cert"),
            contests: DomainStore::new("cont"),
            audit_log: DomainStore::new("caud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            certifications: DomainStore::with_persistence("cert", dir.join("certifications.json")),
            contests: DomainStore::with_persistence("cont", dir.join("contests.json")),
            audit_log: DomainStore::with_persistence("caud", dir.join("certification_audit.json")),
        }
    }

    /// Get the certification record for an election.
    pub fn certification_for_election(&self, election_ref: &str) -> Option<(String, CertificationRecord)> {
        self.certifications.find_all(|c| c.election_ref == election_ref)
            .into_iter().next()
    }

    /// Check if an election has been certified.
    pub fn is_certified(&self, election_ref: &str) -> bool {
        self.certifications.find_all(|c| {
            c.election_ref == election_ref && c.status == CertificationStatus::Certified
        }).into_iter().next().is_some()
    }

    /// Check if an election result is contested.
    pub fn is_contested(&self, election_ref: &str) -> bool {
        self.contests.find_all(|c| {
            c.election_ref == election_ref
                && (c.status == ContestStatus::Filed || c.status == ContestStatus::UnderReview)
        }).into_iter().next().is_some()
    }

    /// Get all contests for an election.
    pub fn contests_for_election(&self, election_ref: &str) -> Vec<(String, Contest)> {
        self.contests.find_all(|c| c.election_ref == election_ref)
    }
}
