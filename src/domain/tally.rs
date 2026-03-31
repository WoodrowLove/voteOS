//! Tally Engine domain types and registry.
//!
//! Module 5: Deterministic vote counting.
//! Given identical sealed vote contents, the tally MUST always produce
//! the same result. No randomness, no ordering sensitivity, no hidden state.

use std::collections::BTreeMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;
use crate::domain::votes::VoteContent;
use crate::domain::elections::VotingMethod;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Status of a tally computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TallyStatus {
    /// Tally has not been computed for this election.
    NotComputed,
    /// Tally computed with a clear winner for every item.
    Computed,
    /// Tally computed but at least one item has a tie or ambiguity.
    Ambiguous,
    /// Tally could not be computed (no votes, invalid data).
    Invalid,
}

/// Result of tallying a single ballot item (race, measure, question).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemTally {
    /// The ballot item being tallied.
    pub ballot_item_ref: String,
    /// Vote counts per choice, sorted by choice_ref for determinism.
    /// Using BTreeMap guarantees consistent ordering.
    pub choice_counts: BTreeMap<String, u64>,
    /// Total valid votes for this item.
    pub total_votes: u64,
    /// Winner(s). Empty if tie or no votes. Multiple if exact tie.
    pub winners: Vec<String>,
    /// Whether this item has an exact tie among top candidates.
    pub is_tie: bool,
    /// Whether any ambiguity exists (tie, no votes, etc.).
    pub is_ambiguous: bool,
    /// Human-readable explanation of the result.
    pub result_summary: String,
}

/// Complete tally result for an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyResult {
    /// Election this tally belongs to.
    pub election_ref: String,
    /// Method used for counting.
    pub method: VotingMethod,
    /// Status of the overall tally.
    pub status: TallyStatus,
    /// Per-item results.
    pub item_tallies: Vec<ItemTally>,
    /// Total sealed votes counted.
    pub total_votes_counted: u64,
    /// Timestamp of computation.
    pub computed_at: String,
    /// Who triggered the computation.
    pub computed_by: String,
    /// Decision ref from legitimacy evaluation.
    pub decision_ref: String,
    /// SHA-256 hash of the input vote contents (for determinism verification).
    pub input_hash: String,
    /// Whether any item is ambiguous.
    pub has_ambiguity: bool,
}

/// Audit entry for tally operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyAuditEntry {
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

pub struct TallyRegistry {
    pub results: DomainStore<TallyResult>,
    pub audit_log: DomainStore<TallyAuditEntry>,
}

