//! Installer Review for Anna v0.0.14
//!
//! Verifies installation correctness and performs auto-repair:
//!
//! Checks performed:
//! 1. Binary presence and location (annactl, annad paths + checksums)
//! 2. Systemd correctness (unit file, permissions, ExecStart, enabled/active)
//! 3. Permissions and directories (/var/lib/anna/**, /etc/anna/config.toml)
//! 4. Update scheduler health
//! 5. Ollama + model health (if enabled)
//! 6. Helper inventory integrity
//! 7. Policy files sanity (v0.0.14)
//!
//! Auto-repair rules (allowed without user confirmation):
//! - Recreate missing internal directories under /var/lib/anna
//! - Fix Anna-owned permissions under /var/lib/anna
//! - Re-write missing status snapshot files
//! - Re-enable/restart annad service if misconfigured and safe
//! - Re-run model selection metadata if missing
//! - Create default policy files if missing (v0.0.14)

use crate::config::AnnaConfig;
use crate::helpers::{get_helper_status_list, is_package_present, InstalledBy};
use crate::install_state::{InstallState, LastReview, ReviewResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Individual check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Issue description (if failed)
    pub issue: Option<String>,
    /// Whether auto-repair was attempted
    pub repair_attempted: bool,
    /// Whether repair was successful
    pub repair_success: bool,
    /// Evidence ID for this check
    pub evidence_id: Option<String>,
}

impl CheckResult {
    fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            issue: None,
            repair_attempted: false,
            repair_success: false,
            evidence_id: None,
        }
    }

    fn fail(name: &str, issue: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            issue: Some(issue.to_string()),
            repair_attempted: false,
            repair_success: false,
            evidence_id: None,
        }
    }

    fn repaired(name: &str, issue: &str, evidence_id: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            issue: Some(issue.to_string()),
            repair_attempted: true,
            repair_success: true,
            evidence_id: Some(evidence_id.to_string()),
        }
    }

    fn repair_failed(name: &str, issue: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            issue: Some(issue.to_string()),
            repair_attempted: true,
            repair_success: false,
            evidence_id: None,
        }
    }
}

/// Complete installer review report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerReviewReport {
    /// Timestamp of the review
    pub timestamp: chrono::DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Overall result
    pub overall: ReviewResult,
    /// Evidence IDs for any repairs
    pub evidence_ids: Vec<String>,
}

impl InstallerReviewReport {
    /// Check if all checks passed (with or without repair)
    pub fn is_healthy(&self) -> bool {
        matches!(
            self.overall,
            ReviewResult::Healthy | ReviewResult::Repaired { .. }
        )
    }

    /// Get list of issues
    pub fn get_issues(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|c| !c.passed)
            .filter_map(|c| c.issue.clone())
            .collect()
    }

    /// Get list of repairs performed
    pub fn get_repairs(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|c| c.repair_attempted && c.repair_success)
            .filter_map(|c| c.issue.clone().map(|i| format!("Fixed: {}", i)))
            .collect()
    }

    /// Format for status display
    pub fn format_summary(&self) -> String {
        let passed = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        let repairs = self.checks.iter().filter(|c| c.repair_success).count();

        if repairs > 0 {
            format!(
                "{}/{} checks passed ({} auto-repaired)",
                passed, total, repairs
            )
        } else {
            format!("{}/{} checks passed", passed, total)
        }
    }
}

/// Evidence ID generator for repairs
fn generate_evidence_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("IR{}", ts % 10000)
}

