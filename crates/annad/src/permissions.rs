//! Permissions management - ensures anna group and socket access.

use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Name of the anna group
pub const ANNA_GROUP: &str = "anna";

/// Check if the anna group exists
pub fn group_exists() -> bool {
    Command::new("getent")
        .args(["group", ANNA_GROUP])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create the anna group
pub fn create_group() -> Result<()> {
    let output = Command::new("groupadd").arg(ANNA_GROUP).output()?;

    if output.status.success() {
        info!("Created group: {}", ANNA_GROUP);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Group already exists is not an error
        if stderr.contains("already exists") {
            Ok(())
        } else {
            anyhow::bail!("Failed to create group: {}", stderr)
        }
    }
}

/// Check if a user is in the anna group
pub fn user_in_group(username: &str) -> bool {
    Command::new("groups")
        .arg(username)
        .output()
        .map(|o| {
            let groups = String::from_utf8_lossy(&o.stdout);
            groups.split_whitespace().any(|g| g == ANNA_GROUP)
        })
        .unwrap_or(false)
}

/// Add a user to the anna group
pub fn add_user_to_group(username: &str) -> Result<()> {
    let output = Command::new("usermod")
        .args(["-aG", ANNA_GROUP, username])
        .output()?;

    if output.status.success() {
        info!("Added user {} to group {}", username, ANNA_GROUP);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to add user to group: {}", stderr)
    }
}

/// Set group ownership on a path
pub fn set_group(path: &str) -> Result<()> {
    let output = Command::new("chgrp").args([ANNA_GROUP, path]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set group on {}: {}", path, stderr)
    }
}

/// Ensure permissions are set up correctly
pub fn ensure_permissions() -> Result<()> {
    // Ensure anna group exists
    if !group_exists() {
        create_group()?;
    }

    // Ensure directories have correct group
    let dirs = ["/var/lib/anna", "/var/log/anna", "/run/anna"];
    for dir in dirs {
        if std::path::Path::new(dir).exists() {
            if let Err(e) = set_group(dir) {
                warn!("Could not set group on {}: {}", dir, e);
            }
        }
    }

    Ok(())
}

/// Check and fix permissions for connecting users
/// Returns list of users that were added to the group
pub fn check_and_fix_user_access() -> Vec<String> {
    let mut fixed = Vec::new();

    // Try to get users who might need access
    // We can check /etc/passwd for regular users (UID >= 1000)
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let username = parts[0];
                let uid: u32 = parts[2].parse().unwrap_or(0);
                let shell = parts.get(6).unwrap_or(&"");

                // Regular users typically have UID >= 1000 and a valid shell
                if uid >= 1000
                    && !shell.contains("nologin")
                    && !shell.contains("false")
                    && !user_in_group(username)
                    && add_user_to_group(username).is_ok()
                {
                    fixed.push(username.to_string());
                }
            }
        }
    }

    fixed
}
