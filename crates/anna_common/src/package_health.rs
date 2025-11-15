//! Package Management Health Detection
//!
//! Detects package database integrity issues and file conflicts.
//!
//! ## Detection Capabilities
//!
//! ### Package Database Health
//! - Pacman database corruption detection
//! - Database lock file checking
//! - Package cache integrity
//! - Sync database freshness
//!
//! ### File Ownership Issues
//! - Unowned files in system directories
//! - Conflicting files (owned by multiple packages)
//! - Orphaned package files
//! - Modified package files
//!
//! ### Upgrade Health
//! - Partial upgrade detection
//! - Held back packages
//! - Package version conflicts
//! - Dependency conflicts
//!
//! ### Package Conflicts
//! - File conflicts between packages
//! - Unsatisfied dependencies
//! - Broken package dependencies
//!
//! ## Example
//!
//! ```rust
//! use anna_common::package_health::PackageHealth;
//!
//! let health = PackageHealth::detect();
//! println!("Database healthy: {}", health.database_healthy);
//! println!("Unowned files: {}", health.unowned_files.len());
//! println!("Conflicting files: {}", health.conflicting_files.len());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Complete package management health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHealth {
    /// Whether pacman database appears healthy
    pub database_healthy: bool,

    /// Database health issues found
    pub database_issues: Vec<DatabaseIssue>,

    /// Unowned files in system directories
    pub unowned_files: Vec<String>,

    /// Files owned by multiple packages (conflicts)
    pub conflicting_files: Vec<FileConflict>,

    /// Partial upgrade warnings
    pub partial_upgrade_warnings: Vec<String>,

    /// Packages held back from upgrade
    pub held_back_packages: Vec<HeldPackage>,

    /// Broken package dependencies
    pub broken_dependencies: Vec<BrokenDependency>,

    /// Modified package files (checksum mismatch)
    pub modified_files: Vec<ModifiedFile>,

    /// Pacman database lock status
    pub database_locked: bool,

    /// Last database sync timestamp
    pub last_sync: Option<String>,
}

/// A package database issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue description
    pub description: String,

    /// Affected package (if applicable)
    pub package: Option<String>,

    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Issue severity level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - requires immediate attention
    Critical,

    /// Warning - should be addressed soon
    Warning,

    /// Info - informational only
    Info,
}

/// A file owned by multiple packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflict {
    /// The conflicting file path
    pub file_path: String,

    /// Packages that claim ownership
    pub packages: Vec<String>,
}

/// A package held back from upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldPackage {
    /// Package name
    pub name: String,

    /// Current version
    pub current_version: String,

    /// Available version
    pub available_version: String,

    /// Reason for being held back
    pub reason: String,
}

/// A broken package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenDependency {
    /// Package with broken dependency
    pub package: String,

    /// Missing or incompatible dependency
    pub dependency: String,

    /// Dependency requirement
    pub requirement: String,

    /// Current state
    pub state: DependencyState,
}

/// Dependency state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyState {
    /// Dependency is missing entirely
    Missing,

    /// Dependency is installed but wrong version
    VersionMismatch,

    /// Dependency is in conflict
    Conflict,
}

/// A modified package file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedFile {
    /// Package name
    pub package: String,

    /// Modified file path
    pub file_path: String,

    /// Modification type
    pub modification_type: ModificationType,
}

/// Type of file modification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModificationType {
    /// File was modified (checksum mismatch)
    Modified,

    /// File was deleted
    Deleted,

    /// File has wrong permissions
    PermissionsMismatch,
}

impl PackageHealth {
    /// Detect all package management health issues
    pub fn detect() -> Self {
        let database_locked = check_database_lock();
        let database_issues = check_database_health();
        let database_healthy = database_issues.is_empty() && !database_locked;

        let unowned_files = find_unowned_files();
        let conflicting_files = find_conflicting_files();
        let partial_upgrade_warnings = check_partial_upgrades();
        let held_back_packages = find_held_back_packages();
        let broken_dependencies = find_broken_dependencies();
        let modified_files = find_modified_files();
        let last_sync = get_last_sync_time();

        Self {
            database_healthy,
            database_issues,
            unowned_files,
            conflicting_files,
            partial_upgrade_warnings,
            held_back_packages,
            broken_dependencies,
            modified_files,
            database_locked,
            last_sync,
        }
    }
}

/// Check if pacman database is locked
fn check_database_lock() -> bool {
    Path::new("/var/lib/pacman/db.lck").exists()
}

