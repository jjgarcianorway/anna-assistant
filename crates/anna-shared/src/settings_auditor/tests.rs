// v0.0.691: Settings Auditor (Phase 267)
// Tests for settings auditor

#[cfg(test)]
mod tests {
    use crate::settings_auditor::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", AuditEventType::Read), "read");
        assert_eq!(format!("{}", AuditEventType::Write), "write");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", AuditSeverity::High), "high");
        assert_eq!(format!("{}", AuditSeverity::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = AuditorConfig::new();
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = AuditorConfig::new()
            .log_reads(true)
            .max_events(500);
        assert!(c.log_reads);
        assert_eq!(c.max_events, 500);
    }

    #[test]
    fn test_event_new() {
        let e = AuditEvent::new(1, AuditEventType::Write, "key");
        assert!(e.is_write());
    }

    #[test]
    fn test_event_values() {
        let e = AuditEvent::new(1, AuditEventType::Write, "key")
            .old_value("old")
            .new_value("new");
        assert_eq!(e.old_value, Some("old".to_string()));
        assert_eq!(e.new_value, Some("new".to_string()));
    }

    #[test]
    fn test_trail_new() {
        let t = AuditTrail::new();
        assert_eq!(t.total_events, 0);
    }

    #[test]
    fn test_trail_add() {
        let mut t = AuditTrail::new();
        t.add(AuditEvent::new(1, AuditEventType::Write, "key"));
        assert_eq!(t.total_events, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = AuditorStats::default();
        s.record(&AuditEvent::new(1, AuditEventType::Write, "key"));
        assert_eq!(s.write_events, 1);
    }

    #[test]
    fn test_auditor_new() {
        let a = SettingsAuditor::new(AuditorConfig::default());
        assert_eq!(a.stats().total_events, 0);
    }

    #[test]
    fn test_auditor_log_write() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_write("key", Some("old"), Some("new"));
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_log_create() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_create("key", "value");
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_log_delete() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_delete("key", "old_value");
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_disabled_reads() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_read("key");
        assert_eq!(a.stats().read_events, 0); // log_reads is false by default
    }

    #[test]
    fn test_registry_new() {
        let r = AuditorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AuditorRegistry::new();
        r.register("a1", SettingsAuditor::new(AuditorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_auditor_query() {
        assert!(is_auditor_query("audit settings"));
        assert!(!is_auditor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = auditor_fun_fact();
        assert!(fact.contains("auditor"));
    }
}
