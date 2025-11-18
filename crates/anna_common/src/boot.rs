//! Boot system detection
//!
//! Detects boot-related system configuration:
//! - Firmware type (UEFI vs BIOS)
//! - Secure Boot status
//! - Boot loader type
//! - EFI variables availability

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Boot firmware type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirmwareType {
    /// UEFI firmware
    Uefi,
    /// Legacy BIOS
    Bios,
    /// Unknown firmware type
    Unknown,
}

/// Secure Boot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecureBootStatus {
    /// Secure Boot is enabled
    Enabled,
    /// Secure Boot is disabled
    Disabled,
    /// Secure Boot not supported (BIOS)
    NotSupported,
    /// Status unknown
    Unknown,
}

/// Boot loader type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootLoader {
    /// systemd-boot
    SystemdBoot,
    /// GRUB
    Grub,
    /// rEFInd
    Refind,
    /// Syslinux
    Syslinux,
    /// Other boot loader
    Other(String),
    /// Unknown boot loader
    Unknown,
}

/// Boot system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootInfo {
    /// Firmware type (UEFI/BIOS)
    pub firmware_type: FirmwareType,
    /// Secure Boot status
    pub secure_boot: SecureBootStatus,
    /// Boot loader
    pub boot_loader: BootLoader,
    /// EFI variables directory exists
    pub efi_vars_available: bool,
    /// ESP (EFI System Partition) mount point
    pub esp_mount: Option<String>,
}

impl BootInfo {
    /// Detect boot system information
    pub fn detect() -> Self {
        let firmware_type = detect_firmware_type();
        let secure_boot = detect_secure_boot(&firmware_type);
        let boot_loader = detect_boot_loader(&firmware_type);
        let efi_vars_available = Path::new("/sys/firmware/efi/efivars").exists();
        let esp_mount = detect_esp_mount();

        Self {
            firmware_type,
            secure_boot,
            boot_loader,
            efi_vars_available,
            esp_mount,
        }
    }
}

/// Detect firmware type (UEFI vs BIOS)
fn detect_firmware_type() -> FirmwareType {
    // Check for EFI directory - most reliable method
    if Path::new("/sys/firmware/efi").exists() {
        return FirmwareType::Uefi;
    }

    // Fallback: check for EFI variables
    if Path::new("/sys/firmware/efi/efivars").exists() {
        return FirmwareType::Uefi;
    }

    // Check using bootctl if available
    if let Ok(output) = Command::new("bootctl").arg("status").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Firmware: UEFI") || stdout.contains("System:") {
                return FirmwareType::Uefi;
            }
        }
    }

    // If no EFI indicators, assume BIOS
    if Path::new("/sys/firmware/efi").exists() {
        FirmwareType::Uefi
    } else {
        FirmwareType::Bios
    }
}

/// Detect Secure Boot status
fn detect_secure_boot(firmware_type: &FirmwareType) -> SecureBootStatus {
    // Secure Boot only available on UEFI
    if *firmware_type != FirmwareType::Uefi {
        return SecureBootStatus::NotSupported;
    }

    // Method 1: Check EFI variable
    let secure_boot_var =
        "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";
    if Path::new(secure_boot_var).exists() {
        if let Ok(data) = fs::read(secure_boot_var) {
            // EFI variables have a 4-byte header, actual data starts at byte 4
            if data.len() > 4 {
                let value = data[4];
                return if value == 1 {
                    SecureBootStatus::Enabled
                } else {
                    SecureBootStatus::Disabled
                };
            }
        }
    }

    // Method 2: Check using bootctl
    if let Ok(output) = Command::new("bootctl").arg("status").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Secure Boot: enabled") {
                return SecureBootStatus::Enabled;
            } else if stdout.contains("Secure Boot: disabled") {
                return SecureBootStatus::Disabled;
            }
        }
    }

    // Method 3: Check dmesg for Secure Boot messages
    if let Ok(output) = Command::new("dmesg").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Secure boot enabled")
                || stdout.contains("secureboot: Secure boot enabled")
            {
                return SecureBootStatus::Enabled;
            } else if stdout.contains("Secure boot disabled")
                || stdout.contains("secureboot: Secure boot disabled")
            {
                return SecureBootStatus::Disabled;
            }
        }
    }

    SecureBootStatus::Unknown
}