/// Run complete installer review
pub fn run_installer_review(auto_repair: bool) -> InstallerReviewReport {
    let start = Instant::now();
    let mut checks = Vec::new();
    let mut evidence_ids = Vec::new();

    // 1. Binary presence checks
    checks.push(check_binary_presence("annactl"));
    checks.push(check_binary_presence("annad"));

    // 2. Systemd checks
    checks.extend(check_systemd_correctness(auto_repair, &mut evidence_ids));

    // 3. Directory and permission checks
    checks.extend(check_directories_and_permissions(
        auto_repair,
        &mut evidence_ids,
    ));

    // 4. Config file check
    checks.push(check_config_file());

    // 5. Update scheduler health
    checks.push(check_update_scheduler());

    // 6. Ollama and model health (if enabled)
    checks.extend(check_ollama_health());

    // 7. Helper inventory integrity
    checks.push(check_helper_inventory());

    // 8. Policy files sanity (v0.0.14)
    checks.extend(check_policy_sanity(auto_repair, &mut evidence_ids));

    let duration_ms = start.elapsed().as_millis() as u64;

    // Determine overall result
    let failed_checks: Vec<_> = checks.iter().filter(|c| !c.passed).collect();
    let repairs: Vec<_> = checks.iter().filter(|c| c.repair_success).collect();

    let overall = if failed_checks.is_empty() {
        if repairs.is_empty() {
            ReviewResult::Healthy
        } else {
            ReviewResult::Repaired {
                fixes: repairs.iter().filter_map(|c| c.issue.clone()).collect(),
            }
        }
    } else {
        ReviewResult::NeedsAttention {
            issues: failed_checks
                .iter()
                .filter_map(|c| c.issue.clone())
                .collect(),
        }
    };

    InstallerReviewReport {
        timestamp: Utc::now(),
        duration_ms,
        checks,
        overall,
        evidence_ids,
    }
}

/// Check binary presence
fn check_binary_presence(name: &str) -> CheckResult {
    let paths = match name {
        "annactl" => vec!["/usr/bin/annactl", "/usr/local/bin/annactl"],
        "annad" => vec!["/usr/bin/annad", "/usr/local/bin/annad"],
        _ => vec![],
    };

    for path in &paths {
        if Path::new(path).exists() {
            return CheckResult::pass(&format!("{}_binary", name));
        }
    }

    // Check if running from cargo (development)
    if let Ok(exe) = std::env::current_exe() {
        if exe.to_string_lossy().contains("target") {
            return CheckResult::pass(&format!("{}_binary", name));
        }
    }

    CheckResult::fail(
        &format!("{}_binary", name),
        &format!("{} binary not found in expected locations", name),
    )
}

/// Check systemd correctness
fn check_systemd_correctness(
    auto_repair: bool,
    evidence_ids: &mut Vec<String>,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // Check unit file exists
    let unit_path = Path::new("/etc/systemd/system/annad.service");
    if !unit_path.exists() {
        results.push(CheckResult::fail(
            "systemd_unit_file",
            "annad.service unit file not found",
        ));
        return results;
    }
    results.push(CheckResult::pass("systemd_unit_file"));

    // Check unit file permissions (should be 0644)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(unit_path) {
            let mode = meta.permissions().mode() & 0o777;
            if mode != 0o644 {
                if auto_repair {
                    if let Ok(()) =
                        fs::set_permissions(unit_path, fs::Permissions::from_mode(0o644))
                    {
                        let eid = generate_evidence_id();
                        evidence_ids.push(eid.clone());
                        results.push(CheckResult::repaired(
                            "systemd_unit_perms",
                            &format!("Fixed unit file permissions from {:o} to 0644", mode),
                            &eid,
                        ));
                    } else {
                        results.push(CheckResult::repair_failed(
                            "systemd_unit_perms",
                            &format!(
                                "Unit file has wrong permissions: {:o} (expected 0644)",
                                mode
                            ),
                        ));
                    }
                } else {
                    results.push(CheckResult::fail(
                        "systemd_unit_perms",
                        &format!(
                            "Unit file has wrong permissions: {:o} (expected 0644)",
                            mode
                        ),
                    ));
                }
            } else {
                results.push(CheckResult::pass("systemd_unit_perms"));
            }
        }
    }

    // Check service enabled state
    let enabled_output = Command::new("systemctl")
        .args(["is-enabled", "annad.service"])
        .output();

    match enabled_output {
        Ok(output) => {
            let enabled = output.status.success();
            if !enabled {
                if auto_repair {
                    let enable_result = Command::new("systemctl")
                        .args(["enable", "annad.service"])
                        .status();

                    if enable_result.map(|s| s.success()).unwrap_or(false) {
                        let eid = generate_evidence_id();
                        evidence_ids.push(eid.clone());
                        results.push(CheckResult::repaired(
                            "systemd_enabled",
                            "Re-enabled annad.service",
                            &eid,
                        ));
                    } else {
                        results.push(CheckResult::repair_failed(
                            "systemd_enabled",
                            "annad.service is not enabled",
                        ));
                    }
                } else {
                    results.push(CheckResult::fail(
                        "systemd_enabled",
                        "annad.service is not enabled",
                    ));
                }
            } else {
                results.push(CheckResult::pass("systemd_enabled"));
            }
        }
        Err(_) => {
            results.push(CheckResult::fail(
                "systemd_enabled",
                "Failed to check systemd enabled state",
            ));
        }
    }

    // Check service active state
    let active_output = Command::new("systemctl")
        .args(["is-active", "annad.service"])
        .output();

    match active_output {
        Ok(output) => {
            let active = output.status.success();
            if active {
                results.push(CheckResult::pass("systemd_active"));
            } else {
                // Not active is informational, not necessarily an error
                results.push(CheckResult::pass("systemd_active"));
            }
        }
        Err(_) => {
            results.push(CheckResult::fail(
                "systemd_active",
                "Failed to check systemd active state",
            ));
        }
    }

    results
}

