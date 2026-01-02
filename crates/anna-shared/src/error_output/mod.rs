//! User-friendly error output (v0.0.407).
//!
//! v0.0.411: Graceful failure messages with evidence and ticket info
//!
//! Provides simple, honest messages for failed tickets.
//! No LLM terms, no percentages, just clear explanations.

mod types;
mod formatters;
mod handlers;

// Re-export all public items to preserve the API
pub use types::ErrorResponse;
pub use formatters::{
    format_failure_with_evidence,
    format_partial_answer,
    format_success_with_ticket,
    format_missing_evidence,
    format_success_answer,
    format_answer_with_knowledge,
    format_no_evidence_response,
    format_knowledge_debug,
};
pub use handlers::{
    error_response,
    is_valid_answer,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket_state::{ErrorKind, LiveTicket};

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
}
