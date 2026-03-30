# VoteOS Pattern Reference — Implementation Blueprint

> Exact code patterns from the proven CivilOS build. The VoteOS agent MUST follow these
> patterns. Do NOT reinvent infrastructure that already works.

---

## 1. SpineClient Pattern

All 11 AxiaSystem capabilities live on the User canister. One canister ID, 7 service accessors.

```rust
// src/spine/client.rs
use std::sync::Arc;
use candid::Principal;
use ic_agent::Agent;

use axia_system_rust_bridge::tools::modules::legitimacy::LegitimacyService;
use axia_system_rust_bridge::tools::modules::transfer::TransferService;
use axia_system_rust_bridge::tools::modules::asset_registration::AssetRegistrationService;
use axia_system_rust_bridge::tools::modules::governance_decision::GovernanceDecisionService;
use axia_system_rust_bridge::tools::modules::attestation::AttestationService;
use axia_system_rust_bridge::tools::modules::explanation::ExplanationService;
use axia_system_rust_bridge::tools::modules::subject::SubjectService;

pub struct SpineClient {
    agent: Arc<Agent>,
    user_canister_id: Principal,
}

impl SpineClient {
    pub fn new(agent: Arc<Agent>, user_canister_id: Principal) -> Self {
        Self { agent, user_canister_id }
    }

    pub fn legitimacy(&self) -> LegitimacyService {
        LegitimacyService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn transfer(&self) -> TransferService {
        TransferService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn asset_registration(&self) -> AssetRegistrationService {
        AssetRegistrationService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn governance_decision(&self) -> GovernanceDecisionService {
        GovernanceDecisionService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn attestation(&self) -> AttestationService {
        AttestationService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn explanation(&self) -> ExplanationService {
        ExplanationService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn subject(&self) -> SubjectService {
        SubjectService::new(self.agent.clone(), self.user_canister_id)
    }
    pub fn agent(&self) -> &Agent { &self.agent }
}
```

**RULE: Copy this exactly. Do not add or remove services. These are the 11 locked capabilities.**

---

## 2. DomainStore Pattern

Generic thread-safe in-memory store with optional file persistence.

```rust
// src/domain/store.rs
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;

pub struct DomainStore<T: Clone + Send + Sync> {
    records: Arc<RwLock<HashMap<String, T>>>,
    prefix: String,
    counter: Arc<RwLock<u64>>,
    persist_path: Option<PathBuf>,
}

impl<T: Clone + Send + Sync + Serialize + DeserializeOwned> DomainStore<T> {
    pub fn new(prefix: &str) -> Self { /* in-memory only */ }
    pub fn with_persistence(prefix: &str, path: PathBuf) -> Self { /* file-backed */ }
    pub fn insert_new(&self, record: T) -> String { /* auto-ID, auto-save */ }
    pub fn get(&self, id: &str) -> Option<T> { /* read */ }
    pub fn update(&self, id: &str, record: T) { /* write + auto-save */ }
    pub fn find_all<F>(&self, predicate: F) -> Vec<(String, T)> where F: Fn(&T) -> bool { /* query */ }
}
```

**ID format:** `{prefix}-{counter}-{YYYYMMDDHHMMSS}`

**RULE: Use DomainStore<T> for ALL domain registries. Do not create custom storage.**

---

## 3. Workflow Pattern

Every workflow is: async fn composing AxiaSystem calls through SpineClient.