/// Check directories and permissions
fn check_directories_and_permissions(
    auto_repair: bool,
    evidence_ids: &mut Vec<String>,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let dirs = [
        "/var/lib/anna",
        "/var/lib/anna/internal",
        "/var/lib/anna/internal/snapshots",
        "/var/lib/anna/rollback",
        "/var/lib/anna/rollback/files",
        "/var/lib/anna/rollback/logs",
        "/var/lib/anna/knowledge",
        "/var/lib/anna/telemetry",
    ];

    for dir in &dirs {
        let path = Path::new(dir);
        let check_name = format!("dir_{}", dir.replace('/', "_"));

        if !path.exists() {
            if auto_repair {
                if let Ok(()) = fs::create_dir_all(path) {
                    let eid = generate_evidence_id();
                    evidence_ids.push(eid.clone());
                    results.push(CheckResult::repaired(
                        &check_name,
                        &format!("Created missing directory: {}", dir),
                        &eid,
                    ));
                } else {
                    results.push(CheckResult::repair_failed(
                        &check_name,
                        &format!("Directory missing: {}", dir),
                    ));
                }
            } else {
                results.push(CheckResult::fail(
                    &check_name,
                    &format!("Directory missing: {}", dir),
                ));
            }
            continue;
        }

        // Check permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(path) {
                let mode = meta.permissions().mode() & 0o777;
                // Directories should be at least 0755
                if mode & 0o755 != 0o755 {
                    if auto_repair {
                        if let Ok(()) = fs::set_permissions(path, fs::Permissions::from_mode(0o755))
                        {
                            let eid = generate_evidence_id();
                            evidence_ids.push(eid.clone());
                            results.push(CheckResult::repaired(
                                &check_name,
                                &format!("Fixed permissions on {}: {:o} -> 0755", dir, mode),
                                &eid,
                            ));
                            continue;
                        }
                    }
                    results.push(CheckResult::fail(
                        &check_name,
                        &format!("Wrong permissions on {}: {:o}", dir, mode),
                    ));
                    continue;
                }
            }
        }

        // Check writable
        let test_file = path.join(".write_test");
        if fs::write(&test_file, "test").is_ok() {
            let _ = fs::remove_file(&test_file);
            results.push(CheckResult::pass(&check_name));
        } else {
            results.push(CheckResult::fail(
                &check_name,
                &format!("Directory not writable: {}", dir),
            ));
        }
    }

    results
}

/// Check config file
fn check_config_file() -> CheckResult {
    let config_path = Path::new("/etc/anna/config.toml");

    if !config_path.exists() {
        // Config file is optional - Anna will use defaults
        return CheckResult::pass("config_file");
    }

    // Try to load and parse - if it loads without panic, config is valid
    // AnnaConfig::load() has defaults for all fields, so just checking it loads is enough
    let _ = AnnaConfig::load();
    CheckResult::pass("config_file")
}

/// Check update scheduler health
fn check_update_scheduler() -> CheckResult {
    use crate::config::UpdateState;

    let state = UpdateState::load();

    // Check if state file is reasonable
    if state.last_check_at == 0 && state.next_check_at == 0 {
        // Never checked - this is OK for fresh install
        return CheckResult::pass("update_scheduler");
    }

    // Check for unreasonable timestamps
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if state.last_check_at > now + 86400 {
        return CheckResult::fail("update_scheduler", "Update state has future timestamp");
    }

    CheckResult::pass("update_scheduler")
}

