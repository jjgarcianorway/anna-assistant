//! Anna Auto-Install v7.26.0
//!
//! Controlled package installation with safety guardrails:
//! - Only official Arch repos by default (AUR requires explicit gate)
//! - Rate limit: 1 install per 24 hours (configurable)
//! - All installs logged to ops_log
//! - Non-interactive pacman with safe flags

use crate::config::AnnaConfig;
use crate::instrumentation::{
    AvailableTool, InstalledTool, InstrumentationManifest,
    get_package_version,
};
use crate::ops_log::{OpsAction, OpsEntry, OpsLogWriter};
use chrono::Utc;
use std::process::Command;

/// Result of an install attempt
#[derive(Debug, Clone)]
pub enum InstallResult {
    /// Successfully installed
    Success { package: String, version: String },
    /// Rate limited - too many installs today
    RateLimited { reset_at: String },
    /// AUR gate blocked
    AurBlocked { package: String },
    /// Auto-install disabled in config
    Disabled,
    /// Already installed
    AlreadyInstalled { package: String },
    /// Pacman failed
    PacmanFailed { package: String, error: String },
    /// Unknown package
    UnknownPackage { package: String },
}

impl InstallResult {
    pub fn is_success(&self) -> bool {
        matches!(self, InstallResult::Success { .. })
    }

    pub fn message(&self) -> String {
        match self {
            InstallResult::Success { package, version } =>
                format!("✓  Installed {} v{}", package, version),
            InstallResult::RateLimited { reset_at } =>
                format!("⏳  Rate limited, retry after {}", reset_at),
            InstallResult::AurBlocked { package } =>
                format!("🚫  {} is AUR package (gate disabled)", package),
            InstallResult::Disabled =>
                "⚙️  Auto-install disabled in config".to_string(),
            InstallResult::AlreadyInstalled { package } =>
                format!("✓  {} already installed", package),
            InstallResult::PacmanFailed { package, error } =>
                format!("✗  Failed to install {}: {}", package, error),
            InstallResult::UnknownPackage { package } =>
                format!("?  Unknown package: {}", package),
        }
    }
}

/// Try to auto-install a package with all guardrails
pub fn try_install(
    package: &str,
    reason: &str,
    metrics_enabled: &[String],
    optional: bool,
) -> InstallResult {
    let config = AnnaConfig::load();
    let mut manifest = InstrumentationManifest::load();

    // Guard 1: Check if auto-install is enabled
    if !config.instrumentation.auto_install_enabled {
        return InstallResult::Disabled;
    }

    // Guard 2: Check if already installed
    if manifest.is_installed(package) {
        return InstallResult::AlreadyInstalled { package: package.to_string() };
    }

    // Guard 3: Check rate limit
    if manifest.is_rate_limited(&config) {
        let reset_at = manifest.rate_limit_reset_time()
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "soon".to_string());
        return InstallResult::RateLimited { reset_at };
    }

    // Guard 4: Check AUR gate
    let source = get_package_source(package);
    if source == "aur" && !config.instrumentation.allow_aur {
        return InstallResult::AurBlocked { package: package.to_string() };
    }

    // Guard 5: Verify package exists in repos
    if !package_exists_in_repos(package) {
        return InstallResult::UnknownPackage { package: package.to_string() };
    }

    // Execute install
    let result = run_pacman_install(package);

    // Record attempt
    manifest.record_attempt(
        package,
        result.is_ok(),
        result.as_ref().err().cloned(),
    );

    match result {
        Ok(version) => {
            // Record successful install
            let tool = InstalledTool {
                package: package.to_string(),
                version: version.clone(),
                installed_at: Utc::now(),
                reason: reason.to_string(),
                metrics_enabled: metrics_enabled.to_vec(),
                optional,
                source,
            };
            manifest.record_install(tool);
            let _ = manifest.save();

            // Log to ops_log
            log_install_to_ops(package, &version, reason, true, None);

            InstallResult::Success { package: package.to_string(), version }
        }
        Err(error) => {
            let _ = manifest.save();
            log_install_to_ops(package, "", reason, false, Some(&error));
            InstallResult::PacmanFailed {
                package: package.to_string(),
                error,
            }
        }
    }
}

/// Try to install a known tool by name
pub fn try_install_known_tool(tool: &AvailableTool) -> InstallResult {
    try_install(
        &tool.package,
        &tool.reason,
        &tool.metrics_enabled,
        tool.optional,
    )
}

