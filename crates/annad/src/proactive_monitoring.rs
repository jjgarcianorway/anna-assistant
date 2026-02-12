//! Proactive Background Monitoring - Anna checks system health proactively.
//!
//! Philosophy: Don't wait for user to ask. Weekly health checks, alert only if critical.
//! NO HARDCODING: Intelligent thresholds based on system patterns.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Proactive monitoring schedule and results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringState {
    /// Last full health check
    pub last_health_check: Option<DateTime<Utc>>,
    /// Last regression scan
    pub last_regression_scan: Option<DateTime<Utc>>,
    /// Last cleanup scan
    pub last_cleanup_scan: Option<DateTime<Utc>>,
    /// Last prediction update
    pub last_prediction_update: Option<DateTime<Utc>>,
    /// Pending alerts (critical issues found)
    pub pending_alerts: Vec<ProactiveAlert>,
}

/// A proactive alert that needs user attention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveAlert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub recommendations: Vec<String>,
    pub detected_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>, // When alert is no longer relevant
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertType {
    DiskCritical,
    MemoryLeak,
    RegressionSevere,
    SecurityBreach,
    ServiceFailure,
    PredictiveCritical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Critical,  // Immediate action required
    High,      // Action needed soon (<7 days)
    Medium,    // Plan accordingly
    Low,       // Informational
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            last_health_check: None,
            last_regression_scan: None,
            last_cleanup_scan: None,
            last_prediction_update: None,
            pending_alerts: Vec::new(),
        }
    }
}

impl MonitoringState {
    /// Load from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&contents) {
                return state;
            }
        }

        Self::default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("/var/lib/anna/monitoring_state.json")
    }

    /// Check if health check is due (weekly).
    pub fn is_health_check_due(&self) -> bool {
        match self.last_health_check {
            None => true, // Never run
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed > Duration::days(7)
            }
        }
    }

    /// Check if any scan is due.
    pub fn is_any_scan_due(&self) -> bool {
        self.is_health_check_due()
            || self.is_regression_scan_due()
            || self.is_cleanup_scan_due()
            || self.is_prediction_update_due()
    }

    fn is_regression_scan_due(&self) -> bool {
        match self.last_regression_scan {
            None => true,
            Some(last) => (Utc::now() - last) > Duration::days(7),
        }
    }

    fn is_cleanup_scan_due(&self) -> bool {
        match self.last_cleanup_scan {
            None => true,
            Some(last) => (Utc::now() - last) > Duration::days(7),
        }
    }

    fn is_prediction_update_due(&self) -> bool {
        match self.last_prediction_update {
            None => true,
            Some(last) => (Utc::now() - last) > Duration::days(3), // More frequent
        }
    }

    /// Add an alert.
    pub fn add_alert(&mut self, alert: ProactiveAlert) {
        // Remove expired alerts
        self.pending_alerts.retain(|a| {
            if let Some(expires) = a.expires_at {
                expires > Utc::now()
            } else {
                true
            }
        });

        // Check for duplicates
        if !self.pending_alerts.iter().any(|a| a.id == alert.id) {
            self.pending_alerts.push(alert);
        }
    }

    /// Get critical alerts.
    pub fn get_critical_alerts(&self) -> Vec<&ProactiveAlert> {
        self.pending_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .collect()
    }

    /// Clear an alert.
    pub fn clear_alert(&mut self, alert_id: &str) {
        self.pending_alerts.retain(|a| a.id != alert_id);
    }
}

