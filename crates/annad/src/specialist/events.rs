//! Ticket Lifecycle Events for fly-on-the-wall display.
//!
//! These events are emitted during ticket processing for internal communications.
//! Phase 10 is plumbing only - no UI rendering in this phase.

use serde::{Deserialize, Serialize};

/// Ticket lifecycle events for internal communications display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum TicketEvent {
    /// New ticket created.
    Created {
        ticket_id: String,
        department: String,
        question_summary: String,
    },

    /// Ticket assigned to specialist.
    Assigned {
        ticket_id: String,
        specialist_id: String,
        specialist_name: String,
        department: String,
    },

    /// Specialist working on ticket.
    Working {
        ticket_id: String,
        specialist_id: String,
        /// What the specialist is doing (e.g., "running df -h").
        action: String,
    },

    /// Ticket escalated to senior.
    Escalated {
        ticket_id: String,
        from_specialist: String,
        to_specialist: String,
        reason: String,
    },

    /// Ticket resolved successfully.
    Resolved {
        ticket_id: String,
        specialist_id: String,
        specialist_name: String,
        confidence: f32,
        learned_recipe: bool,
    },

    /// Ticket failed.
    Failed {
        ticket_id: String,
        specialist_id: Option<String>,
        reason: String,
    },
}

impl TicketEvent {
    /// Create a ticket created event.
    pub fn created(ticket_id: &str, department: &str, question: &str) -> Self {
        TicketEvent::Created {
            ticket_id: ticket_id.to_string(),
            department: department.to_string(),
            question_summary: truncate_question(question, 50),
        }
    }

    /// Create a ticket assigned event.
    pub fn assigned(
        ticket_id: &str,
        specialist_id: &str,
        specialist_name: &str,
        department: &str,
    ) -> Self {
        TicketEvent::Assigned {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.to_string(),
            specialist_name: specialist_name.to_string(),
            department: department.to_string(),
        }
    }

    /// Create a working event.
    pub fn working(ticket_id: &str, specialist_id: &str, action: &str) -> Self {
        TicketEvent::Working {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.to_string(),
            action: action.to_string(),
        }
    }

    /// Create an escalated event.
    pub fn escalated(ticket_id: &str, from: &str, to: &str, reason: &str) -> Self {
        TicketEvent::Escalated {
            ticket_id: ticket_id.to_string(),
            from_specialist: from.to_string(),
            to_specialist: to.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a resolved event.
    pub fn resolved(
        ticket_id: &str,
        specialist_id: &str,
        specialist_name: &str,
        confidence: f32,
        learned_recipe: bool,
    ) -> Self {
        TicketEvent::Resolved {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.to_string(),
            specialist_name: specialist_name.to_string(),
            confidence,
            learned_recipe,
        }
    }

    /// Create a failed event.
    pub fn failed(ticket_id: &str, specialist_id: Option<&str>, reason: &str) -> Self {
        TicketEvent::Failed {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.map(|s| s.to_string()),
            reason: reason.to_string(),
        }
    }

    /// Get the ticket ID for this event.
    pub fn ticket_id(&self) -> &str {
        match self {
            TicketEvent::Created { ticket_id, .. }
            | TicketEvent::Assigned { ticket_id, .. }
            | TicketEvent::Working { ticket_id, .. }
            | TicketEvent::Escalated { ticket_id, .. }
            | TicketEvent::Resolved { ticket_id, .. }
            | TicketEvent::Failed { ticket_id, .. } => ticket_id,
        }
    }
}

/// Truncate a question for display.
fn truncate_question(question: &str, max_len: usize) -> String {
    if question.len() <= max_len {
        question.to_string()
    } else {
        format!("{}...", &question[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_created() {
        let event = TicketEvent::created("CN-0001-13012026", "System", "how much disk space");
        match event {
            TicketEvent::Created {
                ticket_id,
                department,
                question_summary,
            } => {
                assert_eq!(ticket_id, "CN-0001-13012026");
                assert_eq!(department, "System");
                assert_eq!(question_summary, "how much disk space");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_ticket_assigned() {
        let event = TicketEvent::assigned("CN-0001-13012026", "sys-jr", "James", "System");
        assert_eq!(event.ticket_id(), "CN-0001-13012026");
    }

    #[test]
    fn test_truncate_question() {
        let short = "hello";
        assert_eq!(truncate_question(short, 50), "hello");

        let long = "This is a very long question that should be truncated for display";
        let truncated = truncate_question(long, 30);
        assert!(truncated.len() <= 30);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_ticket_escalated() {
        let event = TicketEvent::escalated(
            "CN-0001-13012026",
            "net-jr",
            "net-sr",
            "Complex routing issue",
        );
        match event {
            TicketEvent::Escalated {
                from_specialist,
                to_specialist,
                ..
            } => {
                assert_eq!(from_specialist, "net-jr");
                assert_eq!(to_specialist, "net-sr");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_ticket_resolved() {
        let event = TicketEvent::resolved("CN-0001-13012026", "sys-jr", "James", 0.92, true);
        match event {
            TicketEvent::Resolved {
                confidence,
                learned_recipe,
                ..
            } => {
                assert!((confidence - 0.92).abs() < 0.01);
                assert!(learned_recipe);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
