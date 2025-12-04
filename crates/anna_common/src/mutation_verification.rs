//! Mutation Verification for Anna v0.0.81
//!
//! Real verification that checks actual system state, not placeholders.
//! - Service verification: uses systemctl is-active/is-enabled
//! - Package verification: uses pacman -Qi
//! - Config verification: checks file content and backup existence

use crate::mutation_engine_v1::{
    ConfigEditOp, MutationDetail, MutationPlanV1, ServiceAction, VerificationResult,
};
use crate::service_mutation::get_service_state;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Detailed verification result with actual vs expected values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedVerificationResult {
    /// Check description
    pub description: String,
    /// Whether verification passed
    pub passed: bool,
    /// Expected value/state
    pub expected: String,
    /// Actual value/state found
    pub actual: String,
    /// Additional diagnostic info
    pub diagnostic: Option<String>,
}

/// Run all verifications for a mutation plan, returning detailed results
pub fn verify_mutation_plan_detailed(
    plan: &MutationPlanV1,
) -> Vec<DetailedVerificationResult> {
    let mut results = Vec::new();
    for step in &plan.steps {
        results.extend(verify_mutation_detail(&step.mutation));
    }
    results
}

/// Run all verifications for a mutation plan
pub fn verify_mutation_plan(plan: &MutationPlanV1) -> Vec<VerificationResult> {
    let detailed = verify_mutation_plan_detailed(plan);
    detailed
        .into_iter()
        .map(|d| VerificationResult {
            description: d.description,
            passed: d.passed,
            actual: d.actual,
        })
        .collect()
}

/// Verify a specific mutation detail
fn verify_mutation_detail(detail: &MutationDetail) -> Vec<DetailedVerificationResult> {
    match detail {
        MutationDetail::ServiceControl { service, action } => {
            verify_service_action(service, action)
        }
        MutationDetail::PackageInstall { packages } => {
            verify_package_install(packages)
        }
        MutationDetail::PackageRemove { packages } => {
            verify_package_remove(packages)
        }
        MutationDetail::ConfigEdit { path, operation } => {
            verify_config_edit(path, operation)
        }
    }
}

// =============================================================================
// Service Verification (A1)
// =============================================================================

/// Verify service action based on actual systemd state
pub fn verify_service_action(
    service: &str,
    action: &ServiceAction,
) -> Vec<DetailedVerificationResult> {
    let state = get_service_state(service);
    let mut results = Vec::new();

    match action {
        ServiceAction::Start => {
            results.push(DetailedVerificationResult {
                description: format!("Verify {} is running", service),
                passed: state.active_state == "active",
                expected: "active".to_string(),
                actual: state.active_state.clone(),
                diagnostic: if state.active_state != "active" {
                    Some(get_service_status_diagnostic(service))
                } else {
                    None
                },
            });
        }
        ServiceAction::Stop => {
            let passed = state.active_state == "inactive" || state.active_state == "failed";
            results.push(DetailedVerificationResult {
                description: format!("Verify {} is stopped", service),
                passed,
                expected: "inactive".to_string(),
                actual: state.active_state.clone(),
                diagnostic: if !passed {
                    Some(get_service_status_diagnostic(service))
                } else {
                    None
                },
            });
        }
        ServiceAction::Restart => {
            results.push(DetailedVerificationResult {
                description: format!("Verify {} is running after restart", service),
                passed: state.active_state == "active",
                expected: "active".to_string(),
                actual: state.active_state.clone(),
                diagnostic: if state.active_state != "active" {
                    Some(get_service_status_diagnostic(service))
                } else {
                    None
                },
            });
        }
        ServiceAction::Enable => {
            results.push(DetailedVerificationResult {
                description: format!("Verify {} is enabled at boot", service),
                passed: state.enabled_state == "enabled",
                expected: "enabled".to_string(),
                actual: state.enabled_state.clone(),
                diagnostic: None,
            });
        }
        ServiceAction::Disable => {
            let passed = state.enabled_state == "disabled" || state.enabled_state == "masked";
            results.push(DetailedVerificationResult {
                description: format!("Verify {} is disabled at boot", service),
                passed,
                expected: "disabled".to_string(),
                actual: state.enabled_state.clone(),
                diagnostic: None,
            });
        }
    }

    results
}

