//! Auto-fix execution handlers and history tracking.

use super::AutoFix;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Record of an executed fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixRecord {
    /// Fix ID (e.g., "pacman_cache")
    pub fix_id: String,
    /// Description of what was fixed
    pub description: String,
    /// Command that was executed
    pub command: String,
    /// When the fix was executed
    pub executed_at: DateTime<Utc>,
    /// Whether the fix succeeded
    pub success: bool,
    /// Output from the fix command
    pub output: String,
}

/// Fix history storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FixHistory {
    pub records: Vec<FixRecord>,
}

impl FixHistory {
    fn history_path() -> PathBuf {
        anna_shared::paths::paths().fix_history_file()
    }

    pub fn load() -> Self {
        let path = Self::history_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::history_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }

    pub fn add_record(&mut self, record: FixRecord) {
        self.records.push(record);
        if self.records.len() > 100 {
            self.records.remove(0);
        }
        if let Err(e) = self.save() {
            warn!("Failed to save fix history: {}", e);
        }
    }

    pub fn recent(&self, count: usize) -> Vec<&FixRecord> {
        self.records.iter().rev().take(count).collect()
    }

    pub fn for_fix(&self, fix_id: &str) -> Vec<&FixRecord> {
        self.records.iter().filter(|r| r.fix_id == fix_id).collect()
    }
}

/// Record a fix execution to history
pub fn record_fix(fix: &AutoFix, success: bool, output: &str) {
    let record = FixRecord {
        fix_id: fix.id.to_string(),
        description: fix.description.to_string(),
        command: fix.fix_cmd.to_string(),
        executed_at: Utc::now(),
        success,
        output: output.to_string(),
    };
    let mut history = FixHistory::load();
    history.add_record(record);
    info!("Recorded fix {} to history (success={})", fix.id, success);
}

/// Get fix history summary for display
pub fn get_fix_history_summary() -> String {
    let history = FixHistory::load();
    if history.records.is_empty() {
        return "No fixes have been executed yet.".to_string();
    }
    let recent = history.recent(5);
    let mut summary = String::from("Recent fixes:\n");
    for record in recent {
        let status = if record.success { "OK" } else { "FAILED" };
        let time = record.executed_at.format("%Y-%m-%d %H:%M");
        summary.push_str(&format!("  [{}] {} - {} ({})\n",
            status, record.fix_id, record.description, time));
    }
    summary
}

/// Check if an auto-fix is needed (run the check command)
pub fn check_autofix_needed(fix: &AutoFix) -> bool {
    match Command::new("sh").arg("-c").arg(fix.check_cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = if fix.check_condition.is_empty() {
                let trimmed = stdout.trim();
                !trimmed.is_empty() && trimmed != "0"
            } else {
                stdout.contains(fix.check_condition)
            };
            debug!("AutoFix check {}: output='{}', needed={}", fix.id, stdout.trim(), result);
            result
        }
        Err(e) => {
            debug!("AutoFix check {} failed: {}", fix.id, e);
            false
        }
    }
}

/// Execute an auto-fix (called after user confirmation)
pub fn execute_autofix(fix: &AutoFix) -> Result<String, String> {
    info!("Executing autofix: {}", fix.id);
    match Command::new("sh").arg("-c").arg(fix.fix_cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                info!("AutoFix {} succeeded", fix.id);
                record_fix(fix, true, stdout.trim());
                Ok(format!("Done! {}", stdout.trim()))
            } else {
                record_fix(fix, false, stderr.trim());
                Err(format!("Fix failed: {}", stderr.trim()))
            }
        }
        Err(e) => {
            record_fix(fix, false, &e.to_string());
            Err(format!("Failed to run fix: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_find_autofix_pacman_lock() {
        let fix = find_autofix("pacman says database is locked");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "pacman_db_lock");
    }

    #[test]
    fn test_find_autofix_failed_services() {
        let fix = find_autofix("show me failed services");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "failed_services");
    }

    #[test]
    fn test_find_autofix_orphan() {
        let fix = find_autofix("remove orphan packages");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "orphan_packages");
    }

    #[test]
    fn test_no_false_positive() {
        let fix = find_autofix("what is pacman");
        assert!(fix.is_none());
    }

    #[test]
    fn test_no_autofix_for_info_questions() {
        assert!(find_autofix("what's using my disk space").is_none());
        assert!(find_autofix("how much disk space do I have").is_none());
        assert!(find_autofix("show me disk usage").is_none());
    }

    #[test]
    fn test_autofix_for_action_requests() {
        assert!(find_autofix("my disk is full, please fix it").is_some());
        assert!(find_autofix("clean up my disk space").is_some());
    }

    #[test]
    fn test_find_autofix_wifi() {
        let fix = find_autofix("my wifi won't connect to the network");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "wifi_restart");
    }

    #[test]
    fn test_find_autofix_audio() {
        let fix = find_autofix("no sound from speakers");
        assert!(fix.is_some());
    }

    #[test]
    fn test_find_autofix_bluetooth() {
        let fix = find_autofix("bluetooth device won't pair");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "bluetooth_restart");
    }

    #[test]
    fn test_find_autofix_docker() {
        let fix = find_autofix("docker daemon socket error");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "docker_restart");
    }

    #[test]
    fn test_find_autofix_time() {
        let fix = find_autofix("system clock time is wrong");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "timesyncd_restart");
    }

    #[test]
    fn test_find_autofix_swap() {
        let fix = find_autofix("system is slow and swapping memory");
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().id, "swap_clear");
    }

    #[test]
    fn test_autofix_count() {
        assert_eq!(AUTO_FIXES.len(), 17, "Expected 17 auto-fixes");
    }
}
