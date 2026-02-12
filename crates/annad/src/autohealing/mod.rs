//! Auto-Healing System - Anna fixes common issues automatically.
//!
//! Safe fixes run automatically during learning cycles.
//! Risky fixes require user confirmation via suggestions.

mod types;
mod disk;
mod system;
mod services;

pub use types::{HealingAction, HealingLog, HealingResult};

use anyhow::Result;
use tracing::{debug, info};

/// Run safe automatic healing checks.
pub async fn run_safe_healing() -> Result<Vec<String>> {
    debug!("Running safe auto-healing checks");

    let mut healing_log = HealingLog::load();
    let mut healed = Vec::new();

    // Safe fix 1: Clean pacman cache if >5GB
    if let Some(result) = disk::clean_pacman_cache_if_large(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 2: Remove orphaned packages (if >20)
    if let Some(result) = disk::remove_orphans_if_many(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 3: Clean systemd journal if >1GB
    if let Some(result) = system::clean_journal_if_large(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 4: Fix common permission issues
    if let Some(result) = system::fix_common_permissions(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 5: Clean tmp directories if large (>2GB)
    if let Some(result) = system::clean_tmp_if_large(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 6: Clear systemd failed unit states (after verifying they're not currently failed)
    if let Some(result) = services::clear_failed_unit_states(&mut healing_log).await? {
        healed.push(result);
    }

    // Safe fix 7: Restart persistently failing services (with backoff)
    if let Some(result) = services::restart_failed_services(&mut healing_log).await? {
        healed.push(result);
    }

    healing_log.save()?;

    if !healed.is_empty() {
        info!("Auto-healing: Fixed {} issues", healed.len());
    }

    Ok(healed)
}

/// Get healing history summary.
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
