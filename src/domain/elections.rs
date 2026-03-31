//! Election Management domain types and registry.
//!
//! Module 2: Create, configure, and control election lifecycle.
//! State machine: DRAFT → PUBLISHED → OPEN → CLOSED → TALLIED → CERTIFIED

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::domain::store::DomainStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElectionStatus {
    Draft,
    Published,
    Open,
    Closed,
    Tallied,
    Certified,
    Cancelled,
}

impl ElectionStatus {
    /// Valid transitions for the election state machine.
    pub fn can_transition_to(&self, target: &ElectionStatus) -> bool {
        matches!(
            (self, target),
            (ElectionStatus::Draft, ElectionStatus::Published)
                | (ElectionStatus::Draft, ElectionStatus::Cancelled)
                | (ElectionStatus::Published, ElectionStatus::Open)
                | (ElectionStatus::Published, ElectionStatus::Cancelled)
                | (ElectionStatus::Open, ElectionStatus::Closed)
                | (ElectionStatus::Open, ElectionStatus::Cancelled)
                | (ElectionStatus::Closed, ElectionStatus::Tallied)
                | (ElectionStatus::Tallied, ElectionStatus::Certified)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElectionType {
    General,
    Primary,
    Special,
    Referendum,
    Recall,
    Initiative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyMode {
    SecretBallot,
    RollCall,
    Configurable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VotingMethod {
    Plurality,
    RankedChoice,
    Approval,
    ScoreVoting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionConfig {
    pub privacy_mode: PrivacyMode,
    pub voting_method: VotingMethod,
    pub participation_threshold: Option<f64>,
    pub margin_threshold: Option<f64>,
    pub max_choices_per_item: Option<u32>,
}

impl Default for ElectionConfig {
    fn default() -> Self {
        Self {
            privacy_mode: PrivacyMode::SecretBallot,
            voting_method: VotingMethod::Plurality,
            participation_threshold: None,
            margin_threshold: None,
            max_choices_per_item: Some(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionSchedule {
    pub registration_start: Option<String>,
    pub registration_end: Option<String>,
    pub voting_start: Option<String>,
    pub voting_end: Option<String>,
    pub certification_deadline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Election {
    pub title: String,
    pub description: String,
    pub election_type: ElectionType,
    pub status: ElectionStatus,
    pub config: ElectionConfig,
    pub schedule: ElectionSchedule,
    pub scope: String,
    pub created_by: String,
    pub created_at: String,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionOfficial {
    pub subject_ref: String,
    pub election_ref: String,
    pub role: String,
    pub assigned_by: String,
    pub assigned_at: String,
    pub revoked: bool,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionTransition {
    pub election_ref: String,
    pub from_status: ElectionStatus,
    pub to_status: ElectionStatus,
    pub actor_ref: String,
    pub timestamp: String,
    pub decision_ref: String,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct ElectionRegistry {
    pub elections: DomainStore<Election>,
    pub officials: DomainStore<ElectionOfficial>,
    pub transitions: DomainStore<ElectionTransition>,
}

impl ElectionRegistry {
    pub fn new() -> Self {
        Self {
            elections: DomainStore::new("elec"),
            officials: DomainStore::new("eoff"),
            transitions: DomainStore::new("etrn"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            elections: DomainStore::with_persistence("elec", dir.join("elections.json")),
            officials: DomainStore::with_persistence("eoff", dir.join("election_officials.json")),
            transitions: DomainStore::with_persistence("etrn", dir.join("election_transitions.json")),
        }
    }

    /// Check if a subject is an active official for an election.
    pub fn is_official(&self, subject_ref: &str, election_ref: &str) -> bool {
        !self.officials.find_all(|o| {
            o.subject_ref == subject_ref
                && o.election_ref == election_ref
                && !o.revoked
        }).is_empty()
    }

    /// Get all active officials for an election.
    pub fn officials_for_election(&self, election_ref: &str) -> Vec<(String, ElectionOfficial)> {
        self.officials.find_all(|o| o.election_ref == election_ref && !o.revoked)
    }

    /// Get transition history for an election.
    pub fn transition_history(&self, election_ref: &str) -> Vec<(String, ElectionTransition)> {
        self.transitions.find_all(|t| t.election_ref == election_ref)
    }

    /// Transition an election to a new status. Returns error message if invalid.
    pub fn transition_election(
        &self,
        election_ref: &str,
        target_status: ElectionStatus,
        actor_ref: &str,
        decision_ref: &str,
        reason: Option<String>,
    ) -> Result<(), String> {
        let election = self.elections.get(election_ref)
            .ok_or_else(|| format!("Election {} not found", election_ref))?;

        if !election.status.can_transition_to(&target_status) {
            return Err(format!(
                "Invalid transition: {:?} → {:?}",
                election.status, target_status
            ));
        }

        let timestamp = chrono::Utc::now().to_rfc3339();

        // Record transition
        self.transitions.insert_new(ElectionTransition {
            election_ref: election_ref.to_string(),
            from_status: election.status.clone(),
            to_status: target_status.clone(),
            actor_ref: actor_ref.to_string(),
            timestamp: timestamp.clone(),
            decision_ref: decision_ref.to_string(),
            reason,
        });

        // Update election status
        let mut updated = election;
        updated.status = target_status;
        self.elections.update(election_ref, updated);

        Ok(())
    }
}