/// Get diagnostic info from systemctl status
fn get_service_status_diagnostic(service: &str) -> String {
    let output = Command::new("systemctl")
        .args(["status", service, "--no-pager", "-l"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let lines: Vec<&str> = stdout.lines().take(5).collect();
            if lines.is_empty() {
                stderr.trim().to_string()
            } else {
                lines.join("\n")
            }
        }
        Err(e) => format!("Failed to get status: {}", e),
    }
}

// =============================================================================
// Package Verification (A2)
// =============================================================================

/// Verify packages were installed
pub fn verify_package_install(packages: &[String]) -> Vec<DetailedVerificationResult> {
    let mut results = Vec::new();

    for pkg in packages {
        let (installed, version) = check_package_installed(pkg);

        results.push(DetailedVerificationResult {
            description: format!("Verify {} is installed", pkg),
            passed: installed,
            expected: "installed".to_string(),
            actual: if installed {
                format!("installed ({})", version.unwrap_or_else(|| "?".to_string()))
            } else {
                "not installed".to_string()
            },
            diagnostic: if !installed {
                Some(format!("pacman -Qi {} returned non-zero", pkg))
            } else {
                None
            },
        });
    }

    results
}

/// Verify packages were removed
pub fn verify_package_remove(packages: &[String]) -> Vec<DetailedVerificationResult> {
    let mut results = Vec::new();

    for pkg in packages {
        let (installed, version) = check_package_installed(pkg);

        results.push(DetailedVerificationResult {
            description: format!("Verify {} is removed", pkg),
            passed: !installed,
            expected: "not installed".to_string(),
            actual: if installed {
                format!("still installed ({})", version.unwrap_or_else(|| "?".to_string()))
            } else {
                "not installed".to_string()
            },
            diagnostic: if installed {
                Some(format!("Package {} still present after removal", pkg))
            } else {
                None
            },
        });
    }

    results
}

/// Check if a package is installed and get its version
fn check_package_installed(package: &str) -> (bool, Option<String>) {
    let output = Command::new("pacman").args(["-Qi", package]).output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let version = stdout
                    .lines()
                    .find(|line| line.starts_with("Version"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|v| v.trim().to_string());
                (true, version)
            } else {
                (false, None)
            }
        }
        Err(_) => (false, None),
    }
}

/// Get package info for preview (B2: Smart Previews)
pub fn get_package_info_detailed(package: &str) -> PackageInfoDetailed {
    let repo_info = Command::new("pacman").args(["-Si", package]).output().ok();
    let (installed, installed_version) = check_package_installed(package);

    let mut info = PackageInfoDetailed {
        name: package.to_string(),
        installed,
        installed_version,
        repo_version: None,
        repository: None,
        description: None,
        download_size: None,
        installed_size: None,
        depends: Vec::new(),
        found_in_repos: false,
    };

    if let Some(output) = repo_info {
        if output.status.success() {
            info.found_in_repos = true;
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Version") {
                    info.repo_version = line.split(':').nth(1).map(|v| v.trim().to_string());
                } else if line.starts_with("Repository") {
                    info.repository = line.split(':').nth(1).map(|v| v.trim().to_string());
                } else if line.starts_with("Description") {
                    info.description = line.split(':').nth(1).map(|v| v.trim().to_string());
                } else if line.starts_with("Download Size") {
                    info.download_size = line.split(':').nth(1).map(|v| v.trim().to_string());
                } else if line.starts_with("Installed Size") {
                    info.installed_size = line.split(':').nth(1).map(|v| v.trim().to_string());
                } else if line.starts_with("Depends On") {
                    if let Some(deps) = line.split(':').nth(1) {
                        info.depends = deps
                            .trim()
                            .split_whitespace()
                            .take(5) // Limit to first 5 deps for preview
                            .map(|s| s.to_string())
                            .collect();
                    }
                }
            }
        }
    }

    info
}

/// Detailed package info for smart previews (B2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfoDetailed {
    pub name: String,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub repo_version: Option<String>,
    pub repository: Option<String>,
    pub description: Option<String>,
    pub download_size: Option<String>,
    pub installed_size: Option<String>,
    pub depends: Vec<String>,
    pub found_in_repos: bool,
}

