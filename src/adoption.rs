//! VoteOS Adoption Layer — Legacy Migration & Shadow Validation
//!
//! This module provides the bridge between legacy election systems and VoteOS.
//! It follows the same conceptual discipline as CivilOS's wrapper layer:
//!
//!   Legacy Source → Adapter → Normalizer → Identity Reconciler → Shadow Validator
//!
//! FUNDAMENTAL PRINCIPLES:
//! 1. VoteOS NEVER trusts legacy results blindly — it recomputes independently
//! 2. Identity reconciliation preserves ambiguity explicitly — no silent resolution
//! 3. Shadow mode before cutover — compare, don't replace
//! 4. The adoption layer FEEDS the trust core; it does not distort it
//! 5. Legacy-specific logic stays HERE, not in Modules 1-10

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

// ===========================================================================
// LEGACY RECORD TYPES — what comes in from external systems
// ===========================================================================

/// A voter record from a legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyVoterRecord {
    /// Legacy system's voter ID.
    pub legacy_id: String,
    /// Full name as stored in legacy system.
    pub full_name: String,
    /// Date of birth (if available).
    pub date_of_birth: Option<String>,
    /// Address/jurisdiction info.
    pub jurisdiction: String,
    /// Registration status in legacy system.
    pub legacy_status: String,
    /// Any additional fields from legacy system.
    pub metadata: BTreeMap<String, String>,
}

/// An election configuration from a legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyElectionRecord {
    /// Legacy system's election ID.
    pub legacy_id: String,
    /// Election title.
    pub title: String,
    /// Election type as string (legacy systems vary).
    pub election_type: String,
    /// Jurisdiction scope.
    pub jurisdiction: String,
    /// Voting method as string.
    pub voting_method: String,
    /// Items/races on the ballot.
    pub ballot_items: Vec<LegacyBallotItem>,
    /// Reported outcome from legacy system.
    pub reported_outcome: Option<LegacyOutcome>,
}

/// A ballot item from a legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyBallotItem {
    pub item_id: String,
    pub title: String,
    pub choices: Vec<String>,
}

/// Reported outcome from a legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyOutcome {
    /// Per-item results as reported by legacy system.
    pub item_results: Vec<LegacyItemResult>,
    /// Total votes reported.
    pub total_votes_reported: u64,
    /// Was this outcome certified in the legacy system?
    pub certified: bool,
}

/// Per-item result from legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyItemResult {
    pub item_id: String,
    pub winner: Option<String>,
    pub vote_counts: BTreeMap<String, u64>,
    pub total_votes: u64,
}

/// An election official/admin from a legacy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyOfficialRecord {
    pub legacy_id: String,
    pub full_name: String,
    pub role: String,
    pub jurisdiction: String,
}

// ===========================================================================
// NORMALIZATION — mapping legacy shapes to VoteOS-compatible candidates
// ===========================================================================

/// Status of a normalization attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NormalizationStatus {
    /// Successfully mapped to VoteOS shape.
    Normalized,
    /// Mapped but missing required fields.
    Incomplete,
    /// Data is invalid or contradictory.
    Invalid,
    /// Conflicts with existing VoteOS data.
    Conflict,
    /// Legacy format not supported for this record type.
    Unsupported,
}

/// A normalized voter candidate — ready for VoteOS consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVoterCandidate {
    pub legacy_id: String,
    pub full_name: String,
    pub jurisdiction: String,
    pub normalization_status: NormalizationStatus,
    pub normalization_notes: Vec<String>,
    /// Will be populated during identity reconciliation.
    pub reconciled_subject_ref: Option<String>,
    pub reconciliation_status: Option<ReconciliationStatus>,
}

/// A normalized election candidate — ready for VoteOS import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedElectionCandidate {
    pub legacy_id: String,
    pub title: String,
    pub election_type: String,
    pub jurisdiction: String,
    pub voting_method: String,
    pub ballot_items: Vec<NormalizedBallotItemCandidate>,
    pub normalization_status: NormalizationStatus,
    pub normalization_notes: Vec<String>,
}

/// A normalized ballot item candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBallotItemCandidate {
    pub legacy_item_id: String,
    pub title: String,
    pub choices: Vec<String>,
}

// ===========================================================================
// NORMALIZER — pure functions that map legacy → normalized
// ===========================================================================

