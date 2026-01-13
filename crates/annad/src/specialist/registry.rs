//! Specialist Registry - Static definitions of all specialists.
//!
//! Each specialist has a domain, level, supported actions, and allowed helpers.

use super::domain::Domain;
use crate::translator::intent::IntentAction;
use serde::{Deserialize, Serialize};

/// Specialist experience level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialistLevel {
    Junior,
    Senior,
}

/// Extended specialist definition with execution metadata.
#[derive(Debug, Clone)]
pub struct SpecialistDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub level: SpecialistLevel,
    pub domain: Domain,
    /// IntentActions this specialist can handle.
    pub supported_actions: &'static [IntentAction],
    /// Helper tools this specialist may use.
    pub allowed_helpers: &'static [&'static str],
    /// Keywords for fine-grained routing.
    pub expertise_keywords: &'static [&'static str],
}

impl SpecialistDefinition {
    /// Check if this specialist can handle the given action and keywords.
    pub fn can_handle(&self, action: &IntentAction, keywords: &[&str]) -> bool {
        if !self.supported_actions.contains(action) {
            return false;
        }
        keywords.iter().any(|k| {
            let k_lower = k.to_lowercase();
            self.expertise_keywords
                .iter()
                .any(|e| k_lower.contains(e) || e.contains(&k_lower))
        })
    }

    /// Check if this specialist is a junior.
    pub fn is_junior(&self) -> bool {
        self.level == SpecialistLevel::Junior
    }
}

/// Static registry of all specialists (16 total: 8 domains x 2 levels).
pub static SPECIALIST_REGISTRY: &[SpecialistDefinition] = &[
    // === Network Domain ===
    SpecialistDefinition {
        id: "net-jr",
        name: "Michael",
        level: SpecialistLevel::Junior,
        domain: Domain::Network,
        supported_actions: &[IntentAction::Query, IntentAction::Troubleshoot],
        allowed_helpers: &["ip", "ping", "ss", "nmcli", "iwctl", "resolvectl"],
        expertise_keywords: &["wifi", "ethernet", "dns", "dhcp", "ip", "ping", "connectivity"],
    },
    SpecialistDefinition {
        id: "net-sr",
        name: "Sarah",
        level: SpecialistLevel::Senior,
        domain: Domain::Network,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["ip", "iptables", "nftables", "ss", "tcpdump", "traceroute", "dig"],
        expertise_keywords: &["routing", "vpn", "firewall", "iptables", "nftables", "tunnel"],
    },
    // === Desktop Domain ===
    SpecialistDefinition {
        id: "desk-jr",
        name: "Alex",
        level: SpecialistLevel::Junior,
        domain: Domain::Desktop,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Help],
        allowed_helpers: &["vim", "nvim", "nano", "cat", "grep", "echo"],
        expertise_keywords: &["vim", "neovim", "nano", "editor", "terminal", "shell", "bash", "zsh"],
    },
    SpecialistDefinition {
        id: "desk-sr",
        name: "Emma",
        level: SpecialistLevel::Senior,
        domain: Domain::Desktop,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["hyprctl", "swaymsg", "xrandr", "xdotool", "wlr-randr"],
        expertise_keywords: &["hyprland", "sway", "i3", "gnome", "kde", "wayland", "x11", "compositor"],
    },
    // === System Domain ===
    SpecialistDefinition {
        id: "sys-jr",
        name: "James",
        level: SpecialistLevel::Junior,
        domain: Domain::System,
        supported_actions: &[IntentAction::Query, IntentAction::Troubleshoot],
        allowed_helpers: &["systemctl", "journalctl", "ps", "top", "free", "uptime"],
        expertise_keywords: &["systemd", "service", "boot", "startup", "process", "memory", "cpu"],
    },
    SpecialistDefinition {
        id: "sys-sr",
        name: "Lisa",
        level: SpecialistLevel::Senior,
        domain: Domain::System,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["systemctl", "journalctl", "dmesg", "lsmod", "modprobe", "sysctl"],
        expertise_keywords: &["kernel", "modules", "performance", "optimization", "security"],
    },
    // === Packages Domain ===
    SpecialistDefinition {
        id: "pkg-jr",
        name: "David",
        level: SpecialistLevel::Junior,
        domain: Domain::Packages,
        supported_actions: &[IntentAction::Query, IntentAction::Package],
        allowed_helpers: &["pacman", "yay", "paru", "apt", "dnf"],
        expertise_keywords: &["pacman", "yay", "aur", "install", "update", "package", "dependency"],
    },
    SpecialistDefinition {
        id: "pkg-sr",
        name: "Nina",
        level: SpecialistLevel::Senior,
        domain: Domain::Packages,
        supported_actions: &[IntentAction::Query, IntentAction::Package, IntentAction::Configure],
        allowed_helpers: &["pacman", "makepkg", "pkgctl", "asp", "pacman-key"],
        expertise_keywords: &["makepkg", "pkgbuild", "aur", "conflicts", "downgrade", "keyring"],
    },
    // === Hardware Domain ===
    SpecialistDefinition {
        id: "hw-jr",
        name: "Ryan",
        level: SpecialistLevel::Junior,
        domain: Domain::Hardware,
        supported_actions: &[IntentAction::Query, IntentAction::Troubleshoot],
        allowed_helpers: &["lspci", "lsusb", "lshw", "nvidia-smi", "glxinfo"],
        expertise_keywords: &["gpu", "nvidia", "amd", "intel", "driver", "graphics", "display"],
    },
    SpecialistDefinition {
        id: "hw-sr",
        name: "Sophie",
        level: SpecialistLevel::Senior,
        domain: Domain::Hardware,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["lspci", "dmidecode", "sensors", "fwupdmgr", "efibootmgr"],
        expertise_keywords: &["firmware", "bios", "uefi", "acpi", "power", "sensors", "overclock"],
    },
    // === Audio Domain ===
    SpecialistDefinition {
        id: "audio-jr",
        name: "Chris",
        level: SpecialistLevel::Junior,
        domain: Domain::Audio,
        supported_actions: &[IntentAction::Query, IntentAction::Troubleshoot],
        allowed_helpers: &["wpctl", "pactl", "pw-cli", "aplay", "arecord"],
        expertise_keywords: &["pipewire", "pulseaudio", "sound", "audio", "volume", "speaker"],
    },
    SpecialistDefinition {
        id: "audio-sr",
        name: "Maria",
        level: SpecialistLevel::Senior,
        domain: Domain::Audio,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["wpctl", "pw-cli", "alsamixer", "jack_control", "pw-link"],
        expertise_keywords: &["alsa", "jack", "routing", "latency", "pro audio", "midi"],
    },
    // === Storage Domain ===
    SpecialistDefinition {
        id: "stor-jr",
        name: "Kevin",
        level: SpecialistLevel::Junior,
        domain: Domain::Storage,
        supported_actions: &[IntentAction::Query],
        allowed_helpers: &["df", "du", "lsblk", "findmnt", "mount"],
        expertise_keywords: &["disk", "partition", "mount", "usb", "drive", "storage", "space"],
    },
    SpecialistDefinition {
        id: "stor-sr",
        name: "Rachel",
        level: SpecialistLevel::Senior,
        domain: Domain::Storage,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["lsblk", "fdisk", "btrfs", "zfs", "cryptsetup", "lvm"],
        expertise_keywords: &["lvm", "raid", "btrfs", "zfs", "encryption", "luks", "backup"],
    },
    // === Security Domain ===
    SpecialistDefinition {
        id: "sec-jr",
        name: "Tom",
        level: SpecialistLevel::Junior,
        domain: Domain::Security,
        supported_actions: &[IntentAction::Query, IntentAction::Help],
        allowed_helpers: &["ssh", "ssh-keygen", "gpg", "chmod", "chown", "id"],
        expertise_keywords: &["password", "ssh", "key", "gpg", "permission", "user", "group"],
    },
    SpecialistDefinition {
        id: "sec-sr",
        name: "Elena",
        level: SpecialistLevel::Senior,
        domain: Domain::Security,
        supported_actions: &[IntentAction::Query, IntentAction::Configure, IntentAction::Troubleshoot],
        allowed_helpers: &["ausearch", "aa-status", "ufw", "firewall-cmd", "passwd"],
        expertise_keywords: &["selinux", "apparmor", "audit", "hardening", "vulnerability", "firewall"],
    },
];

