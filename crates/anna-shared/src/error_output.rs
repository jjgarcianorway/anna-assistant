//! User-friendly error output (v0.0.407).
//!
//! v0.0.411: Graceful failure messages with evidence and ticket info
//!
//! Provides simple, honest messages for failed tickets.
//! No LLM terms, no percentages, just clear explanations.

use crate::ticket_state::{ErrorKind, LiveTicket, TicketOutcome, TicketState};

/// User-facing error response
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    /// Main message (1-2 sentences)
    pub message: String,
    /// Optional hint for next steps
    pub hint: Option<String>,
    /// Whether debug mode would help
    pub debug_helpful: bool,
}

impl ErrorResponse {
    /// Format for display
    pub fn format(&self, show_debug_hint: bool) -> String {
        let mut output = self.message.clone();

        if let Some(ref hint) = self.hint {
            output.push_str("\n\n");
            output.push_str(hint);
        }

        if show_debug_hint && self.debug_helpful {
            output.push_str("\n\n(Enable debug mode for details.)");
        }

        output
    }
}

/// Generate user-friendly error response from ticket
pub fn error_response(ticket: &LiveTicket) -> ErrorResponse {
    match (&ticket.state, &ticket.error_kind) {
        // LLM timeout
        (_, Some(ErrorKind::LlmTimeout)) => ErrorResponse {
            message: "This analysis exceeded my time limit. I stopped before making any changes.".to_string(),
            hint: Some("You can try again or run commands manually.".to_string()),
            debug_helpful: true,
        },

        // LLM parse error
        (_, Some(ErrorKind::LlmParseError)) => ErrorResponse {
            message: "I tried to analyze this but the response format was invalid. I did not run any changes.".to_string(),
            hint: Some("Please try again or use a direct command.".to_string()),
            debug_helpful: true,
        },

        // Probe failure
        (_, Some(ErrorKind::ProbeFailure)) => ErrorResponse {
            message: "I could not gather the system information needed. Some commands failed to run.".to_string(),
            hint: Some("Check that the necessary tools are installed.".to_string()),
            debug_helpful: true,
        },

        // Validation failed (v0.0.409: includes forbidden pattern detection)
        (_, Some(ErrorKind::ValidationFailed)) => {
            let detail = ticket.error_detail.as_deref().unwrap_or("");
            let message = if detail.contains("forbidden pattern") {
                "My analysis produced an invalid response that I caught before showing you. No changes were made.".to_string()
            } else {
                "I could not produce a reliable answer after multiple attempts. No changes were made.".to_string()
            };
            ErrorResponse {
                message,
                hint: Some("You can try rephrasing your question or using a direct command.".to_string()),
                debug_helpful: true,
            }
        }

        // Unsupported
        (_, Some(ErrorKind::Unsupported)) => ErrorResponse {
            message: "This type of request is not supported yet.".to_string(),
            hint: Some("Try rephrasing your question or using a more specific command.".to_string()),
            debug_helpful: false,
        },

        // Cancelled
        (_, Some(ErrorKind::Cancelled)) => ErrorResponse {
            message: "The request was cancelled.".to_string(),
            hint: None,
            debug_helpful: false,
        },

        // v0.0.411: Missing evidence
        (_, Some(ErrorKind::MissingEvidence)) => ErrorResponse {
            message: "I cannot answer safely because I did not collect enough evidence.".to_string(),
            hint: Some("Try asking a more specific question or providing more context.".to_string()),
            debug_helpful: true,
        },

        // v0.0.411: Unsafe to answer
        (_, Some(ErrorKind::UnsafeToAnswer)) => ErrorResponse {
            message: "I determined it would be unsafe to answer this without more verification.".to_string(),
            hint: Some("This could affect your system in unpredictable ways. Please consult documentation.".to_string()),
            debug_helpful: true,
        },

        // Internal error
        (_, Some(ErrorKind::InternalError)) => ErrorResponse {
            message: "An internal error occurred.".to_string(),
            hint: Some("Please try again.".to_string()),
            debug_helpful: true,
        },

        // LLM failed state without specific error kind
        (TicketState::LlmFailed, None) => ErrorResponse {
            message: "I could not safely complete this analysis. No changes were made.".to_string(),
            hint: Some("You can try again or use a direct command.".to_string()),
            debug_helpful: true,
        },

        // Generic failed state
        (TicketState::Failed, None) => ErrorResponse {
            message: "I could not complete this request.".to_string(),
            hint: Some("Please try again.".to_string()),
            debug_helpful: true,
        },

        // Not a failure state - shouldn't be called
        _ => ErrorResponse {
            message: "Unexpected state.".to_string(),
            hint: None,
            debug_helpful: false,
        },
    }
}

