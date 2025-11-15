//! Security configuration detection
//!
//! Detects security-related configuration:
//! - Firewall status (ufw, nftables, iptables)
//! - SSH server configuration and security settings
//! - System umask settings

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Firewall type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallType {
    /// UFW (Uncomplicated Firewall)
    Ufw,
    /// nftables
    Nftables,
    /// iptables
    Iptables,
    /// firewalld
    Firewalld,
    /// None detected
    None,
}

/// Firewall status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallStatus {
    /// Active and enabled
    Active,
    /// Inactive but available
    Inactive,
    /// Not installed
    NotInstalled,
}

/// SSH configuration security level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SshSecurityLevel {
    /// Strong security configuration
    Strong,
    /// Moderate security
    Moderate,
    /// Weak security (needs improvement)
    Weak,
    /// SSH not configured/installed
    NotConfigured,
}

/// Security configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    /// Primary firewall type
    pub firewall_type: FirewallType,
    /// Firewall status
    pub firewall_status: FirewallStatus,
    /// Firewall rule count (if available)
    pub firewall_rule_count: Option<usize>,
    /// SSH server is running
    pub ssh_running: bool,
    /// SSH server is enabled at boot
    pub ssh_enabled: bool,
    /// SSH security level
    pub ssh_security_level: SshSecurityLevel,
    /// SSH allows root login
    pub ssh_root_login: Option<bool>,
    /// SSH allows password authentication
    pub ssh_password_auth: Option<bool>,
    /// SSH port (default 22)
    pub ssh_port: Option<u16>,
    /// System umask value
    pub system_umask: Option<String>,
}

impl SecurityInfo {
    /// Detect security configuration
    pub fn detect() -> Self {
        let (firewall_type, firewall_status, firewall_rule_count) = detect_firewall();
        let ssh_running = is_ssh_running();
        let ssh_enabled = is_ssh_enabled();
        let (ssh_security_level, ssh_root_login, ssh_password_auth, ssh_port) =
            analyze_ssh_config();
        let system_umask = detect_umask();

        Self {
            firewall_type,
            firewall_status,
            firewall_rule_count,
            ssh_running,
            ssh_enabled,
            ssh_security_level,
            ssh_root_login,
            ssh_password_auth,
            ssh_port,
            system_umask,
        }
    }
}

/// Detect firewall type and status
fn detect_firewall() -> (FirewallType, FirewallStatus, Option<usize>) {
    // Check UFW first
    if let Ok(output) = Command::new("ufw").arg("status").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let status = if stdout.contains("Status: active") {
                FirewallStatus::Active
            } else {
                FirewallStatus::Inactive
            };

            // Count rules
            let rule_count = stdout.lines().filter(|line| {
                !line.is_empty()
                    && !line.starts_with("Status:")
                    && !line.starts_with("To")
                    && !line.starts_with("--")
                    && !line.starts_with("Logging:")
            }).count();

            return (FirewallType::Ufw, status, Some(rule_count));
        }
    }

    // Check nftables
    if let Ok(output) = Command::new("nft").arg("list").arg("ruleset").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let has_rules = !stdout.trim().is_empty();
            let status = if has_rules {
                FirewallStatus::Active
            } else {
                FirewallStatus::Inactive
            };

            let rule_count = stdout.lines().filter(|line| line.trim().starts_with("rule")).count();

            return (FirewallType::Nftables, status, Some(rule_count));
        }
    }

    // Check firewalld
    if let Ok(output) = Command::new("firewall-cmd").arg("--state").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let status = if stdout.trim() == "running" {
                FirewallStatus::Active
            } else {
                FirewallStatus::Inactive
            };

            return (FirewallType::Firewalld, status, None);
        }
    }

    // Check iptables
    if let Ok(output) = Command::new("iptables").arg("-L").arg("-n").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let rule_count = stdout.lines().filter(|line| {
                !line.is_empty()
                    && !line.starts_with("Chain")
                    && !line.starts_with("target")
                    && line.trim() != ""
            }).count();

            let status = if rule_count > 3 {
                FirewallStatus::Active
            } else {
                FirewallStatus::Inactive
            };

            return (FirewallType::Iptables, status, Some(rule_count));
        }
    }

    (FirewallType::None, FirewallStatus::NotInstalled, None)
}

