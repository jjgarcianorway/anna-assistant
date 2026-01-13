//! Tests for status types.

#[cfg(test)]
mod tests {
    use crate::status::{
        DaemonState, ErrorSummary, LearningStatus, PermissionsAudit, RpgStats, SocketStatus,
        TicketStatus, BuildMetadata,
    };

    #[test]
    fn test_daemon_state_display() {
        assert_eq!(DaemonState::Ready.to_string(), "READY");
        assert_eq!(DaemonState::Starting.to_string(), "STARTING");
        assert_eq!(DaemonState::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_rpg_stats_xp_bar() {
        let mut stats = RpgStats::default();
        stats.xp = 0;
        assert!(stats.xp_bar().contains("0%"));

        stats.xp = 50;
        assert!(stats.xp_bar().contains("50%"));

        stats.xp = 100;
        assert!(stats.xp_bar().contains("100%"));
    }

    #[test]
    fn test_rpg_stats_title_progression() {
        // Verify titles progress correctly
        assert_eq!(RpgStats::get_title(0), "Novice Apprentice");
        assert_eq!(RpgStats::get_title(10), "Junior Technician");
        assert_eq!(RpgStats::get_title(50), "Senior Specialist");
        assert_eq!(RpgStats::get_title(100), "Omniscient Oracle");
    }

    #[test]
    fn test_rpg_stats_calculate_xp() {
        let mut stats = RpgStats::default();
        stats.total_questions = 100;
        stats.instant_answers = 50;
        stats.memory_answers = 25;
        stats.llm_answers = 25;
        stats.recipes_learned = 10;
        stats.reliability = 0.9;

        stats.calculate_xp();

        assert!(stats.xp > 0);
        assert!(stats.xp <= 100);
        assert!(!stats.title.is_empty());
    }

    #[test]
    fn test_socket_status_display() {
        assert_eq!(SocketStatus::Healthy.to_string(), "HEALTHY");
        assert_eq!(SocketStatus::NotFound.to_string(), "NOT_FOUND");
        assert_eq!(SocketStatus::PermissionDenied.to_string(), "PERMISSION_DENIED");
    }

    #[test]
    fn test_ticket_status_display() {
        assert_eq!(TicketStatus::Open.to_string(), "open");
        assert_eq!(TicketStatus::InProgress.to_string(), "in-progress");
        assert_eq!(TicketStatus::Resolved.to_string(), "resolved");
    }

    /// v0.3.29: Test new investigation/experiment ticket states
    #[test]
    fn test_ticket_status_investigation_states() {
        assert_eq!(TicketStatus::Investigating.to_string(), "investigating");
        assert_eq!(TicketStatus::Experimenting.to_string(), "experimenting");
        assert_eq!(TicketStatus::Failed.to_string(), "failed");
        assert_eq!(TicketStatus::Escalated.to_string(), "escalated");
    }

    /// v0.3.29: Test learning status defaults
    #[test]
    fn test_learning_status_defaults() {
        let status = LearningStatus::default();
        assert!(!status.enabled); // Default is false
        assert_eq!(status.candidate_skills, 0);
        assert_eq!(status.probation_skills, 0);
        assert_eq!(status.trusted_skills, 0);
    }

    #[test]
    fn test_build_metadata_display() {
        let mut meta = BuildMetadata::default();
        meta.version = "0.3.22".to_string();
        meta.git_sha = "abc1234".to_string();
        meta.git_dirty = false;

        let display = meta.display();
        assert!(display.contains("0.3.22"));
        assert!(display.contains("abc1234"));

        meta.git_dirty = true;
        let display = meta.display();
        assert!(display.contains("*")); // dirty marker
    }

    #[test]
    fn test_error_summary_add_error() {
        let mut summary = ErrorSummary::default();

        summary.add_error("E001", "Test error", Some("test"), true);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.recent_errors.len(), 1);

        // Add 6 errors, should only keep 5
        for i in 2..=6 {
            summary.add_error(&format!("E{:03}", i), &format!("Error {}", i), None, false);
        }
        assert_eq!(summary.error_count, 6);
        assert_eq!(summary.recent_errors.len(), 5);
    }

    #[test]
    fn test_permissions_audit_check() {
        // This test just verifies the function runs without panic
        let perms = PermissionsAudit::check();
        assert!(!perms.user.is_empty());
    }
}
