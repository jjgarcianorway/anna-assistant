//! Package Mutation Executor for Anna v0.0.81
//!
//! Executes package install/remove mutations:
//! - pacman -S (install)
//! - pacman -R (remove)
//!
//! No AUR support in v0.0.80.
//! v0.0.81: Enhanced previews with size and dependency info.

use crate::mutation_engine_v1::{
    MutationRiskLevel, PackageAction, RollbackStep, StepExecutionResult, StepPreview,
    VerificationCheck,
};
use crate::mutation_verification::get_package_info_detailed;
use crate::privilege::{check_privilege, generate_manual_commands, run_privileged};
use std::process::Command;

/// Check if a package is installed
pub fn is_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get package info for preview
pub fn get_package_info(package: &str) -> PackageInfo {
    let installed = is_package_installed(package);

    // Get version if installed
    let installed_version = if installed {
        Command::new("pacman")
            .args(["-Q", package])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let out = String::from_utf8_lossy(&o.stdout);
                    // Format: "package version"
                    out.split_whitespace().nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
    } else {
        None
    };

    // Get repo version if available
    let repo_version = Command::new("pacman")
        .args(["-Si", package])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let out = String::from_utf8_lossy(&o.stdout);
                for line in out.lines() {
                    if line.starts_with("Version") {
                        return line.split(':').nth(1).map(|s| s.trim().to_string());
                    }
                }
            }
            None
        });

    // Get description
    let description = Command::new("pacman")
        .args(["-Si", package])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let out = String::from_utf8_lossy(&o.stdout);
                for line in out.lines() {
                    if line.starts_with("Description") {
                        return line.split(':').nth(1).map(|s| s.trim().to_string());
                    }
                }
            }
            None
        });

    PackageInfo {
        name: package.to_string(),
        installed,
        installed_version,
        repo_version,
        description,
    }
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub repo_version: Option<String>,
    pub description: Option<String>,
}

impl PackageInfo {
    pub fn format_human(&self) -> String {
        let status = if self.installed {
            format!(
                "installed ({})",
                self.installed_version.as_deref().unwrap_or("?")
            )
        } else {
            "not installed".to_string()
        };

        format!("{}: {}", self.name, status)
    }
}

/// Generate preview for package install
/// v0.0.81: Enhanced with size and dependency info
pub fn preview_package_install(packages: &[String]) -> Result<StepPreview, String> {
    if packages.is_empty() {
        return Err("No packages specified".to_string());
    }

    let mut current_states = Vec::new();
    let mut intended_states = Vec::new();

    for pkg in packages {
        // v0.0.81: Use detailed package info for smarter previews
        let detailed = get_package_info_detailed(pkg);
        current_states.push(detailed.format_preview());

        let intended = if detailed.installed {
            format!("{}: already installed (no change)", pkg)
        } else if detailed.found_in_repos {
            let mut parts = vec![format!(
                "{}: will be installed ({})",
                pkg,
                detailed.repo_version.as_deref().unwrap_or("latest")
            )];
            // Add size info if available
            if let Some(ref size) = detailed.download_size {
                parts.push(format!("  Download: {}", size));
            }
            if let Some(ref size) = detailed.installed_size {
                parts.push(format!("  Installed: {}", size));
            }
            // Add dependency info if available
            if !detailed.depends.is_empty() {
                let deps_str = detailed.depends.join(", ");
                let suffix = if detailed.depends.len() >= 5 { "..." } else { "" };
                parts.push(format!("  Requires: {}{}", deps_str, suffix));
            }
            parts.join("\n")
        } else {
            format!("{}: NOT FOUND in repositories", pkg)
        };
        intended_states.push(intended);
    }

    Ok(StepPreview {
        step_id: format!("pkg-install-{}", packages.join("-")),
        description: format!("Install packages: {}", packages.join(", ")),
        current_state: current_states.join("\n\n"),
        intended_state: intended_states.join("\n\n"),
        diff: None,
    })
}

/// Generate preview for package remove
/// v0.0.81: Enhanced with detailed package info
pub fn preview_package_remove(packages: &[String]) -> Result<StepPreview, String> {
    if packages.is_empty() {
        return Err("No packages specified".to_string());
    }

    let mut current_states = Vec::new();
    let mut intended_states = Vec::new();

    for pkg in packages {
        // v0.0.81: Use detailed package info
        let detailed = get_package_info_detailed(pkg);
        current_states.push(detailed.format_preview());

        let intended = if detailed.installed {
            let mut parts = vec![format!("{}: will be removed", pkg)];
            // Show what space will be freed
            if let Some(ref size) = detailed.installed_size {
                parts.push(format!("  Frees: {}", size));
            }
            parts.join("\n")
        } else {
            format!("{}: not installed (no change)", pkg)
        };
        intended_states.push(intended);
    }

    Ok(StepPreview {
        step_id: format!("pkg-remove-{}", packages.join("-")),
        description: format!("Remove packages: {}", packages.join(", ")),
        current_state: current_states.join("\n\n"),
        intended_state: intended_states.join("\n\n"),
        diff: None,
    })
}

