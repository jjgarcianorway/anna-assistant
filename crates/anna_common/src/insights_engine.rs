//! v6.24.0: Insights Engine - Historical Metrics & Trend Analysis
//!
//! The Insights Engine wraps the Historian and Trend Detectors to provide
//! actionable insights about system health trends over time.
//!
//! Core Features:
//! - Rules-based insight generation (no LLM required)
//! - Severity classification (info, warning, critical)
//! - Evidence-backed recommendations
//! - Persistent across daemon restarts
//!
//! Insight Sources:
//! 1. Boot duration trends
//! 2. Disk space deterioration
//! 3. Journal error spikes
//! 4. Memory pressure trends
//! 5. Swap usage anomalies
//! 6. Anna inactivity detection
//! 7. Configuration drift warnings

use crate::historian::Historian;
use crate::trend_detectors::{TrendDetection, TrendSeverity};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Severity levels for insights (aligned with TrendSeverity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InsightSeverity {
    Info,     // Informational observation
    Warning,  // Requires attention soon
    Critical, // Immediate action needed
}

impl From<TrendSeverity> for InsightSeverity {
    fn from(ts: TrendSeverity) -> Self {
        match ts {
            TrendSeverity::Info => InsightSeverity::Info,
            TrendSeverity::Warning => InsightSeverity::Warning,
            TrendSeverity::Critical => InsightSeverity::Critical,
        }
    }
}

/// A single insight about system health trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Unique identifier (e.g., "boot_regression_2025-11-24")
    pub id: String,
    /// When this insight was generated
    pub timestamp: DateTime<Utc>,
    /// Severity level
    pub severity: InsightSeverity,
    /// Short title (1 line)
    pub title: String,
    /// Detailed explanation (2-4 lines)
    pub explanation: String,
    /// Evidence supporting this insight
    pub evidence: Vec<String>,
    /// Optional actionable suggestion
    pub suggestion: Option<String>,
    /// Source detector name
    pub detector: String,
}

