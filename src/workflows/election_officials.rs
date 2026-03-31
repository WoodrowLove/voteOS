//! Capabilities: assign_election_official, revoke_election_official (governance_action)

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::elections::*;

// ---------------------------------------------------------------------------
// assign_election_official
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignOfficialInput {
    pub assigner_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub subject_ref: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignOfficialResult {
    pub official_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn assign_election_official(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: AssignOfficialInput,
) -> WorkflowResult<AssignOfficialResult> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.assigner_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "assign_election_official".to_string(),
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

    // Step 2: Assign
    let timestamp = chrono::Utc::now().to_rfc3339();
    let official_ref = registry.officials.insert_new(ElectionOfficial {
        subject_ref: input.subject_ref.clone(),
        election_ref: input.election_ref.clone(),
        role: input.role.clone(),
        assigned_by: input.assigner_ref.clone(),
        assigned_at: timestamp,
        revoked: false,
        decision_ref: decision_ref.clone(),
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.assigner_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: official_ref.clone(),
            action_type: "assign_election_official".to_string(),
            summary: format!("Assigned {} as {} for election {}", input.subject_ref, input.role, input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "official_assignment".to_string(),
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

    // Step 4: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(), message: e,
    })?;

    Ok(AssignOfficialResult { official_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// revoke_election_official
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeOfficialInput {
    pub revoker_ref: String,
    pub session_ref: String,
    pub official_record_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeOfficialResult {
    pub official_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn revoke_election_official(
    spine: &SpineClient,
    registry: &ElectionRegistry,
    input: RevokeOfficialInput,
) -> WorkflowResult<RevokeOfficialResult> {
    let official = registry.officials.get(&input.official_record_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Official record {} not found", input.official_record_ref),
        })?;

    if official.revoked {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Official already revoked".to_string(),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.revoker_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "revoke_election_official".to_string(),
            target: Some(input.official_record_ref.clone()),
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

    // Step 2: Revoke
    let mut updated = official;
    updated.revoked = true;
    registry.officials.update(&input.official_record_ref, updated);

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.revoker_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.official_record_ref.clone(),
            action_type: "revoke_election_official".to_string(),
            summary: format!("Revoked official {}: {}", input.official_record_ref, input.reason),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "official_revocation".to_string(),
            additional_context: Some(input.reason),
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

    Ok(RevokeOfficialResult {
        official_ref: input.official_record_ref,
        decision_ref,
        attestation_ref,
    })
}
