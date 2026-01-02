//! Formatting and display functions for health scores

use super::types::{HealthGrade, SystemHealthScore};

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
