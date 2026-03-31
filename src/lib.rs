//! VoteOS — Sovereign Decision, Election, and Governance Legitimacy System
//!
//! Architecture:
//!   AxiaSystem (identity/legitimacy) → Rust Bridge → SpineClient
//!     → Domain State (elections, ballots, votes)
//!     → Workflows (voter registration, vote recording, result aggregation)
//!     → API (HTTP endpoints for election operations)
//!
//! VoteOS is a sovereign sibling to CivilOS.
//! Both depend on AxiaSystem. Neither depends on the other.
//! When deployed together, citizens carry identity between systems.

pub mod persistence;
pub mod spine;
pub mod domain;
pub mod workflows;
pub mod error;
pub mod adoption;
