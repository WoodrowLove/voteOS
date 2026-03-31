//! Module 6: Result Certification workflows
//!
//! Capabilities: certify_result, contest_result, resolve_contest,
//!   get_certified_result, generate_result_bundle

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::certification::*;
use crate::domain::tally::{TallyRegistry, TallyStatus};
use crate::domain::elections::{ElectionRegistry, ElectionStatus};

// ---------------------------------------------------------------------------
// certify_result (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifyResultInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub certification_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifyResultOutput {
    pub certification_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn certify_result(
    spine: &SpineClient,
    cert_registry: &CertificationRegistry,
    tally_registry: &TallyRegistry,
    election_registry: &ElectionRegistry,
    input: CertifyResultInput,
) -> WorkflowResult<CertifyResultOutput> {
    // Precondition: election must be Tallied
    let election = election_registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} not found", input.election_ref),
        })?;

    if election.status != ElectionStatus::Tallied {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!(
                "Can only certify Tallied elections, current status: {:?}",
                election.status
            ),
        });
    }

    // Precondition: tally must exist and not be ambiguous
    let (tally_ref, tally_result) = tally_registry.result_for_election(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("No tally found for election {}", input.election_ref),
        })?;

    if tally_result.status == TallyStatus::Ambiguous {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Cannot certify ambiguous tally — ties or ambiguity must be resolved first".into(),
        });
    }

    if tally_result.status == TallyStatus::Invalid {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Cannot certify invalid tally (no votes or computation error)".into(),
        });
    }

    // Precondition: not already certified
    if cert_registry.is_certified(&input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} is already certified", input.election_ref),
        });
    }

    // Step 1: Evaluate legitimacy (governance_action — elevated authority)
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "certify_result".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("result_certification".to_string()),
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

    // Step 2: Create certification record with tally snapshot
    let timestamp = chrono::Utc::now().to_rfc3339();

    let certification_ref = cert_registry.certifications.insert_new(CertificationRecord {
        election_ref: input.election_ref.clone(),
        tally_ref: tally_ref.clone(),
        tally_snapshot: tally_result,
        status: CertificationStatus::Certified,
        certified_by: Some(input.official_ref.clone()),
        certified_at: Some(timestamp.clone()),
        certification_basis: input.certification_basis.clone(),
        decision_ref: decision_ref.clone(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: timestamp.clone(),
    });

    // Step 3: Transition election to Certified
    election_registry.transition_election(
        &input.election_ref,
        ElectionStatus::Certified,
        &input.official_ref,
        &decision_ref,
        Some(format!("Result certified: {}", input.certification_basis)),
    ).map_err(|e| WorkflowError::PreconditionFailed { step: 3, reason: e })?;

    // Step 4: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: certification_ref.clone(),
            action_type: "certify_result".to_string(),
            summary: format!(
                "Certified result for election {}: {}",
                input.election_ref, input.certification_basis
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "result_certification".to_string(),
            additional_context: Some(input.certification_basis),
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

    // Update certification with attestation ref
    if let Some(mut cert) = cert_registry.certifications.get(&certification_ref) {
        cert.attestation_ref = Some(attestation_ref.clone());
        cert_registry.certifications.update(&certification_ref, cert);
    }

    // Step 5: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "full".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    // Audit
    cert_registry.audit_log.insert_new(CertificationAuditEntry {
        action: "certify_result".into(),
        actor_ref: input.official_ref,
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Result certified, tally ref: {}", tally_ref),
    });

    Ok(CertifyResultOutput {
        certification_ref,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// contest_result (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestResultInput {
    pub challenger_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestResultOutput {
    pub contest_ref: String,
    pub decision_ref: String,
}

pub async fn contest_result(
    spine: &SpineClient,
    cert_registry: &CertificationRegistry,
    input: ContestResultInput,
) -> WorkflowResult<ContestResultOutput> {
    // Precondition: must be certified
    let (cert_ref, _) = cert_registry.certification_for_election(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("No certification found for election {}", input.election_ref),
        })?;

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.challenger_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "contest_result".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("result_certification".to_string()),
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

    // Step 2: File contest
    let timestamp = chrono::Utc::now().to_rfc3339();
    let contest_ref = cert_registry.contests.insert_new(Contest {
        certification_ref: cert_ref.clone(),
        election_ref: input.election_ref.clone(),
        filed_by: input.challenger_ref.clone(),
        reason: input.reason.clone(),
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

    // Audit
    cert_registry.audit_log.insert_new(CertificationAuditEntry {
        action: "contest_result".into(),
        actor_ref: input.challenger_ref,
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Contest filed: {}", input.reason),
    });

    Ok(ContestResultOutput { contest_ref, decision_ref })
}

// ---------------------------------------------------------------------------
// resolve_contest (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveContestInput {
    pub official_ref: String,
    pub session_ref: String,
    pub contest_ref: String,
    pub resolution: String,
    pub upheld: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveContestOutput {
    pub contest_ref: String,
    pub status: ContestStatus,
    pub decision_ref: String,
}

pub async fn resolve_contest(
    spine: &SpineClient,
    cert_registry: &CertificationRegistry,
    input: ResolveContestInput,
) -> WorkflowResult<ResolveContestOutput> {
    let contest = cert_registry.contests.get(&input.contest_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Contest {} not found", input.contest_ref),
        })?;

    if contest.status != ContestStatus::Filed && contest.status != ContestStatus::UnderReview {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Contest already resolved with status: {:?}", contest.status),
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
            operation: "resolve_contest".to_string(),
            target: Some(contest.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("result_certification".to_string()),
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

    // Step 2: Resolve contest
    let timestamp = chrono::Utc::now().to_rfc3339();
    let status = if input.upheld { ContestStatus::Upheld } else { ContestStatus::Dismissed };

    let mut updated_contest = contest;
    updated_contest.status = status.clone();
    updated_contest.resolution = Some(input.resolution.clone());
    updated_contest.resolved_by = Some(input.official_ref.clone());
    updated_contest.resolved_at = Some(timestamp.clone());
    cert_registry.contests.update(&input.contest_ref, updated_contest);

    // If contest dismissed, restore certification to Certified status
    if status == ContestStatus::Dismissed {
        let election_ref = cert_registry.contests.get(&input.contest_ref)
            .map(|c| c.election_ref.clone());
        if let Some(eref) = election_ref {
            if !cert_registry.is_contested(&eref) {
                if let Some((cert_ref, mut cert)) = cert_registry.certification_for_election(&eref) {
                    if cert.status == CertificationStatus::Contested {
                        cert.status = CertificationStatus::Certified;
                        cert_registry.certifications.update(&cert_ref, cert);
                    }
                }
            }
        }
    }

    // Audit
    cert_registry.audit_log.insert_new(CertificationAuditEntry {
        action: "resolve_contest".into(),
        actor_ref: input.official_ref,
        election_ref: "".into(), // contest already has election_ref
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Contest resolved: {:?} — {}", status, input.resolution),
    });

    Ok(ResolveContestOutput {
        contest_ref: input.contest_ref,
        status,
        decision_ref,
    })
}

// ---------------------------------------------------------------------------
// get_certified_result (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifiedResultOutput {
    pub certification: Option<CertificationRecord>,
    pub decision_ref: String,
}

pub async fn get_certified_result(
    spine: &SpineClient,
    cert_registry: &CertificationRegistry,
    requester_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<CertifiedResultOutput> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "get_certified_result".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("result_certification".to_string()),
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

    let certification = cert_registry.certification_for_election(&election_ref).map(|(_, c)| c);
    Ok(CertifiedResultOutput { certification, decision_ref })
}
