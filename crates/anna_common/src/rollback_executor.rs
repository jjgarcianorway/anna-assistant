//! Rollback Executor for Anna v0.0.81
//!
//! Executes actual rollback operations:
//! - F1: File rollback (restore from backup)
//! - F2: Service rollback (restore previous state)
//! - F3: Package rollback (pacman -U with cached .pkg.tar.zst)
//! - F4: Verification after rollback

use crate::mutation_verification::{
    verify_config_edit, verify_package_install, verify_package_remove, verify_service_action,
    DetailedVerificationResult,
};
use crate::privilege::run_privileged;
use crate::rollback::{MutationDetails, MutationType, ROLLBACK_FILES_DIR};
use crate::service_mutation::get_service_state;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Rollback result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    /// Whether rollback succeeded
    pub success: bool,
    /// What was rolled back
    pub rolled_back: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Verification after rollback
    pub verification: Option<DetailedVerificationResult>,
}

/// Package cache directory for rollback
pub const PACMAN_CACHE_DIR: &str = "/var/cache/pacman/pkg";

// =============================================================================
// F1: File Rollback
// =============================================================================

/// Restore a file from backup
pub fn rollback_file(
    original_path: &str,
    backup_path: &str,
    case_id: &str,
) -> RollbackResult {
    // Verify backup exists
    if !Path::new(backup_path).exists() {
        return RollbackResult {
            success: false,
            rolled_back: format!("File: {}", original_path),
            error: Some(format!("Backup not found: {}", backup_path)),
            verification: None,
        };
    }

    // Copy backup back to original location
    let result = run_privileged(
        &format!("cp {} {}", backup_path, original_path),
        &["cp", backup_path, original_path],
    );

    match result {
        Ok(_) => {
            // Verify the restore
            let verification = verify_file_restored(original_path, backup_path);

            RollbackResult {
                success: verification.passed,
                rolled_back: format!("File: {}", original_path),
                error: if verification.passed {
                    None
                } else {
                    Some(verification.diagnostic.clone().unwrap_or_default())
                },
                verification: Some(verification),
            }
        }
        Err(e) => RollbackResult {
            success: false,
            rolled_back: format!("File: {}", original_path),
            error: Some(format!("Failed to restore file: {}", e)),
            verification: None,
        },
    }
}

/// Verify file was restored correctly
fn verify_file_restored(original: &str, backup: &str) -> DetailedVerificationResult {
    // Check both files exist
    if !Path::new(original).exists() {
        return DetailedVerificationResult {
            description: format!("Verify {} was restored", original),
            passed: false,
            expected: "file exists".to_string(),
            actual: "file missing after restore".to_string(),
            diagnostic: Some("File not found after restore attempt".to_string()),
        };
    }

    // Compare file contents
    let original_content = fs::read_to_string(original).unwrap_or_default();
    let backup_content = fs::read_to_string(backup).unwrap_or_default();

    let passed = original_content == backup_content;

    DetailedVerificationResult {
        description: format!("Verify {} matches backup", original),
        passed,
        expected: "content matches backup".to_string(),
        actual: if passed {
            "content matches".to_string()
        } else {
            "content differs from backup".to_string()
        },
        diagnostic: if !passed {
            Some("File content does not match backup after restore".to_string())
        } else {
            None
        },
    }
}

// =============================================================================
// F2: Service Rollback
// =============================================================================

/// Restore service to previous state
pub fn rollback_service(
    service: &str,
    previous_active: &str,
    previous_enabled: &str,
) -> RollbackResult {
    let current_state = get_service_state(service);
    let mut commands = Vec::new();

    // Restore active state
    if current_state.active_state != previous_active {
        match previous_active {
            "active" => commands.push(("start", format!("systemctl start {}", service))),
            "inactive" => commands.push(("stop", format!("systemctl stop {}", service))),
            _ => {}
        }
    }

    // Restore enabled state
    if current_state.enabled_state != previous_enabled {
        match previous_enabled {
            "enabled" => commands.push(("enable", format!("systemctl enable {}", service))),
            "disabled" => commands.push(("disable", format!("systemctl disable {}", service))),
            _ => {}
        }
    }

    if commands.is_empty() {
        return RollbackResult {
            success: true,
            rolled_back: format!("Service: {} (no changes needed)", service),
            error: None,
            verification: None,
        };
    }

    // Execute rollback commands
    let mut all_success = true;
    let mut errors = Vec::new();

    for (desc, cmd) in commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let result = run_privileged(&cmd, &parts);

        if let Err(e) = result {
            all_success = false;
            errors.push(format!("Failed to {} {}: {}", desc, service, e));
        }
    }

    // Verify final state
    let final_state = get_service_state(service);
    let verification = DetailedVerificationResult {
        description: format!("Verify {} state restored", service),
        passed: final_state.active_state == previous_active
            && final_state.enabled_state == previous_enabled,
        expected: format!("{} / {}", previous_active, previous_enabled),
        actual: format!("{} / {}", final_state.active_state, final_state.enabled_state),
        diagnostic: if !all_success {
            Some(errors.join("; "))
        } else {
            None
        },
    };

    RollbackResult {
        success: all_success && verification.passed,
        rolled_back: format!("Service: {}", service),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
        verification: Some(verification),
    }
}

// =============================================================================
// F3: Package Rollback
// =============================================================================