/// Find specialists for a domain.
pub fn get_specialists_for_domain(domain: Domain) -> Vec<&'static SpecialistDefinition> {
    SPECIALIST_REGISTRY
        .iter()
        .filter(|s| s.domain == domain)
        .collect()
}

/// Get junior specialist for domain.
pub fn get_junior(domain: Domain) -> Option<&'static SpecialistDefinition> {
    SPECIALIST_REGISTRY
        .iter()
        .find(|s| s.domain == domain && s.level == SpecialistLevel::Junior)
}

/// Get senior specialist for domain.
pub fn get_senior(domain: Domain) -> Option<&'static SpecialistDefinition> {
    SPECIALIST_REGISTRY
        .iter()
        .find(|s| s.domain == domain && s.level == SpecialistLevel::Senior)
}

/// Get specialist by ID.
pub fn get_by_id(id: &str) -> Option<&'static SpecialistDefinition> {
    SPECIALIST_REGISTRY.iter().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_16_specialists() {
        assert_eq!(SPECIALIST_REGISTRY.len(), 16);
    }

    #[test]
    fn test_each_domain_has_junior_and_senior() {
        let domains = [
            Domain::Network, Domain::Desktop, Domain::System, Domain::Packages,
            Domain::Hardware, Domain::Audio, Domain::Storage, Domain::Security,
        ];
        for domain in domains {
            assert!(get_junior(domain).is_some(), "Missing junior for {:?}", domain);
            assert!(get_senior(domain).is_some(), "Missing senior for {:?}", domain);
        }
    }

    #[test]
    fn test_specialist_can_handle() {
        let junior = get_junior(Domain::Network).unwrap();
        assert!(junior.can_handle(&IntentAction::Query, &["wifi", "problem"]));
        assert!(!junior.can_handle(&IntentAction::Configure, &["wifi"]));
    }

    #[test]
    fn test_get_by_id() {
        let spec = get_by_id("sys-jr").unwrap();
        assert_eq!(spec.name, "James");
        assert_eq!(spec.domain, Domain::System);
    }

    #[test]
    fn test_specialists_have_allowed_helpers() {
        for spec in SPECIALIST_REGISTRY {
            assert!(!spec.allowed_helpers.is_empty(), "{} has no helpers", spec.id);
        }
    }
}
