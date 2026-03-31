//! Module 3: Ballot Operations workflows
//!
//! Capabilities: create_ballot_template, add_ballot_item, remove_ballot_item,
//!   finalize_ballot, issue_ballot, revoke_ballot, resolve_ballot,
//!   track_ballot_issuance, validate_ballot_integrity, audit_ballot_operations

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::ballots::*;

// ---------------------------------------------------------------------------
// create_ballot_template (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBallotTemplateInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBallotTemplateResult {
    pub template_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn create_ballot_template(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: CreateBallotTemplateInput,
) -> WorkflowResult<CreateBallotTemplateResult> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "create_ballot_template".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    // Step 2: Create template
    let timestamp = chrono::Utc::now().to_rfc3339();
    let template_ref = registry.templates.insert_new(BallotTemplate {
        election_ref: input.election_ref.clone(),
        status: BallotStatus::Draft,
        items: Vec::new(),
        created_by: input.official_ref.clone(),
        created_at: timestamp.clone(),
        finalized_at: None,
        finalized_by: None,
        decision_ref: decision_ref.clone(),
        integrity_hash: None,
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: template_ref.clone(),
            action_type: "create_ballot_template".to_string(),
            summary: format!("Created ballot template for election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "ballot_template_creation".to_string(),
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

    // Audit
    registry.audit_log.insert_new(BallotAuditEntry {
        action: "create_ballot_template".into(),
        actor_ref: input.official_ref,
        target_ref: Some(template_ref.clone()),
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: "Created new ballot template".into(),
    });

    Ok(CreateBallotTemplateResult { template_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// add_ballot_item (operation) — local domain operation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBallotItemInput {
    pub official_ref: String,
    pub session_ref: String,
    pub template_ref: String,
    pub item: BallotItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBallotItemResult {
    pub template_ref: String,
    pub item_count: usize,
    pub decision_ref: String,
}

pub async fn add_ballot_item(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: AddBallotItemInput,
) -> WorkflowResult<AddBallotItemResult> {
    let mut template = registry.templates.get(&input.template_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Template {} not found", input.template_ref),
        })?;

    if template.status != BallotStatus::Draft {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Can only add items to Draft ballot templates".into(),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "add_ballot_item".to_string(),
            target: Some(input.template_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    template.items.push(input.item);
    let item_count = template.items.len();
    registry.templates.update(&input.template_ref, template);

    Ok(AddBallotItemResult {
        template_ref: input.template_ref,
        item_count,
        decision_ref,
    })
}

// ---------------------------------------------------------------------------
// remove_ballot_item (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveBallotItemInput {
    pub official_ref: String,
    pub session_ref: String,
    pub template_ref: String,
    pub item_ref: String,
}

pub async fn remove_ballot_item(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: RemoveBallotItemInput,
) -> WorkflowResult<AddBallotItemResult> {
    let mut template = registry.templates.get(&input.template_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Template {} not found", input.template_ref),
        })?;

    if template.status != BallotStatus::Draft {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Can only remove items from Draft ballot templates".into(),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "remove_ballot_item".to_string(),
            target: Some(input.template_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    template.items.retain(|item| item.item_ref != input.item_ref);
    let item_count = template.items.len();
    registry.templates.update(&input.template_ref, template);

    Ok(AddBallotItemResult {
        template_ref: input.template_ref,
        item_count,
        decision_ref,
    })
}

// ---------------------------------------------------------------------------
// finalize_ballot (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeBallotInput {
    pub official_ref: String,
    pub session_ref: String,
    pub template_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeBallotResult {
    pub template_ref: String,
    pub integrity_hash: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn finalize_ballot(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: FinalizeBallotInput,
) -> WorkflowResult<FinalizeBallotResult> {
    let mut template = registry.templates.get(&input.template_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Template {} not found", input.template_ref),
        })?;

    if template.status != BallotStatus::Draft {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Can only finalize Draft ballot templates".into(),
        });
    }

    if template.items.is_empty() {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Cannot finalize empty ballot template".into(),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "finalize_ballot".to_string(),
            target: Some(input.template_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    // Compute integrity hash before finalization
    let integrity_hash = BallotRegistry::compute_integrity_hash(&template);
    let timestamp = chrono::Utc::now().to_rfc3339();

    template.status = BallotStatus::Finalized;
    template.finalized_at = Some(timestamp.clone());
    template.finalized_by = Some(input.official_ref.clone());
    template.integrity_hash = Some(integrity_hash.clone());
    registry.templates.update(&input.template_ref, template);

    // Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.template_ref.clone(),
            action_type: "finalize_ballot".to_string(),
            summary: format!("Finalized ballot template {} with hash {}", input.template_ref, integrity_hash),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "ballot_finalization".to_string(),
            additional_context: Some(integrity_hash.clone()),
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

    // Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(), message: e,
    })?;

    Ok(FinalizeBallotResult { template_ref: input.template_ref, integrity_hash, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// issue_ballot (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBallotInput {
    pub official_ref: String,
    pub session_ref: String,
    pub template_ref: String,
    pub voter_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBallotResult {
    pub issuance_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn issue_ballot(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: IssueBallotInput,
) -> WorkflowResult<IssueBallotResult> {
    // Precondition: template must be finalized
    let template = registry.templates.get(&input.template_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Template {} not found", input.template_ref),
        })?;

    if template.status != BallotStatus::Finalized {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Can only issue from finalized ballot templates".into(),
        });
    }

    // Precondition: voter must not already have an active issuance
    if registry.has_active_issuance(&input.voter_ref, &input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Voter {} already has an active ballot for election {}", input.voter_ref, input.election_ref),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "issue_ballot".to_string(),
            target: Some(input.voter_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    let timestamp = chrono::Utc::now().to_rfc3339();
    let issuance_ref = registry.issuances.insert_new(BallotIssuance {
        template_ref: input.template_ref,
        voter_ref: input.voter_ref.clone(),
        election_ref: input.election_ref.clone(),
        status: IssuanceStatus::Issued,
        issued_at: timestamp.clone(),
        issued_by: input.official_ref.clone(),
        decision_ref: decision_ref.clone(),
        spoiled_at: None,
        replacement_ref: None,
    });

    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: issuance_ref.clone(),
            action_type: "issue_ballot".to_string(),
            summary: format!("Issued ballot to voter {} for election {}", input.voter_ref, input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "ballot_issuance".to_string(),
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

    registry.audit_log.insert_new(BallotAuditEntry {
        action: "issue_ballot".into(),
        actor_ref: input.official_ref,
        target_ref: Some(issuance_ref.clone()),
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Ballot issued to voter {}", input.voter_ref),
    });

    Ok(IssueBallotResult { issuance_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// revoke_ballot (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeBallotInput {
    pub official_ref: String,
    pub session_ref: String,
    pub issuance_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeBallotResult {
    pub issuance_ref: String,
    pub decision_ref: String,
}

pub async fn revoke_ballot(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: RevokeBallotInput,
) -> WorkflowResult<RevokeBallotResult> {
    let mut issuance = registry.issuances.get(&input.issuance_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Issuance {} not found", input.issuance_ref),
        })?;

    if issuance.status != IssuanceStatus::Issued {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Cannot revoke issuance in status {:?}", issuance.status),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "revoke_ballot".to_string(),
            target: Some(input.issuance_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    issuance.status = IssuanceStatus::Spoiled;
    issuance.spoiled_at = Some(chrono::Utc::now().to_rfc3339());
    registry.issuances.update(&input.issuance_ref, issuance);

    Ok(RevokeBallotResult { issuance_ref: input.issuance_ref, decision_ref })
}

// ---------------------------------------------------------------------------
// resolve_ballot, track_ballot_issuance, validate_ballot_integrity,
// audit_ballot_operations (data_access — all follow same pattern)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveBallotInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub template_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveBallotResult {
    pub template: Option<BallotTemplate>,
    pub decision_ref: String,
}

pub async fn resolve_ballot(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: ResolveBallotInput,
) -> WorkflowResult<ResolveBallotResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_ballot".to_string(),
            target: Some(input.template_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    let template = registry.templates.get(&input.template_ref);
    Ok(ResolveBallotResult { template, decision_ref })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackIssuanceInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackIssuanceResult {
    pub total_issued: usize,
    pub decision_ref: String,
}

pub async fn track_ballot_issuance(
    spine: &SpineClient,
    registry: &BallotRegistry,
    input: TrackIssuanceInput,
) -> WorkflowResult<TrackIssuanceResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "track_ballot_issuance".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    let total_issued = registry.issuance_count(&input.election_ref);
    Ok(TrackIssuanceResult { total_issued, decision_ref })
}

pub async fn validate_ballot_integrity(
    spine: &SpineClient,
    registry: &BallotRegistry,
    requester_ref: String,
    session_ref: String,
    template_ref: String,
) -> WorkflowResult<(bool, String)> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "validate_ballot_integrity".to_string(),
            target: Some(template_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("ballot_operations".to_string()),
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

    let template = registry.templates.get(&template_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 2, reason: "Template not found".into(),
        })?;

    let current_hash = BallotRegistry::compute_integrity_hash(&template);
    let valid = template.integrity_hash.as_deref() == Some(&current_hash);

    Ok((valid, decision_ref))
}
