//! Auto-fix module - Anna offers to fix known issues automatically.
//! v0.0.993: Initial implementation
//!
//! When Anna detects a well-known problem, she can offer to fix it
//! with user confirmation.

use std::process::Command;
use tracing::{debug, info};

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
];

/// Find matching auto-fix for a question
pub fn find_autofix(question: &str) -> Option<&'static AutoFix> {
    let q = question.to_lowercase();

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
                Ok(format!("Done! {}", stdout.trim()))
            } else {
                Err(format!("Fix failed: {}", stderr.trim()))
            }
        }
        Err(e) => Err(format!("Failed to run fix: {}", e))
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
}
