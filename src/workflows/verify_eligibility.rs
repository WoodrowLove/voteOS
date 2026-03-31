//! Capabilities: verify_voter_eligibility (data_access), evaluate_eligibility_batch (operation)
//!
//! Check if a citizen is eligible for a specific election.

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::subject::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::voters::VoterRegistry;

// ---------------------------------------------------------------------------
// verify_voter_eligibility
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEligibilityInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub citizen_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEligibilityResult {
    pub eligible: bool,
    pub reason: String,
    pub decision_ref: String,
    pub already_registered: bool,
}

pub async fn verify_voter_eligibility(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: VerifyEligibilityInput,
) -> WorkflowResult<VerifyEligibilityResult> {
    // Step 1: Evaluate legitimacy for data access
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "verify_voter_eligibility".to_string(),
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

    // Step 2: Resolve subject to verify identity exists
    let resolve = spine.subject().resolve_subject_full(ResolveSubjectRequest {
        identification: Identification {
            idType: "reference".to_string(),
            material: IdentificationMaterial {
                identifier: Some(input.citizen_ref.clone()),
                username: None,
                email: None,
                password: None,
                institutional_context: None,
            },
        },
        verification_level: "standard".to_string(),
        financial_account: FinancialAccountLink {
            link: false,
            account_reference: None,
        },
        context: ResolveContext {
            requesting_system: "voteos".to_string(),
            purpose: "eligibility_check".to_string(),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "resolve_subject".into(),
        message: e,
    })?;

    let (subject_exists, standing_ok) = match resolve {
        ResolveSubjectResult::Ok(r) => (true, r.standing.status == "good"),
        ResolveSubjectResult::Err(_) => (false, false),
    };

    let already_registered = registry.is_registered(&input.citizen_ref, &input.election_ref);

    let (eligible, reason) = if !subject_exists {
        (false, "Subject not found in AxiaSystem".to_string())
    } else if !standing_ok {
        (false, "Subject standing is not good".to_string())
    } else if already_registered {
        (true, "Already registered for this election".to_string())
    } else {
        (true, "Subject exists with good standing — eligible".to_string())
    };

    Ok(VerifyEligibilityResult {
        eligible,
        reason,
        decision_ref,
        already_registered,
    })
}

// ---------------------------------------------------------------------------
// evaluate_eligibility_batch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEligibilityInput {
    pub official_ref: String,
    pub session_ref: String,
    pub citizen_refs: Vec<String>,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEligibilityEntry {
    pub citizen_ref: String,
    pub eligible: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEligibilityResult {
    pub decision_ref: String,
    pub results: Vec<BatchEligibilityEntry>,
    pub total_eligible: usize,
    pub total_ineligible: usize,
}

pub async fn evaluate_eligibility_batch(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: BatchEligibilityInput,
) -> WorkflowResult<BatchEligibilityResult> {
    // Step 1: Evaluate legitimacy for batch operation
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "evaluate_eligibility_batch".to_string(),
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

    // Step 2: Check each citizen
    let mut results = Vec::new();
    let mut total_eligible = 0;
    let mut total_ineligible = 0;

    for citizen_ref in &input.citizen_refs {
        let already_registered = registry.is_registered(citizen_ref, &input.election_ref);

        // Try to resolve subject
        let resolve = spine.subject().resolve_subject_full(ResolveSubjectRequest {
            identification: Identification {
                idType: "reference".to_string(),
                material: IdentificationMaterial {
                    identifier: Some(citizen_ref.clone()),
                    username: None,
                    email: None,
                    password: None,
                    institutional_context: None,
                },
            },
            verification_level: "standard".to_string(),
            financial_account: FinancialAccountLink {
                link: false,
                account_reference: None,
            },
            context: ResolveContext {
                requesting_system: "voteos".to_string(),
                purpose: "batch_eligibility_check".to_string(),
            },
        }).await;

        let (eligible, reason) = match resolve {
            Ok(ResolveSubjectResult::Ok(r)) if r.standing.status == "good" => {
                if already_registered {
                    (true, "Already registered".to_string())
                } else {
                    (true, "Eligible — good standing".to_string())
                }
            }
            Ok(ResolveSubjectResult::Ok(_)) => (false, "Standing not good".to_string()),
            Ok(ResolveSubjectResult::Err(e)) => (false, format!("Not found: {}", e.message)),
            Err(e) => (false, format!("Resolution failed: {}", e)),
        };

        if eligible { total_eligible += 1; } else { total_ineligible += 1; }
        results.push(BatchEligibilityEntry { citizen_ref: citizen_ref.clone(), eligible, reason });
    }

    Ok(BatchEligibilityResult {
        decision_ref,
        results,
        total_eligible,
        total_ineligible,
    })
}
