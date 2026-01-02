//! Senior strategic thinking during idle time (v0.0.458).
//!
//! During idle time, senior specialists think strategically:
//! - Analyze patterns in past tickets
//! - Identify recurring issues
//! - Suggest preventive measures
//! - Generate weekly insights
//!
//! v0.0.458: Initial implementation per VISION.md Phase 37.

mod formatters;
mod pattern_detector;
mod types;
mod utils;

// Re-export public types
pub use formatters::format_insights_email;
pub use pattern_detector::PatternDetector;
pub use types::{
    AnalysisCheckpoint, DetectedPattern, InsightCategory, InsightPriority, StrategicInsight,
    StrategicSession, TicketSummary,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teams::Team;

    #[test]
    fn test_session_creation() {
        let session = StrategicSession::new();
        assert!(session.session_id.starts_with("SESS-"));
        assert!(!session.is_complete());
    }

    #[test]
    fn test_session_completion() {
        let mut session = StrategicSession::new();
        session.complete();
        assert!(session.is_complete());
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_checkpoint_resume() {
        let mut session = StrategicSession::new();
        session.create_checkpoint("TKT-001", Team::Storage);

        let checkpoint = session.resume().unwrap();
        assert_eq!(checkpoint.last_ticket_id, "TKT-001");
        assert_eq!(checkpoint.current_team, Team::Storage);
    }

    #[test]
    fn test_pattern_detection() {
        let tickets = vec![
            TicketSummary {
                ticket_id: "1".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 1000,
                resolved_at: Some(1100),
            },
            TicketSummary {
                ticket_id: "2".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 2000,
                resolved_at: Some(2100),
            },
            TicketSummary {
                ticket_id: "3".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 3000,
                resolved_at: Some(3100),
            },
        ];

        let patterns = PatternDetector::detect_patterns(&tickets);
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, "disk_space");
        assert_eq!(patterns[0].occurrence_count, 3);
    }

    #[test]
    fn test_priority_for_count() {
        use pattern_detector::*;
        // Note: priority_for_count is private, so we test via patterns_to_insights
        let pattern = DetectedPattern {
            team: Team::Storage,
            pattern_type: "test".to_string(),
            occurrence_count: 15,
            first_seen: 1000,
            last_seen: 2000,
        };

        let insights = PatternDetector::patterns_to_insights(vec![pattern], "TestSpec");
        assert_eq!(insights[0].priority, InsightPriority::Critical);
    }

    #[test]
    fn test_email_formatting() {
        let mut session = StrategicSession::new();
        session.tickets_analyzed = 50;
        session.add_insight(StrategicInsight {
            id: "INS-001".to_string(),
            team: Team::Storage,
            specialist: "Eva".to_string(),
            category: InsightCategory::CapacityPlanning,
            title: "Disk space pattern".to_string(),
            analysis: "Recurring disk space queries".to_string(),
            recommendations: vec!["Add monitoring".to_string()],
            priority: InsightPriority::High,
            generated_at: 1000,
            ticket_count: 5,
            period_days: 7,
        });

        let email = format_insights_email(&session);
        assert!(email.contains("Weekly Strategic Analysis"));
        assert!(email.contains("Disk space pattern"));
        assert!(email.contains("HIGH"));
    }
}
