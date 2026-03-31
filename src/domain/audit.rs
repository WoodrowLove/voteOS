//! Audit & Oversight domain types and registry.
//!
//! Module 8: Independent verification of election outcomes.
//! Audit can reconstruct results from stored evidence, detect tampering,
//! and generate bundles for external inspection — without breaking secrecy.
//!
//! The audit module is READ-ONLY with respect to election data.
//! It can verify, challenge, and report — but never mutate results.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;
use crate::domain::elections::{Election, ElectionRegistry, VotingMethod};
use crate::domain::ballots::{BallotRegistry, BallotTemplate};
use crate::domain::votes::{VoteRegistry, VoteContent};
use crate::domain::tally::{
    self, TallyRegistry, TallyResult, TallyStatus,
};
use crate::domain::certification::{CertificationRegistry, CertificationRecord};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Status of an audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    /// Audit has not been started.
    NotStarted,
    /// Audit is in progress (bundle assembled, verification pending).
    InProgress,
    /// Audit completed — tally reconstruction matches certified result.
    Verified,
    /// Audit completed — discrepancies found.
    Failed,
    /// Audit failure has triggered a contest.
    Contested,
}

/// A specific discrepancy found during audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Discrepancy {
    /// Category of discrepancy.
    pub category: DiscrepancyCategory,
    /// Human-readable description.
    pub description: String,
    /// Which ballot item is affected (if applicable).
    pub ballot_item_ref: Option<String>,
    /// Expected value.
    pub expected: String,
    /// Actual value found.
    pub actual: String,
}

/// Categories of detectable discrepancies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscrepancyCategory {
    /// Recomputed tally does not match certified tally.
    TallyMismatch,
    /// Vote count differs between registries.
    VoteCountMismatch,
    /// Input hash of sealed votes does not match stored hash.
    InputHashMismatch,
    /// Missing records in one or more registries.
    MissingRecords,
    /// Cross-module consistency failure.
    ConsistencyFailure,
}

/// Self-contained evidence bundle for an election — everything needed
/// to independently verify the result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBundle {
    /// Election metadata.
    pub election: Election,
    /// Election reference.
    pub election_ref: String,
    /// Finalized ballot template.
    pub ballot_template: Option<BallotTemplate>,
    /// Sealed vote contents (no voter identity — secrecy preserved).
    pub sealed_contents: Vec<(String, VoteContent)>,
    /// Number of sealed votes.
    pub sealed_vote_count: u64,
    /// Ballot item refs from the template.
    pub ballot_item_refs: Vec<String>,
    /// The certified tally result (snapshot from certification).
    pub certified_tally: Option<TallyResult>,
    /// The certification record.
    pub certification: Option<CertificationRecord>,
    /// Voting method used.
    pub voting_method: VotingMethod,
}

/// Result of an audit verification — comparison between reconstructed and certified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVerification {
    /// Whether the reconstruction matches the certified result.
    pub matches: bool,
    /// Reconstructed tally (computed fresh from sealed contents).
    pub reconstructed_tally: TallyResult,
    /// List of discrepancies (empty if matches).
    pub discrepancies: Vec<Discrepancy>,
    /// Input hash computed from sealed contents during audit.
    pub reconstructed_input_hash: String,
    /// Input hash from the certified tally (stored at computation time).
    pub certified_input_hash: String,
}

/// Persistent audit record for an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Election being audited.
    pub election_ref: String,
    /// Current audit status.
    pub status: AuditStatus,
    /// Who initiated the audit.
    pub initiated_by: String,
    /// When the audit was initiated.
    pub initiated_at: String,
    /// When verification completed (if applicable).
    pub completed_at: Option<String>,
    /// Verification result (populated after verification).
    pub verification: Option<AuditVerification>,
    /// Decision ref from legitimacy evaluation.
    pub decision_ref: String,
    /// Contest ref (if audit failure triggered a contest).
    pub contest_ref: Option<String>,
}

/// Audit trail entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
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

pub struct AuditRegistry {
    pub records: DomainStore<AuditRecord>,
    pub audit_log: DomainStore<AuditLogEntry>,
}

