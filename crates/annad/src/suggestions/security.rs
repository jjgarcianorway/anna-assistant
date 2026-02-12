//! Security-related suggestion checks.

use chrono::Utc;
use super::types::{Suggestion, SuggestionPriority};

/// Check for unpatched packages with known vulnerabilities
pub async fn check_security_updates() -> Option<Suggestion> {
    // Check for security updates (packages marked as security fixes)
    let output = std::process::Command::new("checkupdates")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let updates = String::from_utf8_lossy(&output.stdout);
    let update_count = updates.lines().count();

    // Check if there are critical package updates (kernel, systemd, openssl, etc.)
    let critical_packages = ["linux", "systemd", "openssl", "openssh", "glibc", "sudo"];
    let critical_updates: Vec<&str> = updates
        .lines()
        .filter(|l| critical_packages.iter().any(|pkg| l.starts_with(pkg)))
        .collect();

    if !critical_updates.is_empty() {
        Some(Suggestion {
            id: "security-updates-available".to_string(),
            priority: SuggestionPriority::High,
            title: format!("{} security-critical packages need updating", critical_updates.len()),
            description: format!(
                "Updates available for critical components: {}. These may contain security fixes.",
                critical_updates.join(", ").chars().take(80).collect::<String>()
            ),
            reasoning: "Security updates patch vulnerabilities that could be exploited.".to_string(),
            action: Some("Ask: 'update my system' to apply security patches".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else if update_count > 20 {
        Some(Suggestion {
            id: "many-updates-available".to_string(),
            priority: SuggestionPriority::Medium,
            title: format!("{} packages outdated", update_count),
            description: "Your system has many pending updates. Keeping packages current improves security and stability.".to_string(),
            reasoning: "Old packages may have unpatched vulnerabilities.".to_string(),
            action: Some("Ask: 'update my system'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check SSH configuration for security issues
pub async fn check_ssh_security() -> Option<Suggestion> {
    let sshd_config = std::path::Path::new("/etc/ssh/sshd_config");
    if !sshd_config.exists() {
        return None; // SSH not installed
    }

    let config = std::fs::read_to_string(sshd_config).ok()?;
    let mut issues = Vec::new();

    // Check for password authentication enabled
    if config.lines().any(|l| {
        l.trim().starts_with("PasswordAuthentication") && l.contains("yes") && !l.trim().starts_with('#')
    }) {
        issues.push("Password authentication enabled (key-based is more secure)");
    }

    // Check for root login allowed
    if config.lines().any(|l| {
        l.trim().starts_with("PermitRootLogin") && l.contains("yes") && !l.trim().starts_with('#')
    }) {
        issues.push("Root login permitted (should be disabled)");
    }

    // Check for default port
    if !config.lines().any(|l| {
        l.trim().starts_with("Port") && !l.contains("22") && !l.trim().starts_with('#')
    }) {
        issues.push("Using default SSH port 22 (changing port reduces automated attacks)");
    }

    if !issues.is_empty() {
        Some(Suggestion {
            id: "ssh-security-issues".to_string(),
            priority: SuggestionPriority::Medium,
            title: "SSH configuration could be more secure".to_string(),
            description: format!("Found {} potential security improvements: {}", issues.len(), issues.join("; ")),
            reasoning: "Hardening SSH prevents unauthorized access attempts.".to_string(),
            action: Some("Ask: 'how can I secure SSH?'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if firewall is configured
pub async fn check_firewall() -> Option<Suggestion> {
    // Check if firewalld or ufw is running
    let firewalld_check = std::process::Command::new("systemctl")
        .args(["is-active", "firewalld"])
        .output()
        .ok();

    let ufw_check = std::process::Command::new("systemctl")
        .args(["is-active", "ufw"])
        .output()
        .ok();

    let firewalld_active = firewalld_check
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    let ufw_active = ufw_check
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    if !firewalld_active && !ufw_active {
        // Check if iptables has rules
        let iptables_check = std::process::Command::new("iptables")
            .args(["-L", "-n"])
            .output()
            .ok();

        let has_rules = iptables_check
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                output.lines().count() > 10 // More than just default chains
            })
            .unwrap_or(false);

        if !has_rules {
            return Some(Suggestion {
                id: "no-firewall".to_string(),
                priority: SuggestionPriority::Medium,
                title: "No firewall detected".to_string(),
                description: "Your system doesn't appear to have an active firewall. This leaves all ports exposed.".to_string(),
                reasoning: "Firewalls block unauthorized network access and protect services.".to_string(),
                action: Some("Ask: 'help me setup a firewall'".to_string()),
                created_at: Utc::now().to_rfc3339(),
                shown_count: 0,
                dismissed: false,
            });
        }
    }

    None
}
