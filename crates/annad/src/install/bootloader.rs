//! Bootloader installation
//!
//! Phase 0.8: systemd-boot and GRUB installation
//! Citation: [archwiki:Systemd-boot], [archwiki:GRUB]

use super::types::{BootloaderType, InstallConfig, StepResult};
use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

/// Install bootloader according to configuration
pub async fn install_bootloader(config: &InstallConfig, dry_run: bool) -> StepResult {
    match config.bootloader {
        BootloaderType::SystemdBoot => install_systemd_boot(dry_run).await,
        BootloaderType::Grub => install_grub(dry_run).await,
    }
}

/// Install systemd-boot bootloader
async fn install_systemd_boot(dry_run: bool) -> StepResult {
    let mut actions = Vec::new();

    // Install systemd-boot
    let cmd = "arch-chroot /mnt bootctl install";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        actions.push("[dry-run] bootctl install".to_string());
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("arch-chroot")
            .args(&["/mnt", "bootctl", "install"])
            .output()
            .context("Failed to install systemd-boot")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("bootctl install failed: {}", stderr));
        }
        actions.push("installed systemd-boot".to_string());
    }

    // Create loader configuration
    if dry_run {
        actions.push("[dry-run] create loader.conf".to_string());
    } else {
        let loader_conf = "default arch.conf\ntimeout 3\nconsole-mode max\neditor no\n";
        std::fs::write("/mnt/boot/loader/loader.conf", loader_conf)
            .context("Failed to write loader.conf")?;
        actions.push("created loader.conf".to_string());
    }

    // Create Arch Linux boot entry
    if dry_run {
        actions.push("[dry-run] create arch.conf entry".to_string());
    } else {
        // Get root partition UUID
        let root_uuid = get_root_uuid().unwrap_or_else(|_| "ROOT_UUID".to_string());

        let entry_conf = format!(
            "title   Arch Linux\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID={} rw\n",
            root_uuid
        );
        std::fs::write("/mnt/boot/loader/entries/arch.conf", entry_conf)
            .context("Failed to write arch.conf entry")?;
        actions.push("created arch.conf entry".to_string());
    }

    Ok(actions.join("; "))
}

/// Install GRUB bootloader
async fn install_grub(dry_run: bool) -> StepResult {
    let mut actions = Vec::new();

    // Install grub package (should already be installed via pacstrap)
    // Install GRUB to disk
    let cmd = "arch-chroot /mnt grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        actions.push("[dry-run] grub-install".to_string());
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("arch-chroot")
            .args(&[
                "/mnt",
                "grub-install",
                "--target=x86_64-efi",
                "--efi-directory=/boot",
                "--bootloader-id=GRUB",
            ])
            .output()
            .context("Failed to install GRUB")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("grub-install failed: {}", stderr));
        }
        actions.push("installed GRUB to /boot".to_string());
    }

    // Generate GRUB configuration
    let gen_cmd = "arch-chroot /mnt grub-mkconfig -o /boot/grub/grub.cfg";
    if dry_run {
        actions.push("[dry-run] grub-mkconfig".to_string());
    } else {
        info!("Executing: {}", gen_cmd);
        let output = Command::new("arch-chroot")
            .args(&["/mnt", "grub-mkconfig", "-o", "/boot/grub/grub.cfg"])
            .output()
            .context("Failed to generate GRUB config")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("grub-mkconfig failed: {}", stderr));
        }
        actions.push("generated GRUB configuration".to_string());
    }

    Ok(actions.join("; "))
}

/// Get root partition UUID from /mnt
fn get_root_uuid() -> Result<String> {
    let output = Command::new("findmnt")
        .args(&["-n", "-o", "UUID", "/mnt"])
        .output()
        .context("Failed to get root UUID")?;

    if output.status.success() {
        let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(uuid)
    } else {
        Err(anyhow::anyhow!("Could not determine root UUID"))
    }
}
