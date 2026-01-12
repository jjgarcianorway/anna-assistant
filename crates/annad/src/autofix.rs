//! Auto-fix module - Anna offers to fix known issues automatically.
//! v0.0.993: Initial implementation
//! v0.0.994: Added pending autofix tracking and yes/no handling
//! v0.0.996: Added fix history tracking for audit/rollback
//!
//! When Anna detects a well-known problem, she can offer to fix it
//! with user confirmation.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Track pending autofixes by session ID
static PENDING_FIXES: RwLock<Option<HashMap<String, &'static str>>> = RwLock::new(None);

/// Set a pending autofix for a session
pub fn set_pending_autofix(session_id: &str, fix_id: &'static str) {
    if let Ok(mut guard) = PENDING_FIXES.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(session_id.to_string(), fix_id);
        debug!("Set pending autofix {} for session {}", fix_id, session_id);
    }
}

/// Get and clear pending autofix for a session
pub fn take_pending_autofix(session_id: &str) -> Option<&'static AutoFix> {
    if let Ok(mut guard) = PENDING_FIXES.write() {
        if let Some(map) = guard.as_mut() {
            if let Some(fix_id) = map.remove(session_id) {
                return AUTO_FIXES.iter().find(|f| f.id == fix_id);
            }
        }
    }
    None
}

/// Check if question is a "yes" confirmation
pub fn is_yes_response(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    matches!(q.as_str(), "yes" | "y" | "yeah" | "yep" | "sure" | "ok" | "do it" | "fix it" | "go ahead")
}

/// Check if question is a "no" rejection
pub fn is_no_response(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    matches!(q.as_str(), "no" | "n" | "nope" | "cancel" | "nevermind" | "never mind" | "don't" | "dont")
}

// ============================================================================
// v0.0.996: Fix History Tracking
// ============================================================================

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
    /// Get the path to the fix history file
    fn history_path() -> PathBuf {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("anna");
        fs::create_dir_all(&data_dir).ok();
        data_dir.join("fix_history.json")
    }

    /// Load fix history from disk
    pub fn load() -> Self {
        let path = Self::history_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save fix history to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::history_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }

    /// Add a fix record
    pub fn add_record(&mut self, record: FixRecord) {
        self.records.push(record);
        // Keep only last 100 records
        if self.records.len() > 100 {
            self.records.remove(0);
        }
        if let Err(e) = self.save() {
            warn!("Failed to save fix history: {}", e);
        }
    }

    /// Get recent fixes (last N)
    pub fn recent(&self, count: usize) -> Vec<&FixRecord> {
        self.records.iter().rev().take(count).collect()
    }

    /// Get fixes for a specific fix_id
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

/// A known problem that Anna can fix automatically
#[derive(Debug, Clone)]
pub struct AutoFix {
    /// Problem identifier
    pub id: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Keywords that trigger this fix
    pub triggers: &'static [&'static str],
    /// Command to check if the problem exists
    pub check_cmd: &'static str,
    /// Condition on check output (contains this string = problem exists)
    pub check_condition: &'static str,
    /// Command to fix the problem (requires sudo confirmation)
    pub fix_cmd: &'static str,
    /// What to tell the user
    pub explanation: &'static str,
}

