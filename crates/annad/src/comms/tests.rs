//! Tests for comms module (v0.0.192).
//! v0.0.266: Added test for team_from_query_class.

#[cfg(test)]
mod tests {
    use anna_shared::teams::Team;

    use crate::comms::{team_from_domain, team_from_query_class, CommsGenerator};
    use crate::progress_tracker::ProgressTracker;

    #[test]
    fn test_comms_generator_creates_messages() {
        let gen = CommsGenerator::new(Team::Desktop, "test-case-123");
        let mut progress = ProgressTracker::new();

        gen.dispatch(&mut progress);
        assert!(!progress.events().is_empty());
    }

    #[test]
    fn test_team_from_domain() {
        assert_eq!(team_from_domain("storage"), Team::Storage);
        assert_eq!(team_from_domain("NETWORK"), Team::Network);
        assert_eq!(team_from_domain("performance"), Team::Performance);
        assert_eq!(team_from_domain("system"), Team::Performance);
        assert_eq!(team_from_domain("services"), Team::Services);
        assert_eq!(team_from_domain("hardware"), Team::Hardware);
        assert_eq!(team_from_domain("logs"), Team::Logs);
        assert_eq!(team_from_domain("unknown"), Team::Desktop);
    }

    #[test]
    fn test_team_from_query_class() {
        // Config queries always go to Desktop team
        assert_eq!(team_from_query_class("configure_editor", "system"), Team::Desktop);
        assert_eq!(team_from_query_class("configure_shell", "system"), Team::Desktop);
        assert_eq!(team_from_query_class("configure_git", "system"), Team::Desktop);

        // Other queries fall back to domain routing
        assert_eq!(team_from_query_class("cpu_info", "system"), Team::Performance);
        assert_eq!(team_from_query_class("disk_usage", "storage"), Team::Storage);
        assert_eq!(team_from_query_class("unknown", "network"), Team::Network);
    }
}
