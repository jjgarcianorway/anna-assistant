// v0.0.636: Listener Tests (Phase 212)
// Tests for settings listener functionality

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::*;

    #[test]
    fn test_listener_type_display() {
        assert_eq!(format!("{}", ListenerType::Passive), "passive");
        assert_eq!(format!("{}", ListenerType::Active), "active");
    }

    #[test]
    fn test_listener_state_display() {
        assert_eq!(format!("{}", ListenerState::Idle), "idle");
        assert_eq!(format!("{}", ListenerState::Listening), "listening");
    }

    #[test]
    fn test_config_new() {
        let c = ListenerConfig::new(ListenerType::Passive);
        assert!(c.auto_start);
        assert_eq!(c.buffer_size, 50);
    }

    #[test]
    fn test_config_builder() {
        let c = ListenerConfig::new(ListenerType::Active)
            .category(SettingsCategory::Privacy)
            .auto_start(false);
        assert!(c.category.is_some());
        assert!(!c.auto_start);
    }

    #[test]
    fn test_event_new() {
        let e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(!e.processed);
    }

    #[test]
    fn test_event_mark() {
        let mut e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        e.mark_processed();
        assert!(e.processed);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ListenerStats::default();
        s.record_received();
        s.record_processed();
        assert_eq!(s.total_received, 1);
        assert_eq!(s.processed, 1);
    }

    #[test]
    fn test_listener_new() {
        let l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        assert!(l.is_listening());
    }

    #[test]
    fn test_listener_pause_resume() {
        let mut l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        l.pause();
        assert!(!l.is_listening());
        l.resume();
        assert!(l.is_listening());
    }

    #[test]
    fn test_listener_receive() {
        let mut l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        let e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(l.receive(e));
        assert_eq!(l.buffer_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsListenerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsListenerRegistry::new();
        r.register(SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_listener_query() {
        assert!(is_listener_query("settings listener"));
        assert!(!is_listener_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = listener_fun_fact();
        assert!(fact.contains("listener"));
    }
}