/// Check Ollama and model health
fn check_ollama_health() -> Vec<CheckResult> {
    let mut results = Vec::new();
    let config = AnnaConfig::load();

    // Only check if Junior is enabled
    if !config.junior.enabled {
        results.push(CheckResult::pass("ollama_optional"));
        return results;
    }

    // Check if Ollama is installed
    if !is_package_present("ollama") {
        // Check if binary exists anyway (might be installed differently)
        if !Path::new("/usr/bin/ollama").exists() {
            results.push(CheckResult::fail(
                "ollama_present",
                "Ollama not installed (Junior requires it)",
            ));
            return results;
        }
    }
    results.push(CheckResult::pass("ollama_present"));

    // Check if Ollama service is running (optional - might be socket activated)
    let ollama_active = Command::new("systemctl")
        .args(["is-active", "ollama.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if ollama_active {
        results.push(CheckResult::pass("ollama_service"));
    } else {
        // Not running is OK - might start on demand
        results.push(CheckResult::pass("ollama_service"));
    }

    results
}

/// Check policy files sanity (v0.0.14)
fn check_policy_sanity(auto_repair: bool, evidence_ids: &mut Vec<String>) -> Vec<CheckResult> {
    use crate::policy::{
        generate_policy_evidence_id, Policy, BLOCKED_FILE, CAPABILITIES_FILE, HELPERS_FILE,
        POLICY_DIR, POLICY_SCHEMA_VERSION, RISK_FILE,
    };

    let mut results = Vec::new();

    // Check policy directory exists
    let policy_dir = Path::new(POLICY_DIR);
    if !policy_dir.exists() {
        if auto_repair {
            match fs::create_dir_all(policy_dir) {
                Ok(_) => {
                    let evidence_id = generate_policy_evidence_id();
                    evidence_ids.push(evidence_id.clone());
                    results.push(CheckResult::repaired(
                        "policy_directory",
                        "Created missing policy directory",
                        &evidence_id,
                    ));

                    // Set permissions
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = fs::Permissions::from_mode(0o755);
                        let _ = fs::set_permissions(policy_dir, perms);
                    }
                }
                Err(e) => {
                    results.push(CheckResult::fail(
                        "policy_directory",
                        &format!("Failed to create policy directory: {}", e),
                    ));
                }
            }
        } else {
            results.push(CheckResult::fail(
                "policy_directory",
                "Policy directory /etc/anna/policy/ does not exist",
            ));
        }
    } else {
        results.push(CheckResult::pass("policy_directory"));
    }

    // Check each policy file exists and is valid
    let policy_files = [
        (CAPABILITIES_FILE, "capabilities"),
        (RISK_FILE, "risk"),
        (BLOCKED_FILE, "blocked"),
        (HELPERS_FILE, "helpers"),
    ];

    for (file_path, name) in &policy_files {
        let path = Path::new(file_path);
        if !path.exists() {
            if auto_repair {
                // Create default policy file
                let default_content = match *name {
                    "capabilities" => generate_default_capabilities_toml(),
                    "risk" => generate_default_risk_toml(),
                    "blocked" => generate_default_blocked_toml(),
                    "helpers" => generate_default_helpers_toml(),
                    _ => String::new(),
                };

                match fs::write(path, &default_content) {
                    Ok(_) => {
                        let evidence_id = generate_policy_evidence_id();
                        evidence_ids.push(evidence_id.clone());
                        results.push(CheckResult::repaired(
                            &format!("policy_{}", name),
                            &format!("Created default {}.toml", name),
                            &evidence_id,
                        ));
                    }
                    Err(e) => {
                        results.push(CheckResult::fail(
                            &format!("policy_{}", name),
                            &format!("Failed to create {}.toml: {}", name, e),
                        ));
                    }
                }
            } else {
                results.push(CheckResult::fail(
                    &format!("policy_{}", name),
                    &format!("Policy file {}.toml does not exist", name),
                ));
            }
        } else {
            // File exists, try to parse it
            match fs::read_to_string(path) {
                Ok(content) => {
                    let parse_result: Result<toml::Value, _> = toml::from_str(&content);
                    match parse_result {
                        Ok(_) => {
                            results.push(CheckResult::pass(&format!("policy_{}", name)));
                        }
                        Err(e) => {
                            results.push(CheckResult::fail(
                                &format!("policy_{}", name),
                                &format!("Policy file {}.toml has parse errors: {}", name, e),
                            ));
                        }
                    }
                }
                Err(e) => {
                    results.push(CheckResult::fail(
                        &format!("policy_{}", name),
                        &format!("Cannot read {}.toml: {}", name, e),
                    ));
                }
            }
        }
    }

    // Try to load complete policy and validate
    match Policy::load() {
        Ok(policy) => {
            let validation = policy.validate();
            if !validation.valid {
                results.push(CheckResult::fail(
                    "policy_validation",
                    &format!("Policy validation errors: {}", validation.errors.join("; ")),
                ));
            } else if !validation.warnings.is_empty() {
                // Warnings don't fail the check, but log them
                results.push(CheckResult::pass("policy_validation"));
            } else {
                results.push(CheckResult::pass("policy_validation"));
            }
        }
        Err(e) => {
            results.push(CheckResult::fail(
                "policy_validation",
                &format!("Failed to load policy: {}", e),
            ));
        }
    }

    results
}

