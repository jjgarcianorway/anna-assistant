//! System audit and compliance checking
//!
//! Phase 0.9: Integrity verification and security audit
//! Citation: [archwiki:Security]

use super::types::{AuditReport, IntegrityStatus, SecurityFinding, ConfigIssue};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use tracing::info;

/// Perform system audit
pub async fn perform_audit() -> Result<AuditReport> {
    info!("Performing system audit");

    // Check package integrity
    let mut integrity = check_package_integrity().await?;

    // Check file permissions
    integrity.extend(check_file_permissions().await?);

    // Check security baseline
    let security_findings = check_security_baseline().await?;

    // Check configuration
    let config_issues = check_configuration().await?;

    // Determine overall compliance
    let compliant = integrity.iter().all(|i| i.passed)
        && security_findings.iter().all(|f| f.severity != "critical")
        && config_issues.is_empty();

    let message = if compliant {
        "System passes all audit checks".to_string()
    } else {
        format!(
            "{} integrity issues, {} security findings, {} config issues",
            integrity.iter().filter(|i| !i.passed).count(),
            security_findings.len(),
            config_issues.len()
        )
    };

    Ok(AuditReport {
        timestamp: Utc::now(),
        compliant,
        integrity,
        security_findings,
        config_issues,
        message,
        citation: "[archwiki:Security]".to_string(),
    })
}

/// Check package signature integrity
async fn check_package_integrity() -> Result<Vec<IntegrityStatus>> {
    info!("Checking package integrity");

    let mut results = Vec::new();

    // Check pacman database
    let output = Command::new("pacman")
        .args(&["-Qkk"])
        .output()
        .context("Failed to run pacman -Qkk")?;

    let passed = output.status.success();

    results.push(IntegrityStatus {
        component: "pacman-database".to_string(),
        check_type: "integrity".to_string(),
        passed,
        details: if passed {
            "All packages verified".to_string()
        } else {
            "Some packages failed verification".to_string()
        },
    });

    // Check signature verification is enabled
    let keyring_check = std::path::Path::new("/etc/pacman.d/gnupg").exists();

    results.push(IntegrityStatus {
        component: "pacman-keyring".to_string(),
        check_type: "signature".to_string(),
        passed: keyring_check,
        details: if keyring_check {
            "GPG keyring present".to_string()
        } else {
            "GPG keyring missing".to_string()
        },
    });

    Ok(results)
}

/// Check critical file permissions
async fn check_file_permissions() -> Result<Vec<IntegrityStatus>> {
    info!("Checking file permissions");

    let mut results = Vec::new();

    // Check critical system files
    let critical_files = vec![
        ("/etc/passwd", 0o644),
        ("/etc/shadow", 0o600),
        ("/etc/sudoers", 0o440),
    ];

    for (file, expected_mode) in critical_files {
        if let Ok(metadata) = std::fs::metadata(file) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode() & 0o777;
                let passed = mode == expected_mode;

                results.push(IntegrityStatus {
                    component: file.to_string(),
                    check_type: "ownership".to_string(),
                    passed,
                    details: if passed {
                        format!("Correct permissions: {:o}", mode)
                    } else {
                        format!("Incorrect permissions: {:o} (expected {:o})", mode, expected_mode)
                    },
                });
            }
        }
    }

    Ok(results)
}

/// Check security baseline
async fn check_security_baseline() -> Result<Vec<SecurityFinding>> {
    info!("Checking security baseline");

    let mut findings = Vec::new();

    // Check if firewall is enabled
    let output = Command::new("systemctl")
        .args(&["is-active", "firewalld"])
        .output();

    if let Ok(output) = output {
        if !output.status.success() {
            findings.push(SecurityFinding {
                severity: "medium".to_string(),
                description: "Firewall is not active".to_string(),
                recommendation: "Enable firewalld: systemctl enable --now firewalld".to_string(),
                reference: "[archwiki:Firewall]".to_string(),
            });
        }
    }

    // Check if SSH is hardened
    if let Ok(sshd_config) = std::fs::read_to_string("/etc/ssh/sshd_config") {
        if !sshd_config.contains("PermitRootLogin no") && !sshd_config.contains("PermitRootLogin prohibit-password") {
            findings.push(SecurityFinding {
                severity: "high".to_string(),
                description: "SSH allows root login".to_string(),
                recommendation: "Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string(),
                reference: "[archwiki:Secure_Shell#Deny_root_login]".to_string(),
            });
        }
    }

    Ok(findings)
}

/// Check system configuration
async fn check_configuration() -> Result<Vec<ConfigIssue>> {
    info!("Checking system configuration");

    let mut issues = Vec::new();

    // Check if fstab has proper options
    if let Ok(fstab) = std::fs::read_to_string("/etc/fstab") {
        for line in fstab.lines() {
            if line.contains("/home") && !line.contains("nodev") {
                issues.push(ConfigIssue {
                    file: "/etc/fstab".to_string(),
                    issue: "/home partition missing 'nodev' option".to_string(),
                    expected: "nodev in mount options".to_string(),
                    actual: line.to_string(),
                });
            }
        }
    }

    Ok(issues)
}
