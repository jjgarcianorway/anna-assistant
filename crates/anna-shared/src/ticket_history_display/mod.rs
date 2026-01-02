//! Ticket History Display (Phase 69)
//!
//! Provides display functions for viewing past ticket history with outcomes,
//! specialists involved, and resolution details.

pub mod formatters;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items for backwards compatibility
pub use formatters::{
    format_duration, format_ticket_history, format_ticket_history_compact,
    format_ticket_history_oneline, format_timestamp, is_ticket_history_query,
    ticket_history_fun_fact,
};
pub use types::{HistoricalTicket, TicketHistory, TicketOutcome};
