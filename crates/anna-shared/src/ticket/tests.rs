//! Tests for ticket module (v0.0.215).

#[cfg(test)]
mod tests {
    use crate::review::ReviewArtifact;
    use crate::teams::Team;
    use crate::ticket::{RiskLevel, Ticket, TicketStatus};
    use crate::trace::EvidenceKind;

    #[test]
    fn test_ticket_creation() {
        let ticket = Ticket::new(
            "test-123".to_string(),
            "how is my computer doing?".to_string(),
            "system".to_string(),
            "investigate".to_string(),
            Team::Performance,
            "SystemHealth".to_string(),
            true,
            vec!["free -h".to_string(), "df -h".to_string()],
            vec![EvidenceKind::Memory, EvidenceKind::Disk],
            RiskLevel::ReadOnly,
        );

        assert_eq!(ticket.status, TicketStatus::New);
        assert_eq!(ticket.team, Team::Performance);
        assert!(ticket.can_retry_junior());
        assert!(ticket.can_escalate());
        assert!(!ticket.is_exhausted());
        assert!(ticket.review_artifacts.is_empty());
    }

    #[test]
    fn test_ticket_review_artifacts() {
        let mut ticket = Ticket::default();
        ticket.team = Team::Storage;

        assert!(!ticket.can_publish());

        // Add a passing review
        let artifact = ReviewArtifact::pass(Team::Storage, "junior", 85);
        ticket.add_review_artifact(artifact);

        assert!(ticket.can_publish());
        assert_eq!(ticket.latest_review().unwrap().score, 85);
    }

    #[test]
    fn test_junior_retry_limits() {
        let mut ticket = Ticket::default();
        ticket.junior_rounds_max = 3;

        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(!ticket.can_retry_junior());
    }

    #[test]
    fn test_senior_escalation_limits() {
        let mut ticket = Ticket::default();
        ticket.senior_rounds_max = 1;

        assert!(ticket.can_escalate());
        ticket.increment_senior();
        assert!(!ticket.can_escalate());
    }

    #[test]
    fn test_exhausted_state() {
        let mut ticket = Ticket::default();
        ticket.junior_rounds_max = 1;
        ticket.senior_rounds_max = 1;

        assert!(!ticket.is_exhausted());

        ticket.increment_junior();
        assert!(!ticket.is_exhausted()); // Can still escalate

        ticket.increment_senior();
        assert!(ticket.is_exhausted()); // All attempts exhausted
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::ReadOnly.to_string(), "read-only");
        assert_eq!(RiskLevel::LowRiskChange.to_string(), "low-risk-change");
        assert_eq!(RiskLevel::HighRiskChange.to_string(), "high-risk-change");
    }

    #[test]
    fn test_ticket_status_display() {
        assert_eq!(TicketStatus::New.to_string(), "new");
        assert_eq!(TicketStatus::Probing.to_string(), "probing");
        assert_eq!(TicketStatus::Verified.to_string(), "verified");
        assert_eq!(TicketStatus::Escalated.to_string(), "escalated");
        assert_eq!(TicketStatus::Failed.to_string(), "failed");
    }
}
