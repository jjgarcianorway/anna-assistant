//! Security monitoring checks (SSH, firewall, logins, ports).

use std::process::Command;

use crate::monitor::types::{Issue, IssueType, Severity};

/// Check SSH configuration for security issues
pub fn check_ssh_security() -> Vec<Issue> {
    let mut issues = Vec::new();
    let sshd_config = "/etc/ssh/sshd_config";

    if let Ok(content) = std::fs::read_to_string(sshd_config) {
        let content_lower = content.to_lowercase();

        // Check for root login enabled
        if content_lower.contains("permitrootlogin yes") {
            issues.push(Issue {
                issue_type: IssueType::RootLoginEnabled,
                severity: Severity::Warning,
                summary: "SSH root login is enabled".to_string(),
                details: "Allowing root login via SSH is a security risk.".to_string(),
                suggested_fix: Some(
                    "Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string(),
                ),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }

        // Check for password authentication (prefer keys)
        if content_lower.contains("passwordauthentication yes") {
            issues.push(Issue {
                issue_type: IssueType::SshSecurity,
                severity: Severity::Info,
                summary: "SSH password auth enabled".to_string(),
                details: "Key-based authentication is more secure than passwords.".to_string(),
                suggested_fix: Some("Consider: PasswordAuthentication no".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }

        // Check for weak ciphers (if explicitly set)
        if content_lower.contains("3des") || content_lower.contains("arcfour") {
            issues.push(Issue {
                issue_type: IssueType::SshSecurity,
                severity: Severity::Warning,
                summary: "Weak SSH ciphers configured".to_string(),
                details: "3DES and RC4 ciphers are considered weak.".to_string(),
                suggested_fix: Some("Remove weak ciphers from sshd_config".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check if firewall is active
pub fn check_firewall() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for iptables rules
    let iptables = Command::new("iptables").args(["-L", "-n"]).output();
    let nftables = Command::new("nft").args(["list", "ruleset"]).output();

    let has_iptables = iptables
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // More than just the default empty chains
            out.lines().count() > 8
        })
        .unwrap_or(false);

    let has_nftables = nftables
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            !out.trim().is_empty() && out.contains("chain")
        })
        .unwrap_or(false);

    // Check for firewalld or ufw
    let firewalld_active = Command::new("systemctl")
        .args(["is-active", "firewalld"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    let ufw_active = Command::new("ufw")
        .arg("status")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("active"))
        .unwrap_or(false);

    if !has_iptables && !has_nftables && !firewalld_active && !ufw_active {
        issues.push(Issue {
            issue_type: IssueType::FirewallInactive,
            severity: Severity::Warning,
            summary: "No firewall detected".to_string(),
            details: "No active firewall rules found. Your system may be exposed.".to_string(),
            suggested_fix: Some("Consider: ufw enable or systemctl start firewalld".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}

/// Check for suspicious login attempts
pub fn check_suspicious_logins() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check auth log for failed attempts
    let output = Command::new("journalctl")
        .args([
            "-u",
            "sshd",
            "--since",
            "24 hours ago",
            "-q",
            "--no-pager",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let failed_count = stdout.matches("Failed password").count()
            + stdout.matches("authentication failure").count()
            + stdout.matches("Invalid user").count();

        if failed_count > 20 {
            issues.push(Issue {
                issue_type: IssueType::SuspiciousLogin,
                severity: Severity::Warning,
                summary: format!("{} failed SSH logins (24h)", failed_count),
                details: "Many failed login attempts detected. Possible brute force attack."
                    .to_string(),
                suggested_fix: Some("Consider: fail2ban or changing SSH port".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        } else if failed_count > 5 {
            issues.push(Issue {
                issue_type: IssueType::SuspiciousLogin,
                severity: Severity::Info,
                summary: format!("{} failed SSH logins (24h)", failed_count),
                details: "Some failed login attempts detected.".to_string(),
                suggested_fix: Some("Check: journalctl -u sshd | grep Failed".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check for unexpected open ports
pub fn check_open_ports() -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("ss").args(["-tlnp"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for common dangerous ports listening on all interfaces
        let dangerous_ports = [
            ("0.0.0.0:23", "Telnet (unencrypted)"),
            ("0.0.0.0:21", "FTP (unencrypted)"),
            ("0.0.0.0:3306", "MySQL (public)"),
            ("0.0.0.0:5432", "PostgreSQL (public)"),
            ("0.0.0.0:27017", "MongoDB (public)"),
            ("0.0.0.0:6379", "Redis (public)"),
            (":23", "Telnet (unencrypted)"),
            (":21", "FTP (unencrypted)"),
            ("*:3306", "MySQL (public)"),
            ("*:5432", "PostgreSQL (public)"),
        ];

        for (port_pattern, desc) in dangerous_ports {
            if stdout.contains(port_pattern) {
                issues.push(Issue {
                    issue_type: IssueType::OpenPort,
                    severity: Severity::Warning,
                    summary: format!("{} exposed", desc),
                    details: format!("Port {} is listening on all interfaces.", port_pattern),
                    suggested_fix: Some("Bind to 127.0.0.1 or use firewall".to_string()),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    notified: false,
                    acknowledged: false,
                });
            }
        }
    }

    issues
}
