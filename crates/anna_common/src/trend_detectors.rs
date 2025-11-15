//! Trend-Based Detectors
//!
//! This module provides intelligent detectors that analyze historical trends
//! from the Historian system to generate proactive warnings and suggestions.
//!
//! Key detectors:
//! - Boot time regression (>20% slower than baseline)
//! - Memory leak detection (sustained growth over 30 days)
//! - Disk growth prediction ("disk will be full in N days")
//! - Service reliability warnings (stability score drops)
//! - Performance degradation alerts (health score decline)

use crate::historian::{Historian, Trend};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A trend-based detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDetection {
    pub detector_name: String,
    pub severity: TrendSeverity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub supporting_data: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendSeverity {
    Info,      // Informational trend observation
    Warning,   // Concerning trend that needs attention
    Critical,  // Critical trend requiring immediate action
}

impl TrendDetection {
    pub fn new(
        detector_name: impl Into<String>,
        severity: TrendSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            detector_name: detector_name.into(),
            severity,
            title: title.into(),
            description: description.into(),
            recommendation: recommendation.into(),
            supporting_data: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: Vec<String>) -> Self {
        self.supporting_data = data;
        self
    }
}

/// Boot Time Regression Detector
///
/// Alerts when boot time is trending up significantly
pub fn detect_boot_regression(historian: &Historian) -> Result<Option<TrendDetection>> {
    let boot_trends = historian.get_boot_trends(30)?;

    // Check if boot time is trending up
    if boot_trends.avg_boot_time_ms > 15000 && matches!(boot_trends.trend, Trend::Up) {
        let data = vec![
            format!("Average boot time: {} ms", boot_trends.avg_boot_time_ms),
            format!("Trend: Increasing over {} days", boot_trends.days_analyzed),
        ];

        return Ok(Some(
            TrendDetection::new(
                "boot_regression",
                TrendSeverity::Warning,
                "Boot Time Increasing",
                format!(
                    "System boot time has been increasing over the last {} days. \
                     Average boot time is now {} ms ({} seconds).",
                    boot_trends.days_analyzed,
                    boot_trends.avg_boot_time_ms,
                    boot_trends.avg_boot_time_ms / 1000
                ),
                "Review recently enabled systemd services with 'systemd-analyze blame' and consider \
                 disabling unnecessary startup units."
            )
            .with_data(data)
        ));
    }

    Ok(None)
}

/// Memory Leak Detector
///
/// Warns about sustained memory growth over 30 days
pub fn detect_memory_leak(historian: &Historian) -> Result<Option<TrendDetection>> {
    let memory_trends = historian.get_memory_trends(30)?;

    // Check for sustained upward memory usage
    if matches!(memory_trends.trend, Trend::Up) && memory_trends.avg_ram_used_mb > 4096 {
        let avg_gb = memory_trends.avg_ram_used_mb as f64 / 1024.0;

        let data = vec![
            format!("Average RAM usage: {:.1} GB", avg_gb),
            format!("Trend: Increasing over {} days", memory_trends.days_analyzed),
        ];

        return Ok(Some(
            TrendDetection::new(
                "memory_leak",
                TrendSeverity::Warning,
                "Memory Usage Increasing",
                format!(
                    "RAM usage has been steadily increasing over the last {} days. \
                     Average usage is now {:.1} GB.",
                    memory_trends.days_analyzed, avg_gb
                ),
                "Investigate long-running processes for memory leaks. Use 'systemctl status' \
                 to check daemon memory usage. Consider restarting services with high memory growth."
            )
            .with_data(data)
        ));
    }

    Ok(None)
}

/// Disk Growth Predictor
///
/// Calculates "disk will be full in N days" predictions for common filesystems
pub fn predict_disk_full(historian: &Historian) -> Result<Vec<TrendDetection>> {
    let mut detections = Vec::new();

    // Analyze common filesystems
    for filesystem in &["/", "/home", "/var"] {
        if let Ok(analysis) = historian.analyze_disk_growth(filesystem, 30) {
            if analysis.growth_rate_gb_per_day > 0.01 {
                if let Some(days_until_full) = analysis.days_until_full {
                    if days_until_full > 0 && days_until_full <= 60 {
                        let severity = if days_until_full <= 7 {
                            TrendSeverity::Critical
                        } else if days_until_full <= 30 {
                            TrendSeverity::Warning
                        } else {
                            TrendSeverity::Info
                        };

                        let data = vec![
                            format!("Filesystem: {}", analysis.filesystem),
                            format!("Current usage: {:.1} GB", analysis.current_used_gb),
                            format!("Growth rate: {:.2} GB/day", analysis.growth_rate_gb_per_day),
                            format!("Days until full: {}", days_until_full),
                        ];

                        detections.push(
                            TrendDetection::new(
                                "disk_full_prediction",
                                severity,
                                format!("Disk Space Running Low: {}", analysis.filesystem),
                                format!(
                                    "Based on current growth trends, {} will be full in approximately {} days. \
                                     Current usage: {:.1} GB, growing at {:.2} GB/day.",
                                    analysis.filesystem,
                                    days_until_full,
                                    analysis.current_used_gb,
                                    analysis.growth_rate_gb_per_day
                                ),
                                if filesystem == &"/" || filesystem == &"/home" {
                                    "Clean up old files, remove unused packages (pacman -Sc), \
                                     and consider moving large files to another partition."
                                } else {
                                    "Investigate what's consuming disk space and clean up unnecessary files."
                                }
                            )
                            .with_data(data)
                        );
                    }
                }
            }
        }
    }

    Ok(detections)
}

/// Service Reliability Detector
///
/// Alerts when service stability score is low
pub fn detect_service_issues(historian: &Historian) -> Result<Vec<TrendDetection>> {
    let service_stability = historian.get_service_stability_scores()?;
    let mut detections = Vec::new();

    for service in service_stability {
        // Alert if stability score is below 80%
        if service.stability_score < 80 && service.total_crashes > 0 {
            let severity = if service.stability_score < 50 {
                TrendSeverity::Critical
            } else if service.stability_score < 70 {
                TrendSeverity::Warning
            } else {
                TrendSeverity::Info
            };

            let data = vec![
                format!("Service: {}", service.service_name),
                format!("Stability score: {}%", service.stability_score),
                format!("Total crashes: {}", service.total_crashes),
            ];

            detections.push(
                TrendDetection::new(
                    "service_reliability",
                    severity,
                    format!("Service Reliability Issue: {}", service.service_name),
                    format!(
                        "The {} service has been unreliable. \
                         Stability score: {}%, with {} crashes recorded.",
                        service.service_name, service.stability_score, service.total_crashes
                    ),
                    format!(
                        "Check service logs with 'journalctl -u {}' to diagnose the issue. \
                         Consider reviewing the service configuration or dependencies.",
                        service.service_name
                    )
                )
                .with_data(data)
            );
        }
    }

    Ok(detections)
}

/// Performance Degradation Detector
///
/// Alerts on overall system health score decline
pub fn detect_performance_degradation(historian: &Historian) -> Result<Option<TrendDetection>> {
    let health_summary = historian.get_health_summary(30)?;

    // Check if any health score is below threshold
    let mut issues = Vec::new();

    if health_summary.avg_stability_score < 70 {
        issues.push(format!(
            "Stability score is {} (below 70)",
            health_summary.avg_stability_score
        ));
    }

    if health_summary.avg_performance_score < 70 {
        issues.push(format!(
            "Performance score is {} (below 70)",
            health_summary.avg_performance_score
        ));
    }

    if health_summary.avg_noise_score < 70 {
        issues.push(format!(
            "Noise score is {} (indicating high error volume)",
            health_summary.avg_noise_score
        ));
    }

    if !issues.is_empty() {
        let severity = if health_summary.avg_stability_score < 50
            || health_summary.avg_performance_score < 50 {
            TrendSeverity::Critical
        } else {
            TrendSeverity::Warning
        };

        let data = vec![
            format!("Stability: {}/100", health_summary.avg_stability_score),
            format!("Performance: {}/100", health_summary.avg_performance_score),
            format!("Noise: {}/100", health_summary.avg_noise_score),
            format!("Analyzed over {} days", health_summary.days_analyzed),
        ];

        return Ok(Some(
            TrendDetection::new(
                "performance_degradation",
                severity,
                "System Health Below Optimal",
                format!(
                    "Overall system health has been suboptimal over the last {} days. {}",
                    health_summary.days_analyzed,
                    issues.join(". ")
                ),
                "Review recent system changes, check for failing services, investigate resource usage, \
                 and consider reviewing the Historian's detailed trends for root cause analysis."
            )
            .with_data(data)
        ));
    }

    Ok(None)
}

/// CPU Spike Pattern Detector
///
/// Detects sustained high CPU usage
pub fn detect_cpu_patterns(historian: &Historian) -> Result<Option<TrendDetection>> {
    let cpu_trends = historian.get_cpu_trends(30)?;

    // If CPU usage is consistently high and trending up
    if cpu_trends.avg_utilization_percent > 70.0 && matches!(cpu_trends.trend, Trend::Up) {
        let data = vec![
            format!("Average CPU usage: {:.1}%", cpu_trends.avg_utilization_percent),
            format!("Trend: Increasing over {} days", cpu_trends.days_analyzed),
        ];

        let severity = if cpu_trends.avg_utilization_percent > 85.0 {
            TrendSeverity::Warning
        } else {
            TrendSeverity::Info
        };

        return Ok(Some(
            TrendDetection::new(
                "cpu_high_usage",
                severity,
                "High CPU Usage Trend",
                format!(
                    "CPU utilization has been high over the last {} days, averaging {:.1}% and increasing.",
                    cpu_trends.days_analyzed, cpu_trends.avg_utilization_percent
                ),
                "Check for resource-intensive processes with 'top' or 'htop'. \
                 Review cron jobs and systemd timers that might be causing load."
            )
            .with_data(data)
        ));
    }

    Ok(None)
}

/// Run all trend detectors and return combined results
pub fn run_all_detectors(historian: &Historian) -> Result<Vec<TrendDetection>> {
    let mut detections = Vec::new();

    // Boot regression
    if let Some(detection) = detect_boot_regression(historian)? {
        detections.push(detection);
    }

    // Memory leak
    if let Some(detection) = detect_memory_leak(historian)? {
        detections.push(detection);
    }

    // Disk growth predictions
    detections.extend(predict_disk_full(historian)?);

    // Service reliability
    detections.extend(detect_service_issues(historian)?);

    // Performance degradation
    if let Some(detection) = detect_performance_degradation(historian)? {
        detections.push(detection);
    }

    // CPU patterns
    if let Some(detection) = detect_cpu_patterns(historian)? {
        detections.push(detection);
    }

    Ok(detections)
}