impl PackageInfoDetailed {
    /// Format for human-readable preview
    pub fn format_preview(&self) -> String {
        let mut lines = Vec::new();

        if !self.found_in_repos && !self.installed {
            lines.push(format!("Package '{}' not found in repositories", self.name));
            return lines.join("\n");
        }

        lines.push(format!("Package: {}", self.name));

        if let Some(ref desc) = self.description {
            lines.push(format!("  Description: {}", desc));
        }

        if let Some(ref repo) = self.repository {
            lines.push(format!("  Repository: {}", repo));
        }

        if self.installed {
            lines.push(format!(
                "  Status: installed ({})",
                self.installed_version.as_deref().unwrap_or("?")
            ));
        } else {
            lines.push("  Status: not installed".to_string());
        }

        if let Some(ref ver) = self.repo_version {
            lines.push(format!("  Available version: {}", ver));
        }

        if let Some(ref size) = self.download_size {
            lines.push(format!("  Download size: {}", size));
        }

        if let Some(ref size) = self.installed_size {
            lines.push(format!("  Installed size: {}", size));
        }

        if !self.depends.is_empty() {
            let deps_str = self.depends.join(", ");
            let suffix = if self.depends.len() >= 5 { "..." } else { "" };
            lines.push(format!("  Dependencies: {}{}", deps_str, suffix));
        }

        lines.join("\n")
    }
}

// =============================================================================
// Config Verification (A3)
// =============================================================================

/// Verify config edit
pub fn verify_config_edit(
    path: &str,
    op: &ConfigEditOp,
) -> Vec<DetailedVerificationResult> {
    let mut results = Vec::new();

    // 1. Verify file exists
    let file_exists = Path::new(path).exists();
    results.push(DetailedVerificationResult {
        description: format!("Verify {} exists", path),
        passed: file_exists,
        expected: "file exists".to_string(),
        actual: if file_exists { "exists" } else { "missing" }.to_string(),
        diagnostic: None,
    });

    if !file_exists {
        return results;
    }

    // 2. Verify edit was applied by checking file content
    if let Ok(content) = fs::read_to_string(path) {
        let edit_applied = match op {
            ConfigEditOp::AddLine { line } => content.lines().any(|l| l.trim() == line.trim()),
            ConfigEditOp::ReplaceLine { new, .. } => {
                content.lines().any(|l| l.trim() == new.trim())
            }
            ConfigEditOp::CommentLine { pattern } => content.lines().any(|l| {
                let trimmed = l.trim();
                trimmed.starts_with('#')
                    && trimmed.trim_start_matches('#').trim() == pattern.trim()
            }),
            ConfigEditOp::UncommentLine { pattern } => {
                let pattern_clean = pattern.trim().trim_start_matches('#');
                content.lines().any(|l| l.trim() == pattern_clean)
            }
        };

        let op_desc = match op {
            ConfigEditOp::AddLine { line } => format!("line '{}' added", truncate(line, 40)),
            ConfigEditOp::ReplaceLine { old, new } => {
                format!("'{}' replaced with '{}'", truncate(old, 20), truncate(new, 20))
            }
            ConfigEditOp::CommentLine { pattern } => {
                format!("'{}' commented", truncate(pattern, 30))
            }
            ConfigEditOp::UncommentLine { pattern } => {
                format!("'{}' uncommented", truncate(pattern, 30))
            }
        };

        results.push(DetailedVerificationResult {
            description: format!("Verify {} in {}", op_desc, path),
            passed: edit_applied,
            expected: "edit applied".to_string(),
            actual: if edit_applied {
                "verified".to_string()
            } else {
                "not found in file".to_string()
            },
            diagnostic: if !edit_applied {
                Some("Expected content not found after edit".to_string())
            } else {
                None
            },
        });
    }

    // 3. Check file permissions are sane
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        let mode = std::os::unix::fs::PermissionsExt::mode(&permissions);
        let is_world_writable = mode & 0o002 != 0;

        results.push(DetailedVerificationResult {
            description: format!("Verify {} has safe permissions", path),
            passed: !is_world_writable,
            expected: "not world-writable".to_string(),
            actual: format!("mode {:o}", mode & 0o777),
            diagnostic: if is_world_writable {
                Some("Warning: file is world-writable".to_string())
            } else {
                None
            },
        });
    }

    results
}

