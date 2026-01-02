// v0.0.635: Settings Broadcaster Tests (Phase 211)
// Unit tests for the broadcaster

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::types::{BroadcastChannel, BroadcastMode};
    use super::super::config::BroadcasterConfig;
    use super::super::message::BroadcastMessage;
    use super::super::listener::ListenerInfo;
    use super::super::stats::BroadcasterStats;
    use super::super::broadcaster::SettingsBroadcaster;
    use super::super::utils::{is_broadcaster_query, broadcaster_fun_fact};

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", BroadcastChannel::Default), "default");
        assert_eq!(format!("{}", BroadcastChannel::System), "system");
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", BroadcastMode::Sync), "sync");
        assert_eq!(format!("{}", BroadcastMode::Async), "async");
    }

    #[test]
    fn test_config_new() {
        let c = BroadcasterConfig::new(BroadcastChannel::Default);
        assert!(c.active);
        assert_eq!(c.max_listeners, 100);
    }

    #[test]
    fn test_config_builder() {
        let c = BroadcasterConfig::new(BroadcastChannel::System)
            .mode(BroadcastMode::Async)
            .max_listeners(50);
        assert_eq!(c.mode, BroadcastMode::Async);
        assert_eq!(c.max_listeners, 50);
    }

    #[test]
    fn test_message_new() {
        let m = BroadcastMessage::new(
            "m1",
            BroadcastChannel::Default,
            SettingsCategory::Privacy,
            "key",
            "payload",
        );
        assert_eq!(m.key, "key");
    }

    #[test]
    fn test_listener_new() {
        let l = ListenerInfo::new("l1", "Test", BroadcastChannel::Default);
        assert_eq!(l.message_count, 0);
    }

    #[test]
    fn test_listener_record() {
        let mut l = ListenerInfo::new("l1", "Test", BroadcastChannel::Default);
        l.record_message();
        assert_eq!(l.message_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = BroadcasterStats::default();
        s.record_broadcast(5);
        assert_eq!(s.total_broadcasts, 1);
        assert_eq!(s.delivered, 5);
    }

    #[test]
    fn test_broadcaster_new() {
        let b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        assert!(b.is_active());
    }

    #[test]
    fn test_broadcaster_add_listener() {
        let mut b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        assert!(b.add_listener(ListenerInfo::new("l1", "Test", BroadcastChannel::Default)));
        assert_eq!(b.listener_count(), 1);
    }

    #[test]
    fn test_broadcaster_broadcast() {
        let mut b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        b.add_listener(ListenerInfo::new("l1", "Test", BroadcastChannel::Default));
        let count = b.broadcast(BroadcastMessage::new(
            "m1",
            BroadcastChannel::Default,
            SettingsCategory::Privacy,
            "key",
            "payload",
        ));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_is_broadcaster_query() {
        assert!(is_broadcaster_query("settings broadcaster"));
        assert!(!is_broadcaster_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = broadcaster_fun_fact();
        assert!(fact.contains("broadcaster"));
    }
}
