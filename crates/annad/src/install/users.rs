//! User creation and permissions
//!
//! Phase 0.8: Create user, add to sudo, configure permissions
//! Citation: [archwiki:Users_and_groups]

use super::types::{InstallConfig, StepResult};
use anyhow::Context;
use std::process::Command;
use tracing::info;

/// Create user and configure permissions
pub async fn create_user(config: &InstallConfig, dry_run: bool) -> StepResult {
    let mut actions = Vec::new();

    // Create user
    let create_cmd = format!(
        "arch-chroot /mnt useradd -m -G wheel -s /bin/bash {}",
        config.username
    );
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", create_cmd);
        actions.push(format!("[dry-run] create user {}", config.username));
    } else {
        info!("Executing: {}", create_cmd);
        let output = Command::new("arch-chroot")
            .args(&[
                "/mnt",
                "useradd",
                "-m",
                "-G",
                "wheel",
                "-s",
                "/bin/bash",
                &config.username,
            ])
            .output()
            .context("Failed to create user")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("useradd failed: {}", stderr));
        }
        actions.push(format!("created user {}", config.username));
    }

    // Configure sudo for wheel group
    if dry_run {
        actions.push("[dry-run] enable wheel group in sudoers".to_string());
    } else {
        // Read sudoers file
        let sudoers_path = "/mnt/etc/sudoers";
        let sudoers = std::fs::read_to_string(sudoers_path).unwrap_or_else(|_| String::new());

        // Uncomment wheel group line
        let updated = sudoers.replace("# %wheel ALL=(ALL:ALL) ALL", "%wheel ALL=(ALL:ALL) ALL");

        std::fs::write(sudoers_path, updated).context("Failed to update sudoers")?;
        actions.push("enabled wheel group in sudoers".to_string());
    }

    // Set root password (prompt in real implementation)
    if dry_run {
        actions.push("[dry-run] set root password".to_string());
    } else {
        info!("Setting root password to 'root' (CHANGE THIS!)");
        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "sh", "-c", "echo 'root:root' | chpasswd"])
            .output();
        actions.push("set root password (default: 'root')".to_string());
    }

    // Set user password (prompt in real implementation)
    if dry_run {
        actions.push(format!("[dry-run] set password for {}", config.username));
    } else {
        info!(
            "Setting password for {} to 'user' (CHANGE THIS!)",
            config.username
        );
        let _ = Command::new("arch-chroot")
            .args(&[
                "/mnt",
                "sh",
                "-c",
                &format!("echo '{}:user' | chpasswd", config.username),
            ])
            .output();
        actions.push(format!(
            "set password for {} (default: 'user')",
            config.username
        ));
    }

    // Create anna group and add user
    if dry_run {
        actions.push("[dry-run] create anna group".to_string());
    } else {
        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "groupadd", "anna"])
            .output();

        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "usermod", "-aG", "anna", &config.username])
            .output();
        actions.push(format!("added {} to anna group", config.username));
    }

    Ok(actions.join("; "))
}