/// Normalize a legacy voter record.
pub fn normalize_voter(record: &LegacyVoterRecord) -> NormalizedVoterCandidate {
    let mut notes = Vec::new();
    let mut status = NormalizationStatus::Normalized;

    if record.full_name.trim().is_empty() {
        notes.push("Missing full_name".into());
        status = NormalizationStatus::Invalid;
    }

    if record.jurisdiction.trim().is_empty() {
        notes.push("Missing jurisdiction".into());
        if status == NormalizationStatus::Normalized {
            status = NormalizationStatus::Incomplete;
        }
    }

    if record.legacy_id.trim().is_empty() {
        notes.push("Missing legacy_id".into());
        status = NormalizationStatus::Invalid;
    }

    // Check legacy status mapping
    let known_statuses = ["active", "registered", "inactive", "suspended", "pending"];
    if !known_statuses.contains(&record.legacy_status.to_lowercase().as_str()) {
        notes.push(format!("Unknown legacy status: '{}'", record.legacy_status));
        if status == NormalizationStatus::Normalized {
            status = NormalizationStatus::Incomplete;
        }
    }

    NormalizedVoterCandidate {
        legacy_id: record.legacy_id.clone(),
        full_name: record.full_name.clone(),
        jurisdiction: record.jurisdiction.clone(),
        normalization_status: status,
        normalization_notes: notes,
        reconciled_subject_ref: None,
        reconciliation_status: None,
    }
}

/// Normalize a legacy election record.
pub fn normalize_election(record: &LegacyElectionRecord) -> NormalizedElectionCandidate {
    let mut notes = Vec::new();
    let mut status = NormalizationStatus::Normalized;

    if record.title.trim().is_empty() {
        notes.push("Missing title".into());
        status = NormalizationStatus::Invalid;
    }

    if record.ballot_items.is_empty() {
        notes.push("No ballot items".into());
        if status == NormalizationStatus::Normalized {
            status = NormalizationStatus::Incomplete;
        }
    }

    // Map voting method
    let supported_methods = ["plurality", "first-past-the-post", "fptp", "simple majority"];
    if !supported_methods.contains(&record.voting_method.to_lowercase().as_str()) {
        notes.push(format!("Unsupported voting method: '{}' — only plurality supported", record.voting_method));
        if status == NormalizationStatus::Normalized {
            status = NormalizationStatus::Unsupported;
        }
    }

    let ballot_items = record.ballot_items.iter().map(|item| {
        NormalizedBallotItemCandidate {
            legacy_item_id: item.item_id.clone(),
            title: item.title.clone(),
            choices: item.choices.clone(),
        }
    }).collect();

    NormalizedElectionCandidate {
        legacy_id: record.legacy_id.clone(),
        title: record.title.clone(),
        election_type: record.election_type.clone(),
        jurisdiction: record.jurisdiction.clone(),
        voting_method: record.voting_method.clone(),
        ballot_items,
        normalization_status: status,
        normalization_notes: notes,
    }
}

/// Normalize a batch of voter records. Returns (normalized, rejected_count).
pub fn normalize_voter_batch(records: &[LegacyVoterRecord]) -> (Vec<NormalizedVoterCandidate>, usize) {
    let mut normalized = Vec::new();
    let mut rejected = 0;

    for record in records {
        let candidate = normalize_voter(record);
        if candidate.normalization_status == NormalizationStatus::Invalid {
            rejected += 1;
        }
        normalized.push(candidate);
    }

    (normalized, rejected)
}

// ===========================================================================
// IDENTITY RECONCILIATION — mapping legacy identity to system identity
// ===========================================================================

/// Outcome of identity reconciliation for a single record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReconciliationStatus {
    /// Successfully matched to exactly one system identity.
    Matched,
    /// Multiple possible matches — requires human resolution.
    Ambiguous,
    /// No match found in system.
    Missing,
    /// Legacy record is invalid for reconciliation.
    Invalid,
}

/// Result of reconciling a batch of voter candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub total_records: usize,
    pub matched: usize,
    pub ambiguous: usize,
    pub missing: usize,
    pub invalid: usize,
    pub entries: Vec<ReconciliationEntry>,
}

/// Individual reconciliation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationEntry {
    pub legacy_id: String,
    pub full_name: String,
    pub status: ReconciliationStatus,
    pub matched_subject_ref: Option<String>,
    pub candidates: Vec<String>,
    pub notes: String,
}