impl AuditRegistry {
    pub fn new() -> Self {
        Self {
            records: DomainStore::new("audt"),
            audit_log: DomainStore::new("alog"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            records: DomainStore::with_persistence("audt", dir.join("audit_records.json")),
            audit_log: DomainStore::with_persistence("alog", dir.join("audit_log.json")),
        }
    }

    /// Get the audit record for an election.
    pub fn audit_for_election(&self, election_ref: &str) -> Option<(String, AuditRecord)> {
        self.records.find_all(|r| r.election_ref == election_ref)
            .into_iter().next()
    }

    /// Check if an election has been audited.
    pub fn is_audited(&self, election_ref: &str) -> bool {
        self.records.find_all(|r| {
            r.election_ref == election_ref && r.status == AuditStatus::Verified
        }).into_iter().next().is_some()
    }
}

// ---------------------------------------------------------------------------
// Pure audit functions (no side effects, no mutation)
// ---------------------------------------------------------------------------

/// Assemble an audit bundle from all registries.
/// This is the evidence package an external auditor would receive.
/// Contains NO voter identity information — secrecy is preserved.
pub fn assemble_audit_bundle(
    election_ref: &str,
    election_registry: &ElectionRegistry,
    ballot_registry: &BallotRegistry,
    vote_registry: &VoteRegistry,
    tally_registry: &TallyRegistry,
    cert_registry: &CertificationRegistry,
) -> Option<AuditBundle> {
    let election = election_registry.elections.get(election_ref)?;

    let ballot_template = ballot_registry.finalized_template(election_ref)
        .map(|(_, t)| t);

    let ballot_item_refs = ballot_template.as_ref()
        .map(|t| t.items.iter().map(|i| i.item_ref.clone()).collect())
        .unwrap_or_default();

    let sealed_contents = vote_registry.sealed_contents(election_ref);
    let sealed_vote_count = sealed_contents.len() as u64;

    let certified_tally = tally_registry.result_for_election(election_ref)
        .map(|(_, t)| t);

    let certification = cert_registry.certification_for_election(election_ref)
        .map(|(_, c)| c);

    let voting_method = election.config.voting_method.clone();

    Some(AuditBundle {
        election,
        election_ref: election_ref.to_string(),
        ballot_template,
        sealed_contents,
        sealed_vote_count,
        ballot_item_refs,
        certified_tally,
        certification,
        voting_method,
    })
}

/// Reconstruct the tally from an audit bundle and compare with the certified result.
///
/// This is the core audit operation. It:
/// 1. Recomputes the tally from sealed vote contents using the same deterministic algorithm
/// 2. Compares every field against the certified tally snapshot
/// 3. Reports any discrepancies
///
/// Uses the EXACT same tally functions as the original computation.
pub fn verify_bundle(bundle: &AuditBundle) -> AuditVerification {
    let mut discrepancies = Vec::new();

    // Step 1: Recompute input hash
    let reconstructed_input_hash = tally::compute_input_hash(&bundle.sealed_contents);

    let certified_input_hash = bundle.certified_tally.as_ref()
        .map(|t| t.input_hash.clone())
        .unwrap_or_default();

    // Step 2: Check input hash matches
    if !certified_input_hash.is_empty() && reconstructed_input_hash != certified_input_hash {
        discrepancies.push(Discrepancy {
            category: DiscrepancyCategory::InputHashMismatch,
            description: "Sealed vote contents hash does not match the hash stored at tally time".into(),
            ballot_item_ref: None,
            expected: certified_input_hash.clone(),
            actual: reconstructed_input_hash.clone(),
        });
    }

    // Step 3: Recompute tally using same deterministic logic
    let (reconstructed_items, reconstructed_ambiguity) = match bundle.voting_method {
        VotingMethod::Plurality => {
            tally::compute_plurality_tally(
                &bundle.election_ref,
                &bundle.sealed_contents,
                &bundle.ballot_item_refs,
            )
        }
        _ => {
            // Unsupported method — cannot reconstruct
            discrepancies.push(Discrepancy {
                category: DiscrepancyCategory::ConsistencyFailure,
                description: format!("Voting method {:?} not supported for audit reconstruction", bundle.voting_method),
                ballot_item_ref: None,
                expected: "Plurality".into(),
                actual: format!("{:?}", bundle.voting_method),
            });
            (Vec::new(), false)
        }
    };

    let reconstructed_total = bundle.sealed_contents.len() as u64;
    let reconstructed_status = if reconstructed_total == 0 {
        TallyStatus::Invalid
    } else if reconstructed_ambiguity {
        TallyStatus::Ambiguous
    } else {
        TallyStatus::Computed
    };

    let reconstructed_tally = TallyResult {
        election_ref: bundle.election_ref.clone(),
        method: bundle.voting_method.clone(),
        status: reconstructed_status,
        item_tallies: reconstructed_items,
        total_votes_counted: reconstructed_total,
        computed_at: "audit-reconstruction".into(),
        computed_by: "audit-system".into(),
        decision_ref: "audit-verification".into(),
        input_hash: reconstructed_input_hash.clone(),
        has_ambiguity: reconstructed_ambiguity,
    };

    // Step 4: Compare with certified tally
    if let Some(ref certified) = bundle.certified_tally {
        // Vote count check
        if certified.total_votes_counted != reconstructed_total {
            discrepancies.push(Discrepancy {
                category: DiscrepancyCategory::VoteCountMismatch,
                description: "Total vote count differs".into(),
                ballot_item_ref: None,
                expected: certified.total_votes_counted.to_string(),
                actual: reconstructed_total.to_string(),
            });
        }

        // Per-item comparison
        for (i, certified_item) in certified.item_tallies.iter().enumerate() {
            let reconstructed_item = reconstructed_tally.item_tallies.get(i);

            match reconstructed_item {
                Some(recon) => {
                    // Compare choice counts
                    if recon.choice_counts != certified_item.choice_counts {
                        discrepancies.push(Discrepancy {
                            category: DiscrepancyCategory::TallyMismatch,
                            description: format!(
                                "Choice counts differ for item '{}'",
                                certified_item.ballot_item_ref
                            ),
                            ballot_item_ref: Some(certified_item.ballot_item_ref.clone()),
                            expected: format!("{:?}", certified_item.choice_counts),
                            actual: format!("{:?}", recon.choice_counts),
                        });
                    }

                    // Compare winners
                    if recon.winners != certified_item.winners {
                        discrepancies.push(Discrepancy {
                            category: DiscrepancyCategory::TallyMismatch,
                            description: format!(
                                "Winners differ for item '{}'",
                                certified_item.ballot_item_ref
                            ),
                            ballot_item_ref: Some(certified_item.ballot_item_ref.clone()),
                            expected: format!("{:?}", certified_item.winners),
                            actual: format!("{:?}", recon.winners),
                        });
                    }

                    // Compare tie/ambiguity flags
                    if recon.is_tie != certified_item.is_tie {
                        discrepancies.push(Discrepancy {
                            category: DiscrepancyCategory::TallyMismatch,
                            description: format!(
                                "Tie flag differs for item '{}'",
                                certified_item.ballot_item_ref
                            ),
                            ballot_item_ref: Some(certified_item.ballot_item_ref.clone()),
                            expected: certified_item.is_tie.to_string(),
                            actual: recon.is_tie.to_string(),
                        });
                    }
                }
                None => {
                    discrepancies.push(Discrepancy {
                        category: DiscrepancyCategory::MissingRecords,
                        description: format!(
                            "Certified item '{}' not found in reconstruction (item index {})",
                            certified_item.ballot_item_ref, i
                        ),
                        ballot_item_ref: Some(certified_item.ballot_item_ref.clone()),
                        expected: "present".into(),
                        actual: "missing".into(),
                    });
                }
            }
        }
    }

    // Step 5: Check sealed vote count vs bundle claim
    if bundle.sealed_vote_count != reconstructed_total {
        discrepancies.push(Discrepancy {
            category: DiscrepancyCategory::VoteCountMismatch,
            description: "Bundle sealed_vote_count does not match actual sealed contents".into(),
            ballot_item_ref: None,
            expected: bundle.sealed_vote_count.to_string(),
            actual: reconstructed_total.to_string(),
        });
    }

    let matches = discrepancies.is_empty();

    AuditVerification {
        matches,
        reconstructed_tally,
        discrepancies,
        reconstructed_input_hash,
        certified_input_hash,
    }
}

/// Verify that the audit bundle preserves ballot secrecy.
/// Returns true if no voter identity can be linked to vote content.
pub fn verify_secrecy(bundle: &AuditBundle) -> bool {
    // AuditBundle contains sealed_contents which are Vec<(String, VoteContent)>.
    // VoteContent has: vote_ref, election_ref, selections.
    // It does NOT have voter_ref — secrecy by structural design.
    //
    // The bundle also does NOT contain VoterParticipation records,
    // which are the only place voter_ref appears alongside vote_ref.
    //
    // Therefore, no information in the bundle can link a voter to their vote.
    //
    // Verify: every VoteContent in the bundle has no voter-identifying fields.
    for (_, content) in &bundle.sealed_contents {
        // VoteContent struct: vote_ref, election_ref, selections
        // No voter_ref field exists in the struct — this is compile-time guaranteed.
        // We verify the data is well-formed.
        if content.vote_ref.is_empty() || content.election_ref.is_empty() {
            return false;
        }
    }
    true
}
