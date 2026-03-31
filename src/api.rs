//! VoteOS HTTP API — thin wrapper over domain workflows.
//!
//! DESIGN PRINCIPLE: No business logic in handlers.
//! Handlers validate input, call workflows, return results.
//! All trust logic lives in domain/ and workflows/.

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State, Json},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use voteos::domain::voters::VoterRegistry;
use voteos::domain::elections::*;
use voteos::domain::ballots::*;
use voteos::domain::votes::*;
use voteos::domain::tally::*;
use voteos::domain::certification::*;
use voteos::domain::proposals::*;
use voteos::domain::audit;
use voteos::domain::export::*;
use voteos::domain::operations::*;

// ---------------------------------------------------------------------------
// Application state — shared across handlers
// ---------------------------------------------------------------------------

pub struct AppState {
    pub voter_registry: VoterRegistry,
    pub election_registry: ElectionRegistry,
    pub ballot_registry: BallotRegistry,
    pub vote_registry: VoteRegistry,
    pub tally_registry: TallyRegistry,
    pub cert_registry: CertificationRegistry,
    pub proposal_registry: ProposalRegistry,
    pub audit_registry: audit::AuditRegistry,
    pub export_registry: ExportRegistry,
    pub ops_registry: OperationsRegistry,
    pub config: ApiConfig,
}

#[derive(Clone)]
pub struct ApiConfig {
    pub api_key: String,
    pub require_auth: bool,
    pub persistence_enabled: bool,
}

// ---------------------------------------------------------------------------
// Standard API response
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<ApiResponse<T>> {
        Json(ApiResponse { success: true, data: Some(data), error: None })
    }

    pub fn err(msg: impl Into<String>) -> (StatusCode, Json<ApiResponse<T>>) {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, data: None, error: Some(msg.into()),
        }))
    }
}

fn unauthorized<T: Serialize>() -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::UNAUTHORIZED, Json(ApiResponse {
        success: false, data: None, error: Some("Unauthorized — provide valid x-api-key".into()),
    }))
}

// ---------------------------------------------------------------------------
// Auth check
// ---------------------------------------------------------------------------

fn check_auth(headers: &HeaderMap, config: &ApiConfig) -> bool {
    if !config.require_auth {
        return true;
    }
    headers.get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|k| k == config.api_key)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/status", get(status))
        // Elections
        .route("/api/elections", get(list_elections))
        .route("/api/elections/create", post(create_election))
        .route("/api/elections/:id/publish", post(publish_election))
        .route("/api/elections/:id/open", post(open_election))
        .route("/api/elections/:id/close", post(close_election))
        .route("/api/elections/:id", get(get_election))
        // Tally
        .route("/api/tally/:election_ref/compute", post(compute_tally))
        .route("/api/tally/:election_ref", get(get_tally))
        // Certification
        .route("/api/certify/:election_ref", post(certify_result))
        // Audit
        .route("/api/audit/:election_ref", get(get_audit))
        .route("/api/audit/:election_ref/verify", post(verify_audit))
        // Export
        .route("/api/export/:election_ref", get(export_result))
        // Operations
        .route("/api/operations/:election_ref/pause", post(pause_election))
        .route("/api/operations/:election_ref/resume", post(resume_election))
        .route("/api/operations/:election_ref/incident", post(flag_incident))
        .route("/api/operations/:election_ref/state", get(get_operational_state))
        // Intelligence
        .route("/api/system/insights", get(system_insights))
        .route("/api/system/pilot-report", get(pilot_report))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Health / Ready / Status
// ---------------------------------------------------------------------------

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "system": "voteos",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Real readiness: verify registries are accessible
    let election_count = state.election_registry.elections.count();
    let cert_count = state.cert_registry.certifications.count();

    Json(serde_json::json!({
        "ready": true,
        "persistence_enabled": state.config.persistence_enabled,
        "auth_enabled": state.config.require_auth,
        "registries": {
            "elections": election_count,
            "certifications": cert_count,
            "voters": state.voter_registry.registrations.count(),
        },
        "axia_integration": "not_connected",
    }))
}

