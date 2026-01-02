//! Error response handlers and validation (v0.0.407).
//!
//! v0.0.411: Graceful failure messages with evidence and ticket info

use crate::ticket_state::{ErrorKind, LiveTicket, TicketState};
use super::types::ErrorResponse;

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