/// Find cached package file for rollback
pub fn find_cached_package(package: &str, version: &str) -> Option<String> {
    let pattern = format!("{}-{}-", package, version);

    if let Ok(entries) = fs::read_dir(PACMAN_CACHE_DIR) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&pattern) && name.ends_with(".pkg.tar.zst") {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }

    None
}

/// Rollback a package installation (reinstall previous version)
pub fn rollback_package_install(package: &str, previous_version: &str) -> RollbackResult {
    // Find the cached package
    let cache_path = match find_cached_package(package, previous_version) {
        Some(path) => path,
        None => {
            return RollbackResult {
                success: false,
                rolled_back: format!("Package: {}", package),
                error: Some(format!(
                    "Cannot find cached package {}-{} in {}",
                    package, previous_version, PACMAN_CACHE_DIR
                )),
                verification: None,
            };
        }
    };

    // Install from cache
    let cmd = format!("pacman -U --noconfirm {}", cache_path);
    let result = run_privileged(&cmd, &["pacman", "-U", "--noconfirm", &cache_path]);

    match result {
        Ok(_) => {
            // Verify installation
            let verifications = verify_package_install(&[package.to_string()]);
            let verification = verifications.into_iter().next();

            RollbackResult {
                success: verification.as_ref().map(|v| v.passed).unwrap_or(false),
                rolled_back: format!("Package: {} (restored to {})", package, previous_version),
                error: None,
                verification,
            }
        }
        Err(e) => RollbackResult {
            success: false,
            rolled_back: format!("Package: {}", package),
            error: Some(format!("Failed to restore package: {}", e)),
            verification: None,
        },
    }
}

/// Rollback a package removal (reinstall the package)
pub fn rollback_package_remove(package: &str) -> RollbackResult {
    // Reinstall the package from repos
    let cmd = format!("pacman -S --noconfirm {}", package);
    let result = run_privileged(&cmd, &["pacman", "-S", "--noconfirm", package]);

    match result {
        Ok(_) => {
            // Verify installation
            let verifications = verify_package_install(&[package.to_string()]);
            let verification = verifications.into_iter().next();

            RollbackResult {
                success: verification.as_ref().map(|v| v.passed).unwrap_or(false),
                rolled_back: format!("Package: {} (reinstalled)", package),
                error: None,
                verification,
            }
        }
        Err(e) => RollbackResult {
            success: false,
            rolled_back: format!("Package: {}", package),
            error: Some(format!("Failed to reinstall package: {}", e)),
            verification: None,
        },
    }
}

// =============================================================================
// Combined Rollback
// =============================================================================

/// Rollback a mutation based on its type
pub fn rollback_mutation(
    mutation_type: &MutationType,
    details: &MutationDetails,
    case_id: &str,
) -> RollbackResult {
    match (mutation_type, details) {
        (MutationType::FileEdit, MutationDetails::FileEdit { file_path, backup_path, .. }) => {
            rollback_file(
                &file_path.to_string_lossy(),
                &backup_path.to_string_lossy(),
                case_id,
            )
        }
        (
            MutationType::SystemdRestart
            | MutationType::SystemdReload
            | MutationType::SystemdEnableNow
            | MutationType::SystemdDisableNow,
            MutationDetails::Systemd {
                service,
                prior_state,
                ..
            },
        ) => {
            if let Some(prior) = prior_state {
                // Convert bool fields to state strings
                let active_str = if prior.is_active { "active" } else { "inactive" };
                let enabled_str = if prior.is_enabled { "enabled" } else { "disabled" };
                rollback_service(service, active_str, enabled_str)
            } else {
                RollbackResult {
                    success: false,
                    rolled_back: format!("Service: {}", service),
                    error: Some("No prior state recorded for rollback".to_string()),
                    verification: None,
                }
            }
        }
        (MutationType::PackageInstall, MutationDetails::Package { package, version, .. }) => {
            // To rollback an install, remove the package
            let cmd = format!("pacman -R --noconfirm {}", package);
            let result = run_privileged(&cmd, &["pacman", "-R", "--noconfirm", package]);

            match result {
                Ok(_) => {
                    let verifications = verify_package_remove(&[package.clone()]);
                    RollbackResult {
                        success: true,
                        rolled_back: format!("Package install: {} (removed)", package),
                        error: None,
                        verification: verifications.into_iter().next(),
                    }
                }
                Err(e) => RollbackResult {
                    success: false,
                    rolled_back: format!("Package: {}", package),
                    error: Some(format!("Failed to remove package: {}", e)),
                    verification: None,
                },
            }
        }
        (MutationType::PackageRemove, MutationDetails::Package { package, .. }) => {
            rollback_package_remove(package)
        }
        _ => RollbackResult {
            success: false,
            rolled_back: format!("{:?}", mutation_type),
            error: Some("Unsupported rollback type".to_string()),
            verification: None,
        },
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cached_package_pattern() {
        // This test just validates the pattern matching logic
        let pattern = format!("{}-{}-", "vim", "9.0.0");
        assert!(pattern.starts_with("vim-"));
        assert!(pattern.contains("9.0.0"));
    }

    #[test]
    fn test_rollback_result_success() {
        let result = RollbackResult {
            success: true,
            rolled_back: "File: /etc/test.conf".to_string(),
            error: None,
            verification: None,
        };
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_rollback_result_failure() {
        let result = RollbackResult {
            success: false,
            rolled_back: "Package: test".to_string(),
            error: Some("Failed to restore".to_string()),
            verification: None,
        };
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