/// v0.0.411: Format failure message with evidence gathered
/// Shows what evidence was collected even when the analysis failed
pub fn format_failure_with_evidence(
    ticket: &LiveTicket,
    probes_gathered: &[String],
) -> String {
    let mut output = String::new();

    output.push_str("Anna: I tried to process this with my internal IT team but something went wrong in my reasoning.\n");
    output.push_str("I am not confident enough to answer safely.\n");

    // Show gathered evidence
    if !probes_gathered.is_empty() {
        output.push_str("\nEvidence I did gather:\n");
        for probe in probes_gathered.iter().take(5) {
            output.push_str(&format!("  - {}\n", truncate_evidence(probe, 60)));
        }
    }

    // Suggest next steps
    output.push_str("\nYou can:\n");
    output.push_str("  - Run these commands manually to check your system\n");
    output.push_str("  - Ask me again in simpler terms\n");
    output.push_str("  - Try a more specific question\n");

    // Add ticket info for debugging
    output.push_str(&format!("\n(Ticket: {} - {})\n", ticket.id, ticket.domain));

    output
}

/// v0.0.411: Format partial answer with explicit disclaimer
pub fn format_partial_answer(
    answer: &str,
    confident_parts: &str,
    uncertain_parts: &str,
    ticket_id: &str,
    handler: &str,
    evidence: &[String],
) -> String {
    let mut output = answer.to_string();

    // Add explicit disclaimer about partial confidence
    output.push_str("\n\nNote: This is a partial answer. ");
    if !confident_parts.is_empty() {
        output.push_str(&format!("I am confident about {}. ", confident_parts));
    }
    if !uncertain_parts.is_empty() {
        output.push_str(&format!("I am not sure about {}.", uncertain_parts));
    }

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(3).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    // Add ticket and handler info
    output.push_str(&format!("\n\nTicket: {}  handled by {}", ticket_id, handler));

    output
}

/// v0.0.411: Format successful answer with ticket info (PART D requirement)
pub fn format_success_with_ticket(
    answer: &str,
    ticket_id: &str,
    handler: &str,
    evidence: &[String],
) -> String {
    let mut output = answer.to_string();

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(3).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    // Add ticket and handler info
    output.push_str(&format!("\n\nTicket: {}  handled by {}", ticket_id, handler));

    output
}

/// v0.0.411: Format "missing evidence" response with retry options
pub fn format_missing_evidence(
    ticket: &LiveTicket,
    missing_probes: &[String],
) -> String {
    let mut output = String::new();

    output.push_str("Anna: I need more information to answer safely.\n");

    if !missing_probes.is_empty() {
        output.push_str("\nI would need:\n");
        for probe in missing_probes.iter().take(4) {
            output.push_str(&format!("  - {}\n", probe));
        }
    }

    output.push_str("\nYou can:\n");
    output.push_str("  - Run these commands and paste the output\n");
    output.push_str("  - Ask me to retry with more probes\n");
    output.push_str("  - Rephrase with more detail about your setup\n");

    output.push_str(&format!("\n(Ticket: {})\n", ticket.id));

    output
}

