//! Anomaly Detection Module for Anna v0.14.0 "Orion III"
//!
//! Statistical outlier detection for predictive maintenance

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::history::{HistoryEntry, HistoryManager};
use crate::profiled::Profiler;

/// Anomaly severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,       // Minor deviation
    Warning,    // Persistent deviation
    Critical,   // > 2σ over 3+ consecutive days
}

impl Severity {
    /// Get emoji for severity
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Critical => "🚨",
        }
    }

    /// Get color for severity
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Info => "\x1b[36m",      // Cyan
            Severity::Warning => "\x1b[33m",   // Yellow
            Severity::Critical => "\x1b[31m",  // Red
        }
    }
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub metric: String,           // "overall_score", "hardware_score", etc.
    pub current_value: f64,       // Current metric value
    pub expected_value: f64,      // Expected/baseline value
    pub deviation: f64,           // Absolute deviation
    pub z_score: f64,             // Z-score (standard deviations from mean)
    pub severity: Severity,       // Classification
    pub description: String,      // Human-readable description
    pub first_detected: u64,      // Timestamp of first detection
    pub consecutive_days: u32,    // How many days this persists
}

impl Anomaly {
    /// Check if anomaly is critical
    pub fn is_critical(&self) -> bool {
        self.severity == Severity::Critical
    }

    /// Get recommendation for this anomaly
    pub fn recommendation(&self) -> String {
        match self.metric.as_str() {
            "overall_score" => "Investigate system-wide issues. Run 'annactl report --verbose' for details.".to_string(),
            "hardware_score" => "Check hardware metrics: 'annactl sensors', 'annactl disk --detail'".to_string(),
            "software_score" => "Review software state: 'annactl radar software', check for updates".to_string(),
            "user_score" => "Review user habits and maintenance routines".to_string(),
            "perf_rpc_latency" => "Anna's RPC latency is degraded. Check daemon health: 'annactl health'".to_string(),
            "perf_memory" => "Anna's memory usage is higher than baseline. Consider restarting daemon.".to_string(),
            "perf_io_latency" => "Anna's I/O performance is degraded. Check disk health and filesystem.".to_string(),
            "perf_cpu" => "Anna's CPU usage is elevated. Check for runaway processes.".to_string(),
            _ => "Review specific metrics for this category".to_string(),
        }
    }
}

/// Anomaly detector
pub struct AnomalyDetector {
    history_mgr: HistoryManager,
    profiler: Option<Profiler>,
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new() -> Result<Self> {
        let history_mgr = HistoryManager::new()?;
        let profiler = Profiler::new().ok();
        Ok(Self { history_mgr, profiler })
    }

    /// Detect all anomalies
    pub fn detect_anomalies(&self) -> Result<Vec<Anomaly>> {
        let entries = self.history_mgr.load_all()?;

        if entries.len() < 7 {
            return Ok(Vec::new()); // Need at least 7 days for baseline
        }

        let mut anomalies = Vec::new();

        // Check overall score
        if let Some(anomaly) = self.check_metric(&entries, "overall_score", |e| e.overall_score as f64)? {
            anomalies.push(anomaly);
        }

        // Check hardware score
        if let Some(anomaly) = self.check_metric(&entries, "hardware_score", |e| e.hardware_score as f64)? {
            anomalies.push(anomaly);
        }

        // Check software score
        if let Some(anomaly) = self.check_metric(&entries, "software_score", |e| e.software_score as f64)? {
            anomalies.push(anomaly);
        }

        // Check user score
        if let Some(anomaly) = self.check_metric(&entries, "user_score", |e| e.user_score as f64)? {
            anomalies.push(anomaly);
        }

        // Check performance metrics (perfwatch integration)
        if let Some(profiler) = &self.profiler {
            if let Ok(perf_anomalies) = self.check_performance_metrics(profiler) {
                anomalies.extend(perf_anomalies);
            }
        }

        Ok(anomalies)
    }

