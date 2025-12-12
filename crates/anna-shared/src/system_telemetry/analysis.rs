//! Telemetry analysis (v0.0.291).
//!
//! Anomaly detection and trend analysis for system telemetry.

use chrono::Utc;
use std::collections::VecDeque;

use super::types::{
    AnomalyCategory, AnomalySeverity, InsightCategory, TelemetryAnomaly, TelemetryInsight,
    TelemetrySample, TrendSummary,
};

/// Analyze CPU usage and detect anomalies
pub fn analyze_cpu(samples: &VecDeque<TelemetrySample>) -> Vec<TelemetryAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(sample) = samples.back() {
        if let Some(cpu) = sample.cpu_usage_percent {
            if cpu > 90.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::HighCpu,
                    severity: AnomalySeverity::Critical,
                    description: format!("CPU usage at {:.1}%", cpu),
                    metric: "cpu_usage_percent".to_string(),
                    value: cpu as f64,
                    threshold: 90.0,
                });
            } else if cpu > 75.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::HighCpu,
                    severity: AnomalySeverity::Warning,
                    description: format!("CPU usage elevated at {:.1}%", cpu),
                    metric: "cpu_usage_percent".to_string(),
                    value: cpu as f64,
                    threshold: 75.0,
                });
            }
        }
    }
    anomalies
}

/// Analyze memory usage and detect anomalies
pub fn analyze_memory(samples: &VecDeque<TelemetrySample>) -> Vec<TelemetryAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(sample) = samples.back() {
        if let (Some(used), Some(total)) = (sample.memory_used_bytes, sample.memory_total_bytes) {
            let percent = (used as f64 / total as f64) * 100.0;
            if percent > 90.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::HighMemory,
                    severity: AnomalySeverity::Critical,
                    description: format!("Memory usage at {:.1}%", percent),
                    metric: "memory_percent".to_string(),
                    value: percent,
                    threshold: 90.0,
                });
            } else if percent > 80.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::HighMemory,
                    severity: AnomalySeverity::Warning,
                    description: format!("Memory usage elevated at {:.1}%", percent),
                    metric: "memory_percent".to_string(),
                    value: percent,
                    threshold: 80.0,
                });
            }
        }
    }
    anomalies
}

/// Analyze disk usage and detect anomalies
pub fn analyze_disk(samples: &VecDeque<TelemetrySample>) -> Vec<TelemetryAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(sample) = samples.back() {
        if let (Some(used), Some(total)) = (sample.disk_used_bytes, sample.disk_total_bytes) {
            let percent = (used as f64 / total as f64) * 100.0;
            if percent > 95.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::LowDisk,
                    severity: AnomalySeverity::Critical,
                    description: format!("Disk usage at {:.1}% - critical!", percent),
                    metric: "disk_percent".to_string(),
                    value: percent,
                    threshold: 95.0,
                });
            } else if percent > 85.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::LowDisk,
                    severity: AnomalySeverity::Warning,
                    description: format!("Disk usage elevated at {:.1}%", percent),
                    metric: "disk_percent".to_string(),
                    value: percent,
                    threshold: 85.0,
                });
            }
        }
    }
    anomalies
}

/// Analyze load average and detect anomalies
pub fn analyze_load(samples: &VecDeque<TelemetrySample>) -> Vec<TelemetryAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(sample) = samples.back() {
        if let Some(load) = sample.load_average_1m {
            // High load is typically > number of CPU cores
            if load > 4.0 {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::HighLoad,
                    severity: AnomalySeverity::Warning,
                    description: format!("Load average at {:.2}", load),
                    metric: "load_average_1m".to_string(),
                    value: load as f64,
                    threshold: 4.0,
                });
            }
        }
    }
    anomalies
}

/// Analyze services and detect anomalies
pub fn analyze_services(samples: &VecDeque<TelemetrySample>) -> Vec<TelemetryAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(sample) = samples.back() {
        for service in &sample.services {
            if !service.running && service.enabled {
                anomalies.push(TelemetryAnomaly {
                    detected_at: Utc::now(),
                    category: AnomalyCategory::ServiceDown,
                    severity: AnomalySeverity::Critical,
                    description: format!("Service '{}' is down but enabled", service.name),
                    metric: format!("service_{}", service.name),
                    value: 0.0,
                    threshold: 1.0,
                });
            }
        }
    }
    anomalies
}

