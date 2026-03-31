//! VoteOS error types for workflow orchestration.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Workflow-level error that wraps capability-level errors
/// with orchestration context.
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum WorkflowError {
    /// A capability in the spine returned an error
    #[error("[{capability}] {code}: {message}")]
    CapabilityError {
        capability: String,
        code: String,
        message: String,
        step: u32,
    },

    /// The workflow was halted because a precondition was not met
    #[error("Precondition failed at step {step}: {reason}")]
    PreconditionFailed {
        step: u32,
        reason: String,
    },

    /// Bridge-level transport error
    #[error("Bridge error in {capability}: {message}")]
    BridgeError {
        capability: String,
        message: String,
    },
}

/// Result type for workflow operations.
pub type WorkflowResult<T> = Result<T, WorkflowError>;
