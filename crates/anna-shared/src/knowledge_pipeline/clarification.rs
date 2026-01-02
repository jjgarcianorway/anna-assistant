//! Clarification protocol for specialists (v0.0.432).
//!
//! When a specialist needs more information to answer correctly,
//! they can request clarification through this protocol.

pub use super::clarification_protocol::ClarificationProtocol;
pub use super::clarification_request::ClarificationRequest;
pub use super::clarification_response::ClarificationResponse;
pub use super::clarification_types::{ClarificationOption, ClarificationType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_clarification() {
        let req =
            ClarificationRequest::choice("Which package manager?", vec!["pacman", "yay", "paru"])
                .with_default("pacman");

        assert_eq!(req.options.len(), 3);
        assert_eq!(req.default, Some("pacman".to_string()));

        let formatted = req.format();
        assert!(formatted.contains("Which package manager?"));
        assert!(formatted.contains("pacman"));
    }

    #[test]
    fn test_confirmation() {
        let req = ClarificationRequest::confirmation("Proceed with installation?")
            .with_reason("This will modify system packages");

        assert!(req.question.contains("Proceed"));
        assert_eq!(req.options.len(), 2);
    }

    #[test]
    fn test_protocol_flow() {
        let mut protocol = ClarificationProtocol::new();

        assert!(!protocol.has_pending());

        protocol.request(ClarificationRequest::value("Enter package name"));
        assert!(protocol.has_pending());
        assert_eq!(protocol.pending_count(), 1);

        let response = ClarificationResponse::with_value("firefox");
        protocol.resolve(response);

        assert!(!protocol.has_pending());
        assert_eq!(protocol.history().len(), 1);
    }

    #[test]
    fn test_skip_with_default() {
        let mut protocol = ClarificationProtocol::new();

        protocol
            .request(ClarificationRequest::choice("Pick one", vec!["a", "b"]).with_default("a"));

        let skipped = protocol.skip();
        assert!(skipped.is_some());
        assert_eq!(skipped.unwrap().value, "a");
        assert!(!protocol.has_pending());
    }
}
