//! Capabilities: configure_voting_method, set_election_schedule,
//!   resolve_election_timeline

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::elections::*;

// ---------------------------------------------------------------------------
// configure_voting_method (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureVotingMethodInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub voting_method: VotingMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureVotingMethodResult {
    pub election_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn configure_voting_method(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: ConfigureVotingMethodInput,
) -> WorkflowResult<ConfigureVotingMethodResult> {
    let election = registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Election {} not found", input.election_ref),
        })?;

    if election.status != ElectionStatus::Draft {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Can only configure voting method in Draft, current: {:?}", election.status),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "configure_voting_method".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("election_management".to_string()),
            city: None, workflow_ref: None, urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "evaluate_legitimacy".into(), message: e,
    })?;

    let decision_ref = match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => r.decision_ref,
        EvaluateLegitimacyResult::Ok(r) => return Err(WorkflowError::PreconditionFailed {
            step: 1, reason: format!("{} ({})", r.reason_summary, r.decision),
        }),
        EvaluateLegitimacyResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "evaluate_legitimacy".into(), code: e.code, message: e.message, step: 1,
        }),
    };

    let mut updated = election;
    updated.config.voting_method = input.voting_method;
    registry.elections.update(&input.election_ref, updated);

    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.election_ref.clone(),
            action_type: "configure_voting_method".to_string(),
            summary: format!("Configured voting method for election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "voting_method_configuration".to_string(),
            additional_context: None,
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "attest_action".into(), message: e,
    })?;

    let attestation_ref = match att {
        AttestActionResult::Ok(r) => r.attestation_ref,
        AttestActionResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "attest_action".into(), code: e.code, message: e.message, step: 3,
        }),
    };

    Ok(ConfigureVotingMethodResult { election_ref: input.election_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// set_election_schedule (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetElectionScheduleInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub schedule: ElectionSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetElectionScheduleResult {
    pub election_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn set_election_schedule(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: SetElectionScheduleInput,
) -> WorkflowResult<SetElectionScheduleResult> {
    let election = registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Election {} not found", input.election_ref),
        })?;

    if election.status != ElectionStatus::Draft && election.status != ElectionStatus::Published {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Can only set schedule in Draft/Published, current: {:?}", election.status),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "set_election_schedule".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("election_management".to_string()),
            city: None, workflow_ref: None, urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "evaluate_legitimacy".into(), message: e,
    })?;

    let decision_ref = match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => r.decision_ref,
        EvaluateLegitimacyResult::Ok(r) => return Err(WorkflowError::PreconditionFailed {
            step: 1, reason: format!("{} ({})", r.reason_summary, r.decision),
        }),
        EvaluateLegitimacyResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "evaluate_legitimacy".into(), code: e.code, message: e.message, step: 1,
        }),
    };

    let mut updated = election;
    updated.schedule = input.schedule;
    registry.elections.update(&input.election_ref, updated);

    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.election_ref.clone(),
            action_type: "set_election_schedule".to_string(),
            summary: format!("Set schedule for election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "election_scheduling".to_string(),
            additional_context: None,
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "attest_action".into(), message: e,
    })?;

    let attestation_ref = match att {
        AttestActionResult::Ok(r) => r.attestation_ref,
        AttestActionResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "attest_action".into(), code: e.code, message: e.message, step: 3,
        }),
    };

    Ok(SetElectionScheduleResult { election_ref: input.election_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// resolve_election_timeline (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveTimelineInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveTimelineResult {
    pub schedule: Option<ElectionSchedule>,
    pub decision_ref: String,
}

pub async fn resolve_election_timeline(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: ResolveTimelineInput,
) -> WorkflowResult<ResolveTimelineResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_election_timeline".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("election_management".to_string()),
            city: None, workflow_ref: None, urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "evaluate_legitimacy".into(), message: e,
    })?;

    let decision_ref = match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => r.decision_ref,
        EvaluateLegitimacyResult::Ok(r) => return Err(WorkflowError::PreconditionFailed {
            step: 1, reason: format!("{} ({})", r.reason_summary, r.decision),
        }),
        EvaluateLegitimacyResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "evaluate_legitimacy".into(), code: e.code, message: e.message, step: 1,
        }),
    };

    let schedule = registry.elections.get(&input.election_ref).map(|e| e.schedule);
    Ok(ResolveTimelineResult { schedule, decision_ref })
}
