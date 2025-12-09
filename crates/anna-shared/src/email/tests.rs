//! Tests for email module (v0.0.206).

#[cfg(test)]
mod tests {
    use crate::email::{format_email, EmailConfig, EmailNotification};
    use crate::ticket_tracker::Ticket;

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert!(config.user_email.is_none());
        assert!(!config.enabled);
    }

    #[test]
    fn test_format_ticket_created_email() {
        let ticket = Ticket::new(
            "CN-0001-06122025".to_string(),
            "How much RAM?".to_string(),
            "Hardware".to_string(),
        );
        let (subject, body) = format_email(&EmailNotification::TicketCreated(&ticket));
        assert!(subject.contains("CN-0001"));
        assert!(body.contains("How much RAM"));
        assert!(body.contains("annactl reply"));
    }

    #[test]
    fn test_format_needs_clarification_email() {
        let mut ticket = Ticket::new(
            "CN-0002-06122025".to_string(),
            "Configure vim".to_string(),
            "Desktop".to_string(),
        );
        ticket.pending_question = Some("Which vim feature?".to_string());

        let (subject, body) = format_email(&EmailNotification::NeedsClarification(&ticket));
        assert!(subject.contains("Clarification"));
        assert!(body.contains("Which vim feature"));
    }

    #[test]
    fn test_format_resolved_email() {
        let mut ticket = Ticket::new(
            "CN-0003-06122025".to_string(),
            "Disk space?".to_string(),
            "Storage".to_string(),
        );
        ticket.resolution = Some("50GB free on /home".to_string());
        ticket.reliability = Some(95);

        let (subject, body) = format_email(&EmailNotification::TicketResolved(&ticket));
        assert!(subject.contains("Resolved"));
        assert!(body.contains("50GB free"));
        assert!(body.contains("95%"));
    }
}
