//! Tests for comms module (v0.0.192).

#[cfg(test)]
mod tests {
    use anna_shared::teams::Team;

    use crate::comms::{team_from_domain, CommsGenerator};
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
}