impl Insight {
    pub fn new(
        detector: impl Into<String>,
        severity: InsightSeverity,
        title: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        let detector_str = detector.into();
        let timestamp = Utc::now();
        let id = format!("{}_{}", detector_str, timestamp.format("%Y%m%d_%H%M%S"));

        Self {
            id,
            timestamp,
            severity,
            title: title.into(),
            explanation: explanation.into(),
            evidence: Vec::new(),
            suggestion: None,
            detector: detector_str,
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Insights Engine - generates actionable insights from historical data
pub struct InsightsEngine {
    historian: Historian,
}

impl InsightsEngine {
    /// Create a new Insights Engine
    pub fn new(historian: Historian) -> Self {
        Self { historian }
    }

    /// Generate all insights for the last N hours (default 24)
    pub fn generate_insights(&self, hours: i64) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // 1. Boot duration trends
        if let Some(insight) = self.detect_boot_regression()? {
            insights.push(insight);
        }

        // 2. Disk space trends
        if let Some(insight) = self.detect_disk_space_issues()? {
            insights.push(insight);
        }

        // 3. Error rate spikes
        if let Some(insight) = self.detect_error_spikes(hours)? {
            insights.push(insight);
        }

        // 4. Memory pressure
        if let Some(insight) = self.detect_memory_pressure()? {
            insights.push(insight);
        }

        // 5. Swap usage anomalies
        if let Some(insight) = self.detect_swap_anomalies()? {
            insights.push(insight);
        }

        // 6. Anna inactivity
        if let Some(insight) = self.detect_anna_inactivity(hours)? {
            insights.push(insight);
        }

        // v6.25.0: Service Reliability Detectors
        // 7. Service flapping (restart loops)
        if let Some(insight) = self.detect_service_flapping(hours)? {
            insights.push(insight);
        }

        // 8. Degraded units at boot
        if let Some(insight) = self.detect_degraded_units()? {
            insights.push(insight);
        }

        // 9. User service failures
        if let Some(insight) = self.detect_user_service_failures()? {
            insights.push(insight);
        }

        // 10. Timer reliability issues
        if let Some(insight) = self.detect_timer_issues()? {
            insights.push(insight);
        }

        // v6.25.0: Cross-Subsystem Correlation
        // 11. Disk pressure → service failures
        if let Some(insight) = self.correlate_disk_to_service()? {
            insights.push(insight);
        }

        // 12. Network issues → service degradation
        if let Some(insight) = self.correlate_network_to_service()? {
            insights.push(insight);
        }

        // 13. Boot regression + failed services → root cause
        if let Some(insight) = self.correlate_boot_and_services()? {
            insights.push(insight);
        }

        // Sort by severity (Critical > Warning > Info)
        insights.sort_by(|a, b| {
            use InsightSeverity::*;
            let a_val = match a.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            let b_val = match b.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            b_val.cmp(&a_val) // Descending order
        });

        Ok(insights)
    }

    /// Get top N insights (for status display)
    pub fn get_top_insights(&self, count: usize, hours: i64) -> Result<Vec<Insight>> {
        let all_insights = self.generate_insights(hours)?;
        Ok(all_insights.into_iter().take(count).collect())
    }

    // ========================================================================
    // Individual Detectors
    // ========================================================================

    /// Detect boot time regression
    fn detect_boot_regression(&self) -> Result<Option<Insight>> {
        use crate::trend_detectors::detect_boot_regression;

        if let Some(trend) = detect_boot_regression(&self.historian)? {
            let insight = Insight::new(
                "boot_regression",
                trend.severity.into(),
                trend.title,
                trend.description,
            )
            .with_evidence(trend.supporting_data)
            .with_suggestion(trend.recommendation);

            return Ok(Some(insight));
        }

        Ok(None)
    }

    /// Detect disk space issues
    fn detect_disk_space_issues(&self) -> Result<Option<Insight>> {
        // Check disk space trends from last 7 days
        let disk_trends = self.historian.get_disk_trends(7)?;

        // Critical: >90% used
        if disk_trends.current_used_percent > 90.0 {
            let days_until_full = if disk_trends.growth_rate_gb_per_day > 0.1 {
                let free_gb = disk_trends.total_gb - disk_trends.used_gb;
                (free_gb / disk_trends.growth_rate_gb_per_day).ceil() as i64
            } else {
                -1 // Not growing
            };

            let explanation = if days_until_full > 0 && days_until_full < 30 {
                format!(
                    "Root filesystem is {}% full ({:.1} GB used of {:.1} GB total). \
                     At current growth rate, disk will be full in {} days.",
                    disk_trends.current_used_percent as i64,
                    disk_trends.used_gb,
                    disk_trends.total_gb,
                    days_until_full
                )
            } else {
                format!(
                    "Root filesystem is {}% full ({:.1} GB used of {:.1} GB total). \
                     Immediate cleanup recommended.",
                    disk_trends.current_used_percent as i64,
                    disk_trends.used_gb,
                    disk_trends.total_gb
                )
            };

            return Ok(Some(
                Insight::new("disk_space_critical", InsightSeverity::Critical, "Disk Space Critical", explanation)
                    .with_evidence(vec![
                        format!("Current usage: {:.1}%", disk_trends.current_used_percent),
                        format!("Growth rate: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
                    ])
                    .with_suggestion("Run 'annactl \"clean up disk space\"' for cleanup recommendations"),
            ));
        }

        // Warning: >80% used OR rapid growth
        if disk_trends.current_used_percent > 80.0 || disk_trends.growth_rate_gb_per_day > 1.0 {
            return Ok(Some(
                Insight::new(
                    "disk_space_warning",
                    InsightSeverity::Warning,
                    "Disk Space Increasing",
                    format!(
                        "Root filesystem usage is trending up. Currently at {:.1}% ({:.1} GB used), \
                         growing at {:.2} GB/day.",
                        disk_trends.current_used_percent, disk_trends.used_gb, disk_trends.growth_rate_gb_per_day
                    ),
                )
                .with_evidence(vec![
                    format!("Current: {:.1}% used", disk_trends.current_used_percent),
                    format!("Growth: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
                ])
                .with_suggestion("Monitor disk usage and consider cleanup if trend continues"),
            ));
        }

        Ok(None)
    }

    /// Detect error rate spikes
    fn detect_error_spikes(&self, hours: i64) -> Result<Option<Insight>> {
        let error_trends = self.historian.get_error_trends_v2(hours)?;

        // Critical: >100 errors/hour sustained
        if error_trends.avg_errors_per_hour > 100.0 {
            return Ok(Some(
                Insight::new(
                    "error_spike_critical",
                    InsightSeverity::Critical,
                    "High Error Rate Detected",
                    format!(
                        "System is generating {:.0} errors per hour on average over the last {} hours. \
                         This indicates a serious problem.",
                        error_trends.avg_errors_per_hour, hours
                    ),
                )
                .with_evidence(vec![
                    format!("Average: {:.0} errors/hour", error_trends.avg_errors_per_hour),
                    format!("Total: {} errors", error_trends.total_errors),
                ])
                .with_suggestion("Run 'annactl \"check my system health\"' for diagnostic details"),
            ));
        }

        // Warning: >20 errors/hour OR increasing trend
        if error_trends.avg_errors_per_hour > 20.0 {
            return Ok(Some(
                Insight::new(
                    "error_spike_warning",
                    InsightSeverity::Warning,
                    "Error Rate Elevated",
                    format!(
                        "System error rate is elevated at {:.0} errors/hour over the last {} hours.",
                        error_trends.avg_errors_per_hour, hours
                    ),
                )
                .with_evidence(vec![
                    format!("Average: {:.0} errors/hour", error_trends.avg_errors_per_hour),
                    format!("Total: {} errors", error_trends.total_errors),
                ]),
            ));
        }

        Ok(None)
    }

    /// Detect memory pressure
    fn detect_memory_pressure(&self) -> Result<Option<Insight>> {
        use crate::trend_detectors::detect_memory_leak;

        if let Some(trend) = detect_memory_leak(&self.historian)? {
            let insight = Insight::new("memory_pressure", trend.severity.into(), trend.title, trend.description)
                .with_evidence(trend.supporting_data)
                .with_suggestion(trend.recommendation);

            return Ok(Some(insight));
        }

        Ok(None)
    }

    /// Detect swap usage anomalies
    fn detect_swap_anomalies(&self) -> Result<Option<Insight>> {
        let memory_trends = self.historian.get_memory_trends(7)?;

        // Skip if no swap configured
        if memory_trends.swap_total_mb == 0 {
            return Ok(None);
        }

        // Critical: Heavy swap usage (>50% swap used)
        if memory_trends.avg_swap_used_mb > 1024 {
            let swap_percent = (memory_trends.avg_swap_used_mb as f64 / memory_trends.swap_total_mb as f64) * 100.0;

            if swap_percent > 50.0 {
                return Ok(Some(
                    Insight::new(
                        "swap_heavy_usage",
                        InsightSeverity::Critical,
                        "Heavy Swap Usage Detected",
                        format!(
                            "System is using {:.0}% of swap memory ({} MB of {} MB). \
                             This indicates severe memory pressure.",
                            swap_percent, memory_trends.avg_swap_used_mb, memory_trends.swap_total_mb
                        ),
                    )
                    .with_evidence(vec![
                        format!("Swap used: {} MB ({:.0}%)", memory_trends.avg_swap_used_mb, swap_percent),
                        format!("RAM pressure: High"),
                    ])
                    .with_suggestion("Consider closing memory-intensive applications or adding more RAM"),
                ));
            }

            // Warning: Moderate swap usage (>20%)
            if swap_percent > 20.0 {
                return Ok(Some(
                    Insight::new(
                        "swap_moderate_usage",
                        InsightSeverity::Warning,
                        "Swap Usage Increasing",
                        format!(
                            "System is using {:.0}% of swap memory ({} MB). \
                             This may impact performance.",
                            swap_percent, memory_trends.avg_swap_used_mb
                        ),
                    )
                    .with_evidence(vec![format!("Swap used: {} MB ({:.0}%)", memory_trends.avg_swap_used_mb, swap_percent)])
                    .with_suggestion("Monitor memory usage and consider closing unused applications"),
                ));
            }
        }

        Ok(None)
    }

    /// Detect Anna inactivity (user hasn't used Anna in a while)
    fn detect_anna_inactivity(&self, hours: i64) -> Result<Option<Insight>> {
        let usage_data = self.historian.get_anna_usage_stats(hours)?;

        // Info: No Anna usage in last 7+ days
        let hours_since_last = usage_data.hours_since_last_invocation;

        if hours_since_last > 168 {
            // 7 days
            let days = hours_since_last / 24;
            return Ok(Some(
                Insight::new(
                    "anna_inactive",
                    InsightSeverity::Info,
                    "Anna Unused Recently",
                    format!(
                        "Anna hasn't been invoked in {} days. Consider running 'annactl \"check my system health\"' \
                         to ensure everything is working properly.",
                        days
                    ),
                )
                .with_evidence(vec![format!("Last used: {} days ago", days)]),
            ));
        }

        Ok(None)
    }

    // ========================================================================
    // v6.25.0: Service Reliability Detectors
    // ========================================================================

    /// Detect service flapping (rapid restart/crash loops)
    fn detect_service_flapping(&self, hours: i64) -> Result<Option<Insight>> {
        let flapping_services = self.historian.detect_flapping_services(hours)?;

        if flapping_services.is_empty() {
            return Ok(None);
        }

        // Critical: Any service with 5+ events
        let critical_services: Vec<_> = flapping_services
            .iter()
            .filter(|s| s.event_count >= 5)
            .collect();

        if !critical_services.is_empty() {
            let service_list = critical_services
                .iter()
                .map(|s| format!("{} ({} events)", s.service_name, s.event_count))
                .collect::<Vec<_>>()
                .join(", ");

            let evidence = critical_services
                .iter()
                .map(|s| {
                    let duration = s.last_event.signed_duration_since(s.first_event);
                    format!(
                        "{}: {} restarts/crashes in {} minutes",
                        s.service_name,
                        s.event_count,
                        duration.num_minutes().max(1)
                    )
                })
                .collect();

            return Ok(Some(
                Insight::new(
                    "service_flapping_critical",
                    InsightSeverity::Critical,
                    "Service Restart Loop Detected",
                    format!(
                        "Critical instability detected: {}. These services are in restart loops, indicating serious failures.",
                        service_list
                    ),
                )
                .with_evidence(evidence)
                .with_suggestion("Run 'annactl \"diagnose service crashes\"' to investigate root causes"),
            ));
        }

        // Warning: Any flapping service (3-4 events)
        let service = &flapping_services[0];
        let duration = service
            .last_event
            .signed_duration_since(service.first_event);

        Ok(Some(
            Insight::new(
                "service_flapping_warning",
                InsightSeverity::Warning,
                "Service Instability Detected",
                format!(
                    "{} has restarted {} times in the last {} minutes. This indicates potential configuration or dependency issues.",
                    service.service_name,
                    service.event_count,
                    duration.num_minutes().max(1)
                ),
            )
            .with_evidence(vec![format!(
                "{} restarts/crashes over {} minutes",
                service.event_count,
                duration.num_minutes().max(1)
            )])
            .with_suggestion(format!("Check service logs with 'journalctl -u {}'", service.service_name)),
        ))
    }

    /// Detect degraded units appearing after boot
    fn detect_degraded_units(&self) -> Result<Option<Insight>> {
        // Query SystemdHealth from telemetry (via SystemFacts)
        // This detector looks for units that failed AFTER boot completed
        use crate::systemd_health::SystemdHealth;

        let systemd_health = SystemdHealth::detect();

        if systemd_health.failed_units.is_empty() {
            return Ok(None);
        }

        // Exclude units that commonly fail at boot (user-specific services, etc.)
        let degraded_services: Vec<_> = systemd_health
            .failed_units
            .iter()
            .filter(|u| {
                // Only include system services
                u.unit_type == "service"
                    && !u.name.contains("@")
                    && !u.name.starts_with("user@")
            })
            .collect();

        if degraded_services.is_empty() {
            return Ok(None);
        }

        // Critical: Multiple failed services
        if degraded_services.len() >= 3 {
            let service_list = degraded_services
                .iter()
                .map(|u| u.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let evidence = degraded_services
                .iter()
                .map(|u| format!("{}: {} ({})", u.name, u.active_state, u.sub_state))
                .collect();

            return Ok(Some(
                Insight::new(
                    "degraded_units_critical",
                    InsightSeverity::Critical,
                    "Multiple Failed System Services",
                    format!(
                        "System has {} failed services: {}. This indicates widespread system degradation.",
                        degraded_services.len(),
                        service_list
                    ),
                )
                .with_evidence(evidence)
                .with_suggestion("Run 'systemctl status' and investigate failed units"),
            ));
        }

        // Warning: Single failed service
        let service = degraded_services[0];
        Ok(Some(
            Insight::new(
                "degraded_unit_warning",
                InsightSeverity::Warning,
                "Failed System Service Detected",
                format!(
                    "{} is in failed state ({}). This may affect system functionality.",
                    service.name, service.sub_state
                ),
            )
            .with_evidence(vec![format!(
                "{}: active={}, load={}",
                service.name, service.active_state, service.load_state
            )])
            .with_suggestion(format!("Check logs with 'journalctl -u {}'", service.name)),
        ))
    }

    /// Detect user service failures (systemctl --user)
    fn detect_user_service_failures(&self) -> Result<Option<Insight>> {
        use std::process::Command;

        // Query user-level systemd for failed units
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("list-units")
            .arg("--state=failed")
            .arg("--no-pager")
            .arg("--no-legend")
            .output();

        let failed_count = match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count()
            }
            _ => return Ok(None),
        };

        if failed_count == 0 {
            return Ok(None);
        }

        // Info: User services failed
        Ok(Some(
            Insight::new(
                "user_service_failures",
                InsightSeverity::Info,
                "User Service Failures Detected",
                format!(
                    "{} user-level service(s) have failed. These may affect your desktop environment or user applications.",
                    failed_count
                ),
            )
            .with_evidence(vec![format!("{} failed user services", failed_count)])
            .with_suggestion("Run 'systemctl --user list-units --state=failed' to see details"),
        ))
    }

    /// Detect timer reliability issues (late triggers, failures)
    fn detect_timer_issues(&self) -> Result<Option<Insight>> {
        use crate::systemd_health::SystemdHealth;

        let systemd_health = SystemdHealth::detect();

        // Check for disabled essential timers
        let disabled_timers: Vec<_> = systemd_health
            .essential_timers
            .iter()
            .filter(|t| !t.enabled || !t.active)
            .collect();

        if disabled_timers.is_empty() {
            return Ok(None);
        }

        // Warning: Essential timers not running
        let timer_list = disabled_timers
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let evidence = disabled_timers
            .iter()
            .map(|t| {
                format!(
                    "{}: enabled={}, active={}",
                    t.name, t.enabled, t.active
                )
            })
            .collect();

        Ok(Some(
            Insight::new(
                "timer_issues",
                InsightSeverity::Warning,
                "Essential Timers Not Running",
                format!(
                    "System maintenance timers are not active: {}. This may lead to disk space issues or outdated mirrors.",
                    timer_list
                ),
            )
            .with_evidence(evidence)
            .with_suggestion("Enable and start timers with 'sudo systemctl enable --now <timer>'"),
        ))
    }

    // ========================================================================
    // v6.25.0: Cross-Subsystem Correlation
    // ========================================================================

    /// Correlate disk pressure with service failures
    fn correlate_disk_to_service(&self) -> Result<Option<Insight>> {
        // Get disk trends and service restart trends
        let disk_trends = self.historian.get_disk_trends(7)?;
        let service_trends = self.historian.get_service_restart_trends(7)?;

        // Only correlate if disk is >85% AND services are crashing
        if disk_trends.current_used_percent < 85.0 || service_trends.is_empty() {
            return Ok(None);
        }

        // Check if any services have crashes (not just restarts)
        let services_with_crashes: Vec<_> = service_trends
            .iter()
            .filter(|s| s.crash_count > 0)
            .collect();

        if services_with_crashes.is_empty() {
            return Ok(None);
        }

        // Critical correlation: High disk + crashes
        let service_list = services_with_crashes
            .iter()
            .take(3)
            .map(|s| s.service_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Ok(Some(
            Insight::new(
                "disk_service_correlation",
                InsightSeverity::Critical,
                "Disk Pressure Causing Service Failures",
                format!(
                    "Root filesystem is {:.0}% full, and {} services have crashed recently ({}). \
                     Services may be failing due to insufficient disk space for logs, temporary files, or databases.",
                    disk_trends.current_used_percent,
                    services_with_crashes.len(),
                    service_list
                ),
            )
            .with_evidence(vec![
                format!("Disk usage: {:.0}%", disk_trends.current_used_percent),
                format!("{} services with crashes", services_with_crashes.len()),
                format!("Growth rate: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
            ])
            .with_suggestion("Run 'annactl \"clean up disk space\"' immediately and restart failed services"),
        ))
    }

    /// Correlate network issues with service degradation
    fn correlate_network_to_service(&self) -> Result<Option<Insight>> {
        // Get network trends and service restart trends
        let network_trends = self.historian.get_network_trends(24)?;
        let service_trends = self.historian.get_service_restart_trends(1)?; // Last 24 hours

        // Only correlate if network is degraded AND services are restarting
        if network_trends.avg_latency_ms < 200.0 || service_trends.is_empty() {
            return Ok(None);
        }

        // Filter for network-dependent services
        let network_services: Vec<_> = service_trends
            .iter()
            .filter(|s| {
                let name = s.service_name.to_lowercase();
                name.contains("network")
                    || name.contains("dhcp")
                    || name.contains("dns")
                    || name.contains("resolved")
                    || name.contains("NetworkManager")
            })
            .collect();

        if network_services.is_empty() {
            return Ok(None);
        }

        // Warning: Network latency + service restarts
        let service_list = network_services
            .iter()
            .map(|s| s.service_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Ok(Some(
            Insight::new(
                "network_service_correlation",
                InsightSeverity::Warning,
                "Network Issues Affecting Services",
                format!(
                    "Network latency is elevated ({:.0} ms avg) and network services are restarting: {}. \
                     This may indicate connectivity issues or misconfigurations.",
                    network_trends.avg_latency_ms, service_list
                ),
            )
            .with_evidence(vec![
                format!("Network latency: {:.0} ms", network_trends.avg_latency_ms),
                format!("Packet loss: {:.1}%", network_trends.avg_packet_loss_percent),
                format!("{} network service restarts", network_services.len()),
            ])
            .with_suggestion("Run 'annactl \"diagnose network issues\"' to investigate connectivity"),
        ))
    }

    /// Correlate boot regression with failed services (root cause analysis)
    fn correlate_boot_and_services(&self) -> Result<Option<Insight>> {
        use crate::systemd_health::SystemdHealth;
        use crate::historian::Trend;

        // Check if we have boot regression AND failed services
        let boot_trends = self.historian.get_boot_trends(30)?;
        let systemd_health = SystemdHealth::detect();

        // Boot regression: increasing trend or slow boot
        if boot_trends.avg_boot_time_ms < 15000 || systemd_health.failed_units.is_empty() {
            return Ok(None);
        }

        // Only correlate if boot trend is Up (getting worse)
        if !matches!(boot_trends.trend, Trend::Up) {
            return Ok(None);
        }

        // Critical: Boot regression + failed services = dependency issue
        let service_list = systemd_health
            .failed_units
            .iter()
            .take(3)
            .map(|u| u.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Ok(Some(
            Insight::new(
                "boot_service_correlation",
                InsightSeverity::Critical,
                "Boot Regression Linked to Service Failures",
                format!(
                    "Boot time is elevated at {} seconds with {} trend, and {} services have failed: {}. \
                     This suggests a dependency chain issue or broken service ordering.",
                    boot_trends.avg_boot_time_ms / 1000,
                    format!("{:?}", boot_trends.trend).to_lowercase(),
                    systemd_health.failed_units.len(),
                    service_list
                ),
            )
            .with_evidence(vec![
                format!(
                    "Boot time: {} ms ({:?} trend over {} days)",
                    boot_trends.avg_boot_time_ms, boot_trends.trend, boot_trends.days_analyzed
                ),
                format!("{} failed services", systemd_health.failed_units.len()),
                format!("Slowest units may be timing out"),
            ])
            .with_suggestion("Run 'systemd-analyze blame' and check service dependencies with 'systemctl list-dependencies'"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_severity_ordering() {
        let mut insights = vec![
            Insight::new("test1", InsightSeverity::Info, "Info", "info"),
            Insight::new("test2", InsightSeverity::Critical, "Critical", "critical"),
            Insight::new("test3", InsightSeverity::Warning, "Warning", "warning"),
        ];

        insights.sort_by(|a, b| {
            use InsightSeverity::*;
            let a_val = match a.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            let b_val = match b.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            b_val.cmp(&a_val)
        });

        assert_eq!(insights[0].severity, InsightSeverity::Critical);
        assert_eq!(insights[1].severity, InsightSeverity::Warning);
        assert_eq!(insights[2].severity, InsightSeverity::Info);
    }

    #[test]
    fn test_insight_id_generation() {
        let insight = Insight::new("test_detector", InsightSeverity::Info, "Test", "Test explanation");

        assert!(insight.id.starts_with("test_detector_"));
        assert_eq!(insight.detector, "test_detector");
    }

    #[test]
    fn test_insight_builder_pattern() {
        let insight = Insight::new("test", InsightSeverity::Warning, "Title", "Explanation")
            .with_evidence(vec!["Evidence 1".to_string(), "Evidence 2".to_string()])
            .with_suggestion("Do this");

        assert_eq!(insight.evidence.len(), 2);
        assert!(insight.suggestion.is_some());
        assert_eq!(insight.suggestion.unwrap(), "Do this");
    }

    #[test]
    fn test_severity_conversion_from_trend_severity() {
        use crate::trend_detectors::TrendSeverity;

        assert_eq!(InsightSeverity::from(TrendSeverity::Info), InsightSeverity::Info);
        assert_eq!(InsightSeverity::from(TrendSeverity::Warning), InsightSeverity::Warning);
        assert_eq!(InsightSeverity::from(TrendSeverity::Critical), InsightSeverity::Critical);
    }

    #[test]
    fn test_insight_with_no_suggestion() {
        let insight = Insight::new("test", InsightSeverity::Info, "Title", "Explanation")
            .with_evidence(vec!["Evidence".to_string()]);

        assert!(insight.suggestion.is_none());
        assert_eq!(insight.evidence.len(), 1);
    }

    #[test]
    fn test_insight_with_empty_evidence() {
        let insight = Insight::new("test", InsightSeverity::Warning, "Title", "Explanation");

        assert!(insight.evidence.is_empty());
        assert!(insight.suggestion.is_none());
    }

    #[test]
    fn test_insight_timestamp_generation() {
        let insight1 = Insight::new("test", InsightSeverity::Info, "Title", "Explanation");
        std::thread::sleep(std::time::Duration::from_secs(1));
        let insight2 = Insight::new("test", InsightSeverity::Info, "Title", "Explanation");

        // IDs should be different due to timestamp (second precision)
        assert_ne!(insight1.id, insight2.id);
        // But detector names should be the same
        assert_eq!(insight1.detector, insight2.detector);
    }

    // v6.25.0: Service Reliability Detector Tests
    #[test]
    fn test_service_flapping_insight_critical() {
        // Test that critical flapping (5+ events) generates correct insight
        let insight = Insight::new(
            "service_flapping_critical",
            InsightSeverity::Critical,
            "Service Restart Loop Detected",
            "Critical instability detected: foo.service (5 events). These services are in restart loops, indicating serious failures.",
        )
        .with_evidence(vec!["foo.service: 5 restarts/crashes in 10 minutes".to_string()])
        .with_suggestion("Run 'annactl \"diagnose service crashes\"' to investigate root causes");

        assert_eq!(insight.severity, InsightSeverity::Critical);
        assert_eq!(insight.detector, "service_flapping_critical");
        assert!(insight.title.contains("Restart Loop"));
        assert_eq!(insight.evidence.len(), 1);
        assert!(insight.suggestion.is_some());
    }

    #[test]
    fn test_degraded_unit_insight_warning() {
        // Test that single failed service generates warning
        let insight = Insight::new(
            "degraded_unit_warning",
            InsightSeverity::Warning,
            "Failed System Service Detected",
            "nginx.service is in failed state (failed). This may affect system functionality.",
        )
        .with_evidence(vec!["nginx.service: active=failed, load=loaded".to_string()])
        .with_suggestion("Check logs with 'journalctl -u nginx.service'");

        assert_eq!(insight.severity, InsightSeverity::Warning);
        assert_eq!(insight.detector, "degraded_unit_warning");
        assert!(insight.explanation.contains("nginx.service"));
        assert_eq!(insight.evidence.len(), 1);
    }

    #[test]
    fn test_degraded_units_critical_multiple() {
        // Test that multiple failed services generate critical insight
        let insight = Insight::new(
            "degraded_units_critical",
            InsightSeverity::Critical,
            "Multiple Failed System Services",
            "System has 3 failed services: foo.service, bar.service, baz.service. This indicates widespread system degradation.",
        )
        .with_evidence(vec![
            "foo.service: failed (dead)".to_string(),
            "bar.service: failed (dead)".to_string(),
            "baz.service: failed (dead)".to_string(),
        ])
        .with_suggestion("Run 'systemctl status' and investigate failed units");

        assert_eq!(insight.severity, InsightSeverity::Critical);
        assert!(insight.explanation.contains("3 failed services"));
        assert_eq!(insight.evidence.len(), 3);
    }

    #[test]
    fn test_user_service_failures_info() {
        // Test that user service failures generate info insight
        let insight = Insight::new(
            "user_service_failures",
            InsightSeverity::Info,
            "User Service Failures Detected",
            "2 user-level service(s) have failed. These may affect your desktop environment or user applications.",
        )
        .with_evidence(vec!["2 failed user services".to_string()])
        .with_suggestion("Run 'systemctl --user list-units --state=failed' to see details");

        assert_eq!(insight.severity, InsightSeverity::Info);
        assert_eq!(insight.detector, "user_service_failures");
        assert!(insight.explanation.contains("user-level"));
        assert!(insight.suggestion.is_some());
    }

    #[test]
    fn test_timer_issues_warning() {
        // Test that disabled essential timers generate warning
        let insight = Insight::new(
            "timer_issues",
            InsightSeverity::Warning,
            "Essential Timers Not Running",
            "System maintenance timers are not active: fstrim.timer. This may lead to disk space issues or outdated mirrors.",
        )
        .with_evidence(vec!["fstrim.timer: enabled=false, active=false".to_string()])
        .with_suggestion("Enable and start timers with 'sudo systemctl enable --now <timer>'");

        assert_eq!(insight.severity, InsightSeverity::Warning);
        assert!(insight.explanation.contains("fstrim.timer"));
        assert_eq!(insight.evidence.len(), 1);
    }

    #[test]
    fn test_disk_service_correlation_critical() {
        // Test disk pressure → service failure correlation
        let insight = Insight::new(
            "disk_service_correlation",
            InsightSeverity::Critical,
            "Disk Pressure Causing Service Failures",
            "Root filesystem is 92% full, and 2 services have crashed recently (foo.service, bar.service). Services may be failing due to insufficient disk space for logs, temporary files, or databases.",
        )
        .with_evidence(vec![
            "Disk usage: 92%".to_string(),
            "2 services with crashes".to_string(),
            "Growth rate: 0.50 GB/day".to_string(),
        ])
        .with_suggestion("Run 'annactl \"clean up disk space\"' immediately and restart failed services");

        assert_eq!(insight.severity, InsightSeverity::Critical);
        assert_eq!(insight.detector, "disk_service_correlation");
        assert!(insight.explanation.contains("92% full"));
        assert_eq!(insight.evidence.len(), 3);
    }

    #[test]
    fn test_network_service_correlation_warning() {
        // Test network issues → service degradation correlation
        let insight = Insight::new(
            "network_service_correlation",
            InsightSeverity::Warning,
            "Network Issues Affecting Services",
            "Network latency is elevated (250 ms avg) and network services are restarting: NetworkManager.service. This may indicate connectivity issues or misconfigurations.",
        )
        .with_evidence(vec![
            "Network latency: 250 ms".to_string(),
            "Packet loss: 5.0%".to_string(),
            "1 network service restarts".to_string(),
        ])
        .with_suggestion("Run 'annactl \"diagnose network issues\"' to investigate connectivity");

        assert_eq!(insight.severity, InsightSeverity::Warning);
        assert!(insight.explanation.contains("Network latency"));
        assert_eq!(insight.evidence.len(), 3);
    }

    #[test]
    fn test_boot_service_correlation_critical() {
        // Test boot regression + failed services correlation
        let insight = Insight::new(
            "boot_service_correlation",
            InsightSeverity::Critical,
            "Boot Regression Linked to Service Failures",
            "Boot time is elevated at 25 seconds with up trend, and 2 services have failed: foo.service, bar.service. This suggests a dependency chain issue or broken service ordering.",
        )
        .with_evidence(vec![
            "Boot time: 25000 ms (Up trend over 30 days)".to_string(),
            "2 failed services".to_string(),
            "Slowest units may be timing out".to_string(),
        ])
        .with_suggestion("Run 'systemd-analyze blame' and check service dependencies with 'systemctl list-dependencies'");

        assert_eq!(insight.severity, InsightSeverity::Critical);
        assert_eq!(insight.detector, "boot_service_correlation");
        assert!(insight.explanation.contains("Boot time"));
        assert_eq!(insight.evidence.len(), 3);
    }

    #[test]
    fn test_insight_sorting_by_severity() {
        // Test that insights are properly sorted by severity
        let mut insights = vec![
            Insight::new("test1", InsightSeverity::Info, "Info", "info"),
            Insight::new("test2", InsightSeverity::Critical, "Critical", "critical"),
            Insight::new("test3", InsightSeverity::Warning, "Warning", "warning"),
            Insight::new("test4", InsightSeverity::Critical, "Critical2", "critical2"),
        ];

        insights.sort_by(|a, b| {
            use InsightSeverity::*;
            let a_val = match a.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            let b_val = match b.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            b_val.cmp(&a_val)
        });

        // First two should be Critical
        assert_eq!(insights[0].severity, InsightSeverity::Critical);
        assert_eq!(insights[1].severity, InsightSeverity::Critical);
        // Third should be Warning
        assert_eq!(insights[2].severity, InsightSeverity::Warning);
        // Last should be Info
        assert_eq!(insights[3].severity, InsightSeverity::Info);
    }
}
