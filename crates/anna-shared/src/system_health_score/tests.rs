//! Tests for system health score module

#[cfg(test)]
mod tests {
    use crate::system_health_score::*;

    #[test]
    fn test_health_grade_from_score() {
        assert_eq!(HealthGrade::from_score(95), HealthGrade::A);
        assert_eq!(HealthGrade::from_score(85), HealthGrade::B);
        assert_eq!(HealthGrade::from_score(75), HealthGrade::C);
        assert_eq!(HealthGrade::from_score(65), HealthGrade::D);
        assert_eq!(HealthGrade::from_score(50), HealthGrade::F);
    }

    #[test]
    fn test_health_grade_display() {
        assert_eq!(HealthGrade::A.display(), "A");
        assert_eq!(HealthGrade::A.description(), "Excellent");
    }

    #[test]
    fn test_health_category_weight() {
        // Weights should sum to 100
        let categories = [
            HealthCategory::Cpu,
            HealthCategory::Memory,
            HealthCategory::Disk,
            HealthCategory::Services,
            HealthCategory::Network,
            HealthCategory::Security,
            HealthCategory::Updates,
            HealthCategory::Daemon,
        ];
        let total: u8 = categories.iter().map(|c| c.weight()).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_health_metric_new() {
        let metric = HealthMetric::new(HealthCategory::Cpu, 85, "50% used");
        assert_eq!(metric.score, 85);
        assert_eq!(metric.grade(), HealthGrade::B);
    }

    #[test]
    fn test_health_metric_recommendation() {
        let metric = HealthMetric::new(HealthCategory::Disk, 40, "95% used")
            .with_recommendation("Clean up disk");
        assert!(metric.recommendation.is_some());
    }

    #[test]
    fn test_system_health_score_add() {
        let mut health = SystemHealthScore::new();
        health.add_metric(HealthMetric::new(HealthCategory::Cpu, 90, "30% used"));
        health.add_metric(HealthMetric::new(HealthCategory::Memory, 85, "50% used"));

        assert_eq!(health.metrics.len(), 2);
    }

    #[test]
    fn test_system_health_score_calculate() {
        let mut health = SystemHealthScore::new();
        health.add_metric(HealthMetric::new(HealthCategory::Cpu, 100, "10%"));
        health.add_metric(HealthMetric::new(HealthCategory::Memory, 100, "20%"));
        health.calculate_overall();

        assert!(health.overall_score > 0);
        assert!(!health.overall_grade.is_empty());
    }

    #[test]
    fn test_system_health_critical_tracking() {
        let mut health = SystemHealthScore::new();
        health.add_metric(HealthMetric::new(HealthCategory::Cpu, 40, "95%")); // Critical
        health.add_metric(HealthMetric::new(HealthCategory::Memory, 70, "80%")); // Warning

        assert_eq!(health.critical_issues, 1);
        assert_eq!(health.warnings, 1);
    }

    #[test]
    fn test_cpu_health() {
        let low = cpu_health(30.0);
        assert_eq!(low.score, 100);

        let high = cpu_health(90.0);
        assert!(high.score < 70);
        assert!(high.recommendation.is_some());
    }

    #[test]
    fn test_memory_health() {
        let low = memory_health(40.0);
        assert_eq!(low.score, 100);

        let high = memory_health(90.0);
        assert!(high.score < 70);
    }

    #[test]
    fn test_disk_health() {
        let low = disk_health(50.0);
        assert_eq!(low.score, 100);

        let high = disk_health(92.0);
        assert!(high.score < 50);
    }

    #[test]
    fn test_services_health() {
        let all_ok = services_health(0, 10);
        assert_eq!(all_ok.score, 100);

        let some_failed = services_health(2, 10);
        assert!(some_failed.score < 80);
        assert!(some_failed.recommendation.is_some());
    }

    #[test]
    fn test_network_health() {
        let connected = network_health(true, Some(30));
        assert_eq!(connected.score, 100);

        let disconnected = network_health(false, None);
        assert_eq!(disconnected.score, 0);
    }

    #[test]
    fn test_daemon_health() {
        let running = daemon_health(true, Some(48));
        assert_eq!(running.score, 100);

        let stopped = daemon_health(false, None);
        assert_eq!(stopped.score, 0);
    }

    #[test]
    fn test_format_health_score() {
        let mut health = SystemHealthScore::new();
        health.add_metric(HealthMetric::new(HealthCategory::Cpu, 90, "30%"));
        health.calculate_overall();

        let output = format_health_score(&health);
        assert!(output.contains("System Health"));
        assert!(output.contains("CPU"));
    }

    #[test]
    fn test_health_bar() {
        assert_eq!(health_bar(50, 10), "[=====     ] 50%");
        assert_eq!(health_bar(100, 10), "[==========] 100%");
    }

    #[test]
    fn test_is_health_query() {
        assert!(is_health_query("show system health"));
        assert!(is_health_query("check health score"));
        assert!(is_health_query("how healthy is my system?"));
        assert!(!is_health_query("how do I install vim?"));
    }

    #[test]
    fn test_health_summary_message() {
        let mut health = SystemHealthScore::new();
        health.overall_score = 95;
        assert!(health_summary_message(&health).contains("excellent"));

        health.overall_score = 40;
        health.critical_issues = 2;
        assert!(health_summary_message(&health).contains("critical"));
    }
}
