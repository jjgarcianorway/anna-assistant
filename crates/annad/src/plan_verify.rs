//! Plan Verification - Verify system state after plan execution.
//! Phase 17: Post-action verification.
//!
//! Provides verification for:
//! - File existence and content
//! - Systemd unit states (masked, enabled, active)
//! - Effective logind configuration
//! - File ownership and permissions

use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

/// Result of a verification check.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// Check passed.
    pub passed: bool,
    /// Description of what was checked.
    pub description: String,
    /// Details (only shown in debug mode).
    pub details: Option<String>,
}

/// Verify a file exists with expected content.
pub fn verify_file_contains(path: &str, pattern: &str) -> VerifyResult {
    let p = Path::new(path);

    if !p.exists() {
        return VerifyResult {
            passed: false,
            description: format!("File {} does not exist", path),
            details: None,
        };
    }

    match fs::read_to_string(p) {
        Ok(content) => {
            let passed = content.contains(pattern);
            VerifyResult {
                passed,
                description: if passed {
                    format!("File {} contains expected content", path)
                } else {
                    format!("File {} missing expected pattern", path)
                },
                details: if passed { None } else { Some(pattern.to_string()) },
            }
        }
        Err(e) => VerifyResult {
            passed: false,
            description: format!("Cannot read {}: {}", path, e),
            details: None,
        },
    }
}

/// Verify file ownership.
pub fn verify_file_owner(path: &str, expected_owner: &str) -> VerifyResult {
    let output = Command::new("stat")
        .args(["-c", "%U:%G", path])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let actual = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let passed = actual == expected_owner;
            VerifyResult {
                passed,
                description: if passed {
                    format!("File {} owned by {}", path, expected_owner)
                } else {
                    format!("File {} has wrong owner: {}", path, actual)
                },
                details: None,
            }
        }
        _ => VerifyResult {
            passed: false,
            description: format!("Cannot check ownership of {}", path),
            details: None,
        },
    }
}

/// Verify systemd unit is masked.
pub fn verify_unit_masked(unit: &str) -> VerifyResult {
    let output = Command::new("systemctl")
        .args(["is-enabled", unit])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let passed = stdout.contains("masked");
            VerifyResult {
                passed,
                description: if passed {
                    format!("Unit {} is masked", unit)
                } else {
                    format!("Unit {} is not masked (state: {})", unit, stdout.trim())
                },
                details: None,
            }
        }
        Err(e) => VerifyResult {
            passed: false,
            description: format!("Cannot check unit {}: {}", unit, e),
            details: None,
        },
    }
}

/// Verify effective logind configuration value.
pub fn verify_logind_setting(setting: &str, expected: &str) -> VerifyResult {
    // Use loginctl show to get effective merged configuration
    let output = Command::new("loginctl")
        .args(["show", "-p", setting])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            // Output format: "SettingName=value"
            let actual = stdout
                .trim()
                .split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();
            let passed = actual == expected;
            VerifyResult {
                passed,
                description: if passed {
                    format!("logind {} = {}", setting, expected)
                } else {
                    format!("logind {} = {} (expected {})", setting, actual, expected)
                },
                details: None,
            }
        }
        _ => {
            // Fallback: check config files directly
            verify_logind_config_file(setting, expected)
        }
    }
}

/// Fallback verification by reading config files.
fn verify_logind_config_file(setting: &str, expected: &str) -> VerifyResult {
    // Check drop-in directory first
    let drop_in_dir = Path::new("/etc/systemd/logind.conf.d");
    if drop_in_dir.exists() {
        if let Ok(entries) = fs::read_dir(drop_in_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let search = format!("{}={}", setting, expected);
                    if content.contains(&search) {
                        return VerifyResult {
                            passed: true,
                            description: format!("logind {} configured as {}", setting, expected),
                            details: Some(entry.path().to_string_lossy().to_string()),
                        };
                    }
                }
            }
        }
    }

    // Check main config
    let main_conf = Path::new("/etc/systemd/logind.conf");
    if main_conf.exists() {
        if let Ok(content) = fs::read_to_string(main_conf) {
            let search = format!("{}={}", setting, expected);
            if content.contains(&search) {
                return VerifyResult {
                    passed: true,
                    description: format!("logind {} configured as {}", setting, expected),
                    details: None,
                };
            }
        }
    }

    VerifyResult {
        passed: false,
        description: format!("logind {} not set to {}", setting, expected),
        details: None,
    }
}

/// Run multiple verifications and return combined result.
pub fn verify_all(checks: Vec<VerifyResult>) -> (bool, Vec<VerifyResult>) {
    let all_passed = checks.iter().all(|c| c.passed);
    (all_passed, checks)
}

/// GDM-specific verification.
pub mod gdm {
    use super::*;

    /// Verify GDM monitor configuration.
    pub fn verify_monitors_xml(width: &str, height: &str) -> Vec<VerifyResult> {
        let path = "/var/lib/gdm/.config/monitors.xml";
        vec![
            verify_file_contains(path, &format!("<width>{}</width>", width)),
            verify_file_contains(path, &format!("<height>{}</height>", height)),
            verify_file_owner(path, "gdm:gdm"),
        ]
    }
}

/// Sleep/suspend verification.
pub mod sleep {
    use super::*;

    /// Verify sleep targets are masked.
    pub fn verify_sleep_disabled() -> Vec<VerifyResult> {
        vec![
            verify_unit_masked("sleep.target"),
            verify_unit_masked("suspend.target"),
            verify_unit_masked("hibernate.target"),
            verify_unit_masked("hybrid-sleep.target"),
            verify_logind_setting("IdleAction", "ignore"),
        ]
    }
}

/// Lid close verification.
pub mod lid {
    use super::*;

    /// Verify lid close behavior.
    pub fn verify_lid_behavior(action: &str) -> Vec<VerifyResult> {
        vec![
            verify_logind_setting("HandleLidSwitch", action),
            verify_logind_setting("HandleLidSwitchExternalPower", action),
            verify_logind_setting("HandleLidSwitchDocked", action),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_file_contains_missing() {
        let result = verify_file_contains("/nonexistent/path", "test");
        assert!(!result.passed);
        assert!(result.description.contains("does not exist"));
    }

    #[test]
    fn test_verify_result_structure() {
        let result = VerifyResult {
            passed: true,
            description: "Test passed".to_string(),
            details: None,
        };
        assert!(result.passed);
    }
}
