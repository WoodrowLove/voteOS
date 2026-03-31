//! SpineClient — unified access to all AxiaSystem capabilities via Bridge.
//!
//! Every VoteOS workflow receives a SpineClient and uses it to call
//! the legitimacy spine: evaluate → execute → attest → explain.

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

/// Unified client providing access to all AxiaSystem capabilities
/// through the Rust Bridge. Workflows compose calls on this client
/// to build election and governance operations.
pub struct SpineClient {
    agent: Arc<Agent>,
    user_canister_id: Principal,
}

impl SpineClient {
    /// Create a new SpineClient targeting a specific User canister.
    ///
    /// All 11 proven capabilities live on the User canister, so a single
    /// canister ID is sufficient for the entire spine.
    pub fn new(agent: Arc<Agent>, user_canister_id: Principal) -> Self {
        Self {
            agent,
            user_canister_id,
        }
    }

    /// Access the legitimacy evaluation service.
    /// Gate for all protected operations.
    pub fn legitimacy(&self) -> LegitimacyService {
        LegitimacyService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the transfer execution service.
    pub fn transfer(&self) -> TransferService {
        TransferService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the asset registration service.
    pub fn asset_registration(&self) -> AssetRegistrationService {
        AssetRegistrationService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the governance decision recording service.
    pub fn governance_decision(&self) -> GovernanceDecisionService {
        GovernanceDecisionService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the action attestation service.
    pub fn attestation(&self) -> AttestationService {
        AttestationService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the decision explanation service.
    pub fn explanation(&self) -> ExplanationService {
        ExplanationService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the subject resolution and authentication service.
    pub fn subject(&self) -> SubjectService {
        SubjectService::new(self.agent.clone(), self.user_canister_id)
    }

    /// Access the underlying ic-agent for direct canister calls.
    pub fn agent(&self) -> &Agent {
        &self.agent
    }
}
