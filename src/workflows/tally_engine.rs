//! Module 5: Tally Engine workflows
//!
//! Capabilities: compute_tally, resolve_tally, verify_tally_determinism,
//!   evaluate_thresholds, audit_tally_operations

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::tally::*;
use crate::domain::votes::VoteRegistry;
use crate::domain::elections::{ElectionRegistry, ElectionStatus, VotingMethod};
use crate::domain::ballots::BallotRegistry;

// ---------------------------------------------------------------------------
// compute_tally (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTallyInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTallyResult {
    pub tally_ref: String,
    pub status: TallyStatus,
    pub has_ambiguity: bool,
    pub total_votes: u64,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn compute_tally(
    spine: &SpineClient,
    tally_registry: &TallyRegistry,
    vote_registry: &VoteRegistry,
    election_registry: &ElectionRegistry,
    ballot_registry: &BallotRegistry,
    input: ComputeTallyInput,
) -> WorkflowResult<ComputeTallyResult> {
    // Precondition: election must be Closed
    let election = election_registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} not found", input.election_ref),
        })?;

    if election.status != ElectionStatus::Closed {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!(
                "Can only compute tally for Closed elections, current status: {:?}",
                election.status
            ),
        });
    }

    // Precondition: tally not already computed
    if tally_registry.has_tally(&input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Tally already computed for election {}", input.election_ref),
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
            operation: "compute_tally".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("tally_engine".to_string()),
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

    // Step 2: Get sealed vote contents
    let contents = vote_registry.sealed_contents(&input.election_ref);
    let input_hash = compute_input_hash(&contents);

    // Step 3: Get ballot items to tally
    let ballot_item_refs = match ballot_registry.finalized_template(&input.election_ref) {
        Some((_, template)) => template.items.iter().map(|i| i.item_ref.clone()).collect(),
        None => Vec::new(),
    };

    // Step 4: Compute tally (pure computation)
    let method = election.config.voting_method.clone();
    let (item_tallies, has_ambiguity) = match method {
        VotingMethod::Plurality => {
            compute_plurality_tally(&input.election_ref, &contents, &ballot_item_refs)
        }
        _ => {
            return Err(WorkflowError::PreconditionFailed {
                step: 4,
                reason: format!("Voting method {:?} not yet implemented", method),
            });
        }
    };

    let total_votes_counted = contents.len() as u64;
    let timestamp = chrono::Utc::now().to_rfc3339();

    let status = if total_votes_counted == 0 {
        TallyStatus::Invalid
    } else if has_ambiguity {
        TallyStatus::Ambiguous
    } else {
        TallyStatus::Computed
    };

    let tally_result = TallyResult {
        election_ref: input.election_ref.clone(),
        method,
        status: status.clone(),
        item_tallies,
        total_votes_counted,
        computed_at: timestamp.clone(),
        computed_by: input.official_ref.clone(),
        decision_ref: decision_ref.clone(),
        input_hash,
        has_ambiguity,
    };

    let tally_ref = tally_registry.results.insert_new(tally_result);

    // Step 5: Transition election to Tallied
    election_registry.transition_election(
        &input.election_ref,
        ElectionStatus::Tallied,
        &input.official_ref,
        &decision_ref,
        Some(format!("Tally computed: {} votes, ambiguity: {}", total_votes_counted, has_ambiguity)),
    ).map_err(|e| WorkflowError::PreconditionFailed { step: 5, reason: e })?;

    // Step 6: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: tally_ref.clone(),
            action_type: "compute_tally".to_string(),
            summary: format!(
                "Computed tally for election {}: {} votes, status: {:?}",
                input.election_ref, total_votes_counted, status
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "tally_computation".to_string(),
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
            step: 6,
        }),
    };

    // Step 7: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    // Audit
    tally_registry.audit_log.insert_new(TallyAuditEntry {
        action: "compute_tally".into(),
        actor_ref: input.official_ref,
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Tally computed: {} votes, status: {:?}", total_votes_counted, status),
    });

    Ok(ComputeTallyResult {
        tally_ref,
        status,
        has_ambiguity,
        total_votes: total_votes_counted,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// resolve_tally (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveTallyResult {
    pub tally: Option<TallyResult>,
    pub decision_ref: String,
}

pub async fn resolve_tally(
    spine: &SpineClient,
    registry: &TallyRegistry,
    requester_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<ResolveTallyResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_tally".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("tally_engine".to_string()),
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

    let tally = registry.result_for_election(&election_ref).map(|(_, t)| t);
    Ok(ResolveTallyResult { tally, decision_ref })
}
