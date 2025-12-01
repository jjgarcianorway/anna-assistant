//! Anna Health / Self-Check v7.7.0 - Dependency and Integrity Checks
//!
//! PHASE 24: Auto-install Arch docs as Anna dependency
//!
//! This module handles Anna's self-health checks:
//! - Detect missing documentation dependencies (arch-wiki-docs, arch-wiki-lite)
//! - Attempt automatic installation via pacman (non-interactive, safe)
//! - Record dependency status for display in `annactl kdb`
//!
//! Rules:
//! - Only runs on Arch-based systems with pacman available
//! - Only attempts installation once per daemon session
//! - Uses sudo pacman -S --noconfirm --needed (safe, non-interactive)
//! - Never blocks daemon startup - runs in background
//! - Records status for honest reporting in kdb

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

/// Packages that provide local Arch documentation
const ARCH_DOCS_PACKAGES: &[&str] = &["arch-wiki-docs", "arch-wiki-lite"];

/// Track whether we've already attempted to install docs this session
static DOCS_INSTALL_ATTEMPTED: AtomicBool = AtomicBool::new(false);

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether Arch docs are available
    pub arch_docs_available: bool,
    /// Which package provides the docs (if any)
    pub arch_docs_package: Option<String>,
    /// Whether we attempted to install docs
    pub docs_install_attempted: bool,
    /// Result of the install attempt (if made)
    pub docs_install_result: Option<String>,
}

impl Default for HealthCheckResult {
    fn default() -> Self {
        Self {
            arch_docs_available: false,
            arch_docs_package: None,
            docs_install_attempted: false,
            docs_install_result: None,
        }
    }
}

/// Check if pacman is available (Arch-based system)
pub fn is_pacman_available() -> bool {
    Command::new("which")
        .arg("pacman")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a package is installed via pacman
pub fn is_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if local Arch docs are available
pub fn check_arch_docs_available() -> (bool, Option<String>) {
    for pkg in ARCH_DOCS_PACKAGES {
        if is_package_installed(pkg) {
            return (true, Some((*pkg).to_string()));
        }
    }
    (false, None)
}

/// Attempt to install Arch docs package (non-interactive)
///
/// Returns (success, message)
pub fn attempt_install_arch_docs() -> (bool, String) {
    // Only attempt once per session
    if DOCS_INSTALL_ATTEMPTED.swap(true, Ordering::SeqCst) {
        return (false, "install already attempted this session".to_string());
    }

    // Check if pacman is available
    if !is_pacman_available() {
        return (false, "pacman not available (not an Arch-based system)".to_string());
    }

    // Check if docs are already installed
    let (available, existing_pkg) = check_arch_docs_available();
    if available {
        return (true, format!("{} already installed", existing_pkg.unwrap_or_default()));
    }

    // Try to install arch-wiki-docs (most complete)
    // Using: sudo pacman -S --noconfirm --needed arch-wiki-docs
    //
    // --noconfirm: Non-interactive
    // --needed: Don't reinstall if already installed
    //
    // Note: This requires either:
    // 1. annad running as root (typical for system service)
    // 2. NOPASSWD sudo configured for pacman
    // 3. polkit rule allowing pacman without password

    let preferred_package = "arch-wiki-docs";

    info!("[HEALTH] Attempting to install {} for rich documentation...", preferred_package);

    // Try without sudo first (if running as root)
    let result = Command::new("pacman")
        .args(["-S", "--noconfirm", "--needed", preferred_package])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                info!("[HEALTH] Successfully installed {}", preferred_package);
                return (true, format!("successfully installed {}", preferred_package));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);

                // If we get a permission error, try with sudo
                if stderr.contains("permission") || stderr.contains("root") || stderr.contains("lock") {
                    let sudo_result = Command::new("sudo")
                        .args(["-n", "pacman", "-S", "--noconfirm", "--needed", preferred_package])
                        .output();

                    match sudo_result {
                        Ok(sudo_output) => {
                            if sudo_output.status.success() {
                                info!("[HEALTH] Successfully installed {} (via sudo)", preferred_package);
                                return (true, format!("successfully installed {} (via sudo)", preferred_package));
                            } else {
                                let sudo_stderr = String::from_utf8_lossy(&sudo_output.stderr);
                                warn!("[HEALTH] Failed to install {}: {}", preferred_package, sudo_stderr.trim());
                                return (false, format!("sudo install failed: {}", sudo_stderr.trim()));
                            }
                        }
                        Err(e) => {
                            warn!("[HEALTH] Failed to run sudo pacman: {}", e);
                            return (false, format!("failed to run sudo: {}", e));
                        }
                    }
                }

                warn!("[HEALTH] Failed to install {}: {}", preferred_package, stderr.trim());
                (false, format!("install failed: {}", stderr.trim()))
            }
        }
        Err(e) => {
            warn!("[HEALTH] Failed to run pacman: {}", e);
            (false, format!("failed to run pacman: {}", e))
        }
    }
}

/// Run full health check
pub fn run_health_check() -> HealthCheckResult {
    let mut result = HealthCheckResult::default();

    // Check current state
    let (available, pkg) = check_arch_docs_available();
    result.arch_docs_available = available;
    result.arch_docs_package = pkg;

    // If not available and not yet attempted, try to install
    if !available && !DOCS_INSTALL_ATTEMPTED.load(Ordering::SeqCst) {
        let (success, message) = attempt_install_arch_docs();
        result.docs_install_attempted = true;
        result.docs_install_result = Some(message);

        // Re-check if install succeeded
        if success {
            let (now_available, now_pkg) = check_arch_docs_available();
            result.arch_docs_available = now_available;
            result.arch_docs_package = now_pkg;
        }
    }

    result
}

/// Log health check results
pub fn log_health_check_results(result: &HealthCheckResult) {
    if result.arch_docs_available {
        if let Some(ref pkg) = result.arch_docs_package {
            info!("[HEALTH] Arch docs available via {}", pkg);
        }
    } else {
        warn!("[HEALTH] Arch docs not available - using pacman and man only for config discovery");
        if let Some(ref install_result) = result.docs_install_result {
            warn!("[HEALTH] Install attempt: {}", install_result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pacman_available() {
        // Just test that the function runs without panic
        let _ = is_pacman_available();
    }

    #[test]
    fn test_check_arch_docs_available() {
        // Just test that the function runs without panic
        let (_, _) = check_arch_docs_available();
    }
}
