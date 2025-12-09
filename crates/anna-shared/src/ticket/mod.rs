//! Ticket types for service desk workflow (v0.0.215).
//!
//! Every user request becomes a Ticket with bounded iteration,
//! junior verification, and optional senior escalation.
//! Tickets are assigned to domain-specialized teams.
//!
//! v0.0.215: Modularized into domain-focused submodules.

mod clarification;
mod ticket_struct;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use ticket_struct::Ticket;
pub use types::{
    default_clarification_max, RiskLevel, TicketStatus, DEFAULT_JUNIOR_ROUNDS_MAX,
    DEFAULT_RELIABILITY_THRESHOLD, DEFAULT_SENIOR_ROUNDS_MAX,
};
