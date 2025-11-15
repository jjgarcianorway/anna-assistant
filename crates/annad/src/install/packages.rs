//! Base system installation and configuration
//!
//! Phase 0.8: Package installation with pacstrap
//! Citation: [archwiki:Installation_guide#Install_essential_packages]

use super::types::{InstallConfig, StepResult};
use anyhow::Context;
use std::process::Command;
use tracing::info;

/// Install base system with pacstrap
pub async fn install_base_system(config: &InstallConfig, dry_run: bool) -> StepResult {
    let mut packages = vec![
        "base",
        "linux",
        "linux-firmware",
        "base-devel",
        "networkmanager",
        "vim",
        "sudo",
    ];

    // Add extra packages from config
    let extra: Vec<&str> = config.extra_packages.iter().map(|s| s.as_str()).collect();
    packages.extend(extra);

    let packages_str = packages.join(" ");
    let cmd = format!("pacstrap -K /mnt {}", packages_str);

    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        return Ok(format!(
            "[dry-run] would install {} packages to /mnt",
            packages.len()
        ));
    }

    info!("Executing: {}", cmd);
    let output = Command::new("pacstrap")
        .args(&["-K", "/mnt"])
        .args(&packages)
        .output()
        .context("Failed to execute pacstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pacstrap failed: {}", stderr));
    }

    Ok(format!("installed {} packages successfully", packages.len()))
}

/// Configure system (fstab, locale, timezone, hostname)
pub async fn configure_system(config: &InstallConfig, dry_run: bool) -> StepResult {
    let mut actions = Vec::new();

    // Generate fstab
    if dry_run {
        actions.push("[dry-run] generate fstab".to_string());
    } else {
        info!("Generating fstab");
        let output = Command::new("genfstab")
            .args(&["-U", "/mnt"])
            .output()
            .context("Failed to generate fstab")?;

        if output.status.success() {
            let fstab_content = String::from_utf8_lossy(&output.stdout);
            std::fs::write("/mnt/etc/fstab", fstab_content.as_bytes())
                .context("Failed to write fstab")?;
            actions.push("generated fstab".to_string());
        } else {
            return Err(anyhow::anyhow!("genfstab failed"));
        }
    }

    // Set hostname
    if dry_run {
        actions.push(format!("[dry-run] set hostname to {}", config.hostname));
    } else {
        std::fs::write("/mnt/etc/hostname", &config.hostname)
            .context("Failed to write hostname")?;
        actions.push(format!("set hostname to {}", config.hostname));
    }

    // Set timezone
    let timezone_cmd = format!(
        "arch-chroot /mnt ln -sf /usr/share/zoneinfo/{} /etc/localtime",
        config.timezone
    );
    if dry_run {
        actions.push(format!("[dry-run] {}", timezone_cmd));
    } else {
        info!("Executing: {}", timezone_cmd);
        let output = Command::new("arch-chroot")
            .args(&[
                "/mnt",
                "ln",
                "-sf",
                &format!("/usr/share/zoneinfo/{}", config.timezone),
                "/etc/localtime",
            ])
            .output()
            .context("Failed to set timezone")?;

        if output.status.success() {
            actions.push(format!("set timezone to {}", config.timezone));
        }

        // Generate hardware clock
        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "hwclock", "--systohc"])
            .output();
        actions.push("synced hardware clock".to_string());
    }

    // Set locale
    if dry_run {
        actions.push(format!("[dry-run] set locale to {}", config.locale));
    } else {
        // Uncomment locale in locale.gen
        let locale_gen = std::fs::read_to_string("/mnt/etc/locale.gen")
            .unwrap_or_else(|_| String::new());
        let locale_line = format!("{} UTF-8", config.locale);
        let updated = locale_gen.replace(&format!("#{}", locale_line), &locale_line);
        std::fs::write("/mnt/etc/locale.gen", updated)
            .context("Failed to update locale.gen")?;

        // Generate locale
        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "locale-gen"])
            .output();

        // Set LANG variable
        std::fs::write("/mnt/etc/locale.conf", format!("LANG={}", config.locale))
            .context("Failed to write locale.conf")?;
        actions.push(format!("set locale to {}", config.locale));
    }

    // Enable NetworkManager
    if dry_run {
        actions.push("[dry-run] enable NetworkManager".to_string());
    } else {
        let _ = Command::new("arch-chroot")
            .args(&["/mnt", "systemctl", "enable", "NetworkManager"])
            .output();
        actions.push("enabled NetworkManager".to_string());
    }

    Ok(actions.join("; "))
}
