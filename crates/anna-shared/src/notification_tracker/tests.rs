// v0.0.533: Notification Tests (Phase 109)
// Test suite for notification tracking system

#[cfg(test)]
mod tests {
    use crate::notification_tracker::*;

    #[test]
    fn test_notification_creation() {
        let notif = NotificationRecord::new(
            "NOTIF-001",
            NotificationChannel::Email,
            NotificationPriority::High,
            "Ticket Updated",
            "Your ticket has been resolved",
            "2024-01-01",
        );
        assert_eq!(notif.subject, "Ticket Updated");
        assert!(notif.is_pending());
    }

    #[test]
    fn test_notification_lifecycle() {
        let mut notif = NotificationRecord::new(
            "N-001",
            NotificationChannel::Libnotify,
            NotificationPriority::Normal,
            "Test",
            "Message",
            "ts",
        );
        notif.mark_sent("ts2");
        assert_eq!(notif.status, DeliveryStatus::Sent);
        notif.mark_delivered();
        assert_eq!(notif.status, DeliveryStatus::Delivered);
    }

    #[test]
    fn test_notification_failed() {
        let mut notif = NotificationRecord::new(
            "N-001",
            NotificationChannel::Email,
            NotificationPriority::Normal,
            "Test",
            "Msg",
            "ts",
        );
        notif.mark_failed("Connection refused");
        assert_eq!(notif.status, DeliveryStatus::Failed);
        assert!(notif.error.is_some());
    }

    #[test]
    fn test_tracker_create() {
        let mut tracker = NotificationTracker::new();
        let id = tracker.create(
            NotificationChannel::Terminal,
            NotificationPriority::Low,
            "Test",
            "Msg",
            "ts",
        );
        assert_eq!(tracker.total(), 1);
        assert!(tracker.get(&id).is_some());
    }

    #[test]
    fn test_set_email() {
        let mut tracker = NotificationTracker::new();
        assert!(!tracker.has_email());
        tracker.set_email("user@example.com");
        assert!(tracker.has_email());
    }

    #[test]
    fn test_should_suppress() {
        let mut tracker = NotificationTracker::new();
        tracker.set_cooldown(1000);
        tracker.record_sent(NotificationChannel::Libnotify, 100);
        assert!(tracker.should_suppress(&NotificationChannel::Libnotify, 500));
        assert!(!tracker.should_suppress(&NotificationChannel::Libnotify, 1200));
    }

    #[test]
    fn test_by_channel() {
        let mut tracker = NotificationTracker::new();
        tracker.create(NotificationChannel::Email, NotificationPriority::Normal, "A", "m", "ts");
        tracker.create(NotificationChannel::Email, NotificationPriority::Normal, "B", "m", "ts");
        tracker.create(NotificationChannel::Wall, NotificationPriority::Normal, "C", "m", "ts");
        assert_eq!(tracker.by_channel(&NotificationChannel::Email).len(), 2);
    }

    #[test]
    fn test_channel_stats() {
        let mut tracker = NotificationTracker::new();
        tracker.create(NotificationChannel::Terminal, NotificationPriority::Normal, "A", "m", "ts");
        tracker.create(NotificationChannel::Log, NotificationPriority::Normal, "B", "m", "ts");
        let stats = tracker.channel_stats();
        assert_eq!(stats.get(&NotificationChannel::Terminal), Some(&1));
    }

    #[test]
    fn test_is_notification_query() {
        assert!(is_notification_query("Send me an email"));
        assert!(is_notification_query("Show notifications"));
        assert!(is_notification_query("Set up alerts"));
        assert!(!is_notification_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notification_fun_fact();
        assert!(fact.contains("spam") || fact.contains("focus"));
    }
}
