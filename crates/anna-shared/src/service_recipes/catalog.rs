//! Service catalog with known services (v0.0.214).

use super::recipe::ServiceRecipe;
use super::types::{ServiceCategory, ServiceRisk};

/// Known services with metadata
pub fn known_services() -> Vec<ServiceRecipe> {
    vec![
        // Network
        ServiceRecipe::new(
            "sshd.service",
            "SSH Server",
            ServiceCategory::Network,
            "OpenSSH server daemon",
        )
        .with_risk(ServiceRisk::Medium)
        .with_aliases(&["ssh", "openssh"]),
        ServiceRecipe::new(
            "NetworkManager.service",
            "NetworkManager",
            ServiceCategory::Network,
            "Network configuration daemon",
        )
        .with_risk(ServiceRisk::High)
        .with_aliases(&["network", "nm"]),
        ServiceRecipe::new(
            "systemd-networkd.service",
            "systemd-networkd",
            ServiceCategory::Network,
            "Systemd network daemon",
        )
        .with_risk(ServiceRisk::High),
        ServiceRecipe::new(
            "cups.service",
            "CUPS",
            ServiceCategory::Network,
            "Printing service",
        )
        .with_risk(ServiceRisk::Low)
        .with_aliases(&["printing", "printer"]),
        ServiceRecipe::new(
            "bluetooth.service",
            "Bluetooth",
            ServiceCategory::Network,
            "Bluetooth daemon",
        )
        .with_risk(ServiceRisk::Low),
        // Display
        ServiceRecipe::new(
            "gdm.service",
            "GDM",
            ServiceCategory::Display,
            "GNOME Display Manager",
        )
        .with_risk(ServiceRisk::High)
        .with_aliases(&["gnome-dm"]),
        ServiceRecipe::new(
            "sddm.service",
            "SDDM",
            ServiceCategory::Display,
            "Simple Desktop Display Manager",
        )
        .with_risk(ServiceRisk::High),
        ServiceRecipe::new(
            "lightdm.service",
            "LightDM",
            ServiceCategory::Display,
            "Light Display Manager",
        )
        .with_risk(ServiceRisk::High),
        // Audio
        ServiceRecipe::new(
            "pipewire.service",
            "PipeWire",
            ServiceCategory::Audio,
            "Audio/video daemon",
        )
        .with_risk(ServiceRisk::Low)
        .with_aliases(&["audio"]),
        ServiceRecipe::new(
            "pulseaudio.service",
            "PulseAudio",
            ServiceCategory::Audio,
            "Sound server",
        )
        .with_risk(ServiceRisk::Low),
        // System
        ServiceRecipe::new(
            "cronie.service",
            "Cron",
            ServiceCategory::System,
            "Task scheduler",
        )
        .with_risk(ServiceRisk::Medium)
        .with_aliases(&["cron", "crond"]),
        ServiceRecipe::new(
            "docker.service",
            "Docker",
            ServiceCategory::System,
            "Container runtime",
        )
        .with_risk(ServiceRisk::Low),
        ServiceRecipe::new(
            "libvirtd.service",
            "Libvirt",
            ServiceCategory::System,
            "Virtualization daemon",
        )
        .with_risk(ServiceRisk::Low)
        .with_aliases(&["libvirt", "virt"]),
        // Protected - refuse to modify
        ServiceRecipe::new(
            "systemd-journald.service",
            "Journald",
            ServiceCategory::System,
            "System logging",
        )
        .with_risk(ServiceRisk::Protected),
        ServiceRecipe::new(
            "dbus.service",
            "D-Bus",
            ServiceCategory::System,
            "Message bus",
        )
        .with_risk(ServiceRisk::Protected),
        ServiceRecipe::new(
            "systemd-udevd.service",
            "Udev",
            ServiceCategory::System,
            "Device manager",
        )
        .with_risk(ServiceRisk::Protected),
    ]
}

/// Find a service recipe by name or alias
pub fn find_service(name: &str) -> Option<ServiceRecipe> {
    let name_lower = name.to_lowercase();
    known_services().into_iter().find(|s| {
        s.name.to_lowercase().starts_with(&name_lower)
            || s.display_name.to_lowercase() == name_lower
            || s.aliases.iter().any(|a| a.to_lowercase() == name_lower)
    })
}
