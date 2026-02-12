//! Auto-Healing System - Anna fixes common issues automatically
//!
//! Safe fixes run automatically during learning cycles.
//! Risky fixes require user confirmation via suggestions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn, debug};

const HEALING_LOG: &str = "/var/lib/anna/healing.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingAction {
    pub timestamp: String,
    pub issue: String,
    pub action: String,
    pub result: HealingResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingResult {
    Success(String),
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingLog {
    pub actions: Vec<HealingAction>,
}

impl Default for HealingLog {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

impl HealingLog {
    /// Load healing log
    pub fn load() -> Self {
        let path = PathBuf::from(HEALING_LOG);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save healing log
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(HEALING_LOG);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record a healing action
    pub fn record(&mut self, issue: String, action: String, result: HealingResult) {
        self.actions.push(HealingAction {
            timestamp: chrono::Utc::now().to_rfc3339(),
            issue,
            action,
            result,
        });

        // Keep last 100 actions
        if self.actions.len() > 100 {
            self.actions.drain(0..self.actions.len() - 100);
        }
    }
}

/// Run safe automatic healing checks
pub async fn run_safe_healing() -> Result<Vec<String>> {
    debug!("Running safe auto-healing checks");

    let mut healing_log = HealingLog::load();
    let mut healed = Vec::new();

    // Safe fix 1: Clean pacman cache if >5GB
    if let Some(result) = clean_pacman_cache_if_large(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 2: Remove orphaned packages (if >20)
    if let Some(result) = remove_orphans_if_many(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 3: Clean systemd journal if >1GB
    if let Some(result) = clean_journal_if_large(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 4: Fix common permission issues
    if let Some(result) = fix_common_permissions(&mut healing_log).await? {
        healed.push(result);
    }

    healing_log.save()?;

    if !healed.is_empty() {
        info!("Auto-healing: Fixed {} issues", healed.len());
    }

    Ok(healed)
}

/// Clean pacman cache if >5GB
async fn clean_pacman_cache_if_large(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("du")
        .args(["-sb", "/var/cache/pacman/pkg"])
        .output()?;

    let size_str = String::from_utf8_lossy(&output.stdout);
    let size_bytes: u64 = size_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    if size_gb > 5.0 {
        info!("Pacman cache is {:.1}GB, cleaning automatically", size_gb);

        // Keep last 3 versions with paccache
        let result = std::process::Command::new("paccache")
            .args(["-rk3"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let message = format!("Cleaned pacman cache ({:.1}GB → kept last 3 versions)", size_gb);
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Success(message.clone()),
                );
                Ok(Some(message))
            }
            Ok(_) => {
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Failed("paccache command failed".to_string()),
                );
                Ok(None)
            }
            Err(e) => {
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Failed(format!("paccache not available: {}", e)),
                );
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// Remove orphaned packages if >20
async fn remove_orphans_if_many(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let orphans = String::from_utf8_lossy(&output.stdout);
    let count = orphans.lines().filter(|l| !l.is_empty()).count();

    if count > 20 {
        info!("Found {} orphaned packages, removing automatically", count);

        // Remove orphans - this is safe, they're not dependencies anymore
        let result = std::process::Command::new("sh")
            .args(["-c", "pacman -Rns --noconfirm $(pacman -Qdtq)"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let message = format!("Removed {} orphaned packages automatically", count);
                log.record(
                    format!("{} orphaned packages", count),
                    "pacman -Rns (orphans)".to_string(),
                    HealingResult::Success(message.clone()),
                );
                Ok(Some(message))
            }
            _ => {
                log.record(
                    format!("{} orphaned packages", count),
                    "pacman -Rns (orphans)".to_string(),
                    HealingResult::Failed("Could not remove orphans".to_string()),
                );
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// Clean systemd journal if >1GB
async fn clean_journal_if_large(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("journalctl")
        .args(["--disk-usage"])
        .output()?;

    let usage_str = String::from_utf8_lossy(&output.stdout);

    // Parse "Archived and active journals take up 1.2G in the file system."
    let size_gb: Option<f32> = usage_str
        .split_whitespace()
        .find(|s| s.ends_with('G'))
        .and_then(|s| s.trim_end_matches('G').parse().ok());

    if let Some(size) = size_gb {
        if size > 1.0 {
            info!("Journal is {:.1}GB, cleaning to last 30 days", size);

            let result = std::process::Command::new("journalctl")
                .args(["--vacuum-time=30d"])
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    let message = format!("Cleaned systemd journal ({:.1}GB → kept last 30 days)", size);
                    log.record(
                        format!("Journal {:.1}GB", size),
                        "journalctl --vacuum-time=30d".to_string(),
                        HealingResult::Success(message.clone()),
                    );
                    Ok(Some(message))
                }
                _ => {
                    log.record(
                        format!("Journal {:.1}GB", size),
                        "journalctl --vacuum-time=30d".to_string(),
                        HealingResult::Failed("Failed to vacuum journal".to_string()),
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Fix common permission issues
async fn fix_common_permissions(log: &mut HealingLog) -> Result<Option<String>> {
    // Check if user is in anna group (common issue after install)
    let username = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

    let output = std::process::Command::new("groups")
        .arg(&username)
        .output()?;

    let groups = String::from_utf8_lossy(&output.stdout);

    if !groups.contains("anna") && username != "root" {
        debug!("User {} not in anna group, suggesting manual fix", username);
        // Don't auto-fix this - requires pkexec/sudo
        // This will be caught by suggestions system
        Ok(None)
    } else {
        Ok(None)
    }
}

/// Get healing history summary
pub fn get_healing_summary() -> String {
    let log = HealingLog::load();

    if log.actions.is_empty() {
        return "No auto-healing actions performed yet.".to_string();
    }

    let recent_actions = log.actions.iter().rev().take(5);

    let mut summary = String::from("Recent auto-healing actions:\n");
    for action in recent_actions {
        match &action.result {
            HealingResult::Success(msg) => {
                summary.push_str(&format!("  ✓ {} - {}\n", action.issue, msg));
            }
            HealingResult::Failed(msg) => {
                summary.push_str(&format!("  ✗ {} - {}\n", action.issue, msg));
            }
            HealingResult::Skipped(msg) => {
                summary.push_str(&format!("  ○ {} - {}\n", action.issue, msg));
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healing_log() {
        let mut log = HealingLog::default();

        log.record(
            "Test issue".to_string(),
            "Test action".to_string(),
            HealingResult::Success("Fixed".to_string()),
        );

        assert_eq!(log.actions.len(), 1);
        assert_eq!(log.actions[0].issue, "Test issue");
    }
}