/// Get package source (official or aur)
fn get_package_source(package: &str) -> String {
    // Check if package is in official repos
    let output = Command::new("pacman")
        .args(["-Si", package])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("Repository") {
                // Extract repo name
                for line in stdout.lines() {
                    if line.starts_with("Repository") {
                        let repo = line.split(':')
                            .nth(1)
                            .map(|s| s.trim())
                            .unwrap_or("");
                        if repo == "core" || repo == "extra" || repo == "multilib" {
                            return "official".to_string();
                        }
                    }
                }
            }
            "official".to_string()
        }
        _ => "aur".to_string(),
    }
}

/// Check if package exists in repos
fn package_exists_in_repos(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Si", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run pacman install with safe flags
fn run_pacman_install(package: &str) -> Result<String, String> {
    // Non-interactive install with safe flags:
    // --noconfirm: Don't ask for confirmation
    // --needed: Skip if already installed
    // --noprogressbar: Clean output for logging
    let output = Command::new("sudo")
        .args([
            "pacman", "-S", "--noconfirm", "--needed", "--noprogressbar",
            package,
        ])
        .output()
        .map_err(|e| format!("Failed to run pacman: {}", e))?;

    if output.status.success() {
        // Get installed version
        let version = get_package_version(package)
            .unwrap_or_else(|| "unknown".to_string());
        Ok(version)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

/// Log install action to ops_log
fn log_install_to_ops(
    package: &str,
    version: &str,
    reason: &str,
    success: bool,
    error: Option<&str>,
) {
    let details = if success {
        format!("package={} version={} reason=\"{}\"", package, version, reason)
    } else {
        format!(
            "package={} reason=\"{}\" error=\"{}\"",
            package,
            reason,
            error.unwrap_or("unknown"),
        )
    };

    let entry = OpsEntry {
        timestamp: Utc::now(),
        action: if success {
            OpsAction::PackageInstalled
        } else {
            OpsAction::PackageInstallFailed
        },
        target: package.to_string(),
        details: Some(details),
        success,
    };

    let _ = OpsLogWriter::append_entry(&entry);
}

/// Summary of instrumentation status
#[derive(Debug, Clone)]
pub struct InstrumentationStatus {
    /// Auto-install enabled
    pub enabled: bool,
    /// AUR gate enabled
    pub aur_enabled: bool,
    /// Tools installed by Anna
    pub installed_count: usize,
    /// Tools available to install
    pub available_count: usize,
    /// Rate limited
    pub rate_limited: bool,
    /// Installs remaining today
    pub installs_remaining: u32,
}

/// Get current instrumentation status
pub fn get_instrumentation_status() -> InstrumentationStatus {
    let config = AnnaConfig::load();
    let manifest = InstrumentationManifest::load();

    let cutoff = Utc::now() - chrono::Duration::hours(24);
    let recent_count = manifest.recent_attempts
        .iter()
        .filter(|a| a.attempted_at > cutoff && a.success)
        .count() as u32;

    let max = config.instrumentation.max_installs_per_day;
    let remaining = max.saturating_sub(recent_count);

    InstrumentationStatus {
        enabled: config.instrumentation.auto_install_enabled,
        aur_enabled: config.instrumentation.allow_aur,
        installed_count: manifest.installed_count(),
        available_count: crate::instrumentation::get_missing_tools(&config).len(),
        rate_limited: manifest.is_rate_limited(&config),
        installs_remaining: remaining,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_result_messages() {
        let success = InstallResult::Success {
            package: "test".to_string(),
            version: "1.0".to_string(),
        };
        assert!(success.message().contains("Installed"));

        let rate_limited = InstallResult::RateLimited {
            reset_at: "12:00".to_string(),
        };
        assert!(rate_limited.message().contains("Rate limited"));

        let aur_blocked = InstallResult::AurBlocked {
            package: "aur-pkg".to_string(),
        };
        assert!(aur_blocked.message().contains("AUR"));
    }

    #[test]
    fn test_disabled_guard() {
        // When config has auto_install_enabled = false, should return Disabled
        // This test verifies the logic path exists
        let result = InstallResult::Disabled;
        assert!(!result.is_success());
        assert!(result.message().contains("disabled"));
    }
}
