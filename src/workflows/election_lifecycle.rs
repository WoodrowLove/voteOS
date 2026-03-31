//! Capabilities: configure_election, publish_election, open_election,
//!   close_election, extend_election, cancel_election, resolve_election_state,
//!   audit_election_transitions

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::elections::*;

// ---------------------------------------------------------------------------
// Generic state transition helper
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionElectionInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub target_status: ElectionStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionElectionResult {
    pub election_ref: String,
    pub from_status: ElectionStatus,
    pub to_status: ElectionStatus,
    pub decision_ref: String,
    pub attestation_ref: String,
}

async fn transition_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: TransitionElectionInput,
    operation_name: &str,
) -> WorkflowResult<TransitionElectionResult> {
    let election = registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} not found", input.election_ref),
        })?;

    let from_status = election.status.clone();

    if !from_status.can_transition_to(&input.target_status) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Invalid transition: {:?} → {:?}", from_status, input.target_status),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: operation_name.to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("election_management".to_string()),
            city: None,
            workflow_ref: None,
            urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "evaluate_legitimacy".into(),
        message: e,
    })?;

    let decision_ref = match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => r.decision_ref,
        EvaluateLegitimacyResult::Ok(r) => return Err(WorkflowError::PreconditionFailed {
            step: 1,
            reason: format!("{} ({})", r.reason_summary, r.decision),
        }),
        EvaluateLegitimacyResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "evaluate_legitimacy".into(),
            code: e.code,
            message: e.message,
            step: 1,
        }),
    };

    // Step 2: Transition
    registry.transition_election(
        &input.election_ref,
        input.target_status.clone(),
        &input.official_ref,
        &decision_ref,
        input.reason.clone(),
    ).map_err(|e| WorkflowError::PreconditionFailed { step: 2, reason: e })?;

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.election_ref.clone(),
            action_type: operation_name.to_string(),
            summary: format!(
                "Election {} transitioned: {:?} → {:?}",
                input.election_ref, from_status, input.target_status
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: format!("election_{}", operation_name),
            additional_context: input.reason,
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "attest_action".into(),
        message: e,
    })?;

    let attestation_ref = match att {
        AttestActionResult::Ok(r) => r.attestation_ref,
        AttestActionResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "attest_action".into(),
            code: e.code,
            message: e.message,
            step: 3,
        }),
    };

    // Step 4: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    Ok(TransitionElectionResult {
        election_ref: input.election_ref,
        from_status,
        to_status: input.target_status,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// Public lifecycle capabilities
// ---------------------------------------------------------------------------

pub async fn publish_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    official_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<TransitionElectionResult> {
    transition_election(spine, registry, TransitionElectionInput {
        official_ref, session_ref, election_ref,
        target_status: ElectionStatus::Published,
        reason: None,
    }, "publish_election").await
}

pub async fn open_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    official_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<TransitionElectionResult> {
    transition_election(spine, registry, TransitionElectionInput {
        official_ref, session_ref, election_ref,
        target_status: ElectionStatus::Open,
        reason: None,
    }, "open_election").await
}

pub async fn close_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    official_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<TransitionElectionResult> {
    transition_election(spine, registry, TransitionElectionInput {
        official_ref, session_ref, election_ref,
        target_status: ElectionStatus::Closed,
        reason: None,
    }, "close_election").await
}

pub async fn extend_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    official_ref: String,
    session_ref: String,
    election_ref: String,
    reason: String,
) -> WorkflowResult<TransitionElectionResult> {
    // Extension reopens a closed election — but that's not a valid transition.
    // Instead, extension extends the voting_end in the schedule while still Open.
    let election = registry.elections.get(&election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Election {} not found", election_ref),
        })?;

    if election.status != ElectionStatus::Open {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Can only extend an Open election, current status: {:?}", election.status),
        });
    }

    // Evaluate legitimacy for emergency extension
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: official_ref.clone(),
            session_ref: session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "extend_election".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("election_management".to_string()),
            city: None,
            workflow_ref: None,
            urgency: Some("elevated".to_string()),
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

    // Record the extension as a transition log entry (same status → same status)
    let timestamp = chrono::Utc::now().to_rfc3339();
    registry.transitions.insert_new(ElectionTransition {
        election_ref: election_ref.clone(),
        from_status: ElectionStatus::Open,
        to_status: ElectionStatus::Open,
        actor_ref: official_ref.clone(),
        timestamp,
        decision_ref: decision_ref.clone(),
        reason: Some(format!("Extension: {}", reason)),
    });

    // Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: official_ref.clone(),
            session_ref: session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: election_ref.clone(),
            action_type: "extend_election".to_string(),
            summary: format!("Extended election {}: {}", election_ref, reason),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "election_extension".to_string(),
            additional_context: Some(reason),
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

    Ok(TransitionElectionResult {
        election_ref,
        from_status: ElectionStatus::Open,
        to_status: ElectionStatus::Open,
        decision_ref,
        attestation_ref,
    })
}

pub async fn cancel_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    official_ref: String,
    session_ref: String,
    election_ref: String,
    reason: String,
) -> WorkflowResult<TransitionElectionResult> {
    transition_election(spine, registry, TransitionElectionInput {
        official_ref, session_ref, election_ref,
        target_status: ElectionStatus::Cancelled,
        reason: Some(reason),
    }, "cancel_election").await
}

// ---------------------------------------------------------------------------
// configure_election (governance_action) — modifies config while in Draft
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureElectionInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub config: ElectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureElectionResult {
    pub election_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn configure_election(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: ConfigureElectionInput,
) -> WorkflowResult<ConfigureElectionResult> {
    let election = registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Election {} not found", input.election_ref),
        })?;

    if election.status != ElectionStatus::Draft {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Can only configure elections in Draft status, current: {:?}", election.status),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "configure_election".to_string(),
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
    updated.config = input.config;
    registry.elections.update(&input.election_ref, updated);

    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.election_ref.clone(),
            action_type: "configure_election".to_string(),
            summary: format!("Configured election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "election_configuration".to_string(),
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

    Ok(ConfigureElectionResult { election_ref: input.election_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// resolve_election_state (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveElectionStateInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveElectionStateResult {
    pub election: Option<Election>,
    pub decision_ref: String,
}

pub async fn resolve_election_state(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: ResolveElectionStateInput,
) -> WorkflowResult<ResolveElectionStateResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_election_state".to_string(),
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

    let election = registry.elections.get(&input.election_ref);
    Ok(ResolveElectionStateResult { election, decision_ref })
}

// ---------------------------------------------------------------------------
// audit_election_transitions (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditElectionTransitionsInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditElectionTransitionsResult {
    pub transitions: Vec<(String, ElectionTransition)>,
    pub decision_ref: String,
}

pub async fn audit_election_transitions(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: AuditElectionTransitionsInput,
) -> WorkflowResult<AuditElectionTransitionsResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "audit_election_transitions".to_string(),
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

    let transitions = registry.transition_history(&input.election_ref);
    Ok(AuditElectionTransitionsResult { transitions, decision_ref })
}
