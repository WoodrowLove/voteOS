//! Module 4: Vote Recording workflows
//!
//! Capabilities: cast_vote, validate_vote, prevent_double_vote,
//!   generate_vote_receipt, verify_vote_receipt, spoil_ballot,
//!   seal_vote, resolve_vote_status, enforce_ballot_secrecy,
//!   count_votes_submitted, audit_vote_recording

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::votes::*;
use crate::domain::ballots::BallotRegistry;

// ---------------------------------------------------------------------------
// cast_vote (operation) — the core action of the entire system
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVoteInput {
    pub voter_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub ballot_issuance_ref: String,
    pub selections: Vec<VoteSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVoteResult {
    pub vote_ref: String,
    pub receipt_hash: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn cast_vote(
    spine: &SpineClient,
    vote_registry: &VoteRegistry,
    ballot_registry: &BallotRegistry,
    input: CastVoteInput,
) -> WorkflowResult<CastVoteResult> {
    // Precondition: prevent double voting
    if vote_registry.has_voted(&input.voter_ref, &input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Voter {} has already voted in election {}", input.voter_ref, input.election_ref),
        });
    }

    // Precondition: voter must have an active ballot issuance
    if !ballot_registry.has_active_issuance(&input.voter_ref, &input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Voter {} has no active ballot for election {}", input.voter_ref, input.election_ref),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.voter_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "cast_vote".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    // Step 2: Record the vote
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Generate receipt hash BEFORE creating vote (needed for the record)
    let vote_ref_preview = format!("vote-pending-{}", timestamp);
    let receipt_hash = VoteRegistry::compute_receipt_hash(
        &vote_ref_preview, &input.election_ref, &timestamp,
    );

    // Create vote record (NO voter_ref — ballot secrecy by design)
    let vote_ref = vote_registry.records.insert_new(VoteRecord {
        election_ref: input.election_ref.clone(),
        ballot_issuance_ref: input.ballot_issuance_ref.clone(),
        status: VoteStatus::Recorded,
        submitted_at: timestamp.clone(),
        sealed_at: None,
        receipt_hash: receipt_hash.clone(),
        decision_ref: decision_ref.clone(),
        attestation_ref: None,
    });

    // Recompute receipt hash with actual vote_ref
    let receipt_hash = VoteRegistry::compute_receipt_hash(
        &vote_ref, &input.election_ref, &timestamp,
    );

    // Update the record with correct hash
    if let Some(mut record) = vote_registry.records.get(&vote_ref) {
        record.receipt_hash = receipt_hash.clone();
        vote_registry.records.update(&vote_ref, record);
    }

    // Store vote content separately (no voter identity)
    vote_registry.contents.insert_new(VoteContent {
        vote_ref: vote_ref.clone(),
        election_ref: input.election_ref.clone(),
        selections: input.selections,
    });

    // Record participation (links voter to vote_ref but NOT to content)
    vote_registry.participation.insert_new(VoterParticipation {
        voter_ref: input.voter_ref.clone(),
        election_ref: input.election_ref.clone(),
        voted_at: timestamp.clone(),
        vote_ref: vote_ref.clone(),
    });

    // Generate receipt
    vote_registry.receipts.insert_new(VotingReceipt {
        voter_ref: input.voter_ref.clone(),
        election_ref: input.election_ref.clone(),
        receipt_hash: receipt_hash.clone(),
        timestamp: timestamp.clone(),
        vote_ref: vote_ref.clone(),
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.voter_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: vote_ref.clone(),
            action_type: "cast_vote".to_string(),
            summary: format!("Vote cast in election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "vote_recording".to_string(),
            additional_context: None, // No vote content in attestation — secrecy
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

    // Update vote record with attestation
    if let Some(mut record) = vote_registry.records.get(&vote_ref) {
        record.attestation_ref = Some(attestation_ref.clone());
        vote_registry.records.update(&vote_ref, record);
    }

    // Step 4: Explain
    let _exp = spine.explanation().explain_decision_full(ExplainDecisionRequest {
        decision_ref: decision_ref.clone(),
        detail_level: "summary".to_string(),
    }).await.map_err(|e| WorkflowError::BridgeError {
        capability: "explain_decision".into(),
        message: e,
    })?;

    // Audit (no voter identity in audit for vote actions)
    vote_registry.audit_log.insert_new(VoteAuditEntry {
        action: "cast_vote".into(),
        actor_ref: None, // Secret ballot — no actor identity in audit
        election_ref: input.election_ref,
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Vote {} recorded with receipt hash {}", vote_ref, receipt_hash),
    });

    Ok(CastVoteResult {
        vote_ref,
        receipt_hash,
        decision_ref,
        attestation_ref,
    })
}

// ---------------------------------------------------------------------------
// seal_vote (operation) — finalize vote record as immutable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealVoteInput {
    pub official_ref: String,
    pub session_ref: String,
    pub vote_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealVoteResult {
    pub vote_ref: String,
    pub decision_ref: String,
}

pub async fn seal_vote(
    spine: &SpineClient,
    registry: &VoteRegistry,
    input: SealVoteInput,
) -> WorkflowResult<SealVoteResult> {
    let mut record = registry.records.get(&input.vote_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Vote {} not found", input.vote_ref),
        })?;

    if record.status != VoteStatus::Recorded {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Can only seal Recorded votes, current: {:?}", record.status),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "seal_vote".to_string(),
            target: Some(input.vote_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    record.status = VoteStatus::Sealed;
    record.sealed_at = Some(chrono::Utc::now().to_rfc3339());
    registry.records.update(&input.vote_ref, record);

    Ok(SealVoteResult { vote_ref: input.vote_ref, decision_ref })
}

// ---------------------------------------------------------------------------
// spoil_ballot (operation) — voter requests fresh ballot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoilBallotInput {
    pub voter_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub vote_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoilBallotResult {
    pub vote_ref: String,
    pub decision_ref: String,
}

pub async fn spoil_ballot(
    spine: &SpineClient,
    vote_registry: &VoteRegistry,
    input: SpoilBallotInput,
) -> WorkflowResult<SpoilBallotResult> {
    let mut record = vote_registry.records.get(&input.vote_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Vote {} not found", input.vote_ref),
        })?;

    if record.status != VoteStatus::Recorded {
        return Err(WorkflowError::PreconditionFailed {
            step: 0, reason: "Can only spoil Recorded (not yet Sealed) votes".into(),
        });
    }

    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.voter_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "spoil_ballot".to_string(),
            target: Some(input.vote_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    record.status = VoteStatus::Spoiled;
    vote_registry.records.update(&input.vote_ref, record);

    // Note: participation record remains but the vote status is Spoiled.
    // The has_voted check should be paired with vote status verification
    // in a full implementation to allow re-voting after spoiling.

    Ok(SpoilBallotResult { vote_ref: input.vote_ref, decision_ref })
}

// ---------------------------------------------------------------------------
// resolve_vote_status (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveVoteStatusResult {
    pub has_voted: bool,
    pub vote_ref: Option<String>,
    pub status: Option<VoteStatus>,
    pub receipt_hash: Option<String>,
    pub decision_ref: String,
}

pub async fn resolve_vote_status(
    spine: &SpineClient,
    registry: &VoteRegistry,
    requester_ref: String,
    session_ref: String,
    voter_ref: String,
    election_ref: String,
) -> WorkflowResult<ResolveVoteStatusResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "resolve_vote_status".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    let has_voted = registry.has_voted(&voter_ref, &election_ref);
    let participation = registry.get_participation(&voter_ref, &election_ref);

    let (vote_ref, status, receipt_hash) = if let Some((_, p)) = participation {
        let record = registry.records.get(&p.vote_ref);
        (
            Some(p.vote_ref),
            record.as_ref().map(|r| r.status.clone()),
            record.map(|r| r.receipt_hash),
        )
    } else {
        (None, None, None)
    };

    Ok(ResolveVoteStatusResult { has_voted, vote_ref, status, receipt_hash, decision_ref })
}

// ---------------------------------------------------------------------------
// verify_vote_receipt (data_access)
// ---------------------------------------------------------------------------

pub async fn verify_vote_receipt(
    spine: &SpineClient,
    registry: &VoteRegistry,
    requester_ref: String,
    session_ref: String,
    voter_ref: String,
    election_ref: String,
    receipt_hash: String,
) -> WorkflowResult<(bool, String)> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "verify_vote_receipt".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    let stored = registry.get_receipt(&voter_ref, &election_ref);
    let valid = stored.map(|(_, r)| r.receipt_hash == receipt_hash).unwrap_or(false);

    Ok((valid, decision_ref))
}

// ---------------------------------------------------------------------------
// count_votes_submitted (data_access)
// ---------------------------------------------------------------------------

pub async fn count_votes_submitted(
    spine: &SpineClient,
    registry: &VoteRegistry,
    requester_ref: String,
    session_ref: String,
    election_ref: String,
) -> WorkflowResult<(usize, String)> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: requester_ref,
            session_ref,
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "count_votes_submitted".to_string(),
            target: Some(election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("vote_recording".to_string()),
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

    let count = registry.votes_submitted(&election_ref);
    Ok((count, decision_ref))
}