/// Reconcile voter candidates against a set of known subject references.
///
/// In production, this would call AxiaSystem resolve_subject.
/// Here we use a lookup table for domain-level proof.
pub fn reconcile_voters(
    candidates: &[NormalizedVoterCandidate],
    known_subjects: &BTreeMap<String, String>, // legacy_id → subject_ref
) -> ReconciliationReport {
    let mut matched = 0;
    let mut ambiguous = 0;
    let mut missing = 0;
    let mut invalid = 0;
    let mut entries = Vec::new();

    for candidate in candidates {
        if candidate.normalization_status == NormalizationStatus::Invalid {
            invalid += 1;
            entries.push(ReconciliationEntry {
                legacy_id: candidate.legacy_id.clone(),
                full_name: candidate.full_name.clone(),
                status: ReconciliationStatus::Invalid,
                matched_subject_ref: None,
                candidates: vec![],
                notes: "Normalization was Invalid — cannot reconcile".into(),
            });
            continue;
        }

        match known_subjects.get(&candidate.legacy_id) {
            Some(subject_ref) => {
                matched += 1;
                entries.push(ReconciliationEntry {
                    legacy_id: candidate.legacy_id.clone(),
                    full_name: candidate.full_name.clone(),
                    status: ReconciliationStatus::Matched,
                    matched_subject_ref: Some(subject_ref.clone()),
                    candidates: vec![subject_ref.clone()],
                    notes: "Exact match by legacy_id".into(),
                });
            }
            None => {
                missing += 1;
                entries.push(ReconciliationEntry {
                    legacy_id: candidate.legacy_id.clone(),
                    full_name: candidate.full_name.clone(),
                    status: ReconciliationStatus::Missing,
                    matched_subject_ref: None,
                    candidates: vec![],
                    notes: "No matching subject found in system".into(),
                });
            }
        }
    }

    ReconciliationReport {
        total_records: candidates.len(),
        matched,
        ambiguous,
        missing,
        invalid,
        entries,
    }
}

/// Create an ambiguous reconciliation entry (for testing multi-match scenarios).
pub fn create_ambiguous_entry(legacy_id: &str, name: &str, candidates: Vec<String>) -> ReconciliationEntry {
    ReconciliationEntry {
        legacy_id: legacy_id.into(),
        full_name: name.into(),
        status: ReconciliationStatus::Ambiguous,
        matched_subject_ref: None,
        candidates,
        notes: "Multiple possible matches — requires human resolution".into(),
    }
}

// ===========================================================================
// SHADOW VALIDATION — comparing legacy outcome vs VoteOS recomputation
// ===========================================================================

/// Result of a shadow validation comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShadowComparisonResult {
    /// Legacy outcome matches VoteOS recomputation exactly.
    Match,
    /// Outcomes are semantically equivalent (e.g., same winner, minor count differences).
    SemanticEquivalent,
    /// True mismatch — outcomes differ in a meaningful way.
    TrueMismatch,
    /// Legacy data too incomplete for meaningful comparison.
    LegacyDataIncomplete,
}

/// Detailed shadow validation report for a single election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowValidationReport {
    pub legacy_election_id: String,
    pub overall_result: ShadowComparisonResult,
    pub item_comparisons: Vec<ItemComparison>,
    pub legacy_total_votes: u64,
    pub voteos_total_votes: u64,
    pub notes: Vec<String>,
}

/// Per-item comparison between legacy and VoteOS results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemComparison {
    pub item_id: String,
    pub legacy_winner: Option<String>,
    pub voteos_winner: Option<String>,
    pub legacy_counts: BTreeMap<String, u64>,
    pub voteos_counts: BTreeMap<String, u64>,
    pub result: ShadowComparisonResult,
    pub notes: String,
}