/// Execute package install
pub fn execute_package_install(packages: &[String]) -> Result<StepExecutionResult, String> {
    if packages.is_empty() {
        return Err("No packages specified".to_string());
    }

    let priv_status = check_privilege();
    if !priv_status.available {
        return Err(priv_status.message);
    }

    // Build args: pacman -S --noconfirm pkg1 pkg2 ...
    let mut args = vec!["-S", "--noconfirm"];
    let pkg_refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
    args.extend(pkg_refs.iter());

    let output = run_privileged("pacman", &args)?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let message = if success {
        format!("Successfully installed: {}", packages.join(", "))
    } else {
        format!("Failed to install packages: {}", stderr.trim())
    };

    Ok(StepExecutionResult {
        step_id: format!("pkg-install-{}", packages.join("-")),
        success,
        message,
        stdout: if stdout.is_empty() {
            None
        } else {
            Some(stdout)
        },
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        exit_code: output.status.code(),
    })
}

/// Execute package remove
pub fn execute_package_remove(packages: &[String]) -> Result<StepExecutionResult, String> {
    if packages.is_empty() {
        return Err("No packages specified".to_string());
    }

    let priv_status = check_privilege();
    if !priv_status.available {
        return Err(priv_status.message);
    }

    // Build args: pacman -R --noconfirm pkg1 pkg2 ...
    let mut args = vec!["-R", "--noconfirm"];
    let pkg_refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
    args.extend(pkg_refs.iter());

    let output = run_privileged("pacman", &args)?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let message = if success {
        format!("Successfully removed: {}", packages.join(", "))
    } else {
        format!("Failed to remove packages: {}", stderr.trim())
    };

    Ok(StepExecutionResult {
        step_id: format!("pkg-remove-{}", packages.join("-")),
        success,
        message,
        stdout: if stdout.is_empty() {
            None
        } else {
            Some(stdout)
        },
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        exit_code: output.status.code(),
    })
}

/// Create verification checks for package install
pub fn create_install_verification(packages: &[String]) -> Vec<VerificationCheck> {
    packages
        .iter()
        .map(|pkg| VerificationCheck {
            description: format!("Verify {} is installed", pkg),
            command: Some(format!("pacman -Q {}", pkg)),
            expected: format!("{} installed", pkg),
        })
        .collect()
}

/// Create verification checks for package remove
pub fn create_remove_verification(packages: &[String]) -> Vec<VerificationCheck> {
    packages
        .iter()
        .map(|pkg| VerificationCheck {
            description: format!("Verify {} is removed", pkg),
            command: Some(format!("pacman -Q {}", pkg)),
            expected: format!("{} not found (removed)", pkg),
        })
        .collect()
}

/// Create rollback steps for package install
pub fn create_install_rollback(packages: &[String]) -> Vec<RollbackStep> {
    vec![RollbackStep {
        description: format!("Remove installed packages: {}", packages.join(", ")),
        undo_action: format!("pacman -R --noconfirm {}", packages.join(" ")),
        backup_path: None,
    }]
}

/// Create rollback steps for package remove
pub fn create_remove_rollback(packages: &[String]) -> Vec<RollbackStep> {
    vec![RollbackStep {
        description: format!("Reinstall removed packages: {}", packages.join(", ")),
        undo_action: format!("pacman -S --noconfirm {}", packages.join(" ")),
        backup_path: None,
    }]
}

/// Get risk level for package action
pub fn get_package_action_risk(action: &PackageAction, packages: &[String]) -> MutationRiskLevel {
    // Installing is generally lower risk than removing
    // More packages = higher risk
    let count = packages.len();

    match action {
        PackageAction::Install => {
            if count > 5 {
                MutationRiskLevel::Medium
            } else {
                MutationRiskLevel::Low
            }
        }
        PackageAction::Remove => {
            if count > 3 {
                MutationRiskLevel::High
            } else {
                MutationRiskLevel::Medium
            }
        }
    }
}

/// Generate manual commands for user when privilege not available
pub fn generate_package_manual_commands(action: &PackageAction, packages: &[String]) -> Vec<String> {
    let cmd = match action {
        PackageAction::Install => format!("sudo pacman -S {}", packages.join(" ")),
        PackageAction::Remove => format!("sudo pacman -R {}", packages.join(" ")),
    };
    vec![cmd]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_package_action_risk() {
        let pkgs = vec!["foo".to_string()];
        assert_eq!(
            get_package_action_risk(&PackageAction::Install, &pkgs),
            MutationRiskLevel::Low
        );
        assert_eq!(
            get_package_action_risk(&PackageAction::Remove, &pkgs),
            MutationRiskLevel::Medium
        );

        let many_pkgs: Vec<String> = (0..10).map(|i| format!("pkg{}", i)).collect();
        assert_eq!(
            get_package_action_risk(&PackageAction::Install, &many_pkgs),
            MutationRiskLevel::Medium
        );
        assert_eq!(
            get_package_action_risk(&PackageAction::Remove, &many_pkgs),
            MutationRiskLevel::High
        );
    }

    #[test]
    fn test_create_install_verification() {
        let pkgs = vec!["vim".to_string(), "git".to_string()];
        let checks = create_install_verification(&pkgs);
        assert_eq!(checks.len(), 2);
        assert!(checks[0].description.contains("vim"));
        assert!(checks[1].description.contains("git"));
    }

    #[test]
    fn test_create_install_rollback() {
        let pkgs = vec!["vim".to_string(), "git".to_string()];
        let rollback = create_install_rollback(&pkgs);
        assert_eq!(rollback.len(), 1);
        assert!(rollback[0].undo_action.contains("pacman -R"));
        assert!(rollback[0].undo_action.contains("vim"));
    }

    #[test]
    fn test_generate_package_manual_commands() {
        let pkgs = vec!["vim".to_string()];
        let cmds = generate_package_manual_commands(&PackageAction::Install, &pkgs);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("pacman -S"));
        assert!(cmds[0].contains("vim"));
    }
}