/// Check pacman database health
fn check_database_health() -> Vec<DatabaseIssue> {
    let mut issues = Vec::new();

    // Check if database directory exists
    if !Path::new("/var/lib/pacman/local").exists() {
        issues.push(DatabaseIssue {
            severity: IssueSeverity::Critical,
            description: "Pacman database directory is missing".to_string(),
            package: None,
            suggested_fix: Some("Run 'pacman -Syu' to rebuild database".to_string()),
        });
        return issues;
    }

    // Check for corrupt package databases
    if let Ok(output) = Command::new("pacman")
        .args(["-Qk"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("error") || stderr.contains("corrupt") {
            issues.push(DatabaseIssue {
                severity: IssueSeverity::Critical,
                description: "Pacman database corruption detected".to_string(),
                package: None,
                suggested_fix: Some("Run 'pacman -Syu' to repair database".to_string()),
            });
        }

        // Parse warnings from -Qk output
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("warning") && line.contains("missing file") {
                if let Some(package) = extract_package_from_qk_line(line) {
                    issues.push(DatabaseIssue {
                        severity: IssueSeverity::Warning,
                        description: format!("Package {} has missing files", package),
                        package: Some(package.clone()),
                        suggested_fix: Some(format!("Reinstall with 'pacman -S {}'", package)),
                    });
                }
            }
        }
    }

    // Check sync database freshness
    if let Ok(metadata) = fs::metadata("/var/lib/pacman/sync") {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                let days = elapsed.as_secs() / 86400;
                if days > 30 {
                    issues.push(DatabaseIssue {
                        severity: IssueSeverity::Warning,
                        description: format!("Package database is {} days old", days),
                        package: None,
                        suggested_fix: Some("Run 'pacman -Sy' to update database".to_string()),
                    });
                }
            }
        }
    }

    issues
}

/// Extract package name from pacman -Qk output line
fn extract_package_from_qk_line(line: &str) -> Option<String> {
    // Format: "package: /path/to/file (missing file)"
    line.split(':').next().map(|s| s.trim().to_string())
}

