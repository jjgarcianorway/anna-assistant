//! Change Tracking - Track system changes and correlate with regressions.
//!
//! Philosophy: Answer "WHY did this happen?" by linking changes to outcomes.
//! NO HARDCODING: Probabilistic correlation, not assumptions.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use tracing::info;

/// Tracks system changes over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeHistory {
    /// Recent changes (last 90 days)
    pub changes: VecDeque<SystemChange>,
}

/// A system change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemChange {
    pub id: String,
    pub change_type: ChangeType,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub details: String,
    /// What was affected (package names, service names, etc.)
    pub affected_components: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    PackageUpdate,
    PackageInstall,
    PackageRemove,
    ConfigChange,
    ServiceRestart,
    KernelUpdate,
    SystemUpdate,
}

impl Default for ChangeHistory {
    fn default() -> Self {
        Self {
            changes: VecDeque::new(),
        }
    }
}

impl ChangeHistory {
    const MAX_CHANGES: usize = 500; // Keep last 500 changes

    /// Load from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(history) = serde_json::from_str(&contents) {
                return history;
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
        PathBuf::from("/var/lib/anna/change_history.json")
    }

    /// Record a change.
    pub fn record_change(&mut self, change: SystemChange) {
        // Remove old changes
        while self.changes.len() >= Self::MAX_CHANGES {
            self.changes.pop_front();
        }

        // Remove changes older than 90 days
        let cutoff = Utc::now() - Duration::days(90);
        self.changes.retain(|c| c.timestamp > cutoff);

        self.changes.push_back(change);

        if let Err(e) = self.save() {
            tracing::warn!("Failed to save change history: {}", e);
        }
    }

    /// Get changes in a time window.
    pub fn get_changes_in_window(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&SystemChange> {
        self.changes
            .iter()
            .filter(|c| c.timestamp >= start && c.timestamp <= end)
            .collect()
    }

    /// Get changes before a specific time.
    pub fn get_changes_before(&self, time: DateTime<Utc>, count: usize) -> Vec<&SystemChange> {
        self.changes
            .iter()
            .rev()
            .filter(|c| c.timestamp <= time)
            .take(count)
            .collect()
    }
}

/// Correlate changes with a regression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCorrelation {
    pub change: SystemChange,
    pub correlation_score: f32, // 0.0-1.0
    pub likelihood_description: String,
    pub reasoning: String,
}

/// Correlate recent changes with a regression.
pub async fn correlate_changes_with_regression(
    regression_metric: &str,
    regression_started: Option<DateTime<Utc>>,
) -> Result<Vec<ChangeCorrelation>> {
    let history = ChangeHistory::load();

    // If we know when regression started, look at changes in that window
    // Otherwise, look at last 14 days
    let start_time = regression_started.unwrap_or_else(|| Utc::now() - Duration::days(14));
    let end_time = regression_started.map(|t| t + Duration::days(1)).unwrap_or_else(Utc::now);

    let changes = history.get_changes_in_window(start_time, end_time);

    if changes.is_empty() {
        return Ok(Vec::new());
    }

    let mut correlations = Vec::new();

    for change in changes {
        let score = calculate_correlation_score(regression_metric, change);

        if score > 0.3 {
            // Only include likely correlations
            let likelihood = if score > 0.8 {
                "Very Likely"
            } else if score > 0.6 {
                "Likely"
            } else if score > 0.4 {
                "Possible"
            } else {
                "Unlikely"
            };

            let reasoning = generate_correlation_reasoning(regression_metric, change);

            correlations.push(ChangeCorrelation {
                change: change.clone(),
                correlation_score: score,
                likelihood_description: likelihood.to_string(),
                reasoning,
            });
        }
    }

    // Sort by correlation score
    correlations.sort_by(|a, b| b.correlation_score.partial_cmp(&a.correlation_score).unwrap());

    Ok(correlations)
}

/// Calculate correlation score between a change and regression.
fn calculate_correlation_score(regression_metric: &str, change: &SystemChange) -> f32 {
    let metric_lower = regression_metric.to_lowercase();
    let change_desc_lower = change.description.to_lowercase();

    let mut score: f32 = 0.0;

    // Boot time regressions
    if metric_lower.contains("boot") {
        if change.change_type == ChangeType::SystemUpdate || change.change_type == ChangeType::KernelUpdate {
            score += 0.7;
        }
        if change.change_type == ChangeType::PackageUpdate {
            // Check if systemd, plymouth, network related
            if change.affected_components.iter().any(|c| {
                c.contains("systemd")
                    || c.contains("plymouth")
                    || c.contains("network")
                    || c.contains("grub")
            }) {
                score += 0.8;
            } else {
                score += 0.3;
            }
        }
    }

    // Memory regressions
    if metric_lower.contains("memory") || metric_lower.contains("ram") {
        if change.change_type == ChangeType::PackageInstall {
            score += 0.5; // New packages can increase memory usage
        }
        if change.change_type == ChangeType::PackageUpdate {
            if change.affected_components.iter().any(|c| {
                c.contains("docker")
                    || c.contains("chrome")
                    || c.contains("firefox")
                    || c.contains("electron")
            }) {
                score += 0.7;
            } else {
                score += 0.3;
            }
        }
    }

    // CPU/Performance regressions
    if metric_lower.contains("cpu") || metric_lower.contains("performance") {
        if change.change_type == ChangeType::PackageUpdate {
            if change.affected_components.iter().any(|c| c.contains("baloo") || c.contains("tracker")) {
                score += 0.8; // Known performance impacters
            } else {
                score += 0.3;
            }
        }
    }

    // Disk I/O regressions
    if metric_lower.contains("disk") {
        if change.change_type == ChangeType::PackageInstall {
            if change.affected_components.iter().any(|c| c.contains("docker") || c.contains("database")) {
                score += 0.6;
            }
        }
    }

    // Time proximity bonus
    // If change happened very recently, increase score slightly
    let time_diff = (Utc::now() - change.timestamp).num_hours();
    if time_diff < 24 {
        score += 0.1;
    } else if time_diff < 72 {
        score += 0.05;
    }

    score.min(1.0_f32)
}

