//! Module 7: Governance Proposals workflows
//!
//! Capabilities: create_proposal, publish_proposal, attach_election,
//!   certify_proposal_result, withdraw_proposal, resolve_proposal

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::proposals::*;
use crate::domain::tally::TallyRegistry;
use crate::domain::certification::CertificationRegistry;

// ---------------------------------------------------------------------------
// create_proposal (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalInput {
    pub official_ref: String,
    pub session_ref: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub jurisdiction_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalOutput {
    pub proposal_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn create_proposal(
    spine: &SpineClient,
    registry: &ProposalRegistry,
    input: CreateProposalInput,
) -> WorkflowResult<CreateProposalOutput> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "create_proposal".to_string(),
            target: Some(input.jurisdiction_scope.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("governance_proposals".to_string()),
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

    // Step 2: Create proposal
    let timestamp = chrono::Utc::now().to_rfc3339();
    let proposal_ref = registry.proposals.insert_new(Proposal {
        title: input.title.clone(),
        description: input.description.clone(),
        proposal_type: input.proposal_type,
        jurisdiction_scope: input.jurisdiction_scope,
        status: ProposalStatus::Draft,
        election_ref: None,
        created_by: input.official_ref.clone(),
        created_at: timestamp.clone(),
        decision_ref: decision_ref.clone(),
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: proposal_ref.clone(),
            action_type: "create_proposal".to_string(),
            summary: format!("Created governance proposal: {}", input.title),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "proposal_creation".to_string(),
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

    registry.audit_log.insert_new(ProposalAuditEntry {
        action: "create_proposal".into(),
        actor_ref: input.official_ref,
        proposal_ref: proposal_ref.clone(),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Created proposal: {}", input.title),
    });

    Ok(CreateProposalOutput { proposal_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// certify_proposal_result (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifyProposalResultInput {
    pub official_ref: String,
    pub session_ref: String,
    pub proposal_ref: String,
    pub approval_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifyProposalResultOutput {
    pub result_ref: String,
    pub outcome: ProposalOutcome,
    pub vote_summary: String,
    pub decision_ref: String,
}

pub async fn certify_proposal_result(
    spine: &SpineClient,
    proposal_registry: &ProposalRegistry,
    tally_registry: &TallyRegistry,
    cert_registry: &CertificationRegistry,
    input: CertifyProposalResultInput,
) -> WorkflowResult<CertifyProposalResultOutput> {
    let proposal = proposal_registry.proposals.get(&input.proposal_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Proposal {} not found", input.proposal_ref),
        })?;

    let election_ref = proposal.election_ref.as_ref()
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Proposal has no linked election".into(),
        })?;

    // Must be certified via Module 6 first
    if !cert_registry.is_certified(election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Linked election is not certified".into(),
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
            operation: "certify_proposal_result".to_string(),
            target: Some(input.proposal_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("governance_proposals".to_string()),
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

    // Step 2: Get tally and determine outcome
    let (_, tally) = tally_registry.result_for_election(election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 2,
            reason: "No tally found for linked election".into(),
        })?;

    // Find the proposal item in the tally (typically the ballot item matching the proposal)
    // For proposals, we look for yes/no counts
    let (yes_count, no_count, total_votes) = if let Some(item) = tally.item_tallies.first() {
        let yes = item.choice_counts.get("yes").copied().unwrap_or(0);
        let no = item.choice_counts.get("no").copied().unwrap_or(0);
        (yes, no, item.total_votes)
    } else {
        (0, 0, 0)
    };

    let (outcome, vote_summary) = determine_proposal_outcome(
        yes_count,
        no_count,
        total_votes,
        input.approval_threshold,
    );

    // Step 3: Record proposal result
    let timestamp = chrono::Utc::now().to_rfc3339();
    let (cert_ref, _) = cert_registry.certification_for_election(election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 3,
            reason: "Certification record not found".into(),
        })?;

    let result_ref = proposal_registry.results.insert_new(ProposalResult {
        proposal_ref: input.proposal_ref.clone(),
        election_ref: election_ref.clone(),
        outcome: outcome.clone(),
        vote_summary: vote_summary.clone(),
        certification_ref: cert_ref,
        certified_at: timestamp.clone(),
        threshold_applied: input.approval_threshold,
        threshold_met: input.approval_threshold.map(|t| {
            let yes_pct = if total_votes > 0 { (yes_count as f64 / total_votes as f64) * 100.0 } else { 0.0 };
            yes_pct >= t
        }),
    });

    // Update proposal status
    let new_status = match outcome {
        ProposalOutcome::Approved | ProposalOutcome::Rejected => ProposalStatus::Certified,
        ProposalOutcome::Ambiguous => ProposalStatus::Rejected,
        ProposalOutcome::Pending => ProposalStatus::Closed,
    };

    let mut updated = proposal;
    updated.status = new_status;
    proposal_registry.proposals.update(&input.proposal_ref, updated);

    // Audit
    proposal_registry.audit_log.insert_new(ProposalAuditEntry {
        action: "certify_proposal_result".into(),
        actor_ref: input.official_ref,
        proposal_ref: input.proposal_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Proposal outcome: {:?} — {}", outcome, vote_summary),
    });

    Ok(CertifyProposalResultOutput {
        result_ref,
        outcome,
        vote_summary,
        decision_ref,
    })
}
