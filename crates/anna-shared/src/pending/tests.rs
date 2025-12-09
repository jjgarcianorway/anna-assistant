//! Pending clarification tests (v0.0.227).

#[cfg(test)]
mod tests {
    use crate::clarify::{ClarifyKind, ClarifyOption};
    use crate::pending::{ParseResult, PendingClarification};

    fn sample_options() -> Vec<ClarifyOption> {
        vec![
            ClarifyOption::new("vim", "Vim"),
            ClarifyOption::new("nano", "Nano"),
            ClarifyOption::new("__other__", "Other"),
            ClarifyOption::new("__cancel__", "Cancel"),
        ]
    }

    #[test]
    fn test_pending_creation() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor do you prefer?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "edit the config",
        );

        assert_eq!(pending.request_id, "req-123");
        assert_eq!(pending.options.len(), 4);
    }

    #[test]
    fn test_parse_number_input() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(
            pending.parse_input("1"),
            ParseResult::Selected("vim".to_string())
        );
        assert_eq!(
            pending.parse_input("2"),
            ParseResult::Selected("nano".to_string())
        );
    }

    #[test]
    fn test_parse_name_input() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(
            pending.parse_input("vim"),
            ParseResult::Selected("vim".to_string())
        );
        assert_eq!(
            pending.parse_input("VIM"),
            ParseResult::Selected("vim".to_string())
        );
    }

    #[test]
    fn test_parse_cancel() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(pending.parse_input("cancel"), ParseResult::Cancelled);
        assert_eq!(pending.parse_input("c"), ParseResult::Cancelled);
        assert_eq!(pending.parse_input("0"), ParseResult::Cancelled);
    }

    #[test]
    fn test_parse_custom() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(
            pending.parse_input("emacs"),
            ParseResult::Custom("emacs".to_string())
        );
    }

    #[test]
    fn test_format_prompt() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor do you prefer?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        let prompt = pending.format_prompt();
        assert!(prompt.contains("Which editor"));
        assert!(prompt.contains("1) Vim"));
        assert!(prompt.contains("2) Nano"));
    }

    #[test]
    fn test_staleness() {
        let mut pending = PendingClarification::new(
            "req-123",
            "test",
            vec![],
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert!(!pending.is_stale()); // Just created

        // Simulate old timestamp
        pending.created_at = 0;
        assert!(pending.is_stale());
    }
}