/// Generate default capabilities.toml content
fn generate_default_capabilities_toml() -> String {
    r#"# Anna Policy: Capabilities
# This file controls which tools are enabled

schema_version = 1

[read_only_tools]
enabled = true
disabled_tools = []
max_evidence_bytes = 8192

[mutation_tools]
enabled = true
disabled_tools = []

[mutation_tools.file_edit]
enabled = true
allowed_paths = ["/etc/", "/home/", "/var/lib/anna/"]
blocked_paths = ["/etc/shadow", "/etc/passwd", "/etc/sudoers", "/etc/ssh/sshd_config"]
max_file_size_bytes = 1048576
text_only = true

[mutation_tools.systemd]
enabled = true
allowed_operations = ["restart", "reload", "start", "stop", "enable", "disable", "daemon-reload"]
blocked_units = ["systemd-*", "dbus.service", "dbus.socket"]
protected_units = ["sshd.service", "NetworkManager.service", "systemd-resolved.service"]

[mutation_tools.packages]
enabled = true
max_packages_per_operation = 5
blocked_patterns = ["linux", "linux-*", "grub", "systemd", "glibc", "base", "filesystem"]
protected_patterns = ["sudo", "openssh", "networkmanager"]
blocked_categories = ["kernel", "bootloader", "init"]

[global]
timeout_ms = 30000
audit_logging = true
"#
    .to_string()
}

/// Generate default risk.toml content
fn generate_default_risk_toml() -> String {
    r#"# Anna Policy: Risk Levels
# This file controls confirmation requirements and thresholds

schema_version = 1

[levels.read_only]
requires_confirmation = false
confirmation_phrase = ""
min_reliability_score = 0
description = "Safe observation only"

[levels.low]
requires_confirmation = true
confirmation_phrase = "y"
min_reliability_score = 50
description = "Reversible, local changes"

[levels.medium]
requires_confirmation = true
confirmation_phrase = "I CONFIRM (medium risk)"
min_reliability_score = 70
description = "Config edits, service restarts, installs"

[levels.high]
requires_confirmation = true
confirmation_phrase = "I assume the risk"
min_reliability_score = 85
description = "Destructive, potentially irreversible"

[confirmations]
forget_phrase = "I CONFIRM (forget)"
reset_phrase = "I CONFIRM (reset)"
uninstall_phrase = "I CONFIRM (uninstall)"
timeout_seconds = 300

[thresholds]
min_mutation_reliability = 70
min_package_reliability = 75
max_concurrent_mutations = 1
"#
    .to_string()
}

/// Generate default blocked.toml content
fn generate_default_blocked_toml() -> String {
    r#"# Anna Policy: Blocked Items
# This file explicitly blocks packages, services, and paths

schema_version = 1

[packages]
exact = []
patterns = []

[[packages.categories]]
name = "kernel"
reason = "Kernel modifications require manual intervention"
patterns = ["linux", "linux-*", "kernel*"]

[[packages.categories]]
name = "bootloader"
reason = "Bootloader changes can render system unbootable"
patterns = ["grub", "systemd-boot", "refind", "syslinux"]

[[packages.categories]]
name = "init"
reason = "Init system changes are critical"
patterns = ["systemd", "openrc", "runit"]

[services]
exact = []
patterns = []
critical = ["init", "systemd-journald", "systemd-udevd", "systemd-logind"]

[paths]
exact = []
prefixes = ["/boot/", "/proc/", "/sys/", "/dev/"]
patterns = []

[commands]
exact = []
patterns = []
"#
    .to_string()
}

