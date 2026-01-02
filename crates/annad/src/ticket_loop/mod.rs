//! Ticket verification loop with bounded retries and escalation (v0.0.401).
//!
//! Wraps the service desk answer with:
//! - Junior verification (bounded by junior_rounds_max)
//! - Senior escalation when junior exhausted
//! - v0.0.297: LLM-based self-healing for failed validations
//! - v0.0.376: Domain-specific validation thresholds
//! - v0.0.401: Specialist learning capture (learn from escalations)
//! - Revision application between rounds
//! - Full transcript visibility

mod evidence;
mod junior;
mod runner;
mod senior;
mod types;

// Re-export public API to preserve existing interface
pub use runner::run_ticket_loop;
pub use types::TicketLoopResult;
