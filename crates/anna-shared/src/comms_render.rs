//! Internal comms rendering based on ticket state (v0.0.407).
//!
//! Generates internal communication messages from ticket state, not LLM text.
//! These are deterministic, predictable messages for the "fly on the wall" experience.

use crate::ticket_state::{ErrorKind, HandlerType, LiveTicket, SolverTier, TicketState};

/// A rendered internal comms message
#[derive(Debug, Clone)]
pub struct CommsMessage {
    /// Time offset (e.g., "[0.2s]")
    pub time_label: String,
    /// Speaker (e.g., "Sofia (Desktop)")
    pub speaker: String,
    /// Message content
    pub message: String,
    /// Severity level for styling
    pub level: CommsLevel,
}

/// Severity level for comms message styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommsLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl CommsMessage {
    /// Format for display
    pub fn format(&self) -> String {
        format!("{} {}: {}", self.time_label, self.speaker, self.message)
    }
}

/// Render internal comms messages from a live ticket
pub fn render_comms(ticket: &LiveTicket, elapsed_secs: f64) -> Vec<CommsMessage> {
    let mut messages = vec![];
    let time_label = format!("[{:.1}s]", elapsed_secs);

    // Get staff name from handler
    let staff = staff_name_from_handler(ticket.handler.as_ref(), &ticket.domain);

    match ticket.state {
        TicketState::Created => {
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: "Anna".to_string(),
                message: format!("Ticket {} created: \"{}\"", ticket.id, truncate(&ticket.user_question, 50)),
                level: CommsLevel::Info,
            });
        }

        TicketState::Planned => {
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: "Translator".to_string(),
                message: format!(
                    "Classified as {}/{} (confidence: {:.0}%)",
                    ticket.domain, ticket.intent, ticket.confidence
                ),
                level: CommsLevel::Info,
            });
        }

        TicketState::ProbesRun => {
            let evidence = ticket.evidence_summary.as_deref().unwrap_or("system data collected");
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: format!("Probes complete: {}", truncate(evidence, 60)),
                level: CommsLevel::Info,
            });
        }

        TicketState::LlmRequested => {
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: "Analyzing data...".to_string(),
                level: CommsLevel::Info,
            });
        }

        TicketState::LlmFailed => {
            let error_msg = match ticket.error_kind.as_ref() {
                Some(ErrorKind::LlmTimeout) => "Analysis timed out",
                Some(ErrorKind::LlmParseError) => "LLM returned invalid format",
                Some(ErrorKind::ProbeFailure) => "Probe execution failed",
                _ => "Processing failed",
            };
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: "Anna".to_string(),
                message: format!("{}, ticket flagged as failed.", error_msg),
                level: CommsLevel::Error,
            });
        }

        TicketState::Answered => {
            let confidence_str = if ticket.confidence >= 80 {
                format!("high confidence ({:.0}%)", ticket.confidence)
            } else if ticket.confidence >= 60 {
                format!("moderate confidence ({:.0}%)", ticket.confidence)
            } else {
                format!("low confidence ({:.0}%)", ticket.confidence)
            };
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: format!("Answer ready with {}.", confidence_str),
                level: CommsLevel::Success,
            });
        }

        TicketState::CommandsRun => {
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: "Commands executed successfully.".to_string(),
                level: CommsLevel::Success,
            });
        }

        TicketState::Success => {
            let handler_desc = handler_description(ticket.handler.as_ref());
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: format!("Ticket resolved via {}.", handler_desc),
                level: CommsLevel::Success,
            });
        }

        TicketState::Failed => {
            let reason = ticket.error_kind.as_ref()
                .map(|k| format!(" ({})", k))
                .unwrap_or_default();
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: "Anna".to_string(),
                message: format!("Ticket marked as failed{}.", reason),
                level: CommsLevel::Error,
            });
        }

        TicketState::DocsAttached => {
            messages.push(CommsMessage {
                time_label: time_label.clone(),
                speaker: staff.clone(),
                message: "Documentation attached.".to_string(),
                level: CommsLevel::Info,
            });
        }
    }

    // Add escalation message if applicable
    if ticket.escalated {
        let path = ticket.escalation_path.as_deref().unwrap_or("escalated");
        messages.push(CommsMessage {
            time_label: format!("[{:.1}s]", elapsed_secs - 0.1),
            speaker: "Anna".to_string(),
            message: format!("Escalation: {}", path),
            level: CommsLevel::Warning,
        });
    }

    // Add retry message if applicable
    if ticket.retry_count > 0 {
        messages.push(CommsMessage {
            time_label: format!("[{:.1}s]", elapsed_secs - 0.05),
            speaker: staff.clone(),
            message: format!("Retried {} time(s).", ticket.retry_count),
            level: CommsLevel::Warning,
        });
    }

    messages
}

