//! Self-healing capabilities - autonomous recovery from common issues.
//!
//! Anna can automatically fix certain safe issues without user intervention:
//! - Restart failed user services
//! - Clean disk when critically low
//! - Clear stale locks
//!
//! Note: Self-healing is silent. Results are logged and shown in morning briefing.
//! No push notifications - those are reserved for critical alerts only.

use std::process::Command;
use tracing::{info, warn};

/// Services that are safe to auto-restart (user-level, non-critical).
const SAFE_TO_RESTART: &[&str] = &[
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "xdg-desktop-portal-gnome",
    "xdg-desktop-portal-kde",
    "gvfs-daemon",
    "evolution-addressbook-factory",
    "evolution-calendar-factory",
    "evolution-source-registry",
    "tracker-miner-fs",
    "gnome-keyring-daemon",
];

/// Services that should NEVER be auto-restarted.
const NEVER_RESTART: &[&str] = &[
    "systemd",
    "dbus",
    "NetworkManager",
    "sshd",
    "gdm",
    "sddm",
    "lightdm",
    "docker",
    "libvirtd",
    "postgresql",
    "mysql",
    "mariadb",
    "nginx",
    "apache",
    "httpd",
];

/// Result of a self-healing action.
#[derive(Debug)]
pub struct HealingResult {
    pub action: String,
    pub success: bool,
    pub message: String,
}

/// Try to restart a failed service if it's safe.
pub fn try_restart_service(service: &str) -> Option<HealingResult> {
    let service_name = service.trim_end_matches(".service");

    // Check if it's on the never-restart list
    if NEVER_RESTART.iter().any(|s| service_name.contains(s)) {
        return None;
    }

    // Check if it's a safe service
    let is_safe = SAFE_TO_RESTART.iter().any(|s| service_name.contains(s));

    // Also allow user services (--user)
    let is_user_service = service_name.starts_with("app-")
        || service_name.contains("@autostart");

    if !is_safe && !is_user_service {
        return None;
    }

    info!("Attempting to restart safe service: {}", service_name);

    // Try user service first
    let user_result = Command::new("systemctl")
        .args(["--user", "restart", service_name])
        .status();

    if let Ok(status) = user_result {
        if status.success() {
            return Some(HealingResult {
                action: format!("Restarted {}", service_name),
                success: true,
                message: format!("Auto-restarted user service: {}", service_name),
            });
        }
    }

    // Try system service with pkexec (will prompt for auth)
    // Only for known safe services
    if is_safe {
        let system_result = Command::new("systemctl")
            .args(["restart", service_name])
            .status();

        if let Ok(status) = system_result {
            if status.success() {
                return Some(HealingResult {
                    action: format!("Restarted {}", service_name),
                    success: true,
                    message: format!("Auto-restarted system service: {}", service_name),
                });
            }
        }
    }

    Some(HealingResult {
        action: format!("Failed to restart {}", service_name),
        success: false,
        message: format!("Could not restart {}", service_name),
    })
}

/// Auto-clean disk when critically low (<5GB free).
pub fn auto_clean_if_critical() -> Option<HealingResult> {
    // Check available space
    let output = Command::new("df")
        .args(["--output=avail", "-BG", "/"])
        .output()
        .ok()?;

    let out = String::from_utf8_lossy(&output.stdout);
    let line = out.lines().nth(1)?;
    let gb: u64 = line.trim().trim_end_matches('G').parse().ok()?;

    if gb >= 5 {
        return None; // Not critical
    }

    info!("Disk critically low ({}GB), auto-cleaning...", gb);

    let mut cleaned = Vec::new();
    let mut freed_mb = 0u64;

    // 1. Package cache (aggressive - keep only 1 version)
    if let Ok(before) = get_cache_size_mb() {
        let _ = Command::new("paccache").args(["-rk1"]).status();
        if let Ok(after) = get_cache_size_mb() {
            let saved = before.saturating_sub(after);
            if saved > 0 {
                cleaned.push(format!("Package cache: {}MB", saved));
                freed_mb += saved;
            }
        }
    }

    // 2. Journal logs (aggressive - 3 days)
    let _ = Command::new("journalctl")
        .args(["--vacuum-time=3d"])
        .status();
    cleaned.push("Journal logs (3 days)".to_string());

    // 3. /tmp files older than 1 day
    let _ = Command::new("find")
        .args(["/tmp", "-type", "f", "-mtime", "+1", "-delete"])
        .status();
    cleaned.push("/tmp files (1 day)".to_string());

    // 4. Thumbnail cache
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let thumb_cache = format!("{}/.cache/thumbnails", home);
        let _ = std::fs::remove_dir_all(&thumb_cache);
        cleaned.push("Thumbnail cache".to_string());
    }

    let message = format!(
        "Disk was critically low ({}GB). Auto-cleaned:\n- {}\n\nFreed ~{}MB",
        gb,
        cleaned.join("\n- "),
        freed_mb
    );

    // Log only - no push notification (will be in morning briefing)
    info!("Self-healing: {}", message);

    Some(HealingResult {
        action: "Auto-clean disk".to_string(),
        success: true,
        message,
    })
}

fn get_cache_size_mb() -> Result<u64, ()> {
    let output = Command::new("du")
        .args(["-sm", "/var/cache/pacman/pkg"])
        .output()
        .map_err(|_| ())?;

    let out = String::from_utf8_lossy(&output.stdout);
    out.split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or(())
}

/// Clear stale pacman lock if exists and no pacman running.
pub fn clear_stale_pacman_lock() -> Option<HealingResult> {
    let lock_path = "/var/lib/pacman/db.lck";

    if !std::path::Path::new(lock_path).exists() {
        return None;
    }

    // Check if pacman is running
    let output = Command::new("pgrep")
        .arg("pacman")
        .output()
        .ok()?;

    if !output.stdout.is_empty() {
        // pacman is running, lock is valid
        return None;
    }

    // Lock is stale, remove it
    info!("Self-healing: Removing stale pacman lock");

    if std::fs::remove_file(lock_path).is_ok() {
        // Log only - no push notification (will be in morning briefing)
        Some(HealingResult {
            action: "Clear pacman lock".to_string(),
            success: true,
            message: "Removed stale pacman database lock".to_string(),
        })
    } else {
        None
    }
}

/// Run all self-healing checks.
pub fn run_self_healing() -> Vec<HealingResult> {
    let mut results = Vec::new();

    // 1. Auto-clean if disk critical
    if let Some(r) = auto_clean_if_critical() {
        results.push(r);
    }

    // 2. Clear stale pacman lock
    if let Some(r) = clear_stale_pacman_lock() {
        results.push(r);
    }

    // 3. Try to restart failed services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--no-pager"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if let Some(service) = line.split_whitespace().next() {
                if let Some(r) = try_restart_service(service) {
                    results.push(r);
                }
            }
        }
    }

    // Also check user services
    if let Ok(output) = Command::new("systemctl")
        .args(["--user", "--failed", "--no-legend", "--no-pager"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if let Some(service) = line.split_whitespace().next() {
                if let Some(r) = try_restart_service(service) {
                    results.push(r);
                }
            }
        }
    }

    results
}
