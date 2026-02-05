//! Config File Editor - Safe configuration file editing with approval.
//!
//! v0.3.124: Anna can edit system config files with user approval and rollback.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A proposed configuration change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// File to edit
    pub file_path: String,
    /// What this change does
    pub description: String,
    /// Old content (for rollback)
    pub old_content: String,
    /// New content to write
    pub new_content: String,
    /// Whether this needs a service restart
    pub needs_restart: Option<String>,
    /// Backup path
    pub backup_path: Option<String>,
}

impl ConfigChange {
    /// Create a new config change.
    pub fn new(file_path: &str, description: &str, old_content: String, new_content: String) -> Self {
        Self {
            file_path: file_path.to_string(),
            description: description.to_string(),
            old_content,
            new_content,
            needs_restart: None,
            backup_path: None,
        }
    }

    /// Set service to restart after change.
    pub fn with_restart(mut self, service: &str) -> Self {
        self.needs_restart = Some(service.to_string());
        self
    }

    /// Read current content of the file.
    pub fn read_current(file_path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(file_path)
    }

    /// Create a backup of the file.
    pub fn create_backup(&mut self) -> std::io::Result<()> {
        let backup_dir = PathBuf::from("/var/lib/anna/config_backups");
        std::fs::create_dir_all(&backup_dir)?;

        let filename = Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config");

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}_{}", filename, timestamp);
        let backup_path = backup_dir.join(backup_name);

        std::fs::copy(&self.file_path, &backup_path)?;
        self.backup_path = Some(backup_path.to_string_lossy().to_string());

        Ok(())
    }

    /// Apply the change.
    pub fn apply(&self) -> std::io::Result<()> {
        // Write new content
        std::fs::write(&self.file_path, &self.new_content)?;

        // Restart service if needed
        if let Some(ref service) = self.needs_restart {
            let _ = std::process::Command::new("sudo")
                .arg("systemctl")
                .arg("restart")
                .arg(service)
                .output();
        }

        Ok(())
    }

    /// Rollback the change.
    pub fn rollback(&self) -> std::io::Result<()> {
        std::fs::write(&self.file_path, &self.old_content)?;

        // Restart service if needed
        if let Some(ref service) = self.needs_restart {
            let _ = std::process::Command::new("sudo")
                .arg("systemctl")
                .arg("restart")
                .arg(service)
                .output();
        }

        Ok(())
    }

    /// Generate a diff for display.
    pub fn generate_diff(&self) -> String {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(&self.old_content, &self.new_content);
        let mut output = Vec::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push(format!("{} {}", sign, change));
        }

        output.join("")
    }

    /// Format for user approval.
    pub fn format_for_approval(&self) -> String {
        let mut lines = vec![
            format!("Configuration Change: {}", self.description),
            format!("File: {}", self.file_path),
            String::new(),
            "Changes:".to_string(),
            self.generate_diff(),
        ];

        if let Some(ref service) = self.needs_restart {
            lines.push(String::new());
            lines.push(format!("Will restart service: {}", service));
        }

        lines.push(String::new());
        lines.push("Backup will be created before applying.".to_string());

        lines.join("\n")
    }
}

/// Common config files Anna might need to edit.
pub fn common_config_files() -> Vec<&'static str> {
    vec![
        "/etc/ssh/sshd_config",
        "/etc/systemd/system.conf",
        "/etc/systemd/logind.conf",
        "/etc/default/grub",
        "/etc/fstab",
        "/etc/pacman.conf",
        "/etc/makepkg.conf",
        "/etc/NetworkManager/NetworkManager.conf",
        "/etc/X11/xorg.conf.d/",
        "/etc/pulse/default.pa",
        "/etc/pipewire/pipewire.conf",
    ]
}

/// Check if a file path is a known config file.
pub fn is_known_config(path: &str) -> bool {
    common_config_files().iter().any(|p| path.starts_with(p))
}

/// Suggest a config change based on a problem.
pub fn suggest_config_fix(problem: &str) -> Option<ConfigChange> {
    let problem_lower = problem.to_lowercase();

    // SSH root login
    if problem_lower.contains("ssh") && problem_lower.contains("root") {
        if let Ok(current) = std::fs::read_to_string("/etc/ssh/sshd_config") {
            if current.contains("PermitRootLogin yes") {
                let new_content = current.replace("PermitRootLogin yes", "PermitRootLogin no");
                return Some(
                    ConfigChange::new(
                        "/etc/ssh/sshd_config",
                        "Disable SSH root login for security",
                        current,
                        new_content,
                    )
                    .with_restart("sshd")
                );
            }
        }
    }

    // Lid switch
    if problem_lower.contains("lid") && problem_lower.contains("suspend") {
        if let Ok(current) = std::fs::read_to_string("/etc/systemd/logind.conf") {
            let new_content = if current.contains("#HandleLidSwitch=") {
                current.replace("#HandleLidSwitch=suspend", "HandleLidSwitch=ignore")
            } else {
                format!("{}\nHandleLidSwitch=ignore\n", current)
            };

            return Some(
                ConfigChange::new(
                    "/etc/systemd/logind.conf",
                    "Disable suspend on lid close",
                    current,
                    new_content,
                )
                .with_restart("systemd-logind")
            );
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_change_creation() {
        let change = ConfigChange::new(
            "/etc/test.conf",
            "Test change",
            "old content".to_string(),
            "new content".to_string(),
        );

        assert_eq!(change.file_path, "/etc/test.conf");
        assert!(change.needs_restart.is_none());
    }

    #[test]
    fn test_is_known_config() {
        assert!(is_known_config("/etc/ssh/sshd_config"));
        assert!(is_known_config("/etc/fstab"));
        assert!(!is_known_config("/home/user/test.txt"));
    }
}
