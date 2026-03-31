//! Module 10: Integration & Export workflows
//!
//! Capabilities: export_certified_result, export_proposal_result,
//!   emit_event, get_exports
//!
//! FUNDAMENTAL PRINCIPLE:
//! VoteOS produces certified decisions. It never executes them.
//! Export is read-only. External systems consume. VoteOS does not call out.

use serde::{Deserialize, Serialize};
use axia_system_rust_bridge::bindings::legitimacy::*;
use crate::spine::SpineClient;
use crate::error::{WorkflowError, WorkflowResult};
use crate::domain::export::*;
use crate::domain::tally::TallyRegistry;
use crate::domain::certification::CertificationRegistry;
use crate::domain::elections::ElectionRegistry;
use crate::domain::proposals::ProposalRegistry;

// ---------------------------------------------------------------------------
// export_certified_result (data_access)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResultInput {
    pub requester_ref: String,
    pub session_ref: String,
    pub election_ref: String,
    pub format: ExportFormat,
}

pub async fn export_certified_result(
    spine: &SpineClient,
    export_registry: &ExportRegistry,
    election_registry: &ElectionRegistry,
    _tally_registry: &TallyRegistry,
    cert_registry: &CertificationRegistry,
    proposal_registry: &ProposalRegistry,
    input: ExportResultInput,
) -> WorkflowResult<CertifiedResultExport> {
    // Precondition: election must be certified
    let election = election_registry.elections.get(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 0,
            reason: format!("Election {} not found", input.election_ref),
        })?;

    if !cert_registry.is_certified(&input.election_ref) {
        return Err(WorkflowError::PreconditionFailed {
            step: 0,
            reason: "Can only export certified election results".into(),
        });
    }

    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.requester_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "data_access".to_string(),
            operation: "export_certified_result".to_string(),
            target: Some(input.election_ref.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),
            department: Some("integration_export".to_string()),
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

    // Step 2: Build export
    let (cert_ref, cert) = cert_registry.certification_for_election(&input.election_ref)
        .ok_or(WorkflowError::PreconditionFailed {
            step: 2,
            reason: "Certification record not found".into(),
        })?;

    let tally = &cert.tally_snapshot;
    let item_results: Vec<ExportItemResult> = tally.item_tallies.iter()
        .map(item_tally_to_export)
        .collect();

    // Check if there's a linked proposal
    let proposal_ref = proposal_registry.proposal_for_election(&input.election_ref)
        .map(|(id, _)| id);
    let proposal_outcome = proposal_ref.as_ref()
        .and_then(|pref| proposal_registry.result_for_proposal(pref))
        .map(|(_, r)| r.outcome);

    let timestamp = chrono::Utc::now().to_rfc3339();
    let export = CertifiedResultExport {
        export_ref: String::new(), // Will be set by insert
        election_ref: input.election_ref.clone(),
        proposal_ref,
        jurisdiction_scope: election.scope.clone(),
        title: election.title.clone(),
        item_results,
        proposal_outcome,
        total_votes: tally.total_votes_counted,
        certification_ref: cert_ref,
        audit_hash: tally.input_hash.clone(),
        certified_at: cert.certified_at.unwrap_or_default(),
        certified_by: cert.certified_by.unwrap_or_default(),
        format: input.format.clone(),
        exported_at: timestamp.clone(),
        consumed: false,
    };

    let export_ref = export_registry.exports.insert_new(export.clone());

    // Update the export_ref in the stored record
    let mut stored = export_registry.exports.get(&export_ref).unwrap();
    stored.export_ref = export_ref.clone();
    export_registry.exports.update(&export_ref, stored.clone());

    // Emit event
    export_registry.events.insert_new(SystemEvent {
        event_type: EventType::ExportGenerated,
        election_ref: input.election_ref.clone(),
        proposal_ref: export.proposal_ref.clone(),
        timestamp: timestamp.clone(),
        payload: serde_json::to_string(&stored).unwrap_or_default(),
    });

    // Audit
    export_registry.audit_log.insert_new(ExportAuditEntry {
        action: "export_certified_result".into(),
        actor_ref: input.requester_ref,
        election_ref: input.election_ref,
        timestamp,
        decision_ref,
        details: format!("Exported as {:?}, {} item results", input.format, export.item_results.len()),
    });

    Ok(stored)
}
