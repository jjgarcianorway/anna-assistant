//! System update handling - intelligent, safe updates.
//!
//! Provides smart update capabilities:
//! - Check for updates
//! - Categorize updates (security, kernel, regular)
//! - Handle updates with proper notifications
//! - Detect if reboot is needed

use std::process::Command;
use tracing::{info, warn};

// Note: push_notification not used - updates shown in morning briefing

/// Update information.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub is_security: bool,
    pub is_kernel: bool,
}

/// Check for available updates.
pub fn check_updates() -> Vec<UpdateInfo> {
    let mut updates = Vec::new();

    // Use checkupdates (from pacman-contrib)
    let output = match Command::new("checkupdates").output() {
        Ok(o) => o,
        Err(_) => return updates,
    };

    if !output.status.success() {
        return updates;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        // Format: "package oldver -> newver"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[2] == "->" {
            let name = parts[0].to_string();
            let old_version = parts[1].to_string();
            let new_version = parts[3].to_string();

            let is_kernel = name.starts_with("linux")
                && !name.contains("firmware")
                && !name.contains("headers");
            let is_security = name.contains("openssl")
                || name.contains("gnutls")
                || name.contains("nss")
                || name.contains("ca-certificates")
                || name.contains("gnupg")
                || name.contains("sudo")
                || name.contains("polkit")
                || name.contains("systemd");

            updates.push(UpdateInfo {
                name,
                old_version,
                new_version,
                is_security,
                is_kernel,
            });
        }
    }

    updates
}

/// Format updates summary for display.
pub fn format_updates_summary(updates: &[UpdateInfo]) -> String {
    if updates.is_empty() {
        return "System is up to date!".to_string();
    }

    let security: Vec<_> = updates.iter().filter(|u| u.is_security).collect();
    let kernel: Vec<_> = updates.iter().filter(|u| u.is_kernel).collect();
    let regular: Vec<_> = updates
        .iter()
        .filter(|u| !u.is_security && !u.is_kernel)
        .collect();

    let mut lines = vec![format!("{} updates available:", updates.len())];

    if !security.is_empty() {
        lines.push(format!("\n[SECURITY] {} critical updates:", security.len()));
        for u in security.iter().take(5) {
            lines.push(format!("  {} {} -> {}", u.name, u.old_version, u.new_version));
        }
    }

    if !kernel.is_empty() {
        lines.push(format!("\n[KERNEL] {} kernel updates:", kernel.len()));
        for u in &kernel {
            lines.push(format!("  {} {} -> {}", u.name, u.old_version, u.new_version));
        }
        lines.push("  (reboot required after update)".to_string());
    }

    if !regular.is_empty() {
        lines.push(format!("\n[REGULAR] {} package updates", regular.len()));
        for u in regular.iter().take(10) {
            lines.push(format!("  {} {} -> {}", u.name, u.old_version, u.new_version));
        }
        if regular.len() > 10 {
            lines.push(format!("  ... and {} more", regular.len() - 10));
        }
    }

    lines.join("\n")
}

/// Check if reboot is needed after updates.
pub fn needs_reboot() -> bool {
    // Check if running kernel differs from installed
    let running = std::fs::read_to_string("/proc/version")
        .ok()
        .and_then(|v| v.split_whitespace().nth(2).map(String::from));

    let installed = Command::new("pacman")
        .args(["-Q", "linux"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .nth(1)
                .map(String::from)
        });

    match (running, installed) {
        (Some(r), Some(i)) => !r.contains(&i.replace("-", ".")),
        _ => false,
    }
}

/// Run proactive update check (called periodically).
/// No longer sends notifications - updates are shown in morning briefing.
pub fn run_update_check() {
    // Just log, don't notify. Updates will be in morning briefing.
    let updates = check_updates();
    if !updates.is_empty() {
        let security: Vec<_> = updates.iter().filter(|u| u.is_security).collect();
        tracing::info!(
            "Updates available: {} total ({} security)",
            updates.len(),
            security.len()
        );
    }
}

/// Quick update check for Telegram queries.
pub fn get_updates_quick() -> String {
    let updates = check_updates();
    let summary = format_updates_summary(&updates);

    let mut result = summary;

    // Add reboot notice if needed
    if needs_reboot() {
        result.push_str("\n\n[!] Reboot recommended - kernel was updated");
    }

    result
}
