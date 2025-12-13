//! System Health Score (Phase 74)
//!
//! Provides a unified system health score combining multiple metrics
//! into a single actionable health assessment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health grade from A to F
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthGrade {
    A,
    B,
    C,
    D,
    F,
}

impl HealthGrade {
    /// Display string
    pub fn display(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    /// Description of the grade
    pub fn description(&self) -> &'static str {
        match self {
            Self::A => "Excellent",
            Self::B => "Good",
            Self::C => "Fair",
            Self::D => "Poor",
            Self::F => "Critical",
        }
    }

    /// From numeric score (0-100)
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::A,
            80..=89 => Self::B,
            70..=79 => Self::C,
            60..=69 => Self::D,
            _ => Self::F,
        }
    }

    /// To numeric score midpoint
    pub fn to_score(&self) -> u8 {
        match self {
            Self::A => 95,
            Self::B => 85,
            Self::C => 75,
            Self::D => 65,
            Self::F => 40,
        }
    }
}

/// Category of health metric
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCategory {
    /// CPU usage
    Cpu,
    /// Memory usage
    Memory,
    /// Disk usage
    Disk,
    /// System services
    Services,
    /// Network connectivity
    Network,
    /// Security
    Security,
    /// Updates
    Updates,
    /// Anna daemon
    Daemon,
}

impl HealthCategory {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Disk => "Disk",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Updates => "Updates",
            Self::Daemon => "Daemon",
        }
    }

    /// Weight for overall score (out of 100)
    pub fn weight(&self) -> u8 {
        match self {
            Self::Cpu => 15,
            Self::Memory => 15,
            Self::Disk => 20,
            Self::Services => 15,
            Self::Network => 10,
            Self::Security => 10,
            Self::Updates => 5,
            Self::Daemon => 10,
        }
    }
}

/// Individual health metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    /// Category
    pub category: HealthCategory,
    /// Score (0-100)
    pub score: u8,
    /// Current value (for display)
    pub value: String,
    /// Threshold that triggered the score
    pub threshold: Option<String>,
    /// Recommendation if score is low
    pub recommendation: Option<String>,
}

impl HealthMetric {
    /// Create a new health metric
    pub fn new(category: HealthCategory, score: u8, value: impl Into<String>) -> Self {
        Self {
            category,
            score: score.min(100),
            value: value.into(),
            threshold: None,
            recommendation: None,
        }
    }

    /// Set recommendation
    pub fn with_recommendation(mut self, recommendation: impl Into<String>) -> Self {
        self.recommendation = Some(recommendation.into());
        self
    }

    /// Get grade for this metric
    pub fn grade(&self) -> HealthGrade {
        HealthGrade::from_score(self.score)
    }
}

/// Overall system health score
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealthScore {
    /// Individual metrics
    pub metrics: HashMap<String, HealthMetric>,
    /// Overall score (0-100)
    pub overall_score: u8,
    /// Overall grade
    pub overall_grade: String,
    /// Critical issues count
    pub critical_issues: u32,
    /// Warning count
    pub warnings: u32,
    /// Last check timestamp
    pub last_check: u64,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl SystemHealthScore {
    /// Create a new empty health score
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: HealthMetric) {
        // Track issues
        if metric.score < 60 {
            self.critical_issues += 1;
        } else if metric.score < 80 {
            self.warnings += 1;
        }

        // Add recommendation if present
        if let Some(ref rec) = metric.recommendation {
            if metric.score < 80 {
                self.recommendations.push(rec.clone());
            }
        }

        self.metrics.insert(metric.category.display().to_string(), metric);
    }

    /// Calculate overall score from metrics
    pub fn calculate_overall(&mut self) {
        if self.metrics.is_empty() {
            self.overall_score = 0;
            self.overall_grade = "N/A".to_string();
            return;
        }

        let mut weighted_sum: u32 = 0;
        let mut total_weight: u32 = 0;

        for metric in self.metrics.values() {
            let weight = metric.category.weight() as u32;
            weighted_sum += metric.score as u32 * weight;
            total_weight += weight;
        }

        if total_weight > 0 {
            self.overall_score = (weighted_sum / total_weight) as u8;
        } else {
            self.overall_score = 0;
        }

        self.overall_grade = HealthGrade::from_score(self.overall_score).display().to_string();
    }

    /// Get metrics below threshold
    pub fn issues(&self) -> Vec<&HealthMetric> {
        self.metrics.values().filter(|m| m.score < 80).collect()
    }

    /// Get critical metrics
    pub fn critical(&self) -> Vec<&HealthMetric> {
        self.metrics.values().filter(|m| m.score < 60).collect()
    }

    /// Is the system healthy?
    pub fn is_healthy(&self) -> bool {
        self.overall_score >= 80 && self.critical_issues == 0
    }
}

/// Create CPU health metric from usage percentage
pub fn cpu_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 50.0 {
        100
    } else if usage_percent < 70.0 {
        85
    } else if usage_percent < 85.0 {
        70
    } else if usage_percent < 95.0 {
        55
    } else {
        30
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Cpu,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("High CPU usage. Check running processes.");
    }

    metric
}

/// Create memory health metric from usage percentage
pub fn memory_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 60.0 {
        100
    } else if usage_percent < 75.0 {
        85
    } else if usage_percent < 85.0 {
        70
    } else if usage_percent < 95.0 {
        50
    } else {
        25
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Memory,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("High memory usage. Consider closing applications.");
    }

    metric
}