/// Detect boot loader type
fn detect_boot_loader(firmware_type: &FirmwareType) -> BootLoader {
    // Check for systemd-boot
    if Path::new("/boot/loader/loader.conf").exists()
        || Path::new("/efi/loader/loader.conf").exists()
        || Path::new("/boot/efi/loader/loader.conf").exists()
    {
        return BootLoader::SystemdBoot;
    }

    // Check for GRUB
    if Path::new("/boot/grub/grub.cfg").exists()
        || Path::new("/boot/grub2/grub.cfg").exists()
        || Path::new("/etc/default/grub").exists()
    {
        return BootLoader::Grub;
    }

    // Check for rEFInd
    if Path::new("/boot/refind_linux.conf").exists() || Path::new("/efi/refind_linux.conf").exists()
    {
        return BootLoader::Refind;
    }

    // Check for Syslinux
    if Path::new("/boot/syslinux/syslinux.cfg").exists() {
        return BootLoader::Syslinux;
    }

    // Try to detect using efibootmgr for UEFI systems
    if *firmware_type == FirmwareType::Uefi {
        if let Ok(output) = Command::new("efibootmgr").arg("-v").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("systemd-boot") || stdout.contains("Linux Boot Manager") {
                    return BootLoader::SystemdBoot;
                } else if stdout.contains("grub") || stdout.contains("GRUB") {
                    return BootLoader::Grub;
                } else if stdout.contains("refind") || stdout.contains("rEFInd") {
                    return BootLoader::Refind;
                }
            }
        }
    }

    BootLoader::Unknown
}

/// Detect ESP (EFI System Partition) mount point
fn detect_esp_mount() -> Option<String> {
    // Common ESP mount points
    let common_mounts = ["/boot", "/boot/efi", "/efi"];

    for mount in &common_mounts {
        if is_esp_mounted(mount) {
            return Some(mount.to_string());
        }
    }

    // Try to detect from /proc/mounts
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mount_point = parts[1];
                let fs_type = parts[2];

                // ESP is usually vfat
                if fs_type == "vfat"
                    && (mount_point.contains("boot") || mount_point.contains("efi"))
                {
                    return Some(mount_point.to_string());
                }
            }
        }
    }

    None
}

/// Check if a path is an ESP mount
fn is_esp_mounted(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }

    // Check if path contains EFI directory
    let efi_path = format!("{}/EFI", path);
    if !Path::new(&efi_path).exists() {
        return false;
    }

    // Try to find mount info
    if let Ok(output) = Command::new("findmnt")
        .arg("-n")
        .arg("-o")
        .arg("FSTYPE")
        .arg(path)
        .output()
    {
        if output.status.success() {
            let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // ESP is typically vfat
            return fstype == "vfat";
        }
    }

    // Fallback: if EFI directory exists, probably ESP
    true
}

impl std::fmt::Display for FirmwareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmwareType::Uefi => write!(f, "UEFI"),
            FirmwareType::Bios => write!(f, "BIOS"),
            FirmwareType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for SecureBootStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecureBootStatus::Enabled => write!(f, "Enabled"),
            SecureBootStatus::Disabled => write!(f, "Disabled"),
            SecureBootStatus::NotSupported => write!(f, "Not Supported"),
            SecureBootStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for BootLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootLoader::SystemdBoot => write!(f, "systemd-boot"),
            BootLoader::Grub => write!(f, "GRUB"),
            BootLoader::Refind => write!(f, "rEFInd"),
            BootLoader::Syslinux => write!(f, "Syslinux"),
            BootLoader::Other(name) => write!(f, "{}", name),
            BootLoader::Unknown => write!(f, "Unknown"),
        }
    }
}
