//! Module 8: Audit & Oversight workflows
//!
//! Capabilities: start_audit, verify_audit, get_audit_result,
//!   get_audit_bundle, trigger_audit_contest

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::audit::*;
use crate::domain::elections::ElectionRegistry;
use crate::domain::ballots::BallotRegistry;
use crate::domain::votes::VoteRegistry;
use crate::domain::tally::TallyRegistry;
use crate::domain::certification::{
    CertificationRegistry, CertificationStatus, Contest, ContestStatus,
};

// ---------------------------------------------------------------------------
// start_audit (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartAuditInput {
    pub auditor_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartAuditOutput {
    pub audit_ref: String,
    pub bundle: AuditBundle,
    pub decision_ref: String,
}

pub async fn start_audit(
    spine: &SpineClient,
    audit_registry: &AuditRegistry,
    election_registry: &ElectionRegistry,
    ballot_registry: &BallotRegistry,
    vote_registry: &VoteRegistry,
    tally_registry: &TallyRegistry,
    cert_registry: &CertificationRegistry,
    input: StartAuditInput,
) -> WorkflowResult<StartAuditOutput> {
    // Precondition: election must exist and be certified
    let _election = election_registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} not found", input.election_ref),
        })?;

    if !cert_registry.is_certified(&input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Can only audit certified elections".into(),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.auditor_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "start_audit".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("audit_oversight".to_string()),
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

    // Step 2: Assemble audit bundle
    let bundle = assemble_audit_bundle(
        &input.election_ref,
        election_registry,
        ballot_registry,
        vote_registry,
        tally_registry,
        cert_registry,
    ).ok_or(WorkflowError::PreconditionFailed {
        step: 2,
        reason: "Failed to assemble audit bundle".into(),
    })?;

    // Step 3: Create audit record in InProgress state
    let timestamp = chrono::Utc::now().to_rfc3339();
    let audit_ref = audit_registry.records.insert_new(AuditRecord {
        election_ref: input.election_ref.clone(),
        status: AuditStatus::InProgress,
        initiated_by: input.auditor_ref.clone(),
        initiated_at: timestamp.clone(),
        completed_at: None,
        verification: None,
        decision_ref: decision_ref.clone(),
        contest_ref: None,
    });

    // Audit log
    audit_registry.audit_log.insert_new(AuditLogEntry {
        action: "start_audit".into(),
        actor_ref: input.auditor_ref,
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Audit initiated, bundle assembled with {} sealed votes", bundle.sealed_vote_count),
    });

    Ok(StartAuditOutput {
        audit_ref,
        bundle,
        decision_ref,
    })
}

