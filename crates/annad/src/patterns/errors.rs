//! Common error patterns with known solutions

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match common error messages
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // System conflicts
    if let Some(u) = match_system_conflicts(q) {
        return Some(u);
    }
    // Hardware/driver errors
    if let Some(u) = match_hardware_errors(q) {
        return Some(u);
    }
    // Service/container errors
    if let Some(u) = match_service_errors(q) {
        return Some(u);
    }
    None
}

fn match_system_conflicts(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str, IntentCategory)] = &[
        // DNS/Network conflicts
        (&["resolved", "networkmanager"], "systemd-resolved NetworkManager conflict", "network", IntentCategory::Troubleshoot),
        (&["dns", "not", "resolv"], "DNS resolution issues", "network", IntentCategory::Troubleshoot),
        // Audio conflicts
        (&["pipewire", "pulseaudio", "conflict"], "PipeWire PulseAudio conflict", "audio", IntentCategory::Troubleshoot),
        (&["pipewire", "pulseaudio", "fight"], "PipeWire PulseAudio conflict", "audio", IntentCategory::Troubleshoot),
        (&["audio", "crackl"], "audio crackling issue", "audio", IntentCategory::Troubleshoot),
        (&["no", "sound"], "no audio output", "audio", IntentCategory::Troubleshoot),
        // Display scaling
        (&["electron", "blurry"], "Electron app HiDPI scaling", "display", IntentCategory::Troubleshoot),
        (&["blurry", "scal"], "display scaling issue", "display", IntentCategory::Troubleshoot),
        (&["everything", "small"], "HiDPI scaling issue", "display", IntentCategory::Troubleshoot),
        (&["everything", "tiny"], "HiDPI scaling issue", "display", IntentCategory::Troubleshoot),
        // XDG/Desktop
        (&["xdg-open", "wrong"], "xdg-open default application", "desktop", IntentCategory::Troubleshoot),
        (&["default", "application", "wrong"], "default application issue", "desktop", IntentCategory::Troubleshoot),
    ];

    for (keywords, interpreted, topic, category) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: category.clone(),
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_hardware_errors(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str)] = &[
        // NVIDIA
        (&["nvidia", "dkms"], "NVIDIA DKMS module issue", "display"),
        (&["nvidia", "driver", "not"], "NVIDIA driver issue", "display"),
        (&["nvidia", "module", "not"], "NVIDIA module not loading", "display"),
        // PCIe errors
        (&["pcieport", "error"], "PCIe port errors in journal", "hardware"),
        (&["pcie", "error"], "PCIe errors", "hardware"),
        // Input devices
        (&["mouse", "freeze"], "mouse cursor freezing", "hardware"),
        (&["cursor", "freeze"], "cursor freezing", "hardware"),
        (&["keyboard", "lag"], "keyboard input lag", "hardware"),
        // Bluetooth
        (&["bluetooth", "disconnect"], "Bluetooth disconnecting", "hardware"),
        (&["bluetooth", "not", "work"], "Bluetooth not working", "hardware"),
        // WiFi
        (&["wifi", "drop"], "WiFi connection dropping", "network"),
        (&["wifi", "not", "work"], "WiFi not working", "network"),
        // Screen
        (&["screen", "flicker"], "screen flickering", "display"),
        (&["display", "flicker"], "display flickering", "display"),
        (&["screen", "tear"], "screen tearing", "display"),
    ];

    for (keywords, interpreted, topic) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_service_errors(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str)] = &[
        // Docker
        (&["docker", "dns"], "Docker DNS resolution", "services"),
        (&["docker", "container", "not", "start"], "Docker container not starting", "services"),
        // Flatpak
        (&["flatpak", "access", "home"], "Flatpak home folder access", "packages"),
        (&["flatpak", "permission"], "Flatpak permissions", "packages"),
        // GNOME keyring
        (&["keyring", "password", "boot"], "GNOME keyring password prompt", "desktop"),
        (&["gnome-keyring", "unlock"], "GNOME keyring unlock issue", "desktop"),
        // Timeshift
        (&["timeshift", "btrfs"], "Timeshift BTRFS snapshot issue", "storage"),
        // Steam/gaming
        (&["steam", "crash"], "Steam game crashing", "gaming"),
        (&["proton", "not", "work"], "Proton compatibility issue", "gaming"),
    ];

    for (keywords, interpreted, topic) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}
