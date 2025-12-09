//! Tests for health brief module (v0.0.207).

#[cfg(test)]
mod tests {
    use crate::health_brief::{BriefSeverity, HealthBrief};

    #[test]
    fn test_all_healthy() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 50, "50G");
        brief.add_disk("/home", 70, "30G");
        brief.finalize();

        assert!(brief.all_healthy);
        assert_eq!(brief.items.len(), 0);
        assert!(brief.summary.contains("healthy"));
    }

    #[test]
    fn test_disk_warning() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 87, "13G");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.items.len(), 1);
        assert_eq!(brief.items[0].severity, BriefSeverity::Warning);
        assert!(brief.items[0].context.as_ref().unwrap().contains("/"));
    }

    #[test]
    fn test_disk_critical() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 96, "4G");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.overall, BriefSeverity::Error);
        assert_eq!(brief.items[0].severity, BriefSeverity::Error);
    }

    #[test]
    fn test_failed_service() {
        let mut brief = HealthBrief::new();
        brief.add_failed_service("nginx.service");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.overall, BriefSeverity::Error);
        assert!(brief.items[0].context.as_ref().unwrap().contains("nginx"));
    }

    #[test]
    fn test_high_cpu() {
        let mut brief = HealthBrief::new();
        brief.add_high_cpu("firefox", 85.5);
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.items[0].severity, BriefSeverity::Warning);
    }

    #[test]
    fn test_format_answer_healthy() {
        let mut brief = HealthBrief::new();
        brief.finalize();

        let answer = brief.format_answer();
        assert!(answer.contains("healthy"));
    }

    #[test]
    fn test_format_answer_with_issues() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/home", 96, "4G");
        brief.add_failed_service("docker.service");
        brief.finalize();

        let answer = brief.format_answer();
        assert!(answer.contains("critical"));
        assert!(answer.contains("/home"));
        assert!(answer.contains("docker"));
    }

    #[test]
    fn test_severity_ordering() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/tmp", 87, "13G"); // Warning
        brief.add_disk("/", 96, "4G"); // Error
        brief.add_disk("/var", 88, "12G"); // Warning
        brief.finalize();

        // Errors should come first
        assert_eq!(brief.items[0].severity, BriefSeverity::Error);
        assert_eq!(brief.items[1].severity, BriefSeverity::Warning);
        assert_eq!(brief.items[2].severity, BriefSeverity::Warning);
    }

    // Golden tests
    #[test]
    fn golden_healthy_summary() {
        let mut brief = HealthBrief::new();
        brief.finalize();
        assert_eq!(brief.summary, "Your system is healthy. No issues detected.");
    }

    #[test]
    fn golden_single_warning() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 87, "13G");
        brief.finalize();
        assert_eq!(brief.summary, "1 warning found.");
    }

    #[test]
    fn golden_multiple_issues() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 96, "4G");
        brief.add_disk("/home", 88, "12G");
        brief.add_failed_service("nginx");
        brief.finalize();
        assert_eq!(brief.summary, "2 critical issues and 1 warning found.");
    }
}
