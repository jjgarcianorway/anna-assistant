//! Tests for the contextual tips system.

#[cfg(test)]
mod tests {
    use super::super::handlers::*;
    use super::super::tips::*;
    use super::super::types::*;

    #[test]
    fn test_context_from_query() {
        let ctx = TipContext::from_query("How do I configure vim?");
        assert!(ctx.topics.contains("editor"));

        let ctx2 = TipContext::from_query("restart docker service");
        assert!(ctx2.topics.contains("containers"));
        assert!(ctx2.topics.contains("services"));
    }

    #[test]
    fn test_get_contextual_tips_editor() {
        let ctx = TipContext::from_query("vim config");
        let tips = get_contextual_tips(&ctx);
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| t.id.contains("editor")));
    }

    #[test]
    fn test_get_contextual_tips_docker() {
        let ctx = TipContext::from_query("docker compose up");
        let tips = get_contextual_tips(&ctx);
        assert!(tips.iter().any(|t| t.id.contains("docker")));
    }

    #[test]
    fn test_get_contextual_tips_general() {
        let ctx = TipContext::default(); // No topics
        let tips = get_contextual_tips(&ctx);
        assert!(!tips.is_empty());
        // Should get general tips
        assert!(tips.iter().any(|t| t.id.starts_with("general")));
    }

    #[test]
    fn test_learning_mode_tips() {
        let ctx = TipContext::default().with_learning_mode(true);
        let tips = get_contextual_tips(&ctx);
        assert!(tips.iter().any(|t| t.id.starts_with("learn")));
    }

    #[test]
    fn test_select_tip() {
        let tips = general_tips();
        let tip = select_tip(&tips, 0);
        assert!(tip.is_some());
    }

    #[test]
    fn test_format_tip_with_action() {
        let tip = ContextualTip {
            id: "test",
            message: "Test message",
            related_action: Some("do thing"),
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("do thing"));
    }

    #[test]
    fn test_format_tip_without_action() {
        let tip = ContextualTip {
            id: "test",
            message: "Test message",
            related_action: None,
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("Test message"));
        assert!(!formatted.contains("try:"));
    }

    #[test]
    fn test_get_tip_for_query() {
        let tip = get_tip_for_query("how to use git", false);
        // May or may not return a tip due to probability
        // Just check it doesn't panic
        if let Some(t) = tip {
            assert!(t.contains("Tip:"));
        }
    }

    #[test]
    fn test_multiple_topics() {
        let ctx = TipContext::from_query("docker network issues");
        assert!(ctx.topics.contains("containers"));
        assert!(ctx.topics.contains("network"));

        let tips = get_contextual_tips(&ctx);
        // Should have tips from both categories
        assert!(tips.len() >= 2);
    }
}