/// Check if SSH server is running
fn is_ssh_running() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg("sshd")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim() == "active";
        }
    }

    // Try alternative service names
    for service in &["ssh", "openssh"] {
        if let Ok(output) = Command::new("systemctl")
            .arg("is-active")
            .arg(service)
            .output()
        {
            if output.status.success()
                && String::from_utf8_lossy(&output.stdout).trim() == "active"
            {
                return true;
            }
        }
    }

    false
}

/// Check if SSH server is enabled at boot
fn is_ssh_enabled() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-enabled")
        .arg("sshd")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim() == "enabled";
        }
    }

    // Try alternative service names
    for service in &["ssh", "openssh"] {
        if let Ok(output) = Command::new("systemctl")
            .arg("is-enabled")
            .arg(service)
            .output()
        {
            if output.status.success()
                && String::from_utf8_lossy(&output.stdout).trim() == "enabled"
            {
                return true;
            }
        }
    }

    false
}

/// Analyze SSH configuration for security
fn analyze_ssh_config() -> (SshSecurityLevel, Option<bool>, Option<bool>, Option<u16>) {
    let ssh_config_path = "/etc/ssh/sshd_config";

    let content = match fs::read_to_string(ssh_config_path) {
        Ok(c) => c,
        Err(_) => return (SshSecurityLevel::NotConfigured, None, None, None),
    };

    let mut root_login = None;
    let mut password_auth = None;
    let mut port = None;
    let mut security_score = 0;
    let mut total_checks = 0;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "permitrootlogin" => {
                total_checks += 1;
                let value = parts[1].to_lowercase();
                root_login = Some(value == "yes" || value == "without-password");
                if value == "no" || value == "prohibit-password" {
                    security_score += 1;
                }
            }
            "passwordauthentication" => {
                total_checks += 1;
                let value = parts[1].to_lowercase();
                password_auth = Some(value == "yes");
                if value == "no" {
                    security_score += 1;
                }
            }
            "port" => {
                if let Ok(p) = parts[1].parse::<u16>() {
                    port = Some(p);
                    total_checks += 1;
                    if p != 22 {
                        security_score += 1; // Non-standard port is slightly more secure
                    }
                }
            }
            "permitemptypasswords" => {
                total_checks += 1;
                if parts[1].to_lowercase() == "no" {
                    security_score += 1;
                }
            }
            "x11forwarding" => {
                total_checks += 1;
                if parts[1].to_lowercase() == "no" {
                    security_score += 1;
                }
            }
            "protocol" => {
                total_checks += 1;
                if parts[1] == "2" {
                    security_score += 1;
                }
            }
            _ => {}
        }
    }

    let security_level = if total_checks == 0 {
        SshSecurityLevel::NotConfigured
    } else {
        let ratio = security_score as f32 / total_checks as f32;
        if ratio >= 0.7 {
            SshSecurityLevel::Strong
        } else if ratio >= 0.4 {
            SshSecurityLevel::Moderate
        } else {
            SshSecurityLevel::Weak
        }
    };

    (security_level, root_login, password_auth, port)
}

/// Detect system umask
fn detect_umask() -> Option<String> {
    // Try to get umask from /etc/profile
    if let Ok(content) = fs::read_to_string("/etc/profile") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("umask") && !line.starts_with('#') {
                if let Some(value) = line.split_whitespace().nth(1) {
                    return Some(value.to_string());
                }
            }
        }
    }

    // Try /etc/bash.bashrc
    if let Ok(content) = fs::read_to_string("/etc/bash.bashrc") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("umask") && !line.starts_with('#') {
                if let Some(value) = line.split_whitespace().nth(1) {
                    return Some(value.to_string());
                }
            }
        }
    }

    // Default umask is typically 022
    Some("022".to_string())
}

impl std::fmt::Display for FirewallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirewallType::Ufw => write!(f, "UFW"),
            FirewallType::Nftables => write!(f, "nftables"),
            FirewallType::Iptables => write!(f, "iptables"),
            FirewallType::Firewalld => write!(f, "firewalld"),
            FirewallType::None => write!(f, "None"),
        }
    }
}

impl std::fmt::Display for FirewallStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirewallStatus::Active => write!(f, "Active"),
            FirewallStatus::Inactive => write!(f, "Inactive"),
            FirewallStatus::NotInstalled => write!(f, "Not Installed"),
        }
    }
}

impl std::fmt::Display for SshSecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshSecurityLevel::Strong => write!(f, "Strong"),
            SshSecurityLevel::Moderate => write!(f, "Moderate"),
            SshSecurityLevel::Weak => write!(f, "Weak"),
            SshSecurityLevel::NotConfigured => write!(f, "Not Configured"),
        }
    }
}
