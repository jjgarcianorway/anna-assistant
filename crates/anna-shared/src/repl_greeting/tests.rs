//! Tests for REPL greeting system.

#[cfg(test)]
mod tests {
    use crate::repl_greeting::{ReplGreeting, SessionContext, SystemStatus};

    #[test]
    fn test_first_time_greeting() {
        let greeting = ReplGreeting::first_time("test_user");
        assert!(greeting.first_time);
        assert_eq!(greeting.user_name, "test_user");
        let rendered = greeting.render();
        assert!(rendered.contains("First time here?"));
    }

    #[test]
    fn test_session_context() {
        let mut ctx = SessionContext::new();
        ctx.add_question("what's my disk usage?", Some("storage"), Some("STG-001"));

        assert_eq!(ctx.questions.len(), 1);
        assert_eq!(ctx.last_domain, Some("storage".to_string()));
        assert!(ctx.context_hint().unwrap().contains("storage"));
    }

    #[test]
    fn test_error_announcements() {
        let mut greeting = ReplGreeting::first_time("test_user");
        assert!(!greeting.has_errors());
        assert_eq!(greeting.system_status, SystemStatus::Ok);

        greeting.add_error("daemon", "Not running", Some("Run: systemctl start annad"));
        assert!(greeting.has_errors());
        assert_eq!(greeting.errors.len(), 1);
        assert_eq!(greeting.system_status, SystemStatus::Warn);

        let rendered = greeting.render();
        assert!(rendered.contains("Issues detected"));
        assert!(rendered.contains("[daemon]"));
        assert!(rendered.contains("Not running"));
        assert!(rendered.contains("Fix:"));
    }
}