/// All known auto-fixes
pub const AUTO_FIXES: &[AutoFix] = &[
    AutoFix {
        id: "pacman_db_lock",
        description: "Pacman database lock file",
        triggers: &["database", "locked", "db.lck", "pacman", "lock"],
        check_cmd: "ls /var/lib/pacman/db.lck 2>/dev/null",
        check_condition: "db.lck",
        fix_cmd: "sudo rm /var/lib/pacman/db.lck",
        explanation: "There's a stale lock file blocking pacman. This usually happens when a previous update was interrupted. I can remove it for you.",
    },
    AutoFix {
        id: "pacman_keyring",
        description: "Pacman keyring issue",
        triggers: &["gpgme", "keyring", "signature", "key", "trust"],
        check_cmd: "pacman-key --list-keys 2>&1 | head -1",
        check_condition: "error",
        fix_cmd: "sudo pacman-key --init && sudo pacman-key --populate archlinux",
        explanation: "The pacman keyring needs to be reinitialized. This happens after major updates or keyring corruption.",
    },
    AutoFix {
        id: "failed_services",
        description: "Failed systemd services",
        triggers: &["failed", "service", "systemctl"],
        check_cmd: "systemctl --failed --no-pager | grep -c failed",
        check_condition: "", // Any non-zero number
        fix_cmd: "systemctl reset-failed",
        explanation: "Some services have failed. I can reset the failed state so systemd will try starting them again.",
    },
    AutoFix {
        id: "pacman_cache",
        description: "Large pacman cache",
        triggers: &["cache", "clean", "disk", "space", "pacman"],
        check_cmd: "du -sh /var/cache/pacman/pkg 2>/dev/null | cut -f1",
        check_condition: "G", // Contains G for gigabytes
        fix_cmd: "sudo paccache -rk2",
        explanation: "The pacman cache is taking up significant space. I can clean it up, keeping only the last 2 versions of each package.",
    },
    AutoFix {
        id: "orphan_packages",
        description: "Orphan packages",
        triggers: &["orphan", "unused", "packages", "remove"],
        check_cmd: "pacman -Qtdq 2>/dev/null | wc -l",
        check_condition: "", // Any non-zero number
        fix_cmd: "sudo pacman -Rns $(pacman -Qtdq) 2>/dev/null || echo 'No orphans found'",
        explanation: "There are orphan packages (installed as dependencies but no longer needed). I can remove them to free up space.",
    },
    AutoFix {
        id: "journal_vacuum",
        description: "Large journal logs",
        triggers: &["journal", "logs", "disk", "space", "vacuum"],
        check_cmd: "journalctl --disk-usage 2>/dev/null | grep -oP '\\d+\\.\\d+G' || echo '0'",
        check_condition: "G", // Contains G for gigabytes
        fix_cmd: "sudo journalctl --vacuum-size=500M",
        explanation: "System journals are taking up space. I can clean up old logs, keeping only the most recent 500MB.",
    },
    // v0.0.995: Network & connectivity fixes
    AutoFix {
        id: "wifi_restart",
        description: "WiFi connection issues",
        triggers: &["wifi", "wireless", "connect", "network", "wlan"],
        check_cmd: "nmcli networking connectivity 2>/dev/null || echo 'none'",
        check_condition: "none",
        fix_cmd: "sudo systemctl restart NetworkManager",
        explanation: "NetworkManager seems to be having issues. Restarting it often fixes WiFi connection problems.",
    },
    AutoFix {
        id: "dns_flush",
        description: "DNS resolution problems",
        triggers: &["dns", "resolve", "domain", "lookup", "name"],
        check_cmd: "resolvectl status 2>&1 | grep -q 'DNS Servers' && echo 'ok' || echo 'fail'",
        check_condition: "fail",
        fix_cmd: "sudo systemctl restart systemd-resolved",
        explanation: "DNS resolution seems broken. Restarting systemd-resolved should fix hostname lookups.",
    },
    AutoFix {
        id: "bluetooth_restart",
        description: "Bluetooth not working",
        triggers: &["bluetooth", "bt", "pair", "device", "wireless"],
        check_cmd: "systemctl is-active bluetooth 2>/dev/null || echo 'inactive'",
        check_condition: "inactive",
        fix_cmd: "sudo systemctl restart bluetooth",
        explanation: "The Bluetooth service isn't running properly. I can restart it for you.",
    },
    // v0.0.995: Audio fixes
    AutoFix {
        id: "pipewire_restart",
        description: "Audio not working (PipeWire)",
        triggers: &["audio", "sound", "pipewire", "speaker", "headphone"],
        check_cmd: "systemctl --user is-active pipewire 2>/dev/null || echo 'inactive'",
        check_condition: "inactive",
        fix_cmd: "systemctl --user restart pipewire pipewire-pulse wireplumber",
        explanation: "PipeWire audio service seems stuck. Restarting it usually fixes audio problems.",
    },
    AutoFix {
        id: "pulseaudio_restart",
        description: "Audio not working (PulseAudio)",
        triggers: &["audio", "sound", "pulseaudio", "volume", "mute"],
        check_cmd: "pactl info 2>&1 | grep -q 'Connection failure' && echo 'fail' || echo 'ok'",
        check_condition: "fail",
        fix_cmd: "pulseaudio -k && pulseaudio --start",
        explanation: "PulseAudio connection failed. I can restart it to restore audio.",
    },
    // v0.0.995: Display & GPU fixes
    AutoFix {
        id: "nvidia_persistence",
        description: "NVIDIA GPU issues",
        triggers: &["nvidia", "gpu", "graphics", "driver", "cuda"],
        check_cmd: "nvidia-smi 2>&1 | grep -q 'NVIDIA-SMI has failed' && echo 'fail' || echo 'ok'",
        check_condition: "fail",
        fix_cmd: "sudo modprobe -r nvidia_uvm nvidia_drm nvidia_modeset nvidia && sudo modprobe nvidia",
        explanation: "The NVIDIA driver seems stuck. I can reload the kernel modules to fix it.",
    },
    AutoFix {
        id: "display_manager_restart",
        description: "Display manager frozen",
        triggers: &["screen", "display", "login", "sddm", "gdm", "lightdm"],
        check_cmd: "systemctl is-active display-manager 2>/dev/null || echo 'inactive'",
        check_condition: "inactive",
        fix_cmd: "sudo systemctl restart display-manager",
        explanation: "The display manager service isn't running. I can restart it (this will log you out!).",
    },
    // v0.0.995: System resource fixes
    AutoFix {
        id: "tmp_cleanup",
        description: "Large /tmp directory",
        triggers: &["tmp", "temp", "temporary", "disk", "space"],
        check_cmd: "du -sh /tmp 2>/dev/null | cut -f1",
        check_condition: "G", // Contains G for gigabytes
        fix_cmd: "sudo find /tmp -type f -atime +7 -delete 2>/dev/null; echo 'Cleaned files older than 7 days'",
        explanation: "The /tmp directory is getting large. I can clean up files that haven't been accessed in a week.",
    },
    AutoFix {
        id: "swap_clear",
        description: "High swap usage",
        triggers: &["swap", "memory", "ram", "slow", "swapping"],
        check_cmd: "free | awk '/Swap:/ {if($3>0) print \"used\"; else print \"empty\"}'",
        check_condition: "used",
        fix_cmd: "sudo swapoff -a && sudo swapon -a",
        explanation: "Swap is being used which can slow things down. I can clear it if you have enough free RAM.",
    },
    // v0.0.995: Service management fixes
    AutoFix {
        id: "docker_restart",
        description: "Docker not responding",
        triggers: &["docker", "container", "daemon", "socket"],
        check_cmd: "docker ps 2>&1 | grep -q 'Cannot connect' && echo 'fail' || echo 'ok'",
        check_condition: "fail",
        fix_cmd: "sudo systemctl restart docker",
        explanation: "Docker daemon isn't responding. I can restart it for you.",
    },
    AutoFix {
        id: "timesyncd_restart",
        description: "System time wrong",
        triggers: &["time", "clock", "date", "ntp", "sync"],
        check_cmd: "timedatectl status | grep -q 'synchronized: no' && echo 'fail' || echo 'ok'",
        check_condition: "fail",
        fix_cmd: "sudo systemctl restart systemd-timesyncd && sudo timedatectl set-ntp true",
        explanation: "System time isn't synchronized. I can restart the time sync service.",
    },
];

