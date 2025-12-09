//! Ticket tracking system for Service Desk Theatre (v0.0.183).
//!
//! v0.0.105: Case numbers, ticket lifecycle, and history.
//! v0.0.113: Async tickets with email notifications and user replies.
//! v0.0.183: Modularized into domain-focused submodules.
//!
//! Ticket format: CN-XXXX-DDMMYYYY
//! - CN: Case Number prefix
//! - XXXX: Sequential number (resets daily)
//! - DDMMYYYY: Date created
//!
//! Async Flow (v0.0.113):
//! - Quick queries: resolved immediately (< 5 seconds)
//! - Complex queries: become async tickets with PendingUser status
//! - Email sent when ticket is created, updated, or needs user input
//! - User replies via email or `annactl reply <case> <message>`

mod message;
mod status;
mod tests;
mod ticket;
mod tracker;

// Re-export main types
pub use message::TicketMessage;
pub use status::TicketStatus;
pub use ticket::Ticket;
pub use tracker::{TicketDomain, TicketStats, TicketTracker};
