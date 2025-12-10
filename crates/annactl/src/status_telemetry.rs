//! Telemetry insights section for status display (v0.0.280).
//!
//! Shows system health trends and anomalies detected by Anna.

use anna_shared::system_telemetry::{AnomalySeverity, InsightCategory, TelemetryStore};
use anna_shared::ui::colors;

const KEY_WIDTH: usize = 22;

/// Print the telemetry insights section
pub fn print_telemetry_section() {
    let store = TelemetryStore::load();

    // Only show if we have data
    if store.samples.is_empty() {
        return;
    }

    println!();
    println!("{}[telemetry]{}", colors::HEADER, colors::RESET);

    // Health score
    let health = store.health_score();
    let health_color = if health >= 80 {
        colors::OK
    } else if health >= 50 {
        colors::WARN
    } else {
        colors::ERR
    };
    kv(
        "health_score",
        &format!("{}{}/100{}", health_color, health, colors::RESET),
    );

    // Sample count and time window
    kv("samples", &format!("{}", store.samples.len()));
    if store.trends.window_hours > 0.0 {
        kv(
            "tracking_window",
            &format!("{:.1} hours", store.trends.window_hours),
        );
    }

    // Trends
    if store.trends.sample_count >= 2 {
        print_trends(&store);
    }

    // Anomalies
    let recent = store.recent_anomalies();
    if !recent.is_empty() {
        kv("anomalies", &format!("{}", recent.len()));
        for anomaly in recent.iter().take(3) {
            let severity_color = match anomaly.severity {
                AnomalySeverity::Critical => colors::ERR,
                AnomalySeverity::Warning => colors::WARN,
                AnomalySeverity::Info => colors::DIM,
            };
            println!(
                "    {}[{}]{} {}",
                severity_color,
                anomaly.category,
                colors::RESET,
                anomaly.description
            );
        }
        if recent.len() > 3 {
            println!(
                "    {}... and {} more{}",
                colors::DIM,
                recent.len() - 3,
                colors::RESET
            );
        }
    }

    // Insights
    let insights = store.generate_insights();
    if !insights.is_empty() {
        kv("insights", "");
        for insight in insights.iter().take(3) {
            let icon = match insight.category {
                InsightCategory::Trend => "~",
                InsightCategory::Alert => "!",
                InsightCategory::Optimization => "*",
                InsightCategory::Learning => "+",
            };
            println!(
                "    {}{}{} {}",
                colors::CYAN, icon, colors::RESET, insight.message
            );
        }
    }
}

/// Print trend indicators
fn print_trends(store: &TelemetryStore) {
    let trends = &store.trends;

    // Only show trends if they're significant
    let mut trend_parts = Vec::new();

    if trends.cpu_trend.abs() > 5.0 {
        let arrow = if trends.cpu_trend > 0.0 { "+" } else { "" };
        trend_parts.push(format!("cpu {}{:.0}%", arrow, trends.cpu_trend));
    }

    if trends.memory_trend.abs() > 5.0 {
        let arrow = if trends.memory_trend > 0.0 { "+" } else { "" };
        trend_parts.push(format!("mem {}{:.0}%", arrow, trends.memory_trend));
    }

    if trends.disk_trend.abs() > 2.0 {
        let arrow = if trends.disk_trend > 0.0 { "+" } else { "" };
        trend_parts.push(format!("disk {}{:.1}%", arrow, trends.disk_trend));
    }

    if !trend_parts.is_empty() {
        kv("trends", &trend_parts.join(", "));
    }
}

fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_formatting() {
        // Just ensure it doesn't panic
        print_telemetry_section();
    }
}