/// Find matching auto-fix for a question
/// v0.1.2: Only trigger autofix when user wants to FIX something, not just asking for info
pub fn find_autofix(question: &str) -> Option<&'static AutoFix> {
    let q = question.to_lowercase();

    // v0.1.2: Require action words - user must be asking to FIX something
    // Information questions like "how much disk space" should not trigger autofix
    let action_words = [
        "fix", "clean", "clear", "free up", "remove", "delete",
        "solve", "resolve", "help me", "can you", "please",
        "get rid", "slow", "problem", "issue", "error", "broken",
        "not working", "failed", "failing"
    ];
    let has_action = action_words.iter().any(|w| q.contains(w));

    // Also check for info-seeking questions that should NOT trigger autofix
    let info_questions = [
        "how much", "how many", "what is", "what are", "show me",
        "list", "check", "status", "tell me", "display"
    ];
    let is_info_question = info_questions.iter().any(|w| q.contains(w));

    // Don't trigger autofix for info questions unless they also have action words
    if is_info_question && !has_action {
        debug!("AutoFix: skipping info question without action words");
        return None;
    }

    for fix in AUTO_FIXES {
        // Count how many triggers match
        let matches = fix.triggers.iter().filter(|t| q.contains(*t)).count();
        // Need at least 2 triggers to match (to avoid false positives)
        if matches >= 2 {
            debug!("AutoFix: {} matched with {} triggers", fix.id, matches);
            return Some(fix);
        }
    }
    None
}

/// Check if an auto-fix is needed (run the check command)
pub fn check_autofix_needed(fix: &AutoFix) -> bool {
    match Command::new("sh").arg("-c").arg(fix.check_cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = if fix.check_condition.is_empty() {
                // For numeric checks, any non-zero, non-empty output means problem exists
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
                // v0.0.996: Record successful fix to history
                record_fix(fix, true, stdout.trim());
                Ok(format!("Done! {}", stdout.trim()))
            } else {
                // v0.0.996: Record failed fix to history
                record_fix(fix, false, stderr.trim());
                Err(format!("Fix failed: {}", stderr.trim()))
            }
        }
        Err(e) => {
            // v0.0.996: Record execution error to history
            record_fix(fix, false, &e.to_string());
            Err(format!("Failed to run fix: {}", e))
        }
    }
}

/// Format an auto-fix offer for the user
pub fn format_autofix_offer(fix: &AutoFix) -> String {
    format!(
        "I can fix this for you:\n\n  {}\n\nThis will run:\n  {}\n\nWant me to do it? (yes/no)",
        fix.explanation,
        fix.fix_cmd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Should not match with just one trigger word
        let fix = find_autofix("what is pacman");
        assert!(fix.is_none());
    }

    // v0.0.995: Tests for new auto-fixes
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
        // Could match pipewire or pulseaudio depending on triggers
        assert!(fix.unwrap().id.contains("audio") || fix.unwrap().id.contains("pipewire") || fix.unwrap().id.contains("pulseaudio"));
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
        // Verify we have all expected auto-fixes
        // 6 original + 11 new = 17 total
        assert_eq!(AUTO_FIXES.len(), 17, "Expected 17 auto-fixes");
    }
}
