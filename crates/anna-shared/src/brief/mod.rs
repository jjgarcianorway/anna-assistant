//! Ticket Brief with team-relevance filtering (v0.0.229).
//!
//! Specialists only see evidence relevant to their domain.
//! Storage team sees disk/lsblk, not memory/systemd noise.
//!
//! v0.0.32: Initial implementation.
//! v0.0.229: Modularized into domain-focused submodules.

mod filtering;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use filtering::{
    evidence_kind_for_probe, is_probe_relevant, relevant_evidence_for_team, PROBE_PATTERNS,
};
pub use types::{build_brief_from_ticket, TicketBrief};