/// Calculate trends from samples
pub fn calculate_trends(samples: &VecDeque<TelemetrySample>) -> TrendSummary {
    let sample_count = samples.len();
    if sample_count < 2 {
        return TrendSummary::default();
    }

    let mut trends = TrendSummary {
        sample_count,
        ..Default::default()
    };

    // Calculate time window
    if let (Some(first), Some(last)) = (samples.front(), samples.back()) {
        let duration = last.timestamp.signed_duration_since(first.timestamp);
        trends.window_hours = duration.num_minutes() as f32 / 60.0;
    }

    // Simple linear trend calculation
    trends.cpu_trend = calculate_metric_trend(samples, |s| s.cpu_usage_percent);
    trends.memory_trend = calculate_metric_trend(samples, |s| {
        match (s.memory_used_bytes, s.memory_total_bytes) {
            (Some(used), Some(total)) if total > 0 => Some((used as f32 / total as f32) * 100.0),
            _ => None,
        }
    });
    trends.disk_trend =
        calculate_metric_trend(samples, |s| match (s.disk_used_bytes, s.disk_total_bytes) {
            (Some(used), Some(total)) if total > 0 => Some((used as f32 / total as f32) * 100.0),
            _ => None,
        });
    trends.load_trend = calculate_metric_trend(samples, |s| s.load_average_1m);

    trends
}

/// Calculate trend for a specific metric
fn calculate_metric_trend<F>(samples: &VecDeque<TelemetrySample>, extractor: F) -> f32
where
    F: Fn(&TelemetrySample) -> Option<f32>,
{
    let values: Vec<f32> = samples.iter().filter_map(|s| extractor(s)).collect();
    if values.len() < 2 {
        return 0.0;
    }

    // Compare first quarter average to last quarter average
    let quarter = values.len() / 4;
    if quarter == 0 {
        return 0.0;
    }

    let first_avg: f32 = values[..quarter].iter().sum::<f32>() / quarter as f32;
    let last_avg: f32 = values[values.len() - quarter..].iter().sum::<f32>() / quarter as f32;

    last_avg - first_avg
}

/// Generate insights from telemetry data
pub fn generate_insights(
    trends: &TrendSummary,
    anomalies: &[TelemetryAnomaly],
) -> Vec<TelemetryInsight> {
    let mut insights = Vec::new();

    // Trend-based insights
    if trends.cpu_trend > 10.0 {
        insights.push(TelemetryInsight {
            category: InsightCategory::Trend,
            message: format!(
                "CPU usage trending up by {:.1}% over the last {:.1} hours",
                trends.cpu_trend, trends.window_hours
            ),
            recommendation: "Consider investigating high CPU processes".to_string(),
        });
    }

    if trends.memory_trend > 10.0 {
        insights.push(TelemetryInsight {
            category: InsightCategory::Trend,
            message: format!(
                "Memory usage trending up by {:.1}% over the last {:.1} hours",
                trends.memory_trend, trends.window_hours
            ),
            recommendation: "Check for memory leaks or excessive caching".to_string(),
        });
    }

    if trends.disk_trend > 5.0 {
        insights.push(TelemetryInsight {
            category: InsightCategory::Trend,
            message: format!(
                "Disk usage growing by {:.1}% over the last {:.1} hours",
                trends.disk_trend, trends.window_hours
            ),
            recommendation: "Review log rotation and cleanup old files".to_string(),
        });
    }

    // Anomaly-based insights
    let critical_count = anomalies
        .iter()
        .filter(|a| a.severity == AnomalySeverity::Critical)
        .count();
    if critical_count > 0 {
        insights.push(TelemetryInsight {
            category: InsightCategory::Alert,
            message: format!("{} critical issues detected", critical_count),
            recommendation: "Immediate attention required".to_string(),
        });
    }

    insights
}

/// Calculate health score from anomalies (0-100)
pub fn calculate_health_score(anomalies: &[TelemetryAnomaly]) -> u8 {
    let mut score: i32 = 100;

    for anomaly in anomalies {
        match anomaly.severity {
            AnomalySeverity::Critical => score -= 25,
            AnomalySeverity::Warning => score -= 10,
            AnomalySeverity::Info => score -= 2,
        }
    }

    score.max(0).min(100) as u8
}