async fn status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "system": "voteos",
        "version": env!("CARGO_PKG_VERSION"),
        "phase": "domain-complete",
        "modules": {
            "voter_registry": "build_complete",
            "election_management": "build_complete",
            "ballot_operations": "build_complete",
            "vote_recording": "build_complete",
            "tally_engine": "build_complete",
            "result_certification": "build_complete",
            "governance_proposals": "build_complete",
            "audit_oversight": "build_complete",
            "election_operations": "build_complete",
            "integration_export": "build_complete",
        },
        "runtime": {
            "persistence": state.config.persistence_enabled,
            "auth": state.config.require_auth,
            "axia_live": false,
        },
        "stats": {
            "elections": state.election_registry.elections.count(),
            "certifications": state.cert_registry.certifications.count(),
            "proposals": state.proposal_registry.proposals.count(),
            "exports": state.export_registry.exports.count(),
        },
    }))
}

// ---------------------------------------------------------------------------
// Election endpoints
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CreateElectionRequest {
    title: String,
    description: String,
    election_type: ElectionType,
    scope: String,
}

async fn create_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateElectionRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let election_ref = state.election_registry.elections.insert_new(Election {
        title: req.title,
        description: req.description,
        election_type: req.election_type,
        status: ElectionStatus::Draft,
        config: ElectionConfig::default(),
        schedule: ElectionSchedule {
            registration_start: None, registration_end: None,
            voting_start: None, voting_end: None,
            certification_deadline: None,
        },
        scope: req.scope,
        created_by: "api".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        decision_ref: "api-create".into(),
    });

    // Initialize operational state
    state.ops_registry.ensure_state(&election_ref, "api");

    (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "election_ref": election_ref })))
}

async fn list_elections(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let elections: Vec<serde_json::Value> = state.election_registry.elections
        .find_all(|_| true)
        .iter()
        .map(|(id, e)| serde_json::json!({
            "election_ref": id,
            "title": e.title,
            "status": e.status,
            "election_type": e.election_type,
        }))
        .collect();

    (StatusCode::OK, ApiResponse::ok(elections))
}

async fn get_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.election_registry.elections.get(&id) {
        Some(e) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({
            "election_ref": id,
            "title": e.title,
            "status": e.status,
            "election_type": e.election_type,
            "scope": e.scope,
            "created_at": e.created_at,
        }))),
        None => ApiResponse::err(format!("Election {} not found", id)),
    }
}

async fn publish_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.election_registry.transition_election(&id, ElectionStatus::Published, "api", "api-pub", None) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "Published" }))),
        Err(e) => ApiResponse::err(e),
    }
}

async fn open_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    if state.ops_registry.is_paused(&id) {
        return ApiResponse::err("Election is paused — resume before opening");
    }

    match state.election_registry.transition_election(&id, ElectionStatus::Open, "api", "api-open", None) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "Open" }))),
        Err(e) => ApiResponse::err(e),
    }
}

async fn close_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.election_registry.transition_election(&id, ElectionStatus::Closed, "api", "api-close", None) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "Closed" }))),
        Err(e) => ApiResponse::err(e),
    }
}

// ---------------------------------------------------------------------------
// Tally endpoints
// ---------------------------------------------------------------------------

