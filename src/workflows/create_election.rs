//! Capability: create_election (governance_action)
//!
//! Create a new election with type, scope, and schedule.
//! Chain: evaluate_legitimacy → create in store → attest → explain

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::elections::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateElectionInput {
    pub official_ref: String,
    pub session_ref: String,
    pub title: String,
    pub description: String,
    pub election_type: ElectionType,
    pub scope: String,
    pub config: Option<ElectionConfig>,
    pub schedule: Option<ElectionSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateElectionResult {
    pub election_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn execute(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: CreateElectionInput,
) -> WorkflowResult<CreateElectionResult> {
    // Step 1: Evaluate legitimacy (governance_action)
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "create_election".to_string(),
            target: Some(input.scope.clone()),
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

    // Step 2: Create election in Draft status
    let timestamp = chrono::Utc::now().to_rfc3339();
    let config = input.config.unwrap_or_default();
    let schedule = input.schedule.unwrap_or(ElectionSchedule {
        registration_start: None,
        registration_end: None,
        voting_start: None,
        voting_end: None,
        certification_deadline: None,
    });

    let election_ref = registry.elections.insert_new(Election {
        title: input.title.clone(),
        description: input.description.clone(),
        election_type: input.election_type,
        status: ElectionStatus::Draft,
        config,
        schedule,
        scope: input.scope.clone(),
        created_by: input.official_ref.clone(),
        created_at: timestamp.clone(),
        decision_ref: decision_ref.clone(),
    });

    // Auto-assign creator as an official
    registry.officials.insert_new(ElectionOfficial {
        subject_ref: input.official_ref.clone(),
        election_ref: election_ref.clone(),
        role: "creator".to_string(),
        assigned_by: input.official_ref.clone(),
        assigned_at: timestamp.clone(),
        revoked: false,
        decision_ref: decision_ref.clone(),
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: election_ref.clone(),
            action_type: "create_election".to_string(),
            summary: format!("Created election: {}", input.title),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "election_creation".to_string(),
            additional_context: Some(input.description),
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

    // Record initial transition
    registry.transitions.insert_new(ElectionTransition {
        election_ref: election_ref.clone(),
        from_status: ElectionStatus::Draft,
        to_status: ElectionStatus::Draft,
        actor_ref: input.official_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        reason: Some("Election created".to_string()),
    });

    Ok(CreateElectionResult {
        election_ref,
        decision_ref,
        attestation_ref,
    })
}