/// Run proactive health check.
pub async fn run_proactive_health_check() -> Result<Vec<ProactiveAlert>> {
    info!("Running proactive health check...");

    let mut alerts = Vec::new();
    let mut state = MonitoringState::load();

    // 1. Check predictions for critical issues
    if state.is_prediction_update_due() {
        let forecast = crate::predictive_maintenance::generate_health_forecast().await?;

        for prediction in &forecast.predictions {
            if prediction.severity == crate::predictive_maintenance::PredictionSeverity::Critical {
                let days = prediction.days_until.unwrap_or(999.0);

                alerts.push(ProactiveAlert {
                    id: format!("pred-{}", uuid::Uuid::new_v4()),
                    alert_type: AlertType::PredictiveCritical,
                    severity: AlertSeverity::Critical,
                    title: format!("Critical: {}", prediction.prediction),
                    description: format!(
                        "{} (in {:.0} days)\nTrend: {}\nAction: {}",
                        prediction.prediction, days, prediction.trend, prediction.recommendation
                    ),
                    recommendations: vec![prediction.recommendation.clone()],
                    detected_at: Utc::now(),
                    expires_at: Some(Utc::now() + Duration::days(days as i64)),
                });
            }
        }

        state.last_prediction_update = Some(Utc::now());
    }

    // 2. Check for severe regressions
    if state.is_regression_scan_due() {
        let regressions = crate::regression_detector::detect_regressions().await?;

        for regression in &regressions {
            if regression.severity == crate::regression_detector::RegressionSeverity::Severe {
                alerts.push(ProactiveAlert {
                    id: format!("regr-{}", uuid::Uuid::new_v4()),
                    alert_type: AlertType::RegressionSevere,
                    severity: AlertSeverity::High,
                    title: format!("Severe Regression: {}", regression.metric),
                    description: format!(
                        "{} regressed by {:.0}%\n{:.1} -> {:.1}",
                        regression.metric,
                        regression.change_pct,
                        regression.baseline_value,
                        regression.current_value
                    ),
                    recommendations: regression
                        .causes
                        .iter()
                        .filter_map(|c| c.fix.clone())
                        .take(3)
                        .collect(),
                    detected_at: Utc::now(),
                    expires_at: None,
                });
            }
        }

        state.last_regression_scan = Some(Utc::now());
    }

    // 3. Check disk usage (critical >95%)
    let disk_pct = crate::briefing::get_disk_usage_percentage();
    if disk_pct > 95.0 {
        alerts.push(ProactiveAlert {
            id: format!("disk-{}", uuid::Uuid::new_v4()),
            alert_type: AlertType::DiskCritical,
            severity: AlertSeverity::Critical,
            title: "Disk Critical".to_string(),
            description: format!("Disk usage at {:.1}% - system may become unstable", disk_pct),
            recommendations: vec![
                "Run cleanup scan immediately".to_string(),
                "Move or delete large files".to_string(),
                "Clear package cache".to_string(),
            ],
            detected_at: Utc::now(),
            expires_at: None,
        });
    }

    // 4. Check for failed services
    if let Ok(output) = crate::core_loop::execute_command("systemctl --failed --no-legend | wc -l") {
        if let Ok(count) = output.trim().parse::<u32>() {
            if count > 0 {
                alerts.push(ProactiveAlert {
                    id: format!("svc-{}", uuid::Uuid::new_v4()),
                    alert_type: AlertType::ServiceFailure,
                    severity: AlertSeverity::High,
                    title: format!("{} service(s) failed", count),
                    description: format!("{} systemd service(s) in failed state", count),
                    recommendations: vec![
                        "Check systemctl status for details".to_string(),
                        "Review journal logs".to_string(),
                    ],
                    detected_at: Utc::now(),
                    expires_at: None,
                });
            }
        }
    }

    // 5. Run cleanup scan if disk >75%
    if disk_pct > 75.0 && state.is_cleanup_scan_due() {
        let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;

        if cleanup.total_cleanable_mb > 500.0 {
            // Only alert if significant
            alerts.push(ProactiveAlert {
                id: format!("cleanup-{}", uuid::Uuid::new_v4()),
                alert_type: AlertType::DiskCritical,
                severity: if disk_pct > 90.0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Medium
                },
                title: format!("Found {:.1}GB cleanable space", cleanup.total_cleanable_mb / 1024.0),
                description: format!(
                    "Disk at {:.1}%. Found {:.1}GB that can be cleaned.",
                    disk_pct,
                    cleanup.total_cleanable_mb / 1024.0
                ),
                recommendations: cleanup.recommendations.clone(),
                detected_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::days(30)),
            });
        }

        state.last_cleanup_scan = Some(Utc::now());
    }

    // Store alerts
    for alert in &alerts {
        state.add_alert(alert.clone());
    }

    state.last_health_check = Some(Utc::now());
    state.save()?;

    info!("Proactive health check complete: {} alerts", alerts.len());

    Ok(alerts)
}

/// Get summary of pending alerts.
pub fn get_pending_alerts_summary() -> String {
    let state = MonitoringState::load();

    if state.pending_alerts.is_empty() {
        return "No pending alerts. System health looks good.".to_string();
    }

    let mut summary = format!("Pending Alerts ({})\n\n", state.pending_alerts.len());

    let critical = state.get_critical_alerts();
    if !critical.is_empty() {
        summary.push_str("CRITICAL:\n");
        for alert in critical {
            summary.push_str(&format!("• {}\n  {}\n\n", alert.title, alert.description));
        }
    }

    let high: Vec<_> = state
        .pending_alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::High)
        .collect();

    if !high.is_empty() {
        summary.push_str("HIGH PRIORITY:\n");
        for alert in high {
            summary.push_str(&format!("• {}\n  {}\n\n", alert.title, alert.description));
        }
    }

    summary
}

/// Check if proactive monitoring should run (weekly or on demand).
pub fn should_run_proactive_check() -> bool {
    let state = MonitoringState::load();
    state.is_any_scan_due()
}
