//! User permissions audit.

use serde::{Deserialize, Serialize};

/// v0.3.20: Permissions audit for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsAudit {
    /// Current user
    pub user: String,
    /// User groups
    pub groups: Vec<String>,
    /// Whether user has sudo access
    pub has_sudo: bool,
    /// Whether user is in wheel group
    pub in_wheel: bool,
    /// Running as root
    pub is_root: bool,
    /// Relevant groups for system administration
    pub admin_groups: Vec<String>,
}

impl PermissionsAudit {
    /// Check current user permissions
    pub fn check() -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let is_root = user == "root" || std::env::var("EUID").map(|e| e == "0").unwrap_or(false);

        // Get groups
        let groups: Vec<String> = std::process::Command::new("groups")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                s.trim()
                    .split_whitespace()
                    .map(|g| g.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let in_wheel = groups.iter().any(|g| g == "wheel");
        let has_sudo = in_wheel || is_root || groups.iter().any(|g| g == "sudo");

        // Filter admin-relevant groups
        let admin_groups: Vec<String> = groups
            .iter()
            .filter(|g| {
                matches!(
                    g.as_str(),
                    "wheel" | "sudo" | "root" | "docker" | "libvirt" | "kvm" | "video" | "audio"
                )
            })
            .cloned()
            .collect();

        Self {
            user,
            groups,
            has_sudo,
            in_wheel,
            is_root,
            admin_groups,
        }
    }
}
