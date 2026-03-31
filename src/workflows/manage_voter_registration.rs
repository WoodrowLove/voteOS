//! Capabilities: suspend_voter_registration, restore_voter_registration,
//!               resolve_voter_record (data_access)

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::voters::{VoterRegistry, VoterRegistration, RegistrationStatus, VoterAuditEntry};

// ---------------------------------------------------------------------------
// suspend_voter_registration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendRegistrationInput {
    pub official_ref: String,
    pub session_ref: String,
    pub registration_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendRegistrationResult {
    pub registration_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn suspend_voter_registration(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: SuspendRegistrationInput,
) -> WorkflowResult<SuspendRegistrationResult> {
    let reg = registry.registrations.get(&input.registration_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Registration {} not found", input.registration_ref),
        })?;

    if reg.status != RegistrationStatus::Registered {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Cannot suspend registration in status {:?}", reg.status),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "suspend_voter_registration".to_string(),
            target: Some(input.registration_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: Suspend
    let mut updated = reg.clone();
    updated.status = RegistrationStatus::Suspended;
    registry.registrations.update(&input.registration_ref, updated);

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.registration_ref.clone(),
            action_type: "suspend_voter_registration".to_string(),
            summary: format!("Suspended voter registration {}: {}", input.registration_ref, input.reason),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "voter_suspension".to_string(),
            additional_context: Some(input.reason.clone()),
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

    // Audit
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "suspend_voter_registration".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(input.registration_ref.clone()),
        election_ref: Some(reg.election_ref),
        timestamp: chrono::Utc::now().to_rfc3339(),
        decision_ref: decision_ref.clone(),
        details: format!("Suspended: {}", input.reason),
    });

    Ok(SuspendRegistrationResult {
        registration_ref: input.registration_ref,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// restore_voter_registration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRegistrationInput {
    pub official_ref: String,
    pub session_ref: String,
    pub registration_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRegistrationResult {
    pub registration_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn restore_voter_registration(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: RestoreRegistrationInput,
) -> WorkflowResult<RestoreRegistrationResult> {
    let reg = registry.registrations.get(&input.registration_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Registration {} not found", input.registration_ref),
        })?;

    if reg.status != RegistrationStatus::Suspended {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Cannot restore registration in status {:?}", reg.status),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "restore_voter_registration".to_string(),
            target: Some(input.registration_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: Restore
    let mut updated = reg.clone();
    updated.status = RegistrationStatus::Registered;
    registry.registrations.update(&input.registration_ref, updated);

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.registration_ref.clone(),
            action_type: "restore_voter_registration".to_string(),
            summary: format!("Restored voter registration {}: {}", input.registration_ref, input.reason),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "voter_restoration".to_string(),
            additional_context: Some(input.reason.clone()),
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

    // Audit
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "restore_voter_registration".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(input.registration_ref.clone()),
        election_ref: Some(reg.election_ref),
        timestamp: chrono::Utc::now().to_rfc3339(),
        decision_ref: decision_ref.clone(),
        details: format!("Restored: {}", input.reason),
    });

    Ok(RestoreRegistrationResult {
        registration_ref: input.registration_ref,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// resolve_voter_record (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveVoterRecordInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub citizen_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveVoterRecordResult {
    pub registration_ref: Option<String>,
    pub registration: Option<VoterRegistration>,
    pub decision_ref: String,
}

pub async fn resolve_voter_record(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: ResolveVoterRecordInput,
) -> WorkflowResult<ResolveVoterRecordResult> {
    // Step 1: Evaluate legitimacy for data access
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_voter_record".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: Read
    let found = registry.find_registration(&input.citizen_ref, &input.election_ref);

    Ok(ResolveVoterRecordResult {
        registration_ref: found.as_ref().map(|(id, _)| id.clone()),
        registration: found.map(|(_, r)| r),
        decision_ref,
    })
}
