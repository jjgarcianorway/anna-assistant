//! Self-healing capabilities - autonomous recovery from common issues.
//!
//! Anna can automatically fix certain safe issues without user intervention:
//! - Restart failed user services (via --user systemctl, no privilege needed)
//! - Restart safe system services (via anna-executor RPC)
//! - Clean disk when critically low (via anna-executor RPC)
//! - Clear stale locks
//!
//! Note: Self-healing is silent. Results are logged and shown in morning briefing.
//! No push notifications - those are reserved for critical alerts only.

use std::process::Command;
use tracing::{info, warn};

use crate::executor_client::{executor_rpc, ExecutorRequest, ExecutorResponse};

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

    if NEVER_RESTART.iter().any(|s| service_name.contains(s)) {
        return None;
    }

    let is_safe = SAFE_TO_RESTART.iter().any(|s| service_name.contains(s));
    let is_user_service = service_name.starts_with("app-")
        || service_name.contains("@autostart");

    if !is_safe && !is_user_service {
        return None;
    }

    info!("Attempting to restart safe service: {}", service_name);

    // Try user service first — no privilege needed
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

    // For safe system services: delegate to anna-executor (privileged)
    if is_safe {
        let req = ExecutorRequest::RestartService { name: service_name.to_string() };
        match executor_rpc(&req) {
            Ok(ExecutorResponse::Ok { .. }) => {
                return Some(HealingResult {
                    action: format!("Restarted {}", service_name),
                    success: true,
                    message: format!("Auto-restarted system service: {}", service_name),
                });
            }
            Ok(ExecutorResponse::Denied { reason }) => {
                warn!("Executor denied restart of {}: {}", service_name, reason);
            }
            Ok(ExecutorResponse::Error { message }) => {
                warn!("Executor error restarting {}: {}", service_name, message);
            }
            Err(e) => {
                warn!("Executor unavailable for restart {}: {}", service_name, e);
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
    let output = Command::new("df")
        .args(["--output=avail", "-BG", "/"])
        .output()
        .ok()?;

    let out = String::from_utf8_lossy(&output.stdout);
    let line = out.lines().nth(1)?;
    let gb: u64 = line.trim().trim_end_matches('G').parse().ok()?;

    if gb >= 5 {
        return None;
    }

    info!("Disk critically low ({}GB), auto-cleaning...", gb);

    let mut cleaned = Vec::new();
    let mut freed_mb = 0u64;

    // 1. Package cache — via executor (privileged)
    if let Ok(before) = get_cache_size_mb() {
        let req = ExecutorRequest::CleanPackageCache { keep_versions: 1 };
        match executor_rpc(&req) {
            Ok(ExecutorResponse::Ok { .. }) => {
                if let Ok(after) = get_cache_size_mb() {
                    let saved = before.saturating_sub(after);
                    if saved > 0 {
                        cleaned.push(format!("Package cache: {}MB", saved));
                        freed_mb += saved;
                    }
                }
            }
            Ok(ExecutorResponse::Denied { reason }) => {
                warn!("Package cache clean denied: {}", reason);
            }
            Ok(ExecutorResponse::Error { message }) => {
                warn!("Package cache clean error: {}", message);
            }
            Err(e) => {
                warn!("Executor unavailable for package cache clean: {}", e);
            }
        }
    }

    // 2. Journal logs — via executor (privileged)
    let req = ExecutorRequest::CleanJournal { keep_days: 3 };
    match executor_rpc(&req) {
        Ok(ExecutorResponse::Ok { .. }) | Ok(ExecutorResponse::Error { .. }) => {
            cleaned.push("Journal logs (3 days)".to_string());
        }
        Ok(ExecutorResponse::Denied { reason }) => {
            warn!("Journal clean denied: {}", reason);
        }
        Err(e) => {
            warn!("Executor unavailable for journal clean: {}", e);
        }
    }

    // 3. /tmp files — via executor (privileged)
    let req = ExecutorRequest::CleanTmpFiles;
    match executor_rpc(&req) {
        Ok(ExecutorResponse::Ok { .. }) | Ok(ExecutorResponse::Error { .. }) => {
            cleaned.push("/tmp files (1 day)".to_string());
        }
        Ok(ExecutorResponse::Denied { reason }) => {
            warn!("Tmp clean denied: {}", reason);
        }
        Err(e) => {
            warn!("Executor unavailable for tmp clean: {}", e);
        }
    }

    // 4. Thumbnail cache — user-owned, no privilege needed
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let thumb_cache = format!("{}/.cache/thumbnails", home);
        let _ = std::fs::remove_dir_all(&thumb_cache);
        cleaned.push("Thumbnail cache".to_string());
    }

    if cleaned.is_empty() {
        return None;
    }

    let message = format!(
        "Disk was critically low ({}GB). Auto-cleaned:\n- {}\n\nFreed ~{}MB",
        gb,
        cleaned.join("\n- "),
        freed_mb
    );

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

    let output = Command::new("pgrep")
        .arg("pacman")
        .output()
        .ok()?;

    if !output.stdout.is_empty() {
        return None;
    }

    info!("Self-healing: Removing stale pacman lock");

    if std::fs::remove_file(lock_path).is_ok() {
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

    if let Some(r) = auto_clean_if_critical() {
        results.push(r);
    }

    if let Some(r) = clear_stale_pacman_lock() {
        results.push(r);
    }

    // Check failed system services
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

    // Check failed user services
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
