// v0.0.584: Metrics Utilities
// Helper functions for metrics formatting and queries

use super::{SettingsMetrics, MetricKind};

/// Format metrics for display
pub fn format_metrics(metrics: &SettingsMetrics) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Metrics ===\n\n");
    output.push_str(&format!("Total Metrics: {}\n", metrics.count()));

    if let Some(uptime) = metrics.uptime() {
        output.push_str(&format!("Uptime: {}s\n\n", uptime.num_seconds()));
    }

    output.push_str("--- Counters ---\n");
    for metric in metrics.by_kind(MetricKind::Counter) {
        output.push_str(&format!("• {}: {:.0} {}\n", metric.name, metric.current, metric.unit));
    }

    output.push_str("\n--- Gauges ---\n");
    for metric in metrics.by_kind(MetricKind::Gauge) {
        output.push_str(&format!("• {}: {:.1} {}\n", metric.name, metric.current, metric.unit));
    }

    output
}

/// Check if query is about metrics
pub fn is_metrics_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("metrics")
        || lower.contains("telemetry")
        || lower.contains("statistics")
}

/// Fun fact about metrics
pub fn settings_metrics_fun_fact() -> &'static str {
    "Anna collects metrics to help you understand your settings usage patterns!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_metrics() {
        let metrics = SettingsMetrics::new();
        let output = format_metrics(&metrics);
        assert!(output.contains("Metrics"));
    }

    #[test]
    fn test_is_metrics_query() {
        assert!(is_metrics_query("show metrics"));
        assert!(is_metrics_query("telemetry data"));
        assert!(!is_metrics_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_metrics_fun_fact();
        assert!(fact.contains("metrics"));
    }
}
