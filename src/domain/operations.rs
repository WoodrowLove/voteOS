//! Election Operations domain types and registry.
//!
//! Module 9: Operational control layer for elections.
//! Scheduling, incident management, operational state.
//!
//! CRITICAL: Operations NEVER alter truth.
//! Operations can schedule, pause, resume, and flag — but cannot
//! modify votes, override tallies, bypass certification, or mutate audit data.

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::domain::store::DomainStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Operational state of an election.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationalStatus {
    /// Normal operation.
    Normal,
    /// Election paused (voting temporarily halted).
    Paused,
    /// Incident flagged — requires attention.
    IncidentFlagged,
    /// Force-closed by operator.
    ForceClosed,
}

/// Scheduling record for an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionScheduleRecord {
    /// Election this schedule applies to.
    pub election_ref: String,
    /// When voting should open (ISO 8601).
    pub opens_at: Option<String>,
    /// When voting should close (ISO 8601).
    pub closes_at: Option<String>,
    /// Timezone for display purposes.
    pub timezone: String,
    /// Whether auto-open/close is enabled.
    pub auto_transition: bool,
    /// Who created the schedule.
    pub created_by: String,
    /// When created.
    pub created_at: String,
}

/// Operational state record for an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalState {
    /// Election reference.
    pub election_ref: String,
    /// Current operational status.
    pub status: OperationalStatus,
    /// Whether election is paused.
    pub paused: bool,
    /// Whether an incident has been flagged.
    pub incident_flag: bool,
    /// Operator notes.
    pub notes: Vec<String>,
    /// Last updated.
    pub updated_at: String,
    /// Last updated by.
    pub updated_by: String,
}

/// Record of an operator action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorAction {
    /// Election this action applies to.
    pub election_ref: String,
    /// Type of action.
    pub action_type: OperatorActionType,
    /// Who performed the action.
    pub performed_by: String,
    /// When performed.
    pub performed_at: String,
    /// Reason for the action.
    pub reason: String,
}

/// Types of operator actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperatorActionType {
    Pause,
    Resume,
    FlagIncident,
    ResolveIncident,
    ForceClose,
    UpdateSchedule,
    AddNote,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct OperationsRegistry {
    pub schedules: DomainStore<ElectionScheduleRecord>,
    pub states: DomainStore<OperationalState>,
    pub actions: DomainStore<OperatorAction>,
}

