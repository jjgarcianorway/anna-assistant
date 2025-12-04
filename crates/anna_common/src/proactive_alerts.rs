//! Proactive Alerts v0.0.58 - High-Signal Issue Detection
//!
//! Anna notices problems without being asked, but in a disciplined, low-noise way:
//! - Detect a small set of high-signal issues
//! - Create alert objects with evidence [E#]
//! - Show in status
//! - Allow "why are you warning me?" queries to route to evidence
//!
//! No auto-fixing. Only detection + reporting.
//!
//! Alert Types (v0.0.58):
//! 1. BOOT_REGRESSION - boot time vs rolling baseline
//! 2. DISK_PRESSURE - / free < 10% or < 15 GiB
//! 3. JOURNAL_ERROR_BURST - unit has >= 20 errors in 10 min
//! 4. SERVICE_FAILED - any systemd unit in failed state
//! 5. THERMAL_THROTTLING - CPU near throttle (best-effort)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::atomic_write::atomic_write_bytes;

// ============================================================================
// Alert Schema v1
// ============================================================================

/// Schema version for alerts.json
pub const PROACTIVE_ALERTS_SCHEMA: u32 = 1;

/// Path to alerts.json (daemon-owned)
pub const PROACTIVE_ALERTS_FILE: &str = "/var/lib/anna/internal/alerts.json";

/// Alert type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertType {
    /// Boot time increased vs rolling baseline
    BootRegression,
    /// Root filesystem low on space
    DiskPressure,
    /// Burst of journal errors from a unit
    JournalErrorBurst,
    /// Systemd unit in failed state
    ServiceFailed,
    /// CPU thermal throttling detected
    ThermalThrottling,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertType::BootRegression => "BOOT_REGRESSION",
            AlertType::DiskPressure => "DISK_PRESSURE",
            AlertType::JournalErrorBurst => "JOURNAL_ERROR_BURST",
            AlertType::ServiceFailed => "SERVICE_FAILED",
            AlertType::ThermalThrottling => "THERMAL_THROTTLING",
        }
    }
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is currently active
    Active,
    /// Condition cleared, alert resolved
    Resolved,
}

/// A proactive alert with full evidence trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveAlert {
    /// Stable ID: hash of type + dedupe_key
    pub id: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Short title (e.g., "Boot time regression")
    pub title: String,
    /// 1-2 line summary
    pub summary: String,
    /// When first detected
    pub first_seen: DateTime<Utc>,
    /// When last refreshed/confirmed
    pub last_seen: DateTime<Utc>,
    /// How many times detected
    pub occurrences: u32,
    /// Evidence IDs from most recent refresh
    pub evidence_ids: Vec<String>,
    /// Optional cooldown until (rate limiting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown_until: Option<DateTime<Utc>>,
    /// Deduplication key (type-specific)
    pub dedupe_key: String,
    /// Current status
    pub status: AlertStatus,
    /// Structured data for this alert type
    #[serde(default)]
    pub data: serde_json::Value,
}

impl ProactiveAlert {
    /// Generate stable ID from type and dedupe_key
    pub fn generate_id(alert_type: AlertType, dedupe_key: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        alert_type.as_str().hash(&mut hasher);
        dedupe_key.hash(&mut hasher);
        format!("alert-{:016x}", hasher.finish())
    }

    /// Create a new alert
    pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        dedupe_key: &str,
        title: &str,
        summary: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(alert_type, dedupe_key),
            alert_type,
            severity,
            title: title.to_string(),
            summary: summary.to_string(),
            first_seen: now,
            last_seen: now,
            occurrences: 1,
            evidence_ids: Vec::new(),
            cooldown_until: None,
            dedupe_key: dedupe_key.to_string(),
            status: AlertStatus::Active,
            data: serde_json::Value::Null,
        }
    }

    /// Add evidence ID
    pub fn with_evidence(mut self, evidence_id: &str) -> Self {
        self.evidence_ids.push(evidence_id.to_string());
        self
    }

    /// Set evidence IDs (replaces)
    pub fn with_evidence_ids(mut self, ids: Vec<String>) -> Self {
        self.evidence_ids = ids;
        self
    }

    /// Set structured data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Format age since first seen
    pub fn age_str(&self) -> String {
        let duration = Utc::now() - self.first_seen;
        if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}m", duration.num_minutes().max(1))
        }
    }

    /// Format for status display (one line)
    pub fn format_short(&self) -> String {
        format!("{} ({}, {})", self.title, self.severity, self.age_str())
    }

    /// Format for detailed display
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] {}", self.id, self.title),
            format!("  Type: {}", self.alert_type),
            format!("  Severity: {}", self.severity),
            format!("  {}", self.summary),
            format!("  First seen: {}", self.first_seen.format("%Y-%m-%d %H:%M")),
            format!("  Occurrences: {}", self.occurrences),
        ];

        if !self.evidence_ids.is_empty() {
            lines.push(format!("  Evidence: [{}]", self.evidence_ids.join(", ")));
        }

        lines.join("\n")
    }
}

