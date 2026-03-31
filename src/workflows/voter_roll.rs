//! Capabilities: define_eligibility_rule, challenge_eligibility,
//!   resolve_eligibility_challenge, generate_voter_roll,
//!   export_voter_statistics, audit_voter_registry

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use axia_system_rust_bridge::bindings::attestation::*;
use axia_system_rust_bridge::bindings::explanation::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::voters::*;

// ---------------------------------------------------------------------------
// define_eligibility_rule (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineEligibilityRuleInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub rule_type: RuleType,
    pub criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineEligibilityRuleResult {
    pub rule_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn define_eligibility_rule(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: DefineEligibilityRuleInput,
) -> WorkflowResult<DefineEligibilityRuleResult> {
    // Step 1: Evaluate legitimacy (governance_action)
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "define_eligibility_rule".to_string(),
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

    // Step 2: Create rule
    let timestamp = chrono::Utc::now().to_rfc3339();
    let rule_ref = registry.rules.insert_new(EligibilityRule {
        election_ref: input.election_ref.clone(),
        rule_type: input.rule_type,
        criteria: input.criteria.clone(),
        defined_by: input.official_ref.clone(),
        defined_at: timestamp.clone(),
        decision_ref: decision_ref.clone(),
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: rule_ref.clone(),
            action_type: "define_eligibility_rule".to_string(),
            summary: format!("Defined eligibility rule for election {}", input.election_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "eligibility_rule_definition".to_string(),
            additional_context: Some(input.criteria.clone()),
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

    // Audit
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "define_eligibility_rule".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(rule_ref.clone()),
        election_ref: Some(input.election_ref),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Rule criteria: {}", input.criteria),
    });

    Ok(DefineEligibilityRuleResult { rule_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// challenge_eligibility (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeEligibilityInput {
    pub challenger_ref: String,
    pub session_ref: String,
    pub voter_registration_ref: String,
    pub election_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeEligibilityResult {
    pub challenge_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn challenge_eligibility(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: ChallengeEligibilityInput,
) -> WorkflowResult<ChallengeEligibilityResult> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.challenger_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "challenge_eligibility".to_string(),
            target: Some(input.voter_registration_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: File challenge
    let timestamp = chrono::Utc::now().to_rfc3339();
    let challenge_ref = registry.challenges.insert_new(EligibilityChallenge {
        voter_registration_ref: input.voter_registration_ref.clone(),
        election_ref: input.election_ref.clone(),
        challenger_ref: input.challenger_ref.clone(),
        reason: input.reason.clone(),
        status: ChallengeStatus::Filed,
        filed_at: timestamp.clone(),
        resolution: None,
        resolved_by: None,
        resolved_at: None,
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.challenger_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: challenge_ref.clone(),
            action_type: "challenge_eligibility".to_string(),
            summary: format!("Filed eligibility challenge against {}", input.voter_registration_ref),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "eligibility_challenge".to_string(),
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

    // Audit
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "challenge_eligibility".to_string(),
        actor_ref: input.challenger_ref,
        target_ref: Some(challenge_ref.clone()),
        election_ref: Some(input.election_ref),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Challenge filed against registration {}", input.voter_registration_ref),
    });

    Ok(ChallengeEligibilityResult { challenge_ref, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// resolve_eligibility_challenge (governance_action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveEligibilityChallengeInput {
    pub official_ref: String,
    pub session_ref: String,
    pub challenge_ref: String,
    pub upheld: bool,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveEligibilityChallengeResult {
    pub challenge_ref: String,
    pub decision_ref: String,
    pub attestation_ref: String,
    pub voter_suspended: bool,
}

pub async fn resolve_eligibility_challenge(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: ResolveEligibilityChallengeInput,
) -> WorkflowResult<ResolveEligibilityChallengeResult> {
    let challenge = registry.challenges.get(&input.challenge_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0, reason: format!("Challenge {} not found", input.challenge_ref),
        })?;

    // Step 1: Evaluate legitimacy (governance_action)
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "governance_action".to_string(),
            operation: "resolve_eligibility_challenge".to_string(),
            target: Some(input.challenge_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: Resolve challenge
    let timestamp = chrono::Utc::now().to_rfc3339();
    let new_status = if input.upheld { ChallengeStatus::Upheld } else { ChallengeStatus::Dismissed };
    let mut updated = challenge.clone();
    updated.status = new_status;
    updated.resolution = Some(input.resolution.clone());
    updated.resolved_by = Some(input.official_ref.clone());
    updated.resolved_at = Some(timestamp.clone());
    registry.challenges.update(&input.challenge_ref, updated);

    // If upheld, suspend the voter registration
    let mut voter_suspended = false;
    if input.upheld {
        if let Some(mut reg) = registry.registrations.get(&challenge.voter_registration_ref) {
            reg.status = RegistrationStatus::Suspended;
            registry.registrations.update(&challenge.voter_registration_ref, reg);
            voter_suspended = true;
        }
    }

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: input.challenge_ref.clone(),
            action_type: "resolve_eligibility_challenge".to_string(),
            summary: format!(
                "Resolved eligibility challenge {} — {}",
                input.challenge_ref,
                if input.upheld { "upheld" } else { "dismissed" }
            ),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "challenge_resolution".to_string(),
            additional_context: Some(input.resolution.clone()),
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
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "resolve_eligibility_challenge".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(input.challenge_ref.clone()),
        election_ref: Some(challenge.election_ref),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Challenge {} — voter suspended: {}", if input.upheld { "upheld" } else { "dismissed" }, voter_suspended),
    });

    Ok(ResolveEligibilityChallengeResult {
        challenge_ref: input.challenge_ref,
        decision_ref,
        attestation_ref,
        voter_suspended,
    })
}

// ---------------------------------------------------------------------------
// generate_voter_roll (operation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateVoterRollInput {
    pub official_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateVoterRollResult {
    pub roll_ref: String,
    pub total_eligible: usize,
    pub decision_ref: String,
    pub attestation_ref: String,
}

pub async fn generate_voter_roll(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: GenerateVoterRollInput,
) -> WorkflowResult<GenerateVoterRollResult> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),
            operation: "generate_voter_roll".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    // Step 2: Gather registered voters
    let voters = registry.voters_for_election(&input.election_ref);
    let voter_refs: Vec<String> = voters.iter().map(|(_, r)| r.citizen_ref.clone()).collect();
    let total_eligible = voter_refs.len();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let roll_ref = registry.rolls.insert_new(VoterRoll {
        election_ref: input.election_ref.clone(),
        voter_refs,
        generated_at: timestamp.clone(),
        generated_by: input.official_ref.clone(),
        total_eligible,
    });

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest {
        caller_context: AttestCallerContext {
            subject_ref: input.official_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: AttestActionDetails {
            action_ref: roll_ref.clone(),
            action_type: "generate_voter_roll".to_string(),
            summary: format!("Generated voter roll for election {} with {} voters", input.election_ref, total_eligible),
        },
        attestation: AttestAttestationDetails {
            decision_ref: decision_ref.clone(),
            purpose: "voter_roll_generation".to_string(),
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
    registry.audit_log.insert_new(VoterAuditEntry {
        action: "generate_voter_roll".to_string(),
        actor_ref: input.official_ref,
        target_ref: Some(roll_ref.clone()),
        election_ref: Some(input.election_ref),
        timestamp,
        decision_ref: decision_ref.clone(),
        details: format!("Generated roll with {} eligible voters", total_eligible),
    });

    Ok(GenerateVoterRollResult { roll_ref, total_eligible, decision_ref, attestation_ref })
}

// ---------------------------------------------------------------------------
// export_voter_statistics (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportVoterStatsInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportVoterStatsResult {
    pub statistics: VoterStatistics,
    pub decision_ref: String,
}

pub async fn export_voter_statistics(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: ExportVoterStatsInput,
) -> WorkflowResult<ExportVoterStatsResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "export_voter_statistics".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    let statistics = registry.election_statistics(&input.election_ref);

    Ok(ExportVoterStatsResult { statistics, decision_ref })
}

// ---------------------------------------------------------------------------
// audit_voter_registry (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVoterRegistryInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVoterRegistryResult {
    pub entries: Vec<(String, VoterAuditEntry)>,
    pub decision_ref: String,
}

pub async fn audit_voter_registry(
    spine: &SpineClient,
    registry: &VoterRegistry,
    input: AuditVoterRegistryInput,
) -> WorkflowResult<AuditVoterRegistryResult> {
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "audit_voter_registry".to_string(),
            target: input.election_ref.clone(),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("voter_registry".to_string()),
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

    let entries = match &input.election_ref {
        Some(eref) => registry.audit_log.find_all(|e| e.election_ref.as_deref() == Some(eref)),
        None => registry.audit_log.find_all(|_| true),
    };

    Ok(AuditVoterRegistryResult { entries, decision_ref })
}
