//! Tests for user_profile module (v0.0.217).

#[cfg(test)]
mod tests {
    use crate::user_profile::{UserProfile, UserPreferences};

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert!(!profile.username.is_empty());
        assert!(profile.preferences.learning_mode);
        assert!(profile.preferences.show_internal_comms);
    }

    #[test]
    fn test_tool_usage_tracking() {
        let mut profile = UserProfile::default();
        profile.record_tool_usage("vim");
        profile.record_tool_usage("vim");
        profile.record_tool_usage("nano");

        assert_eq!(profile.tool_usage.get("vim"), Some(&2));
        assert_eq!(profile.tool_usage.get("nano"), Some(&1));
        assert_eq!(profile.preferred_editor, Some("vim".to_string()));
    }

    #[test]
    fn test_topic_tracking() {
        let mut profile = UserProfile::default();
        profile.record_topic("network");
        profile.record_topic("network");
        profile.record_topic("storage");

        assert_eq!(profile.top_topic(), Some(&"network".to_string()));
    }

    #[test]
    fn test_greeting_new_user() {
        let profile = UserProfile::default();
        let ctx = profile.greeting_context();
        assert!(ctx.is_new_user);
        let greeting = ctx.generate_greeting();
        assert!(greeting.contains("Welcome"));
    }

    #[test]
    fn test_learned_commands() {
        let mut profile = UserProfile::default();
        profile.record_learned_command("free -h");
        profile.record_learned_command("free -h"); // Duplicate
        profile.record_learned_command("df -h");

        assert_eq!(profile.learned_commands.len(), 2);
    }

    #[test]
    fn test_default_preferences() {
        let prefs = UserPreferences::default();
        assert!(prefs.learning_mode);
        assert_eq!(prefs.verbosity, 1);
        assert!(!prefs.auto_confirm_low_risk);
        assert!(prefs.show_internal_comms);
    }
}