async fn compute_tally(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let election = match state.election_registry.elections.get(&election_ref) {
        Some(e) => e,
        None => return ApiResponse::err("Election not found"),
    };

    if election.status != ElectionStatus::Closed {
        return ApiResponse::err(format!("Election must be Closed, current: {:?}", election.status));
    }

    if state.tally_registry.has_tally(&election_ref) {
        return ApiResponse::err("Tally already computed");
    }

    let sealed = state.vote_registry.sealed_contents(&election_ref);
    let input_hash = voteos::domain::tally::compute_input_hash(&sealed);

    let ballot_item_refs: Vec<String> = state.ballot_registry.finalized_template(&election_ref)
        .map(|(_, t)| t.items.iter().map(|i| i.item_ref.clone()).collect())
        .unwrap_or_default();

    let (item_tallies, has_ambiguity) = compute_plurality_tally(&election_ref, &sealed, &ballot_item_refs);
    let total = sealed.len() as u64;

    let status = if total == 0 { TallyStatus::Invalid }
        else if has_ambiguity { TallyStatus::Ambiguous }
        else { TallyStatus::Computed };

    let tally_ref = state.tally_registry.results.insert_new(TallyResult {
        election_ref: election_ref.clone(),
        method: election.config.voting_method.clone(),
        status: status.clone(),
        item_tallies, total_votes_counted: total,
        computed_at: chrono::Utc::now().to_rfc3339(),
        computed_by: "api".into(),
        decision_ref: "api-tally".into(),
        input_hash, has_ambiguity,
    });

    let _ = state.election_registry.transition_election(
        &election_ref, ElectionStatus::Tallied, "api", "api-tally-tr", None);

    (StatusCode::OK, ApiResponse::ok(serde_json::json!({
        "tally_ref": tally_ref,
        "status": status,
        "total_votes": total,
        "has_ambiguity": has_ambiguity,
    })))
}

async fn get_tally(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.tally_registry.result_for_election(&election_ref) {
        Some((id, t)) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({
            "tally_ref": id,
            "status": t.status,
            "total_votes": t.total_votes_counted,
            "has_ambiguity": t.has_ambiguity,
            "items": t.item_tallies,
        }))),
        None => ApiResponse::err("No tally found"),
    }
}

// ---------------------------------------------------------------------------
// Certification endpoint
// ---------------------------------------------------------------------------

async fn certify_result(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let election = match state.election_registry.elections.get(&election_ref) {
        Some(e) => e,
        None => return ApiResponse::err("Election not found"),
    };

    if election.status != ElectionStatus::Tallied {
        return ApiResponse::err(format!("Election must be Tallied, current: {:?}", election.status));
    }

    let (tally_ref, tally) = match state.tally_registry.result_for_election(&election_ref) {
        Some(t) => t,
        None => return ApiResponse::err("No tally found"),
    };

    if tally.status == TallyStatus::Ambiguous {
        return ApiResponse::err("Cannot certify ambiguous tally");
    }
    if tally.status == TallyStatus::Invalid {
        return ApiResponse::err("Cannot certify invalid tally");
    }

    let cert_ref = state.cert_registry.certifications.insert_new(CertificationRecord {
        election_ref: election_ref.clone(),
        tally_ref,
        tally_snapshot: tally,
        status: CertificationStatus::Certified,
        certified_by: Some("api".into()),
        certified_at: Some(chrono::Utc::now().to_rfc3339()),
        certification_basis: "API certification".into(),
        decision_ref: "api-certify".into(),
        attestation_ref: None,
        rejection_reason: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    let _ = state.election_registry.transition_election(
        &election_ref, ElectionStatus::Certified, "api", "api-cert-tr", None);

    (StatusCode::OK, ApiResponse::ok(serde_json::json!({
        "certification_ref": cert_ref,
        "status": "Certified",
    })))
}

// ---------------------------------------------------------------------------
// Audit endpoints
// ---------------------------------------------------------------------------

async fn get_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let bundle = audit::assemble_audit_bundle(
        &election_ref,
        &state.election_registry,
        &state.ballot_registry,
        &state.vote_registry,
        &state.tally_registry,
        &state.cert_registry,
    );

    match bundle {
        Some(b) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({
            "election_ref": election_ref,
            "sealed_vote_count": b.sealed_vote_count,
            "has_certification": b.certification.is_some(),
            "has_tally": b.certified_tally.is_some(),
            "secrecy_preserved": audit::verify_secrecy(&b),
        }))),
        None => ApiResponse::err("Could not assemble audit bundle"),
    }
}