/// Generate reasoning for correlation.
fn generate_correlation_reasoning(regression_metric: &str, change: &SystemChange) -> String {
    let metric_lower = regression_metric.to_lowercase();

    let mut reasoning = format!("{:?} at {}", change.change_type, change.timestamp.format("%Y-%m-%d %H:%M"));

    if metric_lower.contains("boot") {
        reasoning.push_str(". Boot-related changes often affect startup time.");
        if change.affected_components.iter().any(|c| c.contains("systemd")) {
            reasoning.push_str(" Systemd changes directly impact boot sequence.");
        }
    } else if metric_lower.contains("memory") {
        reasoning.push_str(". New packages or updates can increase memory usage.");
        if change.change_type == ChangeType::PackageInstall {
            reasoning.push_str(" Package installations add new processes.");
        }
    } else if metric_lower.contains("cpu") || metric_lower.contains("performance") {
        reasoning.push_str(". Package changes can affect system performance.");
    }

    // Time-based reasoning
    let hours_ago = (Utc::now() - change.timestamp).num_hours();
    if hours_ago < 24 {
        reasoning.push_str(&format!(" (just {} hours ago)", hours_ago));
    } else {
        let days_ago = (Utc::now() - change.timestamp).num_days();
        reasoning.push_str(&format!(" ({} days ago)", days_ago));
    }

    reasoning
}

/// Scan pacman log for recent changes and record them.
pub async fn scan_and_record_recent_changes() -> Result<usize> {
    info!("Scanning for recent system changes...");

    let mut history = ChangeHistory::load();
    let mut new_changes = 0;

    // Get recent pacman log entries (last 7 days)
    let cutoff = Utc::now() - Duration::days(7);

    if let Ok(output) = crate::core_loop::execute_command("grep -E 'installed|upgraded|removed' /var/log/pacman.log | tail -100") {
        for line in output.lines() {
            if let Some(change) = parse_pacman_log_line(line) {
                // Only record if newer than cutoff and not already recorded
                if change.timestamp > cutoff
                    && !history
                        .changes
                        .iter()
                        .any(|c| c.id == change.id && c.timestamp == change.timestamp)
                {
                    history.record_change(change);
                    new_changes += 1;
                }
            }
        }
    }

    if new_changes > 0 {
        info!("Recorded {} new changes", new_changes);
        history.save()?;
    }

    Ok(new_changes)
}

/// Parse a pacman log line into a SystemChange.
fn parse_pacman_log_line(line: &str) -> Option<SystemChange> {
    // Format: [2024-02-12T10:30:45+0100] [ALPM] installed package-name (version)
    let parts: Vec<&str> = line.split(']').collect();
    if parts.len() < 3 {
        return None;
    }

    // Parse timestamp
    let timestamp_str = parts[0].trim_start_matches('[');
    let timestamp = chrono::DateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%z")
        .ok()?
        .with_timezone(&Utc);

    // Parse action and package
    let action_part = parts[2].trim();
    let action_words: Vec<&str> = action_part.split_whitespace().collect();

    if action_words.len() < 2 {
        return None;
    }

    let action = action_words[0];
    let package = action_words[1].to_string();

    let change_type = match action {
        "installed" => ChangeType::PackageInstall,
        "upgraded" => ChangeType::PackageUpdate,
        "removed" => ChangeType::PackageRemove,
        _ => return None,
    };

    Some(SystemChange {
        id: format!("{}-{}-{}", change_type as u8, package, timestamp.timestamp()),
        change_type,
        timestamp,
        description: format!("{} {}", action, package),
        details: line.to_string(),
        affected_components: vec![package],
    })
}

/// Format change correlations for display.
pub fn format_change_correlations(correlations: &[ChangeCorrelation]) -> String {
    if correlations.is_empty() {
        return "No recent system changes correlated with this issue.".to_string();
    }

    let mut response = format!("Possible Causes (Recent Changes):\n\n");

    for (i, corr) in correlations.iter().take(5).enumerate() {
        response.push_str(&format!(
            "{}. [{}] {}\n",
            i + 1,
            corr.likelihood_description,
            corr.change.description
        ));
        response.push_str(&format!("   {}\n", corr.reasoning));

        if !corr.change.affected_components.is_empty() {
            response.push_str(&format!(
                "   Affected: {}\n",
                corr.change.affected_components.join(", ")
            ));
        }

        response.push('\n');
    }

    response
}
