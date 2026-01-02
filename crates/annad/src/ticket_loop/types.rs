//! Types for the ticket verification loop.

use anna_shared::ticket::Ticket;

/// Result of ticket verification loop
pub struct TicketLoopResult {
    /// Final answer after revisions
    pub answer: String,
    /// Final ticket state
    pub ticket: Ticket,
    /// Whether verification passed
    pub verified: bool,
    /// Final reliability score
    pub score: u8,
}
