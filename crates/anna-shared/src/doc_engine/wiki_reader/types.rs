//! Types and error definitions for wiki reader (v0.0.429).

/// Wiki reading errors
#[derive(Debug, Clone)]
pub enum WikiReadError {
    NotFound(String),
    ReadFailed(String),
    NoContent(String),
}

impl std::fmt::Display for WikiReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "Wiki page not found: {}", n),
            Self::ReadFailed(e) => write!(f, "Failed to read wiki page: {}", e),
            Self::NoContent(n) => write!(f, "No content in wiki page: {}", n),
        }
    }
}

impl std::error::Error for WikiReadError {}

/// Get essential wiki pages to sync
pub fn get_essential_wiki_pages() -> Vec<&'static str> {
    vec![
        // System
        "systemd",
        "Systemd/User",
        "Systemd/Timers",
        "Systemd-boot",
        // Packages
        "pacman",
        "Pacman/Tips_and_tricks",
        "Arch_User_Repository",
        "Makepkg",
        // Boot
        "Arch_boot_process",
        "GRUB",
        "Unified_kernel_image",
        "Mkinitcpio",
        // Filesystem
        "File_systems",
        "Fstab",
        "Btrfs",
        "Ext4",
        "Partitioning",
        // Storage
        "Solid_state_drive",
        "SSD/NVMe",
        "TRIM",
        "LVM",
        "RAID",
        // Network
        "Network_configuration",
        "Systemd-networkd",
        "NetworkManager",
        "Wireless",
        "Iwd",
        // Hardware
        "PCI_passthrough",
        "Kernel_module",
        "Power_management",
        "CPU_frequency_scaling",
        // Security
        "Security",
        "Users_and_groups",
        "Sudo",
        "SSH",
        "Firewall",
        // Desktop
        "Xorg",
        "Wayland",
        "Desktop_environment",
        "Display_manager",
        // Audio
        "PipeWire",
        "PulseAudio",
        "ALSA",
        // Troubleshooting
        "General_troubleshooting",
        "Boot_debugging",
    ]
}
