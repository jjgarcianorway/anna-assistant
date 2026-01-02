//! Explicit ticket lifecycle and state machine (v0.0.407).
//!
//! v0.0.411: Added explicit TicketOutcome for truthful stats
//!
//! Provides truthful tracking of ticket outcomes with:
//! - Explicit state transitions
//! - Error classification
//! - Handler tracking
//! - Stats alignment
//!
//! States flow: Created → Planned → ProbesRun → [DocsAttached] →
//!              LlmRequested → Answered/LlmFailed → Success/Failed
//!
//! Outcomes (semantic meaning):
//! - Success: User got correct, grounded answer
//! - Partial: Some info, but limitations explained
//! - CannotAnswerSafely: Not enough evidence, or too risky
//! - ErrorParse: LLM response invalid
//! - ErrorTimeout: LLM or probe timeout
//! - ErrorTool: Probe or helper failed
//! - ErrorInternal: Unexpected internal failure

mod error;
mod handler;
mod live_ticket;
mod outcome;
mod state;
mod transition;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use error::ErrorKind;
pub use handler::{HandlerType, SolverTier};
pub use live_ticket::LiveTicket;
pub use outcome::TicketOutcome;
pub use state::TicketState;
pub use transition::StateTransition;
