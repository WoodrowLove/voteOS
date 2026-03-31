//! Ballot Operations domain types and registry.
//!
//! Module 3: Design, generate, and distribute ballots.
//! Ballot content management — what voters choose from.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::domain::store::DomainStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BallotStatus {
    Draft,
    Finalized,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BallotItemType {
    Race,
    Measure,
    Question,
    Referendum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotChoice {
    pub choice_ref: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotItem {
    pub item_ref: String,
    pub item_type: BallotItemType,
    pub title: String,
    pub description: String,
    pub choices: Vec<BallotChoice>,
    pub max_selections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotTemplate {
    pub election_ref: String,
    pub status: BallotStatus,
    pub items: Vec<BallotItem>,
    pub created_by: String,
    pub created_at: String,
    pub finalized_at: Option<String>,
    pub finalized_by: Option<String>,
    pub decision_ref: String,
    pub integrity_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssuanceStatus {
    Issued,
    Spoiled,
    Replaced,
    Returned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotIssuance {
    pub template_ref: String,
    pub voter_ref: String,
    pub election_ref: String,
    pub status: IssuanceStatus,
    pub issued_at: String,
    pub issued_by: String,
    pub decision_ref: String,
    pub spoiled_at: Option<String>,
    pub replacement_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotAuditEntry {
    pub action: String,
    pub actor_ref: String,
    pub target_ref: Option<String>,
    pub election_ref: String,
    pub timestamp: String,
    pub decision_ref: String,
    pub details: String,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct BallotRegistry {
    pub templates: DomainStore<BallotTemplate>,
    pub issuances: DomainStore<BallotIssuance>,
    pub audit_log: DomainStore<BallotAuditEntry>,
}

impl BallotRegistry {
    pub fn new() -> Self {
        Self {
            templates: DomainStore::new("btpl"),
            issuances: DomainStore::new("biss"),
            audit_log: DomainStore::new("baud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            templates: DomainStore::with_persistence("btpl", dir.join("ballot_templates.json")),
            issuances: DomainStore::with_persistence("biss", dir.join("ballot_issuances.json")),
            audit_log: DomainStore::with_persistence("baud", dir.join("ballot_audit.json")),
        }
    }

    /// Get the ballot template for an election.
    pub fn template_for_election(&self, election_ref: &str) -> Option<(String, BallotTemplate)> {
        self.templates.find_all(|t| t.election_ref == election_ref)
            .into_iter().next()
    }

    /// Get finalized template for an election.
    pub fn finalized_template(&self, election_ref: &str) -> Option<(String, BallotTemplate)> {
        self.templates.find_all(|t| {
            t.election_ref == election_ref && t.status == BallotStatus::Finalized
        }).into_iter().next()
    }

    /// Check if a voter has been issued a ballot for an election.
    pub fn has_active_issuance(&self, voter_ref: &str, election_ref: &str) -> bool {
        !self.issuances.find_all(|i| {
            i.voter_ref == voter_ref
                && i.election_ref == election_ref
                && i.status == IssuanceStatus::Issued
        }).is_empty()
    }

    /// Find a voter's active ballot issuance.
    pub fn find_active_issuance(&self, voter_ref: &str, election_ref: &str) -> Option<(String, BallotIssuance)> {
        self.issuances.find_all(|i| {
            i.voter_ref == voter_ref
                && i.election_ref == election_ref
                && i.status == IssuanceStatus::Issued
        }).into_iter().next()
    }

    /// Count issued ballots for an election.
    pub fn issuance_count(&self, election_ref: &str) -> usize {
        self.issuances.find_all(|i| {
            i.election_ref == election_ref && i.status == IssuanceStatus::Issued
        }).len()
    }

    /// Compute integrity hash for a ballot template.
    pub fn compute_integrity_hash(template: &BallotTemplate) -> String {
        use sha2::{Sha256, Digest};
        let content = serde_json::to_string(&template.items).unwrap_or_default();
        let hash = Sha256::digest(content.as_bytes());
        format!("{:x}", hash)
    }
}