async fn verify_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    let bundle = match audit::assemble_audit_bundle(
        &election_ref,
        &state.election_registry,
        &state.ballot_registry,
        &state.vote_registry,
        &state.tally_registry,
        &state.cert_registry,
    ) {
        Some(b) => b,
        None => return ApiResponse::err("Could not assemble audit bundle"),
    };

    let verification = audit::verify_bundle(&bundle);

    (StatusCode::OK, ApiResponse::ok(serde_json::json!({
        "matches": verification.matches,
        "discrepancy_count": verification.discrepancies.len(),
        "reconstructed_input_hash": verification.reconstructed_input_hash,
        "certified_input_hash": verification.certified_input_hash,
    })))
}

// ---------------------------------------------------------------------------
// Export endpoint
// ---------------------------------------------------------------------------

async fn export_result(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    if !state.cert_registry.is_certified(&election_ref) {
        return ApiResponse::err("Election not certified");
    }

    let (cert_ref, cert) = match state.cert_registry.certification_for_election(&election_ref) {
        Some(c) => c,
        None => return ApiResponse::err("Certification not found"),
    };

    let election = match state.election_registry.elections.get(&election_ref) {
        Some(e) => e,
        None => return ApiResponse::err("Election not found"),
    };

    let item_results: Vec<ExportItemResult> = cert.tally_snapshot.item_tallies.iter()
        .map(item_tally_to_export)
        .collect();

    (StatusCode::OK, ApiResponse::ok(serde_json::json!({
        "election_ref": election_ref,
        "title": election.title,
        "scope": election.scope,
        "total_votes": cert.tally_snapshot.total_votes_counted,
        "certification_ref": cert_ref,
        "audit_hash": cert.tally_snapshot.input_hash,
        "certified_at": cert.certified_at,
        "item_results": item_results,
    })))
}

// ---------------------------------------------------------------------------
// Operations endpoints
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct OperationRequest {
    reason: String,
}

async fn pause_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
    Json(req): Json<OperationRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    state.ops_registry.ensure_state(&election_ref, "api");
    match state.ops_registry.pause(&election_ref, "api", &req.reason) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "Paused" }))),
        Err(e) => ApiResponse::err(e),
    }
}

async fn resume_election(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
    Json(req): Json<OperationRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.ops_registry.resume(&election_ref, "api", &req.reason) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "Resumed" }))),
        Err(e) => ApiResponse::err(e),
    }
}

async fn flag_incident(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
    Json(req): Json<OperationRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    state.ops_registry.ensure_state(&election_ref, "api");
    match state.ops_registry.flag_incident(&election_ref, "api", &req.reason) {
        Ok(()) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({ "status": "IncidentFlagged" }))),
        Err(e) => ApiResponse::err(e),
    }
}

async fn get_operational_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(election_ref): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state.config) { return unauthorized(); }

    match state.ops_registry.state_for_election(&election_ref) {
        Some((_, s)) => (StatusCode::OK, ApiResponse::ok(serde_json::json!({
            "election_ref": election_ref,
            "status": s.status,
            "paused": s.paused,
            "incident_flag": s.incident_flag,
            "notes": s.notes,
        }))),
        None => ApiResponse::err("No operational state found"),
    }
}

// ---------------------------------------------------------------------------
// Intelligence endpoints (read-only, no auth required for observability)
// ---------------------------------------------------------------------------

async fn system_insights(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let snapshot = voteos::intelligence::build_system_snapshot(
        &state.election_registry,
        &state.cert_registry,
        &state.proposal_registry,
        &state.audit_registry,
        &state.ops_registry,
        &state.export_registry,
        state.config.persistence_enabled,
        state.config.require_auth,
    );
    Json(snapshot)
}

async fn pilot_report(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Pilot report with empty adoption data — populated when pilot runs
    let report = voteos::intelligence::build_pilot_report(
        &[],    // No normalized voters cached at API level
        None,   // No reconciliation cached
        &[],    // No shadow validations cached
        &state.audit_registry,
    );
    Json(report)
}