    /// Check performance metrics for anomalies
    fn check_performance_metrics(&self, profiler: &Profiler) -> Result<Vec<Anomaly>> {
        let mut anomalies = Vec::new();

        let recent = profiler.get_recent(7)?;
        if recent.len() < 3 {
            return Ok(anomalies);  // Need at least 3 samples
        }

        let baseline = profiler.load_baseline()?;
        let latest = match recent.last() {
            Some(entry) => entry,
            None => return Ok(anomalies),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check RPC latency
        if latest.rpc_delta_pct > 50.0 {
            anomalies.push(Anomaly {
                metric: "perf_rpc_latency".to_string(),
                current_value: latest.snapshot.rpc_latency_ms as f64,
                expected_value: baseline.avg_rpc_latency_ms as f64,
                deviation: latest.rpc_delta_pct as f64,
                z_score: latest.rpc_delta_pct as f64 / 15.0,  // Normalized
                severity: if latest.rpc_delta_pct > 100.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                description: format!(
                    "RPC latency {:.1}% above baseline ({:.2}ms vs {:.2}ms)",
                    latest.rpc_delta_pct,
                    latest.snapshot.rpc_latency_ms,
                    baseline.avg_rpc_latency_ms
                ),
                first_detected: now,
                consecutive_days: self.count_consecutive_degraded(&recent),
            });
        }

        // Check memory usage
        if latest.memory_delta_pct > 50.0 {
            anomalies.push(Anomaly {
                metric: "perf_memory".to_string(),
                current_value: latest.snapshot.memory_mb as f64,
                expected_value: baseline.avg_memory_mb as f64,
                deviation: latest.memory_delta_pct as f64,
                z_score: latest.memory_delta_pct as f64 / 15.0,
                severity: if latest.memory_delta_pct > 100.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                description: format!(
                    "Memory usage {:.1}% above baseline ({:.1}MB vs {:.1}MB)",
                    latest.memory_delta_pct,
                    latest.snapshot.memory_mb,
                    baseline.avg_memory_mb
                ),
                first_detected: now,
                consecutive_days: self.count_consecutive_degraded(&recent),
            });
        }

        // Check I/O latency
        if latest.io_delta_pct > 50.0 {
            anomalies.push(Anomaly {
                metric: "perf_io_latency".to_string(),
                current_value: latest.snapshot.io_latency_ms as f64,
                expected_value: baseline.avg_io_latency_ms as f64,
                deviation: latest.io_delta_pct as f64,
                z_score: latest.io_delta_pct as f64 / 15.0,
                severity: if latest.io_delta_pct > 100.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                description: format!(
                    "I/O latency {:.1}% above baseline ({:.2}ms vs {:.2}ms)",
                    latest.io_delta_pct,
                    latest.snapshot.io_latency_ms,
                    baseline.avg_io_latency_ms
                ),
                first_detected: now,
                consecutive_days: self.count_consecutive_degraded(&recent),
            });
        }

        Ok(anomalies)
    }

    /// Count consecutive degraded snapshots
    fn count_consecutive_degraded(&self, entries: &[crate::profiled::PerfWatchEntry]) -> u32 {
        use crate::profiled::DegradationLevel;

        let mut count = 0;
        for entry in entries.iter().rev() {
            if entry.degradation != DegradationLevel::Normal {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    fn load_baseline(&self) -> Result<crate::profiled::PerfBaseline> {
        if let Some(profiler) = &self.profiler {
            profiler.load_baseline()
        } else {
            Ok(crate::profiled::PerfBaseline::default())
        }
    }

    /// Check single metric for anomalies
    fn check_metric<F>(
        &self,
        entries: &[HistoryEntry],
        metric_name: &str,
        extractor: F,
    ) -> Result<Option<Anomaly>>
    where
        F: Fn(&HistoryEntry) -> f64,
    {
        if entries.len() < 7 {
            return Ok(None);
        }

        // Use last 14 days as baseline
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let baseline_window = 14 * 86400;
        let baseline_entries: Vec<&HistoryEntry> = entries
            .iter()
            .filter(|e| e.timestamp >= now.saturating_sub(baseline_window))
            .collect();

        if baseline_entries.len() < 7 {
            return Ok(None);
        }

        // Calculate baseline statistics
        let values: Vec<f64> = baseline_entries.iter().map(|e| extractor(e)).collect();

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Current value
        let current_value = extractor(entries.last().unwrap());

        // Z-score
        let z_score = if std_dev > 0.0 {
            (current_value - mean) / std_dev
        } else {
            0.0
        };

        // Deviation
        let deviation = (current_value - mean).abs();

        // IQR outlier detection (alternative method)
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = sorted_values[sorted_values.len() / 4];
        let q3 = sorted_values[(3 * sorted_values.len()) / 4];
        let iqr = q3 - q1;
        let iqr_lower = q1 - 1.5 * iqr;
        let iqr_upper = q3 + 1.5 * iqr;

        // Check if anomaly exists
        let is_zscore_anomaly = z_score.abs() >= 2.0;
        let is_iqr_anomaly = current_value < iqr_lower || current_value > iqr_upper;

        if !is_zscore_anomaly && !is_iqr_anomaly {
            return Ok(None);
        }

        // Count consecutive days with anomaly
        let consecutive_days = self.count_consecutive_anomalies(&baseline_entries, &extractor, mean, std_dev);

        // Classify severity
        let severity = if z_score.abs() >= 3.0 || consecutive_days >= 3 {
            Severity::Critical
        } else if z_score.abs() >= 2.0 || consecutive_days >= 2 {
            Severity::Warning
        } else {
            Severity::Info
        };

        // Description
        let direction = if current_value < mean { "below" } else { "above" };
        let description = format!(
            "{} is {:.1} ({}σ {}, expected ~{:.1})",
            metric_name.replace("_", " "),
            current_value,
            z_score.abs(),
            direction,
            mean
        );

        Ok(Some(Anomaly {
            metric: metric_name.to_string(),
            current_value,
            expected_value: mean,
            deviation,
            z_score,
            severity,
            description,
            first_detected: entries.last().unwrap().timestamp,
            consecutive_days,
        }))
    }

    /// Count consecutive days with anomaly
    fn count_consecutive_anomalies<F>(
        &self,
        entries: &[&HistoryEntry],
        extractor: &F,
        mean: f64,
        std_dev: f64,
    ) -> u32
    where
        F: Fn(&HistoryEntry) -> f64,
    {
        let mut count = 0;

        for entry in entries.iter().rev() {
            let value = extractor(entry);
            let z = if std_dev > 0.0 {
                (value - mean).abs() / std_dev
            } else {
                0.0
            };

            if z >= 2.0 {
                count += 1;
            } else {
                break;
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_emoji() {
        assert_eq!(Severity::Info.emoji(), "ℹ️");
        assert_eq!(Severity::Warning.emoji(), "⚠️");
        assert_eq!(Severity::Critical.emoji(), "🚨");
    }

    #[test]
    fn test_severity_color() {
        assert_eq!(Severity::Info.color(), "\x1b[36m");
        assert_eq!(Severity::Warning.color(), "\x1b[33m");
        assert_eq!(Severity::Critical.color(), "\x1b[31m");
    }

    #[test]
    fn test_anomaly_is_critical() {
        let anomaly = Anomaly {
            metric: "test".to_string(),
            current_value: 3.0,
            expected_value: 8.0,
            deviation: 5.0,
            z_score: 3.0,
            severity: Severity::Critical,
            description: "test".to_string(),
            first_detected: 0,
            consecutive_days: 3,
        };

        assert!(anomaly.is_critical());
    }

    #[test]
    fn test_anomaly_recommendation() {
        let anomaly = Anomaly {
            metric: "hardware_score".to_string(),
            current_value: 3.0,
            expected_value: 8.0,
            deviation: 5.0,
            z_score: 3.0,
            severity: Severity::Critical,
            description: "test".to_string(),
            first_detected: 0,
            consecutive_days: 3,
        };

        let rec = anomaly.recommendation();
        assert!(rec.contains("hardware") || rec.contains("sensors"));
    }

    #[test]
    fn test_anomaly_serialization() {
        let anomaly = Anomaly {
            metric: "overall_score".to_string(),
            current_value: 4.0,
            expected_value: 8.0,
            deviation: 4.0,
            z_score: 2.5,
            severity: Severity::Warning,
            description: "Score dropped significantly".to_string(),
            first_detected: 1699000000,
            consecutive_days: 2,
        };

        let json = serde_json::to_string(&anomaly).unwrap();
        let parsed: Anomaly = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.metric, "overall_score");
        assert_eq!(parsed.severity, Severity::Warning);
    }

    #[test]
    fn test_z_score_calculation() {
        let values = vec![7.0, 8.0, 7.0, 8.0, 7.0, 8.0, 7.0];
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let current = 3.0; // Outlier
        let z_score = (current - mean) / std_dev;

        assert!(z_score.abs() > 2.0); // Should be detected as anomaly
    }

    #[test]
    fn test_iqr_outlier_detection() {
        let mut values = vec![7.0, 8.0, 7.0, 8.0, 7.0, 8.0, 7.0, 8.0];
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = values[values.len() / 4];
        let q3 = values[(3 * values.len()) / 4];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        let outlier = 2.0;
        assert!(outlier < lower_bound); // Should be detected as outlier
    }

    #[test]
    fn test_detector_creation() {
        let detector = AnomalyDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_detect_anomalies_insufficient_data() {
        let detector = AnomalyDetector::new().unwrap();
        let result = detector.detect_anomalies();

        // Should succeed even with no history
        assert!(result.is_ok());
    }
}