```rust
// Universal 4-step pattern (most operations):
// 1. evaluate_legitimacy → is the actor authorized?
// 2. [domain action]     → the actual operation (register_asset, etc.)
// 3. attest_action       → tamper-evident record
// 4. explain_decision    → audit explanation

pub async fn execute(
    spine: &SpineClient,
    registry: &SomeRegistry,  // if domain state needed
    input: SomeInput,
) -> WorkflowResult<SomeResult> {
    // Step 1: Evaluate legitimacy
    let leg = spine.legitimacy().evaluate_legitimacy_full(EvaluateLegitimacyRequest {
        caller_context: CallerContext {
            subject_ref: input.actor_ref.clone(),
            session_ref: input.session_ref.clone(),
        },
        action: EvalActionContext {
            actionType: "operation".to_string(),  // or governance_action, etc.
            operation: "capability_name".to_string(),
            target: Some(input.target.clone()),
        },
        context: EvalRequestContext {
            requesting_system: "voteos".to_string(),  // NOT civilos
            department: Some("election".to_string()),
            city: None,
            workflow_ref: None,
            urgency: Some("normal".to_string()),
        },
    }).await.map_err(|e| WorkflowError::BridgeError { capability: "evaluate_legitimacy".into(), message: e })?;

    let decision_ref = match leg {
        EvaluateLegitimacyResult::Ok(r) if r.decision == "proceed" => r.decision_ref,
        EvaluateLegitimacyResult::Ok(r) => return Err(WorkflowError::PreconditionFailed {
            step: 1, reason: format!("{} ({})", r.reason_summary, r.decision)
        }),
        EvaluateLegitimacyResult::Err(e) => return Err(WorkflowError::CapabilityError {
            capability: "evaluate_legitimacy".into(), code: e.code, message: e.message, step: 1
        }),
    };

    // Step 2: Domain action (varies per workflow)
    // ... register in domain store, call AxiaSystem, etc.

    // Step 3: Attest
    let att = spine.attestation().attest_action_full(AttestActionRequest { ... }).await?;

    // Step 4: Explain
    let exp = spine.explanation().explain_decision_full(ExplainDecisionRequest { ... }).await?;

    Ok(SomeResult { decision_ref, ... })
}
```

**RULE: requesting_system must be "voteos", not "civilos".**

---

## 4. Error Pattern

```rust
// src/error.rs
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum WorkflowError {
    #[error("[{capability}] {code}: {message}")]
    CapabilityError { capability: String, code: String, message: String, step: u32 },

    #[error("Precondition failed at step {step}: {reason}")]
    PreconditionFailed { step: u32, reason: String },

    #[error("Bridge error in {capability}: {message}")]
    BridgeError { capability: String, message: String },
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
```

**RULE: Copy exactly. Three error types are sufficient for all workflows.**

---

## 5. Registry Pattern (Per Module)

Each module has a registry wrapping DomainStore instances.

```rust
// Example: src/domain/elections.rs
pub struct ElectionRegistry {
    elections: DomainStore<Election>,
    configs: DomainStore<ElectionConfig>,
}

impl ElectionRegistry {
    pub fn new() -> Self {
        Self {
            elections: DomainStore::new("elec"),
            configs: DomainStore::new("ecfg"),
        }
    }

    pub fn with_data_dir(dir: &std::path::Path) -> Self {
        Self {
            elections: DomainStore::with_persistence("elec", dir.join("elections.json")),
            configs: DomainStore::with_persistence("ecfg", dir.join("election_configs.json")),
        }
    }

    // Domain operations...
}
```

**RULE: Every registry has new() and with_data_dir() constructors.**

---

## 6. Action Type Reference

| Action Type | Min Assurance | Use For |
|------------|---------------|---------|
| operation | L0 | cast_vote, register_voter, issue_ballot |
| data_access | L0 | resolve_voter_record, check_eligibility |
| governance_action | L1 + role | create_election, certify_result, initiate_recount |
| financial_action | L1 | (unlikely in VoteOS core) |
| structural_change | L2 + role | (unlikely in VoteOS core) |

---

## 7. Test Pattern

```rust
#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_register_voter_strict() {
    let spine = build_test_spine().await;
    let registry = VoterRegistry::new();

    // Setup: create real user via AxiaSystem
    let (actor_ref, session_ref) = setup_real_user(&spine, "election_official").await;

    // Execute: call the workflow
    let result = register_voter::execute(&spine, &registry, RegisterVoterInput {
        official_ref: actor_ref.clone(),
        session_ref: session_ref.clone(),
        citizen_ref: "some-citizen-ref".to_string(),
        election_ref: "elec-1".to_string(),
    }).await;

    // STRICT: must be Ok
    let outcome = result.expect("STRICT HAPPY PATH: register_voter must return Ok(...)");

    // Verify artifacts
    assert!(!outcome.registration_ref.is_empty());
    assert!(!outcome.decision_ref.is_empty());
}
```

**RULE: Tests must use `.expect()` not `match Ok/Err`. Failures get separate test functions.**
