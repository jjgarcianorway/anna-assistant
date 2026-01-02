// v0.0.637: Watcher Tests (Phase 213)
// Test suite for settings watcher components

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::watcher::*;
    use super::super::registry::*;
    use super::super::utils::*;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_watcher_type_display() {
        assert_eq!(format!("{}", WatcherType::Polling), "polling");
        assert_eq!(format!("{}", WatcherType::EventBased), "event_based");
    }

    #[test]
    fn test_interval_display() {
        assert_eq!(format!("{}", WatchInterval::Normal), "normal");
        assert_eq!(format!("{}", WatchInterval::Custom(500)), "custom_500ms");
    }

    #[test]
    fn test_config_new() {
        let c = WatcherConfig::new(WatcherType::Polling);
        assert!(c.active);
    }

    #[test]
    fn test_config_builder() {
        let c = WatcherConfig::new(WatcherType::Hybrid)
            .interval(WatchInterval::Fast)
            .category(SettingsCategory::Privacy);
        assert_eq!(c.interval, WatchInterval::Fast);
    }

    #[test]
    fn test_event_new() {
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(e.old_value.is_none());
    }

    #[test]
    fn test_event_change() {
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "new")
            .old_value("old");
        assert!(e.is_change());
    }

    #[test]
    fn test_watcher_new() {
        let w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        assert!(w.is_active());
    }

    #[test]
    fn test_watcher_poll() {
        let mut w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        w.record_poll(1000);
        assert_eq!(w.last_poll, 1000);
    }

    #[test]
    fn test_watcher_matches() {
        let w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(w.matches(&e));
    }

    #[test]
    fn test_stats_record() {
        let mut s = WatcherStats::default();
        s.record_poll();
        s.record_change();
        assert_eq!(s.total_polls, 1);
        assert_eq!(s.changes_detected, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsWatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsWatcherRegistry::new();
        r.register(Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_watcher_query() {
        assert!(is_watcher_query("settings watcher"));
        assert!(!is_watcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = watcher_fun_fact();
        assert!(fact.contains("watcher"));
    }
}
