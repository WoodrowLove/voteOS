//! Integration & Export domain types.
//!
//! Module 10: Certified result export for external consumption.
//!
//! FUNDAMENTAL PRINCIPLE:
//! VoteOS produces certified decisions. It never executes them.
//! Export delivers outcomes as read-only, verifiable packages.
//! External systems (CivilOS, etc.) consume and act on these decisions.
//!
//! NO direct API calls from VoteOS to CivilOS or any external system.
//! Integration is through attested data, not remote procedure calls.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;
use crate::domain::tally::ItemTally;
use crate::domain::proposals::ProposalOutcome;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Format of the export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportFormat {
    /// Full JSON bundle with all verification data.
    FullBundle,
    /// Summary suitable for public display.
    PublicSummary,
    /// Minimal machine-readable outcome.
    MachineReadable,
}

/// A certified result export — everything an external system needs
/// to consume a VoteOS decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifiedResultExport {
    /// Unique export reference.
    pub export_ref: String,
    /// Election that produced this result.
    pub election_ref: String,
    /// Proposal reference (if this is a governance proposal result).
    pub proposal_ref: Option<String>,
    /// Jurisdictional scope.
    pub jurisdiction_scope: String,
    /// Election title.
    pub title: String,
    /// Per-item results summary.
    pub item_results: Vec<ExportItemResult>,
    /// Proposal outcome (if applicable).
    pub proposal_outcome: Option<ProposalOutcome>,
    /// Total votes counted.
    pub total_votes: u64,
    /// Certification reference.
    pub certification_ref: String,
    /// Input hash from tally (for verification).
    pub audit_hash: String,
    /// When the result was certified.
    pub certified_at: String,
    /// Who certified the result.
    pub certified_by: String,
    /// Format of this export.
    pub format: ExportFormat,
    /// When this export was generated.
    pub exported_at: String,
    /// Whether the export has been consumed (for tracking, not enforcement).
    pub consumed: bool,
}

/// Per-item result in an export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportItemResult {
    /// Ballot item reference.
    pub ballot_item_ref: String,
    /// Winner(s).
    pub winners: Vec<String>,
    /// Whether this item had a tie.
    pub is_tie: bool,
    /// Total votes for this item.
    pub total_votes: u64,
    /// Vote summary (e.g., "Alice: 52.3%, Bob: 47.7%").
    pub summary: String,
}

/// Internal event representing a significant system occurrence.
/// These can later be used for webhooks, event buses, or integration triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Type of event.
    pub event_type: EventType,
    /// Associated election reference.
    pub election_ref: String,
    /// Associated proposal reference (if applicable).
    pub proposal_ref: Option<String>,
    /// When the event occurred.
    pub timestamp: String,
    /// Event payload (serialized details).
    pub payload: String,
}

/// Types of system events that can be emitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    ElectionPublished,
    ElectionOpened,
    ElectionClosed,
    TallyComputed,
    ResultCertified,
    ResultContested,
    ProposalPublished,
    ProposalCertified,
    AuditCompleted,
    ExportGenerated,
}

/// Audit entry for export operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAuditEntry {
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

pub struct ExportRegistry {
    pub exports: DomainStore<CertifiedResultExport>,
    pub events: DomainStore<SystemEvent>,
    pub audit_log: DomainStore<ExportAuditEntry>,
}

impl ExportRegistry {
    pub fn new() -> Self {
        Self {
            exports: DomainStore::new("exprt"),
            events: DomainStore::new("event"),
            audit_log: DomainStore::new("eaud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            exports: DomainStore::with_persistence("exprt", dir.join("exports.json")),
            events: DomainStore::with_persistence("event", dir.join("system_events.json")),
            audit_log: DomainStore::with_persistence("eaud", dir.join("export_audit.json")),
        }
    }

    /// Get exports for an election.
    pub fn exports_for_election(&self, election_ref: &str) -> Vec<(String, CertifiedResultExport)> {
        self.exports.find_all(|e| e.election_ref == election_ref)
    }

    /// Get events for an election.
    pub fn events_for_election(&self, election_ref: &str) -> Vec<(String, SystemEvent)> {
        self.events.find_all(|e| e.election_ref == election_ref)
    }

    /// Check if a result has been exported.
    pub fn has_export(&self, election_ref: &str) -> bool {
        !self.exports_for_election(election_ref).is_empty()
    }
}

// ---------------------------------------------------------------------------
// Pure export functions
// ---------------------------------------------------------------------------

/// Build an ExportItemResult from a tally ItemTally.
pub fn item_tally_to_export(item: &ItemTally) -> ExportItemResult {
    ExportItemResult {
        ballot_item_ref: item.ballot_item_ref.clone(),
        winners: item.winners.clone(),
        is_tie: item.is_tie,
        total_votes: item.total_votes,
        summary: item.result_summary.clone(),
    }
}