/// Get staff name from handler type
fn staff_name_from_handler(handler: Option<&HandlerType>, domain: &str) -> String {
    match handler {
        Some(HandlerType::Recipe { name }) => {
            format!("Recipe ({})", truncate(name, 20))
        }
        Some(HandlerType::Deterministic { route }) => {
            format!("Direct ({})", truncate(route, 20))
        }
        Some(HandlerType::LlmSolver { tier, .. }) => {
            let tier_str = match tier {
                SolverTier::Junior => "Jr",
                SolverTier::Senior => "Sr",
            };
            let domain_cap = capitalize(domain);
            format!("{} ({})", domain_cap, tier_str)
        }
        None => capitalize(domain),
    }
}

/// Get handler description
fn handler_description(handler: Option<&HandlerType>) -> String {
    match handler {
        Some(HandlerType::Recipe { name }) => format!("recipe:{}", name),
        Some(HandlerType::Deterministic { route }) => format!("deterministic:{}", route),
        Some(HandlerType::LlmSolver { tier, model }) => format!("LLM:{}/{}", tier, model),
        None => "unknown".to_string(),
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// Format a simple one-line comms summary
pub fn one_line_summary(ticket: &LiveTicket) -> String {
    let status = match ticket.state {
        TicketState::Success => "OK",
        TicketState::Failed | TicketState::LlmFailed => "FAILED",
        TicketState::Answered | TicketState::CommandsRun => "ANSWERED",
        _ => "PROCESSING",
    };

    let handler = handler_description(ticket.handler.as_ref());
    let duration = ticket.duration_ms() as f64 / 1000.0;

    format!(
        "[{}] {} - {} ({:.1}s)",
        status, ticket.id, handler, duration
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_success_comms() {
        let mut ticket = LiveTicket::new("TEST-001", "What is my disk usage?");
        ticket.mark_planned("storage", "diagnose");
        ticket.handler = Some(HandlerType::LlmSolver {
            tier: SolverTier::Junior,
            model: "qwen2.5".to_string(),
        });
        ticket.mark_answered("Your disk is 75% full", 85);
        ticket.mark_success();

        let messages = render_comms(&ticket, 1.5);
        assert!(!messages.is_empty());
        assert!(messages.iter().any(|m| m.level == CommsLevel::Success));
    }

    #[test]
    fn test_render_failure_comms() {
        let mut ticket = LiveTicket::new("TEST-002", "Test");
        ticket.mark_planned("system", "diagnose");
        ticket.mark_llm_failed(ErrorKind::LlmTimeout, None);

        let messages = render_comms(&ticket, 15.0);
        assert!(messages.iter().any(|m| m.level == CommsLevel::Error));
        assert!(messages.iter().any(|m| m.message.contains("timed out")));
    }

    #[test]
    fn test_one_line_summary() {
        let mut ticket = LiveTicket::new("TEST-003", "Test");
        ticket.handler = Some(HandlerType::Recipe { name: "check_disk".to_string() });
        ticket.mark_success();

        let summary = one_line_summary(&ticket);
        assert!(summary.contains("OK"));
        assert!(summary.contains("TEST-003"));
        assert!(summary.contains("recipe:check_disk"));
    }

    #[test]
    fn test_staff_name() {
        let junior = HandlerType::LlmSolver {
            tier: SolverTier::Junior,
            model: "test".to_string(),
        };
        assert_eq!(staff_name_from_handler(Some(&junior), "storage"), "Storage (Jr)");

        let recipe = HandlerType::Recipe { name: "check_disk".to_string() };
        assert!(staff_name_from_handler(Some(&recipe), "storage").contains("Recipe"));
    }
}
