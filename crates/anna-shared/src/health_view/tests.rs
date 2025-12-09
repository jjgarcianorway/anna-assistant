//! Tests for health view module (v0.0.210).

#[cfg(test)]
mod tests {
    use crate::health_view::{build_health_summary, has_health_issues};
    use crate::snapshot::SystemSnapshot;

    #[test]
    fn test_healthy_system() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 50);
        snapshot.set_memory(16_000_000_000, 8_000_000_000); // 50%

        let summary = build_health_summary(&snapshot, None);
        assert!(summary.nothing_to_report);
        assert!(summary.critical.is_empty());
        assert!(summary.warnings.is_empty());
        assert_eq!(
            summary.format(),
            "No critical issues detected. No warnings detected."
        );
    }

    #[test]
    fn test_disk_warning_only() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 87); // above 85% threshold
        snapshot.set_memory(16_000_000_000, 8_000_000_000);

        let summary = build_health_summary(&snapshot, None);
        assert!(!summary.nothing_to_report);
        assert!(summary.critical.is_empty());
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.format().contains("87%"));
    }

    #[test]
    fn test_critical_disk() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 96); // above 95% critical

        let summary = build_health_summary(&snapshot, None);
        assert_eq!(summary.critical.len(), 1);
        assert!(summary.format().contains("CRITICAL"));
    }

    #[test]
    fn test_failed_services() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_failed_service("nginx.service");
        snapshot.add_failed_service("docker.service");

        let summary = build_health_summary(&snapshot, None);
        assert_eq!(summary.critical.len(), 2);
        assert!(summary.format().contains("nginx.service"));
        assert!(summary.format().contains("docker.service"));
    }

    #[test]
    fn test_mixed_issues_sorted() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 96); // critical
        snapshot.add_disk("/home", 87); // warning
        snapshot.add_failed_service("nginx.service");
        snapshot.set_memory(16_000_000_000, 14_000_000_000); // 87.5% - warning

        let summary = build_health_summary(&snapshot, None);

        // Should have disk critical, service critical, disk warning, memory warning
        assert_eq!(summary.critical.len(), 2); // disk critical + service
        assert_eq!(summary.warnings.len(), 2); // disk warning + memory

        // Format should show critical first (v0.0.265: ASCII icons)
        let formatted = summary.format();
        let critical_pos = formatted.find("CRITICAL").unwrap();
        let warning_pos = formatted.find("[!]").unwrap();
        assert!(critical_pos < warning_pos);
    }

    #[test]
    fn test_change_detection() {
        let mut prev = SystemSnapshot::new();
        prev.add_failed_service("nginx.service");

        let mut curr = SystemSnapshot::new();
        // nginx recovered, but docker failed
        curr.add_failed_service("docker.service");

        let summary = build_health_summary(&curr, Some(&prev));
        assert_eq!(summary.changed_since_last.len(), 2);

        let recovered = summary.changed_since_last.iter().find(|c| c.positive);
        assert!(recovered.is_some());
        assert!(recovered.unwrap().description.contains("nginx"));
    }

    #[test]
    fn test_has_health_issues() {
        let mut healthy = SystemSnapshot::new();
        healthy.add_disk("/", 50);
        assert!(!has_health_issues(&healthy));

        let mut warning = SystemSnapshot::new();
        warning.add_disk("/", 87);
        assert!(has_health_issues(&warning));

        let mut failed = SystemSnapshot::new();
        failed.add_failed_service("test.service");
        assert!(has_health_issues(&failed));
    }
}