impl OperationsRegistry {
    pub fn new() -> Self {
        Self {
            schedules: DomainStore::new("sched"),
            states: DomainStore::new("opst"),
            actions: DomainStore::new("oact"),
        }
    }

    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            schedules: DomainStore::with_persistence("sched", dir.join("schedules.json")),
            states: DomainStore::with_persistence("opst", dir.join("operational_states.json")),
            actions: DomainStore::with_persistence("oact", dir.join("operator_actions.json")),
        }
    }

    /// Get the schedule for an election.
    pub fn schedule_for_election(&self, election_ref: &str) -> Option<(String, ElectionScheduleRecord)> {
        self.schedules.find_all(|s| s.election_ref == election_ref)
            .into_iter().next()
    }

    /// Get the operational state for an election.
    pub fn state_for_election(&self, election_ref: &str) -> Option<(String, OperationalState)> {
        self.states.find_all(|s| s.election_ref == election_ref)
            .into_iter().next()
    }

    /// Check if an election is paused.
    pub fn is_paused(&self, election_ref: &str) -> bool {
        self.state_for_election(election_ref)
            .map(|(_, s)| s.paused)
            .unwrap_or(false)
    }

    /// Check if an election has an active incident.
    pub fn has_incident(&self, election_ref: &str) -> bool {
        self.state_for_election(election_ref)
            .map(|(_, s)| s.incident_flag)
            .unwrap_or(false)
    }

    /// Get action history for an election.
    pub fn actions_for_election(&self, election_ref: &str) -> Vec<(String, OperatorAction)> {
        self.actions.find_all(|a| a.election_ref == election_ref)
    }

    /// Initialize operational state for an election (if not exists).
    pub fn ensure_state(&self, election_ref: &str, operator: &str) -> String {
        if let Some((id, _)) = self.state_for_election(election_ref) {
            return id;
        }
        self.states.insert_new(OperationalState {
            election_ref: election_ref.to_string(),
            status: OperationalStatus::Normal,
            paused: false,
            incident_flag: false,
            notes: Vec::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            updated_by: operator.to_string(),
        })
    }

    /// Pause an election. Returns error if already paused.
    pub fn pause(&self, election_ref: &str, operator: &str, reason: &str) -> Result<(), String> {
        let (state_id, mut state) = self.state_for_election(election_ref)
            .ok_or_else(|| "No operational state found".to_string())?;

        if state.paused {
            return Err("Election is already paused".into());
        }

        state.paused = true;
        state.status = OperationalStatus::Paused;
        state.updated_at = chrono::Utc::now().to_rfc3339();
        state.updated_by = operator.to_string();
        state.notes.push(format!("PAUSED: {}", reason));
        self.states.update(&state_id, state);

        self.actions.insert_new(OperatorAction {
            election_ref: election_ref.to_string(),
            action_type: OperatorActionType::Pause,
            performed_by: operator.to_string(),
            performed_at: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
        });

        Ok(())
    }

    /// Resume a paused election.
    pub fn resume(&self, election_ref: &str, operator: &str, reason: &str) -> Result<(), String> {
        let (state_id, mut state) = self.state_for_election(election_ref)
            .ok_or_else(|| "No operational state found".to_string())?;

        if !state.paused {
            return Err("Election is not paused".into());
        }

        state.paused = false;
        state.status = if state.incident_flag {
            OperationalStatus::IncidentFlagged
        } else {
            OperationalStatus::Normal
        };
        state.updated_at = chrono::Utc::now().to_rfc3339();
        state.updated_by = operator.to_string();
        state.notes.push(format!("RESUMED: {}", reason));
        self.states.update(&state_id, state);

        self.actions.insert_new(OperatorAction {
            election_ref: election_ref.to_string(),
            action_type: OperatorActionType::Resume,
            performed_by: operator.to_string(),
            performed_at: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
        });

        Ok(())
    }

    /// Flag an incident on an election.
    pub fn flag_incident(&self, election_ref: &str, operator: &str, description: &str) -> Result<(), String> {
        let (state_id, mut state) = self.state_for_election(election_ref)
            .ok_or_else(|| "No operational state found".to_string())?;

        state.incident_flag = true;
        state.status = OperationalStatus::IncidentFlagged;
        state.updated_at = chrono::Utc::now().to_rfc3339();
        state.updated_by = operator.to_string();
        state.notes.push(format!("INCIDENT: {}", description));
        self.states.update(&state_id, state);

        self.actions.insert_new(OperatorAction {
            election_ref: election_ref.to_string(),
            action_type: OperatorActionType::FlagIncident,
            performed_by: operator.to_string(),
            performed_at: chrono::Utc::now().to_rfc3339(),
            reason: description.to_string(),
        });

        Ok(())
    }

    /// Resolve an incident.
    pub fn resolve_incident(&self, election_ref: &str, operator: &str, resolution: &str) -> Result<(), String> {
        let (state_id, mut state) = self.state_for_election(election_ref)
            .ok_or_else(|| "No operational state found".to_string())?;

        if !state.incident_flag {
            return Err("No active incident".into());
        }

        state.incident_flag = false;
        state.status = if state.paused {
            OperationalStatus::Paused
        } else {
            OperationalStatus::Normal
        };
        state.updated_at = chrono::Utc::now().to_rfc3339();
        state.updated_by = operator.to_string();
        state.notes.push(format!("RESOLVED: {}", resolution));
        self.states.update(&state_id, state);

        self.actions.insert_new(OperatorAction {
            election_ref: election_ref.to_string(),
            action_type: OperatorActionType::ResolveIncident,
            performed_by: operator.to_string(),
            performed_at: chrono::Utc::now().to_rfc3339(),
            reason: resolution.to_string(),
        });

        Ok(())
    }
}