impl TallyRegistry {
    pub fn new() -> Self {
        Self {
            results: DomainStore::new("taly"),
            audit_log: DomainStore::new("taud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            results: DomainStore::with_persistence("taly", dir.join("tally_results.json")),
            audit_log: DomainStore::with_persistence("taud", dir.join("tally_audit.json")),
        }
    }

    /// Get the tally result for an election.
    pub fn result_for_election(&self, election_ref: &str) -> Option<(String, TallyResult)> {
        self.results.find_all(|r| r.election_ref == election_ref)
            .into_iter().next()
    }

    /// Check if a tally has been computed for an election.
    pub fn has_tally(&self, election_ref: &str) -> bool {
        self.result_for_election(election_ref).is_some()
    }
}

// ---------------------------------------------------------------------------
// Deterministic tally computation (pure functions)
// ---------------------------------------------------------------------------

/// Compute a deterministic hash of vote contents for reproducibility verification.
pub fn compute_input_hash(contents: &[(String, VoteContent)]) -> String {
    use sha2::{Sha256, Digest};
    // Sort by content ID for deterministic ordering
    let mut sorted: Vec<&(String, VoteContent)> = contents.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let serialized = serde_json::to_string(&sorted.iter().map(|(id, c)| {
        (id.as_str(), &c.selections)
    }).collect::<Vec<_>>()).unwrap_or_default();

    let hash = Sha256::digest(serialized.as_bytes());
    format!("{:x}", hash)
}

/// Compute plurality tally for a single ballot item.
///
/// Each vote contributes exactly one count to the chosen option.
/// Result is deterministic regardless of input order.
pub fn compute_plurality_item(
    ballot_item_ref: &str,
    contents: &[(String, VoteContent)],
) -> ItemTally {
    // BTreeMap ensures deterministic iteration order
    let mut choice_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_votes: u64 = 0;

    for (_, content) in contents {
        for selection in &content.selections {
            if selection.ballot_item_ref == ballot_item_ref {
                *choice_counts.entry(selection.choice_ref.clone()).or_insert(0) += 1;
                total_votes += 1;
            }
        }
    }

    // Handle no votes
    if total_votes == 0 {
        return ItemTally {
            ballot_item_ref: ballot_item_ref.to_string(),
            choice_counts,
            total_votes: 0,
            winners: Vec::new(),
            is_tie: false,
            is_ambiguous: true,
            result_summary: "No votes cast for this item".to_string(),
        };
    }

    // Find maximum count
    let max_count = choice_counts.values().copied().max().unwrap_or(0);

    // Collect all choices with the maximum count (sorted for determinism)
    let winners: Vec<String> = choice_counts
        .iter()
        .filter(|(_, &count)| count == max_count)
        .map(|(choice, _)| choice.clone())
        .collect(); // Already sorted because BTreeMap

    let is_tie = winners.len() > 1;
    let is_ambiguous = is_tie;

    let result_summary = if is_tie {
        format!(
            "Tie between {} choices with {} votes each",
            winners.len(),
            max_count
        )
    } else {
        format!(
            "{} wins with {} of {} votes ({:.1}%)",
            winners[0],
            max_count,
            total_votes,
            (max_count as f64 / total_votes as f64) * 100.0
        )
    };

    ItemTally {
        ballot_item_ref: ballot_item_ref.to_string(),
        choice_counts,
        total_votes,
        winners,
        is_tie,
        is_ambiguous,
        result_summary,
    }
}

/// Compute a complete plurality tally for all items in an election.
///
/// This is a pure function. Given identical inputs, it always produces
/// the same output. No side effects, no randomness.
pub fn compute_plurality_tally(
    _election_ref: &str,
    contents: &[(String, VoteContent)],
    ballot_item_refs: &[String],
) -> (Vec<ItemTally>, bool) {
    let mut item_tallies = Vec::new();
    let mut has_ambiguity = false;

    for item_ref in ballot_item_refs {
        let item_tally = compute_plurality_item(item_ref, contents);
        if item_tally.is_ambiguous {
            has_ambiguity = true;
        }
        item_tallies.push(item_tally);
    }

    // If no items were tallied, that's also ambiguous
    if item_tallies.is_empty() && !ballot_item_refs.is_empty() {
        has_ambiguity = true;
    }

    (item_tallies, has_ambiguity)
}

/// Evaluate participation threshold.
/// Returns (met, actual_percentage).
pub fn evaluate_participation_threshold(
    total_votes: u64,
    total_eligible: u64,
    threshold: f64,
) -> (bool, f64) {
    if total_eligible == 0 {
        return (false, 0.0);
    }
    let percentage = (total_votes as f64 / total_eligible as f64) * 100.0;
    (percentage >= threshold, percentage)
}

/// Evaluate margin threshold for a single item.
/// Returns (met, actual_margin_percentage).
pub fn evaluate_margin_threshold(
    item_tally: &ItemTally,
    threshold: f64,
) -> (bool, f64) {
    if item_tally.total_votes == 0 || item_tally.is_tie {
        return (false, 0.0);
    }

    let counts: Vec<u64> = item_tally.choice_counts.values().copied().collect();
    if counts.len() < 2 {
        // Only one choice — margin is 100%
        return (true, 100.0);
    }

    let mut sorted_counts = counts.clone();
    sorted_counts.sort_unstable_by(|a, b| b.cmp(a));

    let first = sorted_counts[0] as f64;
    let second = sorted_counts[1] as f64;
    let margin = ((first - second) / item_tally.total_votes as f64) * 100.0;

    (margin >= threshold, margin)
}