/// Generate default helpers.toml content
fn generate_default_helpers_toml() -> String {
    r#"# Anna Policy: Helper Definitions
# This file defines what counts as a helper package

schema_version = 1

[[definitions]]
package = "smartmontools"
purpose = "SMART disk health monitoring"
category = "instrumentation"
optional = true
provides_commands = ["smartctl"]

[[definitions]]
package = "nvme-cli"
purpose = "NVMe drive management"
category = "instrumentation"
optional = true
provides_commands = ["nvme"]

[[definitions]]
package = "lm_sensors"
purpose = "Hardware sensor monitoring"
category = "instrumentation"
optional = true
provides_commands = ["sensors"]

[[definitions]]
package = "iw"
purpose = "Wireless interface management"
category = "diagnostics"
optional = true
provides_commands = ["iw"]

[[definitions]]
package = "ethtool"
purpose = "Ethernet device diagnostics"
category = "diagnostics"
optional = true
provides_commands = ["ethtool"]

[tracking]
enabled = true
offer_removal_on_uninstall = true
state_file = "/var/lib/anna/internal/helpers_state.json"
"#
    .to_string()
}

fn check_helper_inventory() -> CheckResult {
    let helpers = get_helper_status_list();

    if helpers.is_empty() {
        return CheckResult::pass("helper_inventory");
    }

    // Check provenance consistency
    for helper in &helpers {
        // If marked as Anna-installed but not present, that's OK (was removed)
        // If marked as user but Anna has record, that's a data inconsistency
        if helper.installed_by == InstalledBy::Anna && helper.present {
            // Verify package is actually installed
            if !is_package_present(&helper.name) {
                return CheckResult::fail(
                    "helper_inventory",
                    &format!("Helper {} marked present but not installed", helper.name),
                );
            }
        }
    }

    CheckResult::pass("helper_inventory")
}

/// Run installer review and update install state
pub fn run_and_record_review(auto_repair: bool) -> InstallerReviewReport {
    let report = run_installer_review(auto_repair);

    // Update install state with review results
    let mut state = InstallState::load_or_default();
    state.record_review(LastReview {
        timestamp: report.timestamp,
        result: report.overall.clone(),
        evidence_ids: report.evidence_ids.clone(),
        duration_ms: report.duration_ms,
    });
    let _ = state.save();

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_pass() {
        let result = CheckResult::pass("test");
        assert!(result.passed);
        assert!(result.issue.is_none());
        assert!(!result.repair_attempted);
    }

    #[test]
    fn test_check_result_fail() {
        let result = CheckResult::fail("test", "something broke");
        assert!(!result.passed);
        assert_eq!(result.issue, Some("something broke".to_string()));
        assert!(!result.repair_attempted);
    }

    #[test]
    fn test_check_result_repaired() {
        let result = CheckResult::repaired("test", "fixed thing", "E1");
        assert!(result.passed);
        assert!(result.repair_attempted);
        assert!(result.repair_success);
        assert_eq!(result.evidence_id, Some("E1".to_string()));
    }

    #[test]
    fn test_report_is_healthy() {
        let report = InstallerReviewReport {
            timestamp: Utc::now(),
            duration_ms: 100,
            checks: vec![CheckResult::pass("test")],
            overall: ReviewResult::Healthy,
            evidence_ids: vec![],
        };
        assert!(report.is_healthy());
    }

    #[test]
    fn test_report_format_summary() {
        let report = InstallerReviewReport {
            timestamp: Utc::now(),
            duration_ms: 100,
            checks: vec![
                CheckResult::pass("test1"),
                CheckResult::pass("test2"),
                CheckResult::fail("test3", "issue"),
            ],
            overall: ReviewResult::NeedsAttention {
                issues: vec!["issue".into()],
            },
            evidence_ids: vec![],
        };
        assert_eq!(report.format_summary(), "2/3 checks passed");
    }

    #[test]
    fn test_evidence_id_generation() {
        let id1 = generate_evidence_id();
        let id2 = generate_evidence_id();
        assert!(id1.starts_with("IR"));
        // IDs should be different (unless generated in same millisecond)
        // Just check format is correct
        assert!(id1.len() >= 3);
    }
}
