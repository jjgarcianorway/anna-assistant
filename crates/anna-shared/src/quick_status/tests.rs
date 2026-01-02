//! Tests for quick status functionality.

#[cfg(test)]
mod tests {
    use crate::quick_status::*;

    #[test]
    fn test_health_level_symbol() {
        assert_eq!(HealthLevel::Good.symbol(), "[OK]");
        assert_eq!(HealthLevel::Warning.symbol(), "[!]");
        assert_eq!(HealthLevel::Critical.symbol(), "[X]");
        assert_eq!(HealthLevel::Unknown.symbol(), "[?]");
    }

    #[test]
    fn test_status_item_format() {
        let item = StatusItem::new("Test", HealthLevel::Good, "All good");
        assert!(item.format().contains("[OK]"));
        assert!(item.format().contains("Test"));
        assert!(item.format().contains("All good"));

        let item_with_value = item.with_value("100%");
        assert!(item_with_value.format().contains("100%"));
    }

    #[test]
    fn test_quick_status_overall() {
        let mut status = QuickStatus::new();

        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        assert_eq!(status.overall, HealthLevel::Good);

        status.add(StatusItem::new("B", HealthLevel::Warning, "Warn"));
        assert_eq!(status.overall, HealthLevel::Warning);

        status.add(StatusItem::new("C", HealthLevel::Critical, "Bad"));
        assert_eq!(status.overall, HealthLevel::Critical);
    }

    #[test]
    fn test_generate_summary() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.add(StatusItem::new("B", HealthLevel::Good, "OK"));
        status.generate_summary();
        assert_eq!(status.summary, "All systems operational");

        let mut status2 = QuickStatus::new();
        status2.add(StatusItem::new("A", HealthLevel::Warning, "Warn"));
        status2.generate_summary();
        assert!(status2.summary.contains("1 warning"));

        let mut status3 = QuickStatus::new();
        status3.add(StatusItem::new("A", HealthLevel::Critical, "Bad"));
        status3.add(StatusItem::new("B", HealthLevel::Critical, "Bad"));
        status3.generate_summary();
        assert!(status3.summary.contains("2 critical issues"));
    }

    #[test]
    fn test_memory_status() {
        let ok = memory_status(50.0);
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = memory_status(80.0);
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = memory_status(95.0);
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_disk_status() {
        let ok = disk_status(50.0, "/");
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = disk_status(90.0, "/home");
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = disk_status(98.0, "/");
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_cpu_status() {
        let ok = cpu_status(2.0, 4); // 0.5 per core
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = cpu_status(6.0, 4); // 1.5 per core
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = cpu_status(12.0, 4); // 3.0 per core
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_service_status() {
        let running = service_status("nginx", true, false);
        assert_eq!(running.health, HealthLevel::Good);

        let stopped = service_status("nginx", false, false);
        assert_eq!(stopped.health, HealthLevel::Warning);

        let failed = service_status("nginx", false, true);
        assert_eq!(failed.health, HealthLevel::Critical);
    }

    #[test]
    fn test_format_quick_status_oneline() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.set_summary("All good");

        let output = format_quick_status_oneline(&status);
        assert!(output.contains("[OK]"));
        assert!(output.contains("All good"));
    }

    #[test]
    fn test_is_quick_status_query() {
        assert!(is_quick_status_query("quick status"));
        assert!(is_quick_status_query("any problems?"));
        assert!(is_quick_status_query("health check"));
        assert!(is_quick_status_query("how's the system?"));

        assert!(!is_quick_status_query("restart nginx"));
        assert!(!is_quick_status_query("show disk usage"));
    }

    #[test]
    fn test_has_critical() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        assert!(!status.has_critical());

        status.add(StatusItem::new("B", HealthLevel::Critical, "Bad"));
        assert!(status.has_critical());
    }

    #[test]
    fn test_all_good() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.add(StatusItem::new("B", HealthLevel::Good, "OK"));
        assert!(status.all_good());

        status.add(StatusItem::new("C", HealthLevel::Warning, "Warn"));
        assert!(!status.all_good());
    }
}
