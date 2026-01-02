// v0.0.639: Settings Notifier - Tests (Phase 215)
// Unit tests for settings notifier module

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::channel::NotifyChannel;
    use super::super::priority::NotifyPriority;
    use super::super::config::NotifierConfig;
    use super::super::notification::Notification;
    use super::super::stats::NotifierStats;
    use super::super::notifier::SettingsNotifier;
    use super::super::registry::{
        SettingsNotifierRegistry,
        is_notifier_query,
        notifier_fun_fact,
    };

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", NotifyChannel::Internal), "internal");
        assert_eq!(format!("{}", NotifyChannel::Log), "log");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", NotifyPriority::Normal), "normal");
        assert_eq!(format!("{}", NotifyPriority::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = NotifierConfig::new(NotifyChannel::Internal);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = NotifierConfig::new(NotifyChannel::Log)
            .priority_threshold(NotifyPriority::High)
            .debounce_ms(100);
        assert_eq!(c.priority_threshold, NotifyPriority::High);
        assert_eq!(c.debounce_ms, 100);
    }

    #[test]
    fn test_notification_new() {
        let n = Notification::new(
            "n1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        );
        assert!(n.message.is_empty());
    }

    #[test]
    fn test_notification_message() {
        let n = Notification::new(
            "n1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        )
        .message("test");
        assert_eq!(n.message, "test");
    }

    #[test]
    fn test_stats_record() {
        let mut s = NotifierStats::default();
        s.record_sent(NotifyPriority::Normal);
        s.record_suppressed();
        assert_eq!(s.total_sent, 1);
        assert_eq!(s.suppressed, 1);
    }

    #[test]
    fn test_notifier_new() {
        let n = SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal));
        assert!(n.is_enabled());
    }

    #[test]
    fn test_notifier_queue() {
        let mut n = SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal));
        let notif = Notification::new(
            "not1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        );
        assert!(n.queue(notif));
        assert_eq!(n.pending_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsNotifierRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsNotifierRegistry::new();
        r.register(SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_notifier_query() {
        assert!(is_notifier_query("settings notifier"));
        assert!(!is_notifier_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notifier_fun_fact();
        assert!(fact.contains("notifier"));
    }
}