/// Check if backup exists for a config file
pub fn verify_backup_exists(path: &str, case_id: &str) -> DetailedVerificationResult {
    let filename = path.replace('/', "_").trim_start_matches('_').to_string();
    let backup_path = format!("/var/lib/anna/rollback/files/{}/{}", case_id, filename);

    let exists = Path::new(&backup_path).exists();

    DetailedVerificationResult {
        description: format!("Verify backup exists for {}", path),
        passed: exists,
        expected: "backup file exists".to_string(),
        actual: if exists {
            format!("backup at {}", backup_path)
        } else {
            "no backup found".to_string()
        },
        diagnostic: if !exists {
            Some(format!("Expected backup at {}", backup_path))
        } else {
            None
        },
    }
}

/// Truncate string for display
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

// =============================================================================
// Service Unit Resolution (B3)
// =============================================================================

/// Resolve a service name to its systemd unit
pub fn resolve_service_unit(name: &str) -> ServiceUnitResolution {
    let unit_name = match name.to_lowercase().as_str() {
        "docker" => "docker.service",
        "networkmanager" | "network-manager" | "nm" => "NetworkManager.service",
        "sshd" | "ssh" | "openssh" => "sshd.service",
        "bluetooth" | "bt" => "bluetooth.service",
        _ => {
            if name.ends_with(".service")
                || name.ends_with(".socket")
                || name.ends_with(".timer")
            {
                name
            } else {
                return ServiceUnitResolution {
                    input: name.to_string(),
                    resolved_unit: format!("{}.service", name),
                    mapping_reason: "assumed .service suffix".to_string(),
                    exists: check_unit_exists(&format!("{}.service", name)),
                };
            }
        }
    };

    ServiceUnitResolution {
        input: name.to_string(),
        resolved_unit: unit_name.to_string(),
        mapping_reason: if name == unit_name {
            "exact match".to_string()
        } else {
            format!("'{}' maps to '{}'", name, unit_name)
        },
        exists: check_unit_exists(unit_name),
    }
}

/// Check if a systemd unit exists
fn check_unit_exists(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["cat", unit])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Service unit resolution result (B3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUnitResolution {
    pub input: String,
    pub resolved_unit: String,
    pub mapping_reason: String,
    pub exists: bool,
}

impl ServiceUnitResolution {
    /// Format for human-readable preview
    pub fn format_human(&self) -> String {
        if self.input == self.resolved_unit {
            format!("Using unit: {}", self.resolved_unit)
        } else {
            format!(
                "I'm treating '{}' as {} ({})",
                self.input, self.resolved_unit, self.mapping_reason
            )
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_service_unit() {
        let res = resolve_service_unit("docker");
        assert_eq!(res.resolved_unit, "docker.service");

        let res = resolve_service_unit("networkmanager");
        assert_eq!(res.resolved_unit, "NetworkManager.service");

        let res = resolve_service_unit("sshd.service");
        assert_eq!(res.resolved_unit, "sshd.service");

        let res = resolve_service_unit("custom");
        assert_eq!(res.resolved_unit, "custom.service");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_package_info_format() {
        let info = PackageInfoDetailed {
            name: "vim".to_string(),
            installed: true,
            installed_version: Some("9.0.0".to_string()),
            repo_version: Some("9.0.1".to_string()),
            repository: Some("extra".to_string()),
            description: Some("Vi Improved".to_string()),
            download_size: Some("1.5 MiB".to_string()),
            installed_size: Some("3.2 MiB".to_string()),
            depends: vec!["glibc".to_string(), "ncurses".to_string()],
            found_in_repos: true,
        };

        let preview = info.format_preview();
        assert!(preview.contains("vim"));
        assert!(preview.contains("Vi Improved"));
        assert!(preview.contains("9.0.1"));
    }

    #[test]
    fn test_package_not_found_format() {
        let info = PackageInfoDetailed {
            name: "nonexistent-pkg".to_string(),
            installed: false,
            installed_version: None,
            repo_version: None,
            repository: None,
            description: None,
            download_size: None,
            installed_size: None,
            depends: Vec::new(),
            found_in_repos: false,
        };

        let preview = info.format_preview();
        assert!(preview.contains("not found"));
    }
}
