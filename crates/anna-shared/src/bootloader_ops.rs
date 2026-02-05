//! Bootloader Operations - Replace or configure bootloaders.
//!
//! v0.3.127: Support for limine, systemd-boot, GRUB replacement, snapper integration.

use serde::{Deserialize, Serialize};

/// Supported bootloaders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bootloader {
    Grub,
    Limine,
    SystemdBoot,
    Refind,
}

impl Bootloader {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "grub" | "grub2" => Some(Self::Grub),
            "limine" => Some(Self::Limine),
            "systemd-boot" | "systemd boot" => Some(Self::SystemdBoot),
            "refind" | "rEFInd" => Some(Self::Refind),
            _ => None,
        }
    }

    pub fn package_name(&self) -> &str {
        match self {
            Self::Grub => "grub",
            Self::Limine => "limine",
            Self::SystemdBoot => "systemd", // Part of systemd
            Self::Refind => "refind",
        }
    }

    pub fn config_path(&self) -> &str {
        match self {
            Self::Grub => "/etc/default/grub",
            Self::Limine => "/boot/limine.cfg",
            Self::SystemdBoot => "/boot/loader/loader.conf",
            Self::Refind => "/boot/EFI/refind/refind.conf",
        }
    }
}

/// Detect current bootloader.
pub fn detect_current_bootloader() -> Option<Bootloader> {
    // Check for GRUB
    if std::path::Path::new("/boot/grub/grub.cfg").exists() {
        return Some(Bootloader::Grub);
    }

    // Check for systemd-boot
    if std::path::Path::new("/boot/loader/loader.conf").exists() {
        return Some(Bootloader::SystemdBoot);
    }

    // Check for limine
    if std::path::Path::new("/boot/limine.cfg").exists() {
        return Some(Bootloader::Limine);
    }

    // Check for rEFInd
    if std::path::Path::new("/boot/EFI/refind").exists() {
        return Some(Bootloader::Refind);
    }

    None
}

/// Generate installation steps for limine.
pub fn limine_install_steps() -> Vec<String> {
    vec![
        "# Install limine package".to_string(),
        "pacman -S --noconfirm limine".to_string(),
        "".to_string(),
        "# Deploy limine to ESP".to_string(),
        "limine-deploy /dev/sdX".to_string(), // User needs to specify disk
        "".to_string(),
        "# Create limine config".to_string(),
        "cat > /boot/limine.cfg << 'EOF'".to_string(),
        "TIMEOUT=5".to_string(),
        "".to_string(),
        ":Arch Linux".to_string(),
        "    PROTOCOL=linux".to_string(),
        "    KERNEL_PATH=boot:///vmlinuz-linux".to_string(),
        "    CMDLINE=root=UUID=<ROOT_UUID> rw".to_string(),
        "    MODULE_PATH=boot:///initramfs-linux.img".to_string(),
        "EOF".to_string(),
    ]
}

/// Generate steps to replace GRUB with limine.
pub fn replace_grub_with_limine_steps(root_uuid: &str, esp_device: &str) -> Vec<String> {
    vec![
        "# Backup GRUB configuration".to_string(),
        format!("cp -r /boot/grub /boot/grub.backup.{}", chrono::Utc::now().format("%Y%m%d")),
        "".to_string(),
        "# Install limine".to_string(),
        "pacman -S --noconfirm limine".to_string(),
        "".to_string(),
        "# Deploy limine to ESP".to_string(),
        format!("limine-deploy {}", esp_device),
        "".to_string(),
        "# Create limine config".to_string(),
        "cat > /boot/limine.cfg << 'EOF'".to_string(),
        "# Limine configuration".to_string(),
        "TIMEOUT=5".to_string(),
        "DEFAULT_ENTRY=1".to_string(),
        "".to_string(),
        ":Arch Linux".to_string(),
        "    PROTOCOL=linux".to_string(),
        "    KERNEL_PATH=boot:///vmlinuz-linux".to_string(),
        format!("    CMDLINE=root=UUID={} rw quiet", root_uuid),
        "    MODULE_PATH=boot:///initramfs-linux.img".to_string(),
        "".to_string(),
        ":Arch Linux (fallback)".to_string(),
        "    PROTOCOL=linux".to_string(),
        "    KERNEL_PATH=boot:///vmlinuz-linux".to_string(),
        format!("    CMDLINE=root=UUID={} rw", root_uuid),
        "    MODULE_PATH=boot:///initramfs-linux-fallback.img".to_string(),
        "EOF".to_string(),
        "".to_string(),
        "# Optional: Remove GRUB (commented for safety)".to_string(),
        "# pacman -Rns grub".to_string(),
        "".to_string(),
        "# IMPORTANT: Test boot before removing GRUB completely!".to_string(),
    ]
}

/// Get root filesystem UUID.
pub fn get_root_uuid() -> Option<String> {
    let output = std::process::Command::new("findmnt")
        .args(&["-n", "-o", "UUID", "/"])
        .output()
        .ok()?;

    if output.status.success() {
        let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !uuid.is_empty() {
            return Some(uuid);
        }
    }

    None
}

/// Get ESP (EFI System Partition) device.
pub fn get_esp_device() -> Option<String> {
    let output = std::process::Command::new("findmnt")
        .args(&["-n", "-o", "SOURCE", "/boot"])
        .output()
        .ok()?;

    if output.status.success() {
        let device = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !device.is_empty() {
            // Extract disk device (e.g., /dev/sda1 -> /dev/sda)
            if let Some(disk) = device.strip_suffix(char::is_numeric) {
                return Some(disk.to_string());
            }
            return Some(device);
        }
    }

    None
}

/// Format bootloader replacement plan for user approval.
pub fn format_bootloader_replacement_plan(from: Bootloader, to: Bootloader, root_uuid: &str, esp_device: &str) -> String {
    let steps = match (from, to) {
        (Bootloader::Grub, Bootloader::Limine) => replace_grub_with_limine_steps(root_uuid, esp_device),
        _ => vec!["Unsupported bootloader replacement".to_string()],
    };

    format!(
        "Bootloader Replacement Plan\n\
        ===========================\n\
        \n\
        From: {:?}\n\
        To: {:?}\n\
        Root UUID: {}\n\
        ESP Device: {}\n\
        \n\
        Risk Level: HIGH - Boot failure possible if something goes wrong\n\
        \n\
        Steps:\n\
        {}\n\
        \n\
        IMPORTANT:\n\
        - Full system backup recommended before proceeding\n\
        - GRUB will be kept installed as fallback\n\
        - Test new bootloader before removing GRUB\n\
        - Have a live USB ready for recovery\n\
        \n\
        Do you want to proceed with this bootloader replacement?",
        from,
        to,
        root_uuid,
        esp_device,
        steps.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootloader_detection() {
        let _current = detect_current_bootloader();
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_bootloader_from_str() {
        assert_eq!(Bootloader::from_str("limine"), Some(Bootloader::Limine));
        assert_eq!(Bootloader::from_str("grub"), Some(Bootloader::Grub));
        assert_eq!(Bootloader::from_str("systemd-boot"), Some(Bootloader::SystemdBoot));
    }
}
