//! Voter Registry domain types and registry.
//!
//! Module 1: Who can vote in which elections.
//! VoteOS does NOT onboard citizens — it verifies eligibility of identities
//! already in AxiaSystem.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::domain::store::DomainStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RegistrationStatus {
    Registered,
    Pending,
    Suspended,
    Ineligible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterRegistration {
    pub citizen_ref: String,
    pub election_ref: String,
    pub status: RegistrationStatus,
    pub registered_at: String,
    pub registered_by: String,
    pub eligibility_basis: String,
    pub decision_ref: String,
    pub attestation_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    AgeMinimum,
    Jurisdiction,
    Standing,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityRule {
    pub election_ref: String,
    pub rule_type: RuleType,
    pub criteria: String,
    pub defined_by: String,
    pub defined_at: String,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChallengeStatus {
    Filed,
    UnderReview,
    Upheld,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityChallenge {
    pub voter_registration_ref: String,
    pub election_ref: String,
    pub challenger_ref: String,
    pub reason: String,
    pub status: ChallengeStatus,
    pub filed_at: String,
    pub resolution: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterRoll {
    pub election_ref: String,
    pub voter_refs: Vec<String>,
    pub generated_at: String,
    pub generated_by: String,
    pub total_eligible: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterAuditEntry {
    pub action: String,
    pub actor_ref: String,
    pub target_ref: Option<String>,
    pub election_ref: Option<String>,
    pub timestamp: String,
    pub decision_ref: String,
    pub details: String,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct VoterRegistry {
    pub registrations: DomainStore<VoterRegistration>,
    pub rules: DomainStore<EligibilityRule>,
    pub challenges: DomainStore<EligibilityChallenge>,
    pub rolls: DomainStore<VoterRoll>,
    pub audit_log: DomainStore<VoterAuditEntry>,
}

impl VoterRegistry {
    pub fn new() -> Self {
        Self {
            registrations: DomainStore::new("vreg"),
            rules: DomainStore::new("vrul"),
            challenges: DomainStore::new("vchl"),
            rolls: DomainStore::new("vrol"),
            audit_log: DomainStore::new("vaud"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            registrations: DomainStore::with_persistence("vreg", dir.join("voter_registrations.json")),
            rules: DomainStore::with_persistence("vrul", dir.join("eligibility_rules.json")),
            challenges: DomainStore::with_persistence("vchl", dir.join("eligibility_challenges.json")),
            rolls: DomainStore::with_persistence("vrol", dir.join("voter_rolls.json")),
            audit_log: DomainStore::with_persistence("vaud", dir.join("voter_audit.json")),
        }
    }

    /// Check if a citizen is already registered for an election.
    pub fn is_registered(&self, citizen_ref: &str, election_ref: &str) -> bool {
        !self.registrations.find_all(|r| {
            r.citizen_ref == citizen_ref
                && r.election_ref == election_ref
                && r.status == RegistrationStatus::Registered
        }).is_empty()
    }

    /// Find a voter's registration for a specific election.
    pub fn find_registration(&self, citizen_ref: &str, election_ref: &str) -> Option<(String, VoterRegistration)> {
        self.registrations.find_all(|r| {
            r.citizen_ref == citizen_ref && r.election_ref == election_ref
        }).into_iter().next()
    }

    /// Get all registered voters for an election.
    pub fn voters_for_election(&self, election_ref: &str) -> Vec<(String, VoterRegistration)> {
        self.registrations.find_all(|r| {
            r.election_ref == election_ref && r.status == RegistrationStatus::Registered
        })
    }

    /// Get eligibility rules for an election.
    pub fn rules_for_election(&self, election_ref: &str) -> Vec<(String, EligibilityRule)> {
        self.rules.find_all(|r| r.election_ref == election_ref)
    }

    /// Get voter registration statistics for an election (no PII).
    pub fn election_statistics(&self, election_ref: &str) -> VoterStatistics {
        let all = self.registrations.find_all(|r| r.election_ref == election_ref);
        let registered = all.iter().filter(|(_, r)| r.status == RegistrationStatus::Registered).count();
        let pending = all.iter().filter(|(_, r)| r.status == RegistrationStatus::Pending).count();
        let suspended = all.iter().filter(|(_, r)| r.status == RegistrationStatus::Suspended).count();
        let ineligible = all.iter().filter(|(_, r)| r.status == RegistrationStatus::Ineligible).count();
        VoterStatistics { election_ref: election_ref.to_string(), total: all.len(), registered, pending, suspended, ineligible }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterStatistics {
    pub election_ref: String,
    pub total: usize,
    pub registered: usize,
    pub pending: usize,
    pub suspended: usize,
    pub ineligible: usize,
}