// ---------------------------------------------------------------------------
// verify_audit (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyAuditInput {
    pub auditor_ref: String,
    pub session_ref: String,
    pub audit_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyAuditOutput {
    pub audit_ref: String,
    pub status: AuditStatus,
    pub matches: bool,
    pub discrepancy_count: usize,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn verify_audit(
    spine: &SpineClient,
    audit_registry: &AuditRegistry,
    election_registry: &ElectionRegistry,
    ballot_registry: &BallotRegistry,
    vote_registry: &VoteRegistry,
    tally_registry: &TallyRegistry,
    cert_registry: &CertificationRegistry,
    input: VerifyAuditInput,
) -> WorkflowResult<VerifyAuditOutput> {
    let record = audit_registry.records.get(&input.audit_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Audit record {} not found", input.audit_ref),
        })?;

    if record.status != AuditStatus::InProgress {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Audit must be InProgress to verify, current: {:?}", record.status),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.auditor_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "verify_audit".to_string(),
            target: Some(record.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("audit_oversight".to_string()),
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

    // Step 2: Reassemble bundle and verify
    let bundle = assemble_audit_bundle(
        &record.election_ref,
        election_registry,
        ballot_registry,
        vote_registry,
        tally_registry,
        cert_registry,
    ).ok_or(WorkflowError::PreconditionFailed {
        step: 2,
        reason: "Failed to reassemble audit bundle for verification".into(),
    })?;

    let verification = verify_bundle(&bundle);
    let matches = verification.matches;
    let discrepancy_count = verification.discrepancies.len();

    let status = if matches {
        AuditStatus::Verified
    } else {
        AuditStatus::Failed
    };

    // Step 3: Update audit record
    let timestamp = chrono::Utc::now().to_rfc3339();
    let election_ref_for_log = record.election_ref.clone();
    let mut updated = record;
    updated.status = status.clone();
    updated.completed_at = Some(timestamp.clone());
    updated.verification = Some(verification);
    audit_registry.records.update(&input.audit_ref, updated);

    // Step 4: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.auditor_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.audit_ref.clone(),
            action_type: "verify_audit".to_string(),
            summary: format!(
                "Audit verification for election {}: {} (discrepancies: {})",
                election_ref_for_log, if matches { "VERIFIED" } else { "FAILED" }, discrepancy_count
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "audit_verification".to_string(),
            additional_context: None,
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
            step: 4,
        }),
    };

    // Step 5: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "full".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    // Audit log
    audit_registry.audit_log.insert_new(AuditLogEntry {
        action: "verify_audit".into(),
        actor_ref: input.auditor_ref,
        election_ref: election_ref_for_log,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Audit verification: {:?}, {} discrepancies", status, discrepancy_count),
    });

    Ok(VerifyAuditOutput {
        audit_ref: input.audit_ref,
        status,
        matches,
        discrepancy_count,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// trigger_audit_contest (operation) — link audit failure to contest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAuditContestInput {
    pub auditor_ref: String,
    pub session_ref: String,
    pub audit_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAuditContestOutput {
    pub contest_ref: String,
    pub audit_ref: String,
    pub decision_ref: String,
}

pub async fn trigger_audit_contest(
    spine: &SpineClient,
    audit_registry: &AuditRegistry,
    cert_registry: &CertificationRegistry,
    input: TriggerAuditContestInput,
) -> WorkflowResult<TriggerAuditContestOutput> {
    let record = audit_registry.records.get(&input.audit_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Audit record {} not found", input.audit_ref),
        })?;

    if record.status != AuditStatus::Failed {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Can only trigger contest from Failed audit, current: {:?}", record.status),
        });
    }

    let (cert_ref, _) = cert_registry.certification_for_election(&record.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "No certification found for contested election".into(),
        })?;

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.auditor_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "trigger_audit_contest".to_string(),
            target: Some(record.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("audit_oversight".to_string()),
            city: None,
            workflow_ref: None,
            urgency: Some("elevated".to_string()),
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

    // Step 2: Build discrepancy summary from audit
    let discrepancy_summary = record.verification.as_ref()
        .map(|v| {
            v.discrepancies.iter()
                .map(|d| format!("{:?}: {}", d.category, d.description))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_else(|| "Audit failed — details in audit record".into());

    // Step 3: Create contest
    let timestamp = chrono::Utc::now().to_rfc3339();
    let contest_ref = cert_registry.contests.insert_new(Contest {
        certification_ref: cert_ref.clone(),
        election_ref: record.election_ref.clone(),
        filed_by: input.auditor_ref.clone(),
        reason: format!("Audit failure: {}", discrepancy_summary),
        filed_at: timestamp.clone(),
        status: ContestStatus::Filed,
        resolution: None,
        resolved_by: None,
        resolved_at: None,
        decision_ref: decision_ref.clone(),
    });

    // Update certification status to Contested
    if let Some(mut cert) = cert_registry.certifications.get(&cert_ref) {
        cert.status = CertificationStatus::Contested;
        cert_registry.certifications.update(&cert_ref, cert);
    }

    // Update audit record with contest ref
    let mut updated = audit_registry.records.get(&input.audit_ref).unwrap();
    updated.status = AuditStatus::Contested;
    updated.contest_ref = Some(contest_ref.clone());
    audit_registry.records.update(&input.audit_ref, updated);

    // Audit log
    audit_registry.audit_log.insert_new(AuditLogEntry {
        action: "trigger_audit_contest".into(),
        actor_ref: input.auditor_ref,
        election_ref: record.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Contest {} created from audit failure", contest_ref),
    });

    Ok(TriggerAuditContestOutput {
        contest_ref,
        audit_ref: input.audit_ref,
        decision_ref,
    })
}

// ---------------------------------------------------------------------------
// get_audit_result (data_access)
// ---------------------------------------------------------------------------

pub async fn get_audit_result(
    spine: &SpineClient,
    audit_registry: &AuditRegistry,
    requester_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<Option<AuditRecord>> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "get_audit_result".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("audit_oversight".to_string()),
            city: None,
            workflow_ref: None,
            urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "evaluate_legitimacy".into(),
        message: e,
    })?;

    match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => {},
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

    Ok(audit_registry.audit_for_election(&election_ref).map(|(_, r)| r))
}