/// Create disk health metric from usage percentage
pub fn disk_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 70.0 {
        100
    } else if usage_percent < 80.0 {
        85
    } else if usage_percent < 90.0 {
        65
    } else if usage_percent < 95.0 {
        40
    } else {
        20
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Disk,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("Low disk space. Clean up or expand storage.");
    }

    metric
}

/// Create services health metric
pub fn services_health(failed_count: u32, total_count: u32) -> HealthMetric {
    let score = if failed_count == 0 {
        100
    } else if failed_count == 1 {
        80
    } else if failed_count <= 3 {
        60
    } else {
        40
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Services,
        score,
        format!("{}/{} running", total_count - failed_count, total_count),
    );

    if failed_count > 0 {
        metric = metric.with_recommendation(format!(
            "{} service{} failed. Check systemctl status.",
            failed_count,
            if failed_count == 1 { "" } else { "s" }
        ));
    }

    metric
}

/// Create network health metric
pub fn network_health(connected: bool, latency_ms: Option<u32>) -> HealthMetric {
    let score = if !connected {
        0
    } else if let Some(latency) = latency_ms {
        if latency < 50 {
            100
        } else if latency < 100 {
            90
        } else if latency < 200 {
            75
        } else {
            60
        }
    } else {
        85 // Connected but no latency data
    };

    let value = if !connected {
        "Disconnected".to_string()
    } else if let Some(latency) = latency_ms {
        format!("{}ms latency", latency)
    } else {
        "Connected".to_string()
    };

    let mut metric = HealthMetric::new(HealthCategory::Network, score, value);

    if !connected {
        metric = metric.with_recommendation("No network connection. Check connectivity.");
    }

    metric
}

/// Create daemon health metric
pub fn daemon_health(running: bool, uptime_hours: Option<u64>) -> HealthMetric {
    let score = if !running {
        0
    } else if let Some(hours) = uptime_hours {
        if hours > 24 {
            100
        } else if hours > 1 {
            90
        } else {
            80
        }
    } else {
        90
    };

    let value = if !running {
        "Not running".to_string()
    } else if let Some(hours) = uptime_hours {
        format!("{}h uptime", hours)
    } else {
        "Running".to_string()
    };

    let mut metric = HealthMetric::new(HealthCategory::Daemon, score, value);

    if !running {
        metric = metric.with_recommendation("Anna daemon not running. Start with systemctl.");
    }

    metric
}

/// Format health score as full display
pub fn format_health_score(health: &SystemHealthScore) -> String {
    let mut lines = Vec::new();

    lines.push("=== System Health Score ===".to_string());
    lines.push(String::new());

    // Overall
    let grade = HealthGrade::from_score(health.overall_score);
    lines.push(format!(
        "Overall: {} ({}) - {}",
        health.overall_score,
        grade.display(),
        grade.description()
    ));

    if health.critical_issues > 0 {
        lines.push(format!("Critical Issues: {}", health.critical_issues));
    }
    if health.warnings > 0 {
        lines.push(format!("Warnings: {}", health.warnings));
    }

    lines.push(String::new());

    // Individual metrics
    lines.push("--- Component Health ---".to_string());
    for (name, metric) in &health.metrics {
        let grade = metric.grade();
        lines.push(format!(
            "  {}: {} ({}) - {}",
            name,
            metric.score,
            grade.display(),
            metric.value
        ));
    }

    // Recommendations
    if !health.recommendations.is_empty() {
        lines.push(String::new());
        lines.push("--- Recommendations ---".to_string());
        for rec in &health.recommendations {
            lines.push(format!("  * {}", rec));
        }
    }

    lines.join("\n")
}

/// Format health score compact
pub fn format_health_score_compact(health: &SystemHealthScore) -> String {
    let grade = HealthGrade::from_score(health.overall_score);
    let issues = if health.critical_issues > 0 {
        format!(" ({} critical)", health.critical_issues)
    } else if health.warnings > 0 {
        format!(" ({} warnings)", health.warnings)
    } else {
        String::new()
    };

    format!(
        "Health: {} ({}){}",
        health.overall_score,
        grade.display(),
        issues
    )
}

/// Format health score one-line
pub fn format_health_score_oneline(health: &SystemHealthScore) -> String {
    let grade = HealthGrade::from_score(health.overall_score);
    format!(
        "System Health: {}/100 ({}) - {}",
        health.overall_score,
        grade.display(),
        grade.description()
    )
}

/// Generate health bar visualization
pub fn health_bar(score: u8, width: usize) -> String {
    let filled = ((score as f64 / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}] {}%", "=".repeat(filled), " ".repeat(empty), score)
}

/// Check if query is asking about health score
pub fn is_health_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "health score",
        "system health",
        "health check",
        "how healthy",
        "health status",
        "system status",
        "overall health",
        "check health",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Generate a health summary message
pub fn health_summary_message(health: &SystemHealthScore) -> String {
    let grade = HealthGrade::from_score(health.overall_score);

    match grade {
        HealthGrade::A => "System is in excellent health. All systems go!".to_string(),
        HealthGrade::B => "System health is good. Minor optimizations possible.".to_string(),
        HealthGrade::C => format!(
            "System health is fair. {} issue{} to address.",
            health.warnings + health.critical_issues,
            if health.warnings + health.critical_issues == 1 { "" } else { "s" }
        ),
        HealthGrade::D => format!(
            "System health is poor. {} critical issue{} need attention.",
            health.critical_issues,
            if health.critical_issues == 1 { "" } else { "s" }
        ),
        HealthGrade::F => "System health is critical! Immediate action required.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
