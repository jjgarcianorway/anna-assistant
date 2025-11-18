use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Security features and mandatory access control detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatures {
    pub selinux: SeLinuxStatus,
    pub apparmor: AppArmorStatus,
    pub polkit: PolkitConfig,
    pub sudoers: SudoersConfig,
    pub kernel_lockdown: KernelLockdown,
    pub security_issues: Vec<SecurityIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeLinuxStatus {
    pub installed: bool,
    pub enabled: bool,
    pub mode: Option<SeLinuxMode>,
    pub policy: Option<String>,
    pub version: Option<String>,
    pub denials_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeLinuxMode {
    Enforcing,
    Permissive,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppArmorStatus {
    pub installed: bool,
    pub enabled: bool,
    pub loaded: bool,
    pub profiles_loaded: u32,
    pub profiles_enforcing: u32,
    pub profiles_complaining: u32,
    pub profiles_unconfined: u32,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkitConfig {
    pub installed: bool,
    pub version: Option<String>,
    pub service_running: bool,
    pub rules_count: u32,
    pub custom_rules: Vec<PolkitRule>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkitRule {
    pub file: String,
    pub rule_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudoersConfig {
    pub installed: bool,
    pub version: Option<String>,
    pub config_valid: bool,
    pub config_files: Vec<SudoersFile>,
    pub passwordless_users: Vec<String>,
    pub passwordless_groups: Vec<String>,
    pub all_access_users: Vec<String>,
    pub timestamp_timeout: Option<u32>,
    pub use_pty: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudoersFile {
    pub path: String,
    pub valid: bool,
    pub entries_count: u32,
    pub includes_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelLockdown {
    pub supported: bool,
    pub mode: Option<LockdownMode>,
    pub integrity_protected: bool,
    pub confidentiality_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockdownMode {
    None,
    Integrity,
    Confidentiality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub category: String,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityFeatures {
    /// Detect all security features and configurations
    pub fn detect() -> Self {
        let selinux = detect_selinux();
        let apparmor = detect_apparmor();
        let polkit = detect_polkit();
        let sudoers = detect_sudoers();
        let kernel_lockdown = detect_kernel_lockdown();
        let security_issues =
            analyze_security_issues(&selinux, &apparmor, &polkit, &sudoers, &kernel_lockdown);

        Self {
            selinux,
            apparmor,
            polkit,
            sudoers,
            kernel_lockdown,
            security_issues,
        }
    }
}

fn detect_selinux() -> SeLinuxStatus {
    // Check if SELinux is installed
    let installed =
        Path::new("/etc/selinux/config").exists() || Command::new("sestatus").output().is_ok();

    if !installed {
        return SeLinuxStatus {
            installed: false,
            enabled: false,
            mode: None,
            policy: None,
            version: None,
            denials_count: 0,
        };
    }

    // Get SELinux status
    let (enabled, mode, policy) = if let Ok(output) = Command::new("sestatus").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let enabled = !stdout.contains("disabled");

        let mode = if stdout.contains("enforcing") {
            Some(SeLinuxMode::Enforcing)
        } else if stdout.contains("permissive") {
            Some(SeLinuxMode::Permissive)
        } else {
            Some(SeLinuxMode::Disabled)
        };

        let policy = stdout
            .lines()
            .find(|l| l.contains("Loaded policy name:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string());

        (enabled, mode, policy)
    } else {
        (false, None, None)
    };

    // Get version
    let version = Command::new("sestatus")
        .arg("-v")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find(|l| l.contains("SELinux version:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|v| v.trim().to_string())
        });

    // Count denials from audit log
    let denials_count = if Path::new("/var/log/audit/audit.log").exists() {
        Command::new("grep")
            .args(["-c", "avc.*denied", "/var/log/audit/audit.log"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
    } else {
        0
    };

    SeLinuxStatus {
        installed,
        enabled,
        mode,
        policy,
        version,
        denials_count,
    }
}

fn detect_apparmor() -> AppArmorStatus {
    // Check if AppArmor is installed
    let installed = Path::new("/sys/kernel/security/apparmor").exists()
        || Command::new("aa-status").output().is_ok();

    if !installed {
        return AppArmorStatus {
            installed: false,
            enabled: false,
            loaded: false,
            profiles_loaded: 0,
            profiles_enforcing: 0,
            profiles_complaining: 0,
            profiles_unconfined: 0,
            version: None,
        };
    }

    // Check if AppArmor is enabled in kernel
    let enabled = fs::read_to_string("/sys/module/apparmor/parameters/enabled")
        .ok()
        .map(|s| s.trim() == "Y")
        .unwrap_or(false);

    // Get AppArmor status via aa-status
    let (loaded, profiles_loaded, profiles_enforcing, profiles_complaining, profiles_unconfined) =
        if let Ok(output) = Command::new("aa-status").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            let loaded = output.status.success();

            let profiles_loaded = stdout
                .lines()
                .find(|l| l.contains("profiles are loaded"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);

            let profiles_enforcing = stdout
                .lines()
                .find(|l| l.contains("profiles are in enforce mode"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);

            let profiles_complaining = stdout
                .lines()
                .find(|l| l.contains("profiles are in complain mode"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);

            let profiles_unconfined = stdout
                .lines()
                .find(|l| l.contains("processes are unconfined"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);

            (
                loaded,
                profiles_loaded,
                profiles_enforcing,
                profiles_complaining,
                profiles_unconfined,
            )
        } else {
            (false, 0, 0, 0, 0)
        };

    // Get AppArmor version
    let version = Command::new("apparmor_parser")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.to_string()));

    AppArmorStatus {
        installed,
        enabled,
        loaded,
        profiles_loaded,
        profiles_enforcing,
        profiles_complaining,
        profiles_unconfined,
        version,
    }
}

fn detect_polkit() -> PolkitConfig {
    // Check if polkit is installed
    let installed = Command::new("pkaction").arg("--version").output().is_ok();

    if !installed {
        return PolkitConfig {
            installed: false,
            version: None,
            service_running: false,
            rules_count: 0,
            custom_rules: Vec::new(),
            issues: Vec::new(),
        };
    }

    // Get polkit version
    let version = Command::new("pkaction")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.to_string()));

    // Check if polkit service is running
    let service_running = Command::new("systemctl")
        .args(["is-active", "polkit"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Count polkit actions
    let rules_count = Command::new("pkaction")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().count() as u32)
        .unwrap_or(0);

    // Find custom polkit rules
    let mut custom_rules = Vec::new();
    let mut issues = Vec::new();

    // Check /etc/polkit-1/rules.d/
    if let Ok(entries) = fs::read_dir("/etc/polkit-1/rules.d") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".rules") {
                    custom_rules.push(PolkitRule {
                        file: entry.path().to_string_lossy().to_string(),
                        rule_type: "JavaScript".to_string(),
                        description: None,
                    });
                }
            }
        }
    }

    // Check /etc/polkit-1/localauthority/
    if let Ok(entries) = fs::read_dir("/etc/polkit-1/localauthority") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                    for sub_entry in sub_entries.flatten() {
                        if let Some(name) = sub_entry.file_name().to_str() {
                            if name.ends_with(".pkla") {
                                custom_rules.push(PolkitRule {
                                    file: sub_entry.path().to_string_lossy().to_string(),
                                    rule_type: "INI".to_string(),
                                    description: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for common issues
    if !service_running && installed {
        issues.push("polkit service is not running".to_string());
    }

    PolkitConfig {
        installed,
        version,
        service_running,
        rules_count,
        custom_rules,
        issues,
    }
}

fn detect_sudoers() -> SudoersConfig {
    // Check if sudo is installed
    let installed = Path::new("/etc/sudoers").exists()
        || Command::new("sudo").arg("--version").output().is_ok();

    if !installed {
        return SudoersConfig {
            installed: false,
            version: None,
            config_valid: false,
            config_files: Vec::new(),
            passwordless_users: Vec::new(),
            passwordless_groups: Vec::new(),
            all_access_users: Vec::new(),
            timestamp_timeout: None,
            use_pty: false,
            issues: Vec::new(),
        };
    }

    // Get sudo version
    let version = Command::new("sudo")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.to_string()));

    // Validate sudoers config
    let config_valid = Command::new("visudo")
        .arg("-c")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let mut config_files = Vec::new();
    let mut passwordless_users = Vec::new();
    let mut passwordless_groups = Vec::new();
    let mut all_access_users = Vec::new();
    let mut timestamp_timeout = None;
    let mut use_pty = false;
    let mut issues = Vec::new();

    // Parse main sudoers file
    if let Ok(content) = fs::read_to_string("/etc/sudoers") {
        let entries_count = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .count() as u32;
        let includes_count = content
            .lines()
            .filter(|l| l.trim().starts_with("@include") || l.trim().starts_with("#include"))
            .count() as u32;

        config_files.push(SudoersFile {
            path: "/etc/sudoers".to_string(),
            valid: config_valid,
            entries_count,
            includes_count,
        });

        // Parse for security-relevant settings
        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for NOPASSWD
            if line.contains("NOPASSWD") {
                if let Some(user_part) = line.split_whitespace().next() {
                    if user_part.starts_with('%') {
                        passwordless_groups.push(user_part[1..].to_string());
                    } else {
                        passwordless_users.push(user_part.to_string());
                    }
                }
            }

            // Check for ALL=(ALL) ALL
            if line.contains("ALL=(ALL)") && line.contains("ALL") {
                if let Some(user_part) = line.split_whitespace().next() {
                    if !user_part.starts_with('%') && user_part != "root" {
                        all_access_users.push(user_part.to_string());
                    }
                }
            }

            // Check timestamp_timeout
            if line.starts_with("Defaults") && line.contains("timestamp_timeout") {
                if let Some(timeout_str) = line.split('=').nth(1) {
                    timestamp_timeout = timeout_str.trim().parse::<u32>().ok();
                }
            }

            // Check use_pty
            if line.starts_with("Defaults") && line.contains("use_pty") {
                use_pty = true;
            }
        }
    }

    // Parse sudoers.d directory
    if let Ok(entries) = fs::read_dir("/etc/sudoers.d") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let entries_count = content
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                    .count() as u32;

                config_files.push(SudoersFile {
                    path: entry.path().to_string_lossy().to_string(),
                    valid: true, // visudo -c checks all files
                    entries_count,
                    includes_count: 0,
                });

                // Parse for NOPASSWD and ALL access in sudoers.d files
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if line.contains("NOPASSWD") {
                        if let Some(user_part) = line.split_whitespace().next() {
                            if user_part.starts_with('%') {
                                let group = user_part[1..].to_string();
                                if !passwordless_groups.contains(&group) {
                                    passwordless_groups.push(group);
                                }
                            } else if !passwordless_users.contains(&user_part.to_string()) {
                                passwordless_users.push(user_part.to_string());
                            }
                        }
                    }

                    if line.contains("ALL=(ALL)") && line.contains("ALL") {
                        if let Some(user_part) = line.split_whitespace().next() {
                            if !user_part.starts_with('%') && user_part != "root"
                                && !all_access_users.contains(&user_part.to_string()) {
                                    all_access_users.push(user_part.to_string());
                                }
                        }
                    }
                }
            }
        }
    }

    // Identify security issues
    if !config_valid {
        issues.push("sudoers configuration is invalid (visudo -c failed)".to_string());
    }

    if !passwordless_users.is_empty() {
        issues.push(format!(
            "{} user(s) have passwordless sudo access",
            passwordless_users.len()
        ));
    }

    if !passwordless_groups.is_empty() {
        issues.push(format!(
            "{} group(s) have passwordless sudo access",
            passwordless_groups.len()
        ));
    }

    if !use_pty {
        issues.push("use_pty is not enabled (recommended for security)".to_string());
    }

    if timestamp_timeout.is_none() {
        issues.push("timestamp_timeout is not explicitly set".to_string());
    }

    SudoersConfig {
        installed,
        version,
        config_valid,
        config_files,
        passwordless_users,
        passwordless_groups,
        all_access_users,
        timestamp_timeout,
        use_pty,
        issues,
    }
}

fn detect_kernel_lockdown() -> KernelLockdown {
    // Check if lockdown is supported
    let lockdown_path = Path::new("/sys/kernel/security/lockdown");

    if !lockdown_path.exists() {
        return KernelLockdown {
            supported: false,
            mode: None,
            integrity_protected: false,
            confidentiality_protected: false,
        };
    }

    // Read lockdown mode
    let mode_str = fs::read_to_string(lockdown_path).unwrap_or_default();

    let mode = if mode_str.contains("[none]") {
        Some(LockdownMode::None)
    } else if mode_str.contains("[integrity]") {
        Some(LockdownMode::Integrity)
    } else if mode_str.contains("[confidentiality]") {
        Some(LockdownMode::Confidentiality)
    } else {
        None
    };

    let integrity_protected = matches!(
        mode,
        Some(LockdownMode::Integrity) | Some(LockdownMode::Confidentiality)
    );
    let confidentiality_protected = matches!(mode, Some(LockdownMode::Confidentiality));

    KernelLockdown {
        supported: true,
        mode,
        integrity_protected,
        confidentiality_protected,
    }
}

fn analyze_security_issues(
    selinux: &SeLinuxStatus,
    apparmor: &AppArmorStatus,
    polkit: &PolkitConfig,
    sudoers: &SudoersConfig,
    lockdown: &KernelLockdown,
) -> Vec<SecurityIssue> {
    let mut issues = Vec::new();

    // Check for lack of MAC (Mandatory Access Control)
    if !selinux.enabled && !apparmor.enabled {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Medium,
            category: "Mandatory Access Control".to_string(),
            description: "Neither SELinux nor AppArmor is enabled".to_string(),
            recommendation:
                "Consider enabling AppArmor for additional security (pacman -S apparmor)"
                    .to_string(),
        });
    }

    // Check SELinux in permissive mode
    if selinux.enabled {
        if matches!(selinux.mode, Some(SeLinuxMode::Permissive)) {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Low,
                category: "SELinux".to_string(),
                description: "SELinux is in permissive mode (not enforcing)".to_string(),
                recommendation: "Set SELinux to enforcing mode for better security".to_string(),
            });
        }

        if selinux.denials_count > 100 {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Medium,
                category: "SELinux".to_string(),
                description: format!(
                    "{} SELinux denials found in audit log",
                    selinux.denials_count
                ),
                recommendation: "Review audit log with 'ausearch -m avc' to identify policy issues"
                    .to_string(),
            });
        }
    }

    // Check AppArmor profiles
    if apparmor.enabled && apparmor.profiles_complaining > 0 {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Low,
            category: "AppArmor".to_string(),
            description: format!(
                "{} AppArmor profiles in complain mode",
                apparmor.profiles_complaining
            ),
            recommendation: "Consider setting profiles to enforce mode after testing".to_string(),
        });
    }

    // Check polkit issues
    for issue in &polkit.issues {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Medium,
            category: "Polkit".to_string(),
            description: issue.clone(),
            recommendation: "Ensure polkit service is running for proper privilege management"
                .to_string(),
        });
    }

    // Check sudoers issues
    if !sudoers.passwordless_users.is_empty() {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::High,
            category: "Sudo".to_string(),
            description: format!(
                "Users with passwordless sudo: {}",
                sudoers.passwordless_users.join(", ")
            ),
            recommendation: "Remove NOPASSWD from sudoers to require password authentication"
                .to_string(),
        });
    }

    if !sudoers.use_pty {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Low,
            category: "Sudo".to_string(),
            description: "use_pty is not enabled in sudoers".to_string(),
            recommendation: "Add 'Defaults use_pty' to /etc/sudoers for better security"
                .to_string(),
        });
    }

    // Check kernel lockdown
    if lockdown.supported
        && matches!(lockdown.mode, Some(LockdownMode::None)) {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Medium,
                category: "Kernel Lockdown".to_string(),
                description: "Kernel lockdown is not enabled".to_string(),
                recommendation: "Enable kernel lockdown with 'lockdown=integrity' or 'lockdown=confidentiality' kernel parameter".to_string(),
            });
        }

    issues
}