/// Compare a legacy outcome against VoteOS tally results.
///
/// This is the core shadow validation function.
/// It compares per-item, checking winners and counts.
pub fn shadow_validate(
    legacy: &LegacyOutcome,
    voteos_item_tallies: &[crate::domain::tally::ItemTally],
    legacy_election_id: &str,
) -> ShadowValidationReport {
    let mut item_comparisons = Vec::new();
    let mut notes = Vec::new();
    let mut overall = ShadowComparisonResult::Match;

    let voteos_total: u64 = voteos_item_tallies.iter()
        .map(|t| t.total_votes)
        .max()
        .unwrap_or(0);

    if legacy.item_results.is_empty() {
        return ShadowValidationReport {
            legacy_election_id: legacy_election_id.into(),
            overall_result: ShadowComparisonResult::LegacyDataIncomplete,
            item_comparisons: vec![],
            legacy_total_votes: legacy.total_votes_reported,
            voteos_total_votes: voteos_total,
            notes: vec!["Legacy outcome has no item results".into()],
        };
    }

    for legacy_item in &legacy.item_results {
        // Find corresponding VoteOS item
        let voteos_item = voteos_item_tallies.iter()
            .find(|t| t.ballot_item_ref == legacy_item.item_id);

        match voteos_item {
            Some(vi) => {
                let winners_match = legacy_item.winner.as_deref() ==
                    vi.winners.first().map(|s| s.as_str());
                let counts_match = legacy_item.vote_counts == vi.choice_counts;

                let result = if counts_match && winners_match {
                    ShadowComparisonResult::Match
                } else if winners_match && !counts_match {
                    ShadowComparisonResult::SemanticEquivalent
                } else {
                    ShadowComparisonResult::TrueMismatch
                };

                let note = match &result {
                    ShadowComparisonResult::Match => "Exact match".into(),
                    ShadowComparisonResult::SemanticEquivalent =>
                        format!("Same winner but counts differ: legacy {:?} vs voteos {:?}",
                            legacy_item.vote_counts, vi.choice_counts),
                    ShadowComparisonResult::TrueMismatch =>
                        format!("MISMATCH: legacy winner {:?} vs voteos winner {:?}",
                            legacy_item.winner, vi.winners),
                    _ => String::new(),
                };

                if result == ShadowComparisonResult::TrueMismatch {
                    overall = ShadowComparisonResult::TrueMismatch;
                } else if result == ShadowComparisonResult::SemanticEquivalent
                    && overall == ShadowComparisonResult::Match {
                    overall = ShadowComparisonResult::SemanticEquivalent;
                }

                item_comparisons.push(ItemComparison {
                    item_id: legacy_item.item_id.clone(),
                    legacy_winner: legacy_item.winner.clone(),
                    voteos_winner: vi.winners.first().cloned(),
                    legacy_counts: legacy_item.vote_counts.clone(),
                    voteos_counts: vi.choice_counts.clone(),
                    result,
                    notes: note,
                });
            }
            None => {
                notes.push(format!("Legacy item '{}' not found in VoteOS tally", legacy_item.item_id));
                item_comparisons.push(ItemComparison {
                    item_id: legacy_item.item_id.clone(),
                    legacy_winner: legacy_item.winner.clone(),
                    voteos_winner: None,
                    legacy_counts: legacy_item.vote_counts.clone(),
                    voteos_counts: BTreeMap::new(),
                    result: ShadowComparisonResult::LegacyDataIncomplete,
                    notes: "Item not found in VoteOS tally".into(),
                });
                if overall == ShadowComparisonResult::Match {
                    overall = ShadowComparisonResult::LegacyDataIncomplete;
                }
            }
        }
    }

    // Check total vote count
    if legacy.total_votes_reported != voteos_total {
        notes.push(format!(
            "Total vote count differs: legacy {} vs VoteOS {}",
            legacy.total_votes_reported, voteos_total
        ));
    }

    ShadowValidationReport {
        legacy_election_id: legacy_election_id.into(),
        overall_result: overall,
        item_comparisons,
        legacy_total_votes: legacy.total_votes_reported,
        voteos_total_votes: voteos_total,
        notes,
    }
}

// ===========================================================================
// JSON ADAPTER — load legacy data from JSON files
// ===========================================================================

/// Load legacy voter records from a JSON string.
pub fn load_voters_from_json(json: &str) -> Result<Vec<LegacyVoterRecord>, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse voter JSON: {}", e))
}

/// Load a legacy election record from a JSON string.
pub fn load_election_from_json(json: &str) -> Result<LegacyElectionRecord, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse election JSON: {}", e))
}

/// Load legacy official records from a JSON string.
pub fn load_officials_from_json(json: &str) -> Result<Vec<LegacyOfficialRecord>, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse officials JSON: {}", e))
}