/// Format a successful answer with evidence (simplified)
///
/// Guidelines:
/// - Max 4-6 lines of main answer
/// - Optional "Evidence" section with 1-3 bullets
/// - No internal LLM terms
/// - No percent success meta statements
pub fn format_success_answer(
    answer: &str,
    evidence: &[String],
    max_evidence: usize,
) -> String {
    let mut output = answer.to_string();

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(max_evidence.min(3)).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    output
}

/// Truncate evidence item
fn truncate_evidence(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// v0.0.408: Format answer with knowledge item evidence
pub fn format_answer_with_knowledge(
    answer: &str,
    knowledge_titles: &[String],
    max_evidence: usize,
) -> String {
    let mut output = answer.to_string();

    let to_show: Vec<_> = knowledge_titles.iter().take(max_evidence.min(4)).collect();
    if !to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for title in to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(title, 80)));
        }
    }

    output
}

/// v0.0.408: Format a "cannot answer" response with suggestions
pub fn format_no_evidence_response(
    reason: &str,
    suggestions: &[String],
) -> String {
    let mut output = String::from("I cannot safely answer this from local data.");

    if !reason.is_empty() {
        output.push_str(&format!("\n\n{}", reason));
    }

    if !suggestions.is_empty() {
        output.push_str("\n\nYou can try:");
        for suggestion in suggestions.iter().take(5) {
            output.push_str(&format!("\n  - {}", suggestion));
        }
    }

    output
}

/// v0.0.408: Format knowledge search summary for debug
pub fn format_knowledge_debug(
    keywords: &[String],
    found_count: usize,
    source_types: &[String],
) -> String {
    format!(
        "Knowledge search: {} keywords, {} items found from [{}]",
        keywords.len(),
        found_count,
        source_types.join(", ")
    )
}