// ============================================================================
// Alerts State (daemon-owned)
// ============================================================================

/// Alerts state stored in alerts.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveAlertsState {
    /// Schema version
    pub schema_version: u32,
    /// When last checked
    pub last_check: DateTime<Utc>,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Active alerts (keyed by ID for fast lookup)
    pub alerts: HashMap<String, ProactiveAlert>,
    /// Recently resolved (for "what changed" queries)
    #[serde(default)]
    pub recently_resolved: Vec<ProactiveAlert>,
}

impl Default for ProactiveAlertsState {
    fn default() -> Self {
        Self {
            schema_version: PROACTIVE_ALERTS_SCHEMA,
            last_check: Utc::now(),
            check_interval_secs: 60,
            alerts: HashMap::new(),
            recently_resolved: Vec::new(),
        }
    }
}

impl ProactiveAlertsState {
    /// Load from disk
    pub fn load() -> Self {
        let path = Path::new(PROACTIVE_ALERTS_FILE);
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    /// Save to disk atomically
    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(PROACTIVE_ALERTS_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write_bytes(PROACTIVE_ALERTS_FILE, content.as_bytes())
    }

    /// Add or update an alert
    pub fn upsert_alert(&mut self, alert: ProactiveAlert) {
        if let Some(existing) = self.alerts.get_mut(&alert.id) {
            // Update existing
            existing.last_seen = Utc::now();
            existing.occurrences += 1;
            existing.evidence_ids = alert.evidence_ids;
            existing.summary = alert.summary;
            existing.data = alert.data;
            // Escalate severity if needed
            if alert.severity > existing.severity {
                existing.severity = alert.severity;
            }
            // Reactivate if was resolved
            existing.status = AlertStatus::Active;
        } else {
            // Add new
            self.alerts.insert(alert.id.clone(), alert);
        }
    }

    /// Mark an alert as resolved (move to recently_resolved)
    pub fn resolve_alert(&mut self, id: &str) {
        if let Some(mut alert) = self.alerts.remove(id) {
            alert.status = AlertStatus::Resolved;
            // Keep only last 10 resolved
            self.recently_resolved.push(alert);
            if self.recently_resolved.len() > 10 {
                self.recently_resolved.remove(0);
            }
        }
    }

    /// Get active alerts sorted by severity (critical first)
    pub fn get_active(&self) -> Vec<&ProactiveAlert> {
        let mut alerts: Vec<_> = self
            .alerts
            .values()
            .filter(|a| a.status == AlertStatus::Active)
            .collect();
        alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
        alerts
    }

    /// Get counts by severity
    pub fn count_by_severity(&self) -> AlertCounts {
        let active = self.get_active();
        AlertCounts {
            critical: active
                .iter()
                .filter(|a| a.severity == AlertSeverity::Critical)
                .count(),
            warning: active
                .iter()
                .filter(|a| a.severity == AlertSeverity::Warning)
                .count(),
            info: active
                .iter()
                .filter(|a| a.severity == AlertSeverity::Info)
                .count(),
        }
    }

    /// Get top N alerts for status display
    pub fn get_top_alerts(&self, n: usize) -> Vec<&ProactiveAlert> {
        self.get_active().into_iter().take(n).collect()
    }

    /// Check if an alert exists and is active
    pub fn has_active_alert(&self, alert_type: AlertType, dedupe_key: &str) -> bool {
        let id = ProactiveAlert::generate_id(alert_type, dedupe_key);
        self.alerts
            .get(&id)
            .map(|a| a.status == AlertStatus::Active)
            .unwrap_or(false)
    }

    /// Cleanup old resolved alerts (> 7 days)
    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(7);
        self.recently_resolved.retain(|a| a.last_seen > cutoff);
    }

