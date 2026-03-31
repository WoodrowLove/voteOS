//! Capability: register_voter (operation)
//!
//! Add an eligible citizen to a voter roll for a specific election.
//! Chain: evaluate_legitimacy → register in store → attest → explain

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::voters::{VoterRegistry, VoterRegistration, RegistrationStatus, VoterAuditEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterVoterInput {
    pub official_ref: String,
    pub session_ref: String,
    pub citizen_ref: String,
    pub election_ref: String,
    pub eligibility_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterVoterResult {
    pub registration_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn execute(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: RegisterVoterInput,
) -> WorkflowResult<RegisterVoterResult> {
    // Precondition: not already registered
    if registry.is_registered(&input.citizen_ref, &input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!(
                "Citizen {} already registered for election {}",
                input.citizen_ref, input.election_ref
            ),
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
            operation: "register_voter".to_string(),
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

    // Step 2: Domain action — register voter
    let timestamp = chrono::Utc::now().to_rfc3339();
    let registration_ref = registry.registrations.insert_new(VoterRegistration {
        citizen_ref: input.citizen_ref.clone(),
        election_ref: input.election_ref.clone(),
        status: RegistrationStatus::Registered,
        registered_at: timestamp.clone(),
        registered_by: input.official_ref.clone(),
        eligibility_basis: input.eligibility_basis.clone(),
        decision_ref: decision_ref.clone(),
        attestation_ref: None,
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: registration_ref.clone(),
            action_type: "register_voter".to_string(),
            summary: format!(
                "Registered voter {} for election {}",
                input.citizen_ref, input.election_ref
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "voter_registration".to_string(),
            additional_context: Some(input.eligibility_basis.clone()),
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

    // Update registration with attestation ref
    if let Some(mut reg) = registry.registrations.get(&registration_ref) {
        reg.attestation_ref = Some(attestation_ref.clone());
        registry.registrations.update(&registration_ref, reg);
    }

    // Step 4: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    // Audit log
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "register_voter".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(registration_ref.clone()),
        election_ref: Some(input.election_ref),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Registered citizen {} — basis: {}", input.citizen_ref, input.eligibility_basis),
    });

    Ok(RegisterVoterResult {
        registration_ref,
        decision_ref,
        attestation_ref,
    })
}