/// Check if an answer should be considered valid
///
/// v0.0.409: Also checks for forbidden patterns like "unknown is installed"
///
/// An answer is valid if:
/// - It's not empty
/// - It's not just whitespace
/// - It doesn't contain obvious error markers
/// - It doesn't contain forbidden patterns
pub fn is_valid_answer(answer: &str) -> bool {
    let trimmed = answer.trim();

    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_lowercase();

    // Check for obvious error markers
    let error_markers = [
        "failed to parse",
        "internal error",
        "timeout",
        "error occurred",
        "could not process",
    ];

    for marker in error_markers {
        if lower.contains(marker) {
            return false;
        }
    }

    // v0.0.409: Check for forbidden patterns
    let forbidden_patterns = [
        "unknown is installed",
        "unknown is not installed",
        "**unknown**",
        "2 is installed",
        "1 is installed",
    ];

    for pattern in forbidden_patterns {
        if lower.contains(pattern) {
            return false;
        }
    }

    // Check for minimum content (not just a single word)
    trimmed.split_whitespace().count() >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_timeout() {
        let mut ticket = LiveTicket::new("TEST-001", "Test");
        ticket.error_kind = Some(ErrorKind::LlmTimeout);

        let response = error_response(&ticket);
        assert!(response.message.contains("time limit"));
        assert!(response.debug_helpful);
    }

    #[test]
    fn test_error_response_parse_error() {
        let mut ticket = LiveTicket::new("TEST-002", "Test");
        ticket.error_kind = Some(ErrorKind::LlmParseError);

        let response = error_response(&ticket);
        assert!(response.message.contains("invalid"));
        assert!(response.message.contains("did not run"));
    }

    #[test]
    fn test_format_with_debug_hint() {
        let mut ticket = LiveTicket::new("TEST-003", "Test");
        ticket.error_kind = Some(ErrorKind::LlmTimeout);

        let response = error_response(&ticket);
        let formatted = response.format(true);
        assert!(formatted.contains("debug mode"));
    }

    #[test]
    fn test_format_success_answer() {
        let answer = "Your disk is 75% full.";
        let evidence = vec![
            "df -h shows /dev/sda1 at 75%".to_string(),
            "/home uses 50GB".to_string(),
        ];

        let formatted = format_success_answer(answer, &evidence, 3);
        assert!(formatted.contains("Your disk is 75% full"));
        assert!(formatted.contains("Evidence:"));
        assert!(formatted.contains("df -h"));
    }

    #[test]
    fn test_is_valid_answer() {
        assert!(is_valid_answer("Your disk is 75% full."));
        assert!(!is_valid_answer(""));
        assert!(!is_valid_answer("  "));
        assert!(!is_valid_answer("ok")); // Too short
        assert!(!is_valid_answer("Failed to parse response"));
    }

    #[test]
    fn test_is_valid_answer_forbidden_patterns() {
        // v0.0.409: These should all be invalid
        assert!(!is_valid_answer("unknown is installed on your system"));
        assert!(!is_valid_answer("unknown is not installed here"));
        assert!(!is_valid_answer("**unknown** was found"));
        assert!(!is_valid_answer("2 is installed on your machine"));
        assert!(!is_valid_answer("Yes, 1 is installed and running"));
    }

    #[test]
    fn test_error_response_validation_failed() {
        let mut ticket = LiveTicket::new("TEST-004", "Test");
        ticket.error_kind = Some(ErrorKind::ValidationFailed);
        ticket.error_detail = Some("forbidden pattern: unknown is installed".to_string());

        let response = error_response(&ticket);
        assert!(response.message.contains("invalid response"));
        assert!(response.message.contains("caught"));
    }

    #[test]
    fn test_format_failure_with_evidence() {
        let mut ticket = LiveTicket::new("DSK-0101", "What is my disk usage?");
        ticket.domain = "storage".to_string();
        ticket.error_kind = Some(ErrorKind::LlmParseError);

        let probes = vec![
            "df -h: /dev/sda1 75% used".to_string(),
            "lsblk: disk info".to_string(),
        ];

        let output = format_failure_with_evidence(&ticket, &probes);

        assert!(output.contains("I tried to process"));
        assert!(output.contains("Evidence I did gather"));
        assert!(output.contains("df -h"));
        assert!(output.contains("DSK-0101"));
    }

    #[test]
    fn test_format_partial_answer() {
        let output = format_partial_answer(
            "Your system has 16GB RAM.",
            "the total RAM",
            "current usage breakdown",
            "DSK-0102",
            "Sofia (Desktop)",
            &["meminfo: 16GB".to_string()],
        );

        assert!(output.contains("16GB RAM"));
        assert!(output.contains("partial answer"));
        assert!(output.contains("confident about the total RAM"));
        assert!(output.contains("not sure about current usage"));
        assert!(output.contains("DSK-0102"));
        assert!(output.contains("Sofia (Desktop)"));
    }

    #[test]
    fn test_format_success_with_ticket() {
        let output = format_success_with_ticket(
            "vim is installed at /usr/bin/vim.",
            "PKG-0001",
            "Tomas (Packages)",
            &["command -v vim: /usr/bin/vim".to_string()],
        );

        assert!(output.contains("vim is installed"));
        assert!(output.contains("PKG-0001"));
        assert!(output.contains("Tomas (Packages)"));
        assert!(output.contains("Evidence:"));
    }

    #[test]
    fn test_error_response_missing_evidence() {
        let mut ticket = LiveTicket::new("TEST-005", "Test");
        ticket.error_kind = Some(ErrorKind::MissingEvidence);

        let response = error_response(&ticket);
        assert!(response.message.contains("evidence"));
        assert!(response.debug_helpful);
    }

    #[test]
    fn test_error_response_unsafe() {
        let mut ticket = LiveTicket::new("TEST-006", "Test");
        ticket.error_kind = Some(ErrorKind::UnsafeToAnswer);

        let response = error_response(&ticket);
        assert!(response.message.contains("unsafe"));
    }
}
