//! Governance Proposals domain types and registry.
//!
//! Module 7: Proposals as first-class governance entities.
//! A proposal defines WHAT is being decided. An election defines HOW it's decided.
//! Proposals link to elections — they do not replace them.
//!
//! VoteOS decides. It never executes.
//! A certified proposal outcome is a statement, not an action.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Type of governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// Ballot measure (yes/no on a specific policy).
    Measure,
    /// Citizen-initiated proposal.
    Initiative,
    /// Government-referred question to voters.
    Referendum,
    /// Policy change requiring approval.
    Policy,
    /// Advisory (non-binding) question.
    Advisory,
}

/// Lifecycle status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal drafted, not yet public.
    Draft,
    /// Published for review — not yet on a ballot.
    Published,
    /// Attached to an election and voting is active.
    Voting,
    /// Voting closed, awaiting tally/certification.
    Closed,
    /// Result certified — outcome is final.
    Certified,
    /// Proposal rejected (insufficient support, procedural failure).
    Rejected,
    /// Proposal withdrawn before voting.
    Withdrawn,
}

/// Certified outcome of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalOutcome {
    /// Proposal approved by voters.
    Approved,
    /// Proposal rejected by voters.
    Rejected,
    /// Result is ambiguous (tie, threshold not met).
    Ambiguous,
    /// Not yet determined.
    Pending,
}

/// A governance proposal — defines what is being decided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Human-readable title.
    pub title: String,
    /// Full description of what the proposal does.
    pub description: String,
    /// Type of proposal.
    pub proposal_type: ProposalType,
    /// Jurisdictional scope (e.g., "city", "district-3", "statewide").
    pub jurisdiction_scope: String,
    /// Current lifecycle status.
    pub status: ProposalStatus,
    /// Reference to the linked election (set when attached to ballot).
    pub election_ref: Option<String>,
    /// Who created the proposal.
    pub created_by: String,
    /// When created.
    pub created_at: String,
    /// Decision ref from legitimacy evaluation.
    pub decision_ref: String,
}

/// Certified result of a proposal — links proposal outcome to election certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalResult {
    /// Proposal this result applies to.
    pub proposal_ref: String,
    /// Election through which the proposal was voted on.
    pub election_ref: String,
    /// Determined outcome.
    pub outcome: ProposalOutcome,
    /// Summary of the vote (e.g., "Yes: 65%, No: 35%").
    pub vote_summary: String,
    /// Reference to the certification record.
    pub certification_ref: String,
    /// When the result was certified.
    pub certified_at: String,
    /// Approval threshold that was applied (if any).
    pub threshold_applied: Option<f64>,
    /// Whether the threshold was met.
    pub threshold_met: Option<bool>,
}

/// Audit entry for proposal operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalAuditEntry {
    pub action: String,
    pub actor_ref: String,
    pub proposal_ref: String,
    pub timestamp: String,
    pub decision_ref: String,
    pub details: String,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct ProposalRegistry {
    pub proposals: DomainStore<Proposal>,
    pub results: DomainStore<ProposalResult>,
    pub audit_log: DomainStore<ProposalAuditEntry>,
}

impl ProposalRegistry {
    pub fn new() -> Self {
        Self {
            proposals: DomainStore::new("prop"),
            results: DomainStore::new("pres"),
            audit_log: DomainStore::new("paud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            proposals: DomainStore::with_persistence("prop", dir.join("proposals.json")),
            results: DomainStore::with_persistence("pres", dir.join("proposal_results.json")),
            audit_log: DomainStore::with_persistence("paud", dir.join("proposal_audit.json")),
        }
    }

    /// Get a proposal by election reference.
    pub fn proposal_for_election(&self, election_ref: &str) -> Option<(String, Proposal)> {
        self.proposals.find_all(|p| {
            p.election_ref.as_deref() == Some(election_ref)
        }).into_iter().next()
    }

    /// Get the certified result for a proposal.
    pub fn result_for_proposal(&self, proposal_ref: &str) -> Option<(String, ProposalResult)> {
        self.results.find_all(|r| r.proposal_ref == proposal_ref)
            .into_iter().next()
    }

    /// Check if a proposal has been certified.
    pub fn is_certified(&self, proposal_ref: &str) -> bool {
        self.proposals.get(proposal_ref)
            .map(|p| p.status == ProposalStatus::Certified)
            .unwrap_or(false)
    }

    /// Get all proposals for a jurisdiction.
    pub fn proposals_for_jurisdiction(&self, scope: &str) -> Vec<(String, Proposal)> {
        self.proposals.find_all(|p| p.jurisdiction_scope == scope)
    }
}

// ---------------------------------------------------------------------------
// Outcome determination — maps election tally to proposal outcome
// ---------------------------------------------------------------------------

/// Determine proposal outcome from election tally results.
///
/// For a proposal (typically yes/no), the outcome is:
/// - Approved: "yes" votes > "no" votes (and threshold met if applicable)
/// - Rejected: "no" votes >= "yes" votes
/// - Ambiguous: exact tie
///
/// This is a pure function.
pub fn determine_proposal_outcome(
    yes_count: u64,
    no_count: u64,
    total_votes: u64,
    approval_threshold: Option<f64>,
) -> (ProposalOutcome, String) {
    if total_votes == 0 {
        return (
            ProposalOutcome::Ambiguous,
            "No votes cast — outcome cannot be determined".into(),
        );
    }

    if yes_count == no_count {
        return (
            ProposalOutcome::Ambiguous,
            format!("Exact tie: Yes {} / No {} of {} total", yes_count, no_count, total_votes),
        );
    }

    let yes_pct = (yes_count as f64 / total_votes as f64) * 100.0;

    // Check threshold if applicable
    if let Some(threshold) = approval_threshold {
        if yes_pct < threshold {
            return (
                ProposalOutcome::Rejected,
                format!(
                    "Yes {:.1}% ({}/{}) — below {:.1}% threshold",
                    yes_pct, yes_count, total_votes, threshold
                ),
            );
        }
    }

    if yes_count > no_count {
        (
            ProposalOutcome::Approved,
            format!(
                "Approved: Yes {:.1}% ({}/{}), No {:.1}% ({}/{})",
                yes_pct, yes_count, total_votes,
                (no_count as f64 / total_votes as f64) * 100.0, no_count, total_votes
            ),
        )
    } else {
        (
            ProposalOutcome::Rejected,
            format!(
                "Rejected: No {:.1}% ({}/{}), Yes {:.1}% ({}/{})",
                (no_count as f64 / total_votes as f64) * 100.0, no_count, total_votes,
                yes_pct, yes_count, total_votes
            ),
        )
    }
}