/// Find files in system directories not owned by any package
fn find_unowned_files() -> Vec<String> {
    let mut unowned = Vec::new();

    // Check common system directories for unowned files
    let check_dirs = vec![
        "/usr/bin",
        "/usr/lib",
        "/usr/share",
        "/etc",
    ];

    for dir in check_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten().take(100) { // Limit to prevent excessive checking
                if let Ok(path) = entry.path().canonicalize() {
                    let path_str = path.to_string_lossy();

                    // Check if file is owned by any package
                    if let Ok(output) = Command::new("pacman")
                        .args(["-Qo", &path_str])
                        .output()
                    {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.contains("error: No package owns") {
                            unowned.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    unowned
}

/// Find files owned by multiple packages
fn find_conflicting_files() -> Vec<FileConflict> {
    let mut conflicts = Vec::new();
    let mut file_owners: HashMap<String, Vec<String>> = HashMap::new();

    // Get list of all installed packages
    if let Ok(output) = Command::new("pacman").args(["-Q"]).output() {
        let packages = String::from_utf8_lossy(&output.stdout);

        for line in packages.lines().take(50) { // Limit packages checked to prevent slowness
            if let Some(package) = line.split_whitespace().next() {
                // Get files owned by this package
                if let Ok(output) = Command::new("pacman")
                    .args(["-Ql", package])
                    .output()
                {
                    let files = String::from_utf8_lossy(&output.stdout);
                    for file_line in files.lines() {
                        if let Some((_pkg, file)) = file_line.split_once(' ') {
                            let file = file.trim();
                            if !file.ends_with('/') { // Skip directories
                                file_owners
                                    .entry(file.to_string())
                                    .or_insert_with(Vec::new)
                                    .push(package.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Find files with multiple owners
    for (file, owners) in file_owners {
        if owners.len() > 1 {
            conflicts.push(FileConflict {
                file_path: file,
                packages: owners,
            });
        }
    }

    conflicts
}

/// Check for partial upgrade warnings
fn check_partial_upgrades() -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for packages that need updating
    if let Ok(output) = Command::new("checkupdates").output() {
        let updates = String::from_utf8_lossy(&output.stdout);
        let update_count = updates.lines().count();

        if update_count > 0 {
            warnings.push(format!(
                "{} packages have available updates",
                update_count
            ));
        }

        // Check for core system package updates
        for line in updates.lines() {
            if line.contains("linux ") || line.contains("systemd ") || line.contains("pacman ") {
                warnings.push(format!("Critical system package needs update: {}", line));
            }
        }
    }

    warnings
}

/// Find packages held back from upgrade
fn find_held_back_packages() -> Vec<HeldPackage> {
    let mut held_back = Vec::new();

    // Check for packages in IgnorePkg
    if let Ok(content) = fs::read_to_string("/etc/pacman.conf") {
        for line in content.lines() {
            if line.trim_start().starts_with("IgnorePkg") {
                if let Some(packages) = line.split('=').nth(1) {
                    for pkg in packages.split_whitespace() {
                        // Get current version
                        if let Ok(output) = Command::new("pacman")
                            .args(["-Q", pkg])
                            .output()
                        {
                            if output.status.success() {
                                let info = String::from_utf8_lossy(&output.stdout);
                                if let Some((name, version)) = info.trim().split_once(' ') {
                                    // Try to get available version
                                    if let Ok(sync_output) = Command::new("pacman")
                                        .args(["-Si", pkg])
                                        .output()
                                    {
                                        let sync_info = String::from_utf8_lossy(&sync_output.stdout);
                                        let available_version = sync_info
                                            .lines()
                                            .find(|l| l.trim_start().starts_with("Version"))
                                            .and_then(|l| l.split(':').nth(1))
                                            .map(|v| v.trim().to_string())
                                            .unwrap_or_else(|| version.to_string());

                                        held_back.push(HeldPackage {
                                            name: name.to_string(),
                                            current_version: version.to_string(),
                                            available_version,
                                            reason: "Listed in IgnorePkg".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    held_back
}

/// Find broken package dependencies
fn find_broken_dependencies() -> Vec<BrokenDependency> {
    let mut broken = Vec::new();

    // Run pacman -Dk to check for dependency issues
    if let Ok(output) = Command::new("pacman")
        .args(["-Dk"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("requires") && line.contains("missing") {
                if let Some(dep) = parse_dependency_error(line) {
                    broken.push(dep);
                }
            }
        }
    }

    broken
}

/// Parse dependency error from pacman output
fn parse_dependency_error(line: &str) -> Option<BrokenDependency> {
    // Format: "package: requires dependency"
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 2 {
        let package = parts[0].trim();
        let rest = parts[1].trim();

        if rest.starts_with("requires") {
            let dep_parts: Vec<&str> = rest.split_whitespace().collect();
            if dep_parts.len() >= 2 {
                let dependency = dep_parts[1].to_string();

                return Some(BrokenDependency {
                    package: package.to_string(),
                    dependency: dependency.clone(),
                    requirement: rest.to_string(),
                    state: DependencyState::Missing,
                });
            }
        }
    }

    None
}

/// Find files modified from package original
fn find_modified_files() -> Vec<ModifiedFile> {
    let mut modified = Vec::new();

    // Run pacman -Qk to check all packages (limited sample)
    if let Ok(output) = Command::new("sh")
        .args([
            "-c",
            "pacman -Q | head -20 | awk '{print $1}' | xargs pacman -Qk 2>&1"
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut current_package = String::new();

        for line in stdout.lines() {
            // Package headers
            if let Some(pkg) = line.strip_suffix(": ") {
                current_package = pkg.to_string();
                continue;
            }

            // Modified file entries
            if !current_package.is_empty() {
                if line.contains("missing file") {
                    if let Some(path) = extract_file_path(line) {
                        modified.push(ModifiedFile {
                            package: current_package.clone(),
                            file_path: path,
                            modification_type: ModificationType::Deleted,
                        });
                    }
                } else if line.contains("modified") {
                    if let Some(path) = extract_file_path(line) {
                        modified.push(ModifiedFile {
                            package: current_package.clone(),
                            file_path: path,
                            modification_type: ModificationType::Modified,
                        });
                    }
                }
            }
        }
    }

    modified
}

/// Extract file path from pacman -Qk output line
fn extract_file_path(line: &str) -> Option<String> {
    // Format: "    /path/to/file (modified)"
    let parts: Vec<&str> = line.trim().split('(').collect();
    if !parts.is_empty() {
        return Some(parts[0].trim().to_string());
    }
    None
}

/// Get last sync time
fn get_last_sync_time() -> Option<String> {
    if let Ok(metadata) = fs::metadata("/var/lib/pacman/sync/core.db") {
        if let Ok(modified) = metadata.modified() {
            if let Ok(datetime) = modified.duration_since(std::time::UNIX_EPOCH) {
                // Convert to readable timestamp
                return Some(format!("{} seconds ago", datetime.as_secs()));
            }
        }
    }
    None
}