    /// Get age of alerts snapshot
    pub fn snapshot_age_str(&self) -> String {
        let duration = Utc::now() - self.last_check;
        if duration.num_minutes() < 1 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else {
            format!("{}h ago", duration.num_hours())
        }
    }
}

/// Alert counts for status display
#[derive(Debug, Clone, Default)]
pub struct AlertCounts {
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
}

impl AlertCounts {
    pub fn total(&self) -> usize {
        self.critical + self.warning + self.info
    }

    pub fn format(&self) -> String {
        if self.total() == 0 {
            "No active alerts".to_string()
        } else {
            format!(
                "Critical: {}, Warnings: {}, Info: {}",
                self.critical, self.warning, self.info
            )
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_id_stable() {
        let id1 = ProactiveAlert::generate_id(AlertType::DiskPressure, "/");
        let id2 = ProactiveAlert::generate_id(AlertType::DiskPressure, "/");
        assert_eq!(id1, id2);

        let id3 = ProactiveAlert::generate_id(AlertType::DiskPressure, "/home");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_alert_upsert() {
        let mut state = ProactiveAlertsState::default();

        let alert1 = ProactiveAlert::new(
            AlertType::DiskPressure,
            AlertSeverity::Warning,
            "/",
            "Low disk space",
            "Root filesystem is 92% full",
        );
        state.upsert_alert(alert1);

        assert_eq!(state.alerts.len(), 1);

        // Upsert again
        let alert2 = ProactiveAlert::new(
            AlertType::DiskPressure,
            AlertSeverity::Critical,
            "/",
            "Low disk space",
            "Root filesystem is 97% full",
        );
        state.upsert_alert(alert2);

        // Should still be 1 alert, but updated
        assert_eq!(state.alerts.len(), 1);
        let alert = state.alerts.values().next().unwrap();
        assert_eq!(alert.occurrences, 2);
        assert_eq!(alert.severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_alert_resolve() {
        let mut state = ProactiveAlertsState::default();

        let alert = ProactiveAlert::new(
            AlertType::ServiceFailed,
            AlertSeverity::Critical,
            "nginx.service",
            "Service failed",
            "nginx.service is in failed state",
        );
        let id = alert.id.clone();
        state.upsert_alert(alert);

        assert_eq!(state.get_active().len(), 1);

        state.resolve_alert(&id);

        assert_eq!(state.get_active().len(), 0);
        assert_eq!(state.recently_resolved.len(), 1);
    }

    #[test]
    fn test_alert_counts() {
        let mut state = ProactiveAlertsState::default();

        state.upsert_alert(ProactiveAlert::new(
            AlertType::ServiceFailed,
            AlertSeverity::Critical,
            "a",
            "A",
            "A",
        ));
        state.upsert_alert(ProactiveAlert::new(
            AlertType::DiskPressure,
            AlertSeverity::Warning,
            "b",
            "B",
            "B",
        ));
        state.upsert_alert(ProactiveAlert::new(
            AlertType::BootRegression,
            AlertSeverity::Warning,
            "c",
            "C",
            "C",
        ));

        let counts = state.count_by_severity();
        assert_eq!(counts.critical, 1);
        assert_eq!(counts.warning, 2);
        assert_eq!(counts.info, 0);
    }

    #[test]
    fn test_top_alerts() {
        let mut state = ProactiveAlertsState::default();

        state.upsert_alert(ProactiveAlert::new(
            AlertType::DiskPressure,
            AlertSeverity::Warning,
            "a",
            "A",
            "A",
        ));
        state.upsert_alert(ProactiveAlert::new(
            AlertType::ServiceFailed,
            AlertSeverity::Critical,
            "b",
            "B",
            "B",
        ));
        state.upsert_alert(ProactiveAlert::new(
            AlertType::BootRegression,
            AlertSeverity::Info,
            "c",
            "C",
            "C",
        ));

        let top = state.get_top_alerts(2);
        assert_eq!(top.len(), 2);
        // Critical should be first
        assert_eq!(top[0].severity, AlertSeverity::Critical);
    }
}
