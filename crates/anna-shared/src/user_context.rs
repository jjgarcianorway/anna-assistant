//! User context detection and command execution.
//!
//! The daemon runs as root, but most commands should run as the
//! logged-in user to get correct results (home dir, packages, etc.).

use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::{debug, warn};

/// Context for the logged-in user
#[derive(Debug, Clone)]
pub struct UserContext {
    /// Username
    pub username: String,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// Home directory
    pub home: String,
    /// Default shell
    pub shell: String,
}

impl UserContext {
    /// Detect the currently logged-in user.
    /// Returns None if no user is logged in or detection fails.
    pub fn detect() -> Option<Self> {
        // Try SUDO_USER first (if daemon was started via sudo)
        if let Ok(user) = std::env::var("SUDO_USER") {
            if !user.is_empty() && user != "root" {
                if let Some(ctx) = Self::from_username(&user) {
                    debug!("Detected user from SUDO_USER: {}", user);
                    return Some(ctx);
                }
            }
        }

        // Use loginctl to find logged-in users
        let output = Command::new("loginctl")
            .args(["list-users", "--no-legend"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse loginctl output: "UID USERNAME"
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let username = parts[1];
                // Skip root
                if username != "root" {
                    if let Some(ctx) = Self::from_username(username) {
                        debug!("Detected user from loginctl: {}", username);
                        return Some(ctx);
                    }
                }
            }
        }

        // Fallback: check who owns the display
        if let Ok(display_user) = Self::detect_from_display() {
            debug!("Detected user from display: {}", display_user.username);
            return Some(display_user);
        }

        warn!("Could not detect logged-in user");
        None
    }

    /// Create UserContext from username by looking up /etc/passwd
    fn from_username(username: &str) -> Option<Self> {
        let output = Command::new("getent")
            .args(["passwd", username])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let line = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = line.trim().split(':').collect();

        // passwd format: username:x:uid:gid:gecos:home:shell
        if parts.len() >= 7 {
            let uid = parts[2].parse().ok()?;
            let gid = parts[3].parse().ok()?;
            let home = parts[5].to_string();
            let shell = parts[6].to_string();

            // Validate home directory exists
            if !std::path::Path::new(&home).exists() {
                warn!("User {} home directory {} doesn't exist", username, home);
                return None;
            }

            return Some(UserContext {
                username: username.to_string(),
                uid,
                gid,
                home,
                shell,
            });
        }

        None
    }

    /// Try to detect user from display ownership
    fn detect_from_display() -> Result<Self> {
        // Check who owns /dev/tty1 or similar
        let output = Command::new("who")
            .output()
            .map_err(|e| anyhow!("who failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let username = parts[0];
                if username != "root" {
                    if let Some(ctx) = Self::from_username(username) {
                        return Ok(ctx);
                    }
                }
            }
        }

        Err(anyhow!("No display user found"))
    }

    /// Execute a command as this user.
    /// Uses runuser for proper session setup.
    pub fn execute(&self, cmd: &str) -> Result<String> {
        // Use runuser to execute as the user with their environment
        let output = Command::new("runuser")
            .args([
                "-u", &self.username,
                "--",
                "sh", "-c", cmd
            ])
            .env("HOME", &self.home)
            .env("USER", &self.username)
            .env("LOGNAME", &self.username)
            .output()
            .map_err(|e| anyhow!("runuser failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = stdout.to_string();
        if !stderr.is_empty() && !stderr.contains("cannot set groups") {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&format!("(stderr: {})", stderr.trim()));
        }

        Ok(result)
    }

    /// Execute a command as this user, with their full environment.
    /// Uses su - for login shell setup (slower but more complete).
    pub fn execute_with_env(&self, cmd: &str) -> Result<String> {
        let output = Command::new("su")
            .args([
                "-", &self.username,
                "-c", cmd
            ])
            .output()
            .map_err(|e| anyhow!("su failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = stdout.to_string();
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&format!("(stderr: {})", stderr.trim()));
        }

        Ok(result)
    }

    /// Get the user's config directory (~/.config)
    pub fn config_dir(&self) -> String {
        format!("{}/.config", self.home)
    }

    /// Expand ~ in a path to user's home
    pub fn expand_home(&self, path: &str) -> String {
        if path.starts_with("~/") {
            format!("{}/{}", self.home, &path[2..])
        } else if path == "~" {
            self.home.clone()
        } else {
            path.to_string()
        }
    }
}

/// Check if a command needs root privileges.
/// Returns true for commands that must run as root.
pub fn needs_root(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    let first_word = cmd.split_whitespace().next().unwrap_or("");

    // Commands that explicitly use sudo
    if first_word == "sudo" {
        return true;
    }

    // System-level commands that need root
    let root_commands = [
        "systemctl start",
        "systemctl stop",
        "systemctl restart",
        "systemctl enable",
        "systemctl disable",
        "systemctl daemon-reload",
        "journalctl -u",  // Service-specific logs need root
        "pacman -S",
        "pacman -R",
        "pacman -U",
        "modprobe",
        "insmod",
        "rmmod",
        "mount ",
        "umount",
        "fdisk",
        "parted",
        "mkfs",
        "blkid",  // Often needs root for full info
        "dmidecode",
        "hdparm",
        "smartctl",
    ];

    for pattern in &root_commands {
        if cmd_lower.contains(pattern) {
            return true;
        }
    }

    // Reading system files that are root-only
    if cmd_lower.contains("cat /etc/shadow")
        || cmd_lower.contains("cat /etc/gshadow")
    {
        return true;
    }

    false
}

/// Check if a command accesses user-specific data.
/// These commands should always run as the user.
pub fn is_user_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Commands that access user's home or config
    if cmd_lower.contains("~/")
        || cmd_lower.contains("$home")
        || cmd_lower.contains(".config")
        || cmd_lower.contains(".local")
        || cmd_lower.contains(".cache")
    {
        return true;
    }

    // User-specific tools
    let user_commands = [
        "flatpak list",
        "flatpak info",
        "systemctl --user",
        "dconf",
        "gsettings",
        "xdg-",
        "fish ",  // Fish shell
        "fish_",
        "nvim ",
        "vim ",
    ];

    for pattern in &user_commands {
        if cmd_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Global cached user context
static USER_CONTEXT: std::sync::OnceLock<Option<UserContext>> = std::sync::OnceLock::new();

/// Get the cached user context (detects once, caches forever)
pub fn get_user_context() -> Option<&'static UserContext> {
    USER_CONTEXT.get_or_init(|| UserContext::detect()).as_ref()
}

/// Force re-detection of user context
pub fn refresh_user_context() -> Option<UserContext> {
    UserContext::detect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_root() {
        assert!(needs_root("sudo pacman -S neovim"));
        assert!(needs_root("systemctl restart nginx"));
        assert!(needs_root("journalctl -u sshd"));
        assert!(!needs_root("ls -la"));
        assert!(!needs_root("pacman -Qi neovim"));
        assert!(!needs_root("systemctl --user status"));
    }

    #[test]
    fn test_is_user_command() {
        assert!(is_user_command("cat ~/.config/fish/config.fish"));
        assert!(is_user_command("ls ~/.local/share"));
        assert!(is_user_command("systemctl --user status pipewire"));
        assert!(!is_user_command("cat /etc/os-release"));
        assert!(!is_user_command("uname -r"));
    }
}
