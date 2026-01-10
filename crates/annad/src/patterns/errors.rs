//! Common error patterns with known solutions
//! v0.0.915: Added suggested_commands for diagnostics

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, category, and diagnostic commands
type ErrorPattern = (&'static [&'static str], &'static str, &'static str, IntentCategory, &'static [&'static str]);

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
    let patterns: &[ErrorPattern] = &[
        // DNS/Network conflicts
        (&["resolved", "networkmanager"], "systemd-resolved NetworkManager conflict", "network", IntentCategory::Troubleshoot,
            &["systemctl status systemd-resolved", "cat /etc/resolv.conf",
              "echo 'FIX: ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf'"]),
        (&["dns", "not", "resolv"], "DNS resolution issues", "network", IntentCategory::Troubleshoot,
            &["cat /etc/resolv.conf", "ping -c 1 8.8.8.8", "nslookup google.com"]),
        // Audio conflicts
        (&["pipewire", "pulseaudio", "conflict"], "PipeWire PulseAudio conflict", "audio", IntentCategory::Troubleshoot,
            &["systemctl --user status pipewire pulseaudio",
              "echo 'FIX: systemctl --user mask pulseaudio && systemctl --user enable pipewire'"]),
        (&["pipewire", "pulseaudio", "fight"], "PipeWire PulseAudio conflict", "audio", IntentCategory::Troubleshoot,
            &["pacman -Q | grep -E 'pipewire|pulse'", "systemctl --user status pipewire"]),
        (&["audio", "crackl"], "audio crackling issue", "audio", IntentCategory::Troubleshoot,
            &["cat /etc/pipewire/pipewire.conf 2>/dev/null | grep -i quant",
              "echo 'FIX: Increase default.clock.quantum in /etc/pipewire/pipewire.conf'"]),
        (&["no", "sound"], "no audio output", "audio", IntentCategory::Troubleshoot,
            &["pactl info", "wpctl status", "aplay -l"]),
        // Display scaling
        (&["electron", "blurry"], "Electron app HiDPI scaling", "display", IntentCategory::Troubleshoot,
            &["echo 'FIX: --force-device-scale-factor=1 or set GDK_SCALE=2'"]),
        (&["blurry", "scal"], "display scaling issue", "display", IntentCategory::Troubleshoot,
            &["echo $GDK_SCALE", "echo 'Check: Settings > Displays > Scale'"]),
        (&["everything", "small"], "HiDPI scaling issue", "display", IntentCategory::Troubleshoot,
            &["echo 'FIX: gsettings set org.gnome.desktop.interface scaling-factor 2'"]),
        (&["everything", "tiny"], "HiDPI scaling issue", "display", IntentCategory::Troubleshoot,
            &["gsettings get org.gnome.desktop.interface scaling-factor"]),
        // XDG/Desktop
        (&["xdg-open", "wrong"], "xdg-open default application", "desktop", IntentCategory::Troubleshoot,
            &["xdg-mime query default text/html", "echo 'FIX: xdg-mime default <app>.desktop <mimetype>'"]),
        (&["default", "application", "wrong"], "default application issue", "desktop", IntentCategory::Troubleshoot,
            &["cat ~/.config/mimeapps.list 2>/dev/null | head -20"]),
    ];

    for (keywords, interpreted, topic, category, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: category.clone(),
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Hardware pattern with topic and commands
type HardwarePattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_hardware_errors(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // NVIDIA
        (&["nvidia", "dkms"], "NVIDIA DKMS module issue", "display",
            &["dkms status", "journalctl -b | grep -i nvidia | tail -10",
              "echo 'FIX: sudo dkms autoinstall'"]),
        (&["nvidia", "driver", "not"], "NVIDIA driver issue", "display",
            &["nvidia-smi", "lspci -k | grep -A2 -i nvidia"]),
        (&["nvidia", "module", "not"], "NVIDIA module not loading", "display",
            &["lsmod | grep nvidia", "dmesg | grep -i nvidia | tail -10"]),
        // PCIe errors
        (&["pcieport", "error"], "PCIe port errors in journal", "hardware",
            &["dmesg | grep -i pcie | tail -10",
              "echo 'FIX: Add pcie_aspm=off to kernel params if needed'"]),
        (&["pcie", "error"], "PCIe errors", "hardware",
            &["dmesg | grep -i pcie | tail -10"]),
        // Input devices
        (&["mouse", "freeze"], "mouse cursor freezing", "hardware",
            &["xinput list", "dmesg | grep -i mouse | tail -5"]),
        (&["cursor", "freeze"], "cursor freezing", "hardware",
            &["xinput list", "echo 'Check compositor vsync settings'"]),
        (&["keyboard", "lag"], "keyboard input lag", "hardware",
            &["xinput list", "cat /sys/module/hid_apple/parameters/* 2>/dev/null"]),
        // Bluetooth
        (&["bluetooth", "disconnect"], "Bluetooth disconnecting", "hardware",
            &["systemctl status bluetooth", "bluetoothctl show",
              "journalctl -u bluetooth -b | tail -20"]),
        (&["bluetooth", "not", "work"], "Bluetooth not working", "hardware",
            &["systemctl status bluetooth", "rfkill list", "hciconfig -a"]),
        // WiFi
        (&["wifi", "drop"], "WiFi connection dropping", "network",
            &["journalctl -u NetworkManager -b | tail -20", "dmesg | grep -i wifi | tail -10"]),
        (&["wifi", "not", "work"], "WiFi not working", "network",
            &["nmcli device status", "rfkill list", "ip link"]),
        // Screen
        (&["screen", "flicker"], "screen flickering", "display",
            &["xrandr --verbose | grep -i refresh",
              "echo 'FIX: Check refresh rate or add nvidia-drm.modeset=1'"]),
        (&["display", "flicker"], "display flickering", "display",
            &["xrandr --verbose | grep -i refresh"]),
        (&["screen", "tear"], "screen tearing", "display",
            &["echo 'FIX: Enable ForceCompositionPipeline in nvidia-settings'",
              "cat /etc/X11/xorg.conf.d/*.conf 2>/dev/null | grep -i tear"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_service_errors(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // Docker
        (&["docker", "dns"], "Docker DNS resolution", "services",
            &["cat /etc/docker/daemon.json 2>/dev/null",
              "echo 'FIX: Add {\"dns\": [\"8.8.8.8\"]} to daemon.json'"]),
        (&["docker", "container", "not", "start"], "Docker container not starting", "services",
            &["docker ps -a | head -5", "docker logs <container> 2>&1 | tail -20"]),
        // Flatpak
        (&["flatpak", "access", "home"], "Flatpak home folder access", "packages",
            &["flatpak override --user --show",
              "echo 'FIX: flatpak override --user --filesystem=home <app>'"]),
        (&["flatpak", "permission"], "Flatpak permissions", "packages",
            &["flatpak info --show-permissions <app>",
              "echo 'Use Flatseal to manage permissions'"]),
        // GNOME keyring
        (&["keyring", "password", "boot"], "GNOME keyring password prompt", "desktop",
            &["echo 'FIX: Set empty password with seahorse or disable autologin'"]),
        (&["gnome-keyring", "unlock"], "GNOME keyring unlock issue", "desktop",
            &["echo 'FIX: rm ~/.local/share/keyrings/* and re-login'"]),
        // Timeshift
        (&["timeshift", "btrfs"], "Timeshift BTRFS snapshot issue", "storage",
            &["btrfs subvolume list /", "echo 'Ensure @ and @home subvolumes exist'"]),
        // Steam/gaming
        (&["steam", "crash"], "Steam game crashing", "gaming",
            &["cat ~/.steam/steam/logs/console_log.txt 2>/dev/null | tail -30",
              "echo 'Check Proton version compatibility'"]),
        (&["proton", "not", "work"], "Proton compatibility issue", "gaming",
            &["echo 'Check: protondb.com for game compatibility'",
              "echo 'Try: PROTON_USE_WINED3D=1 or different Proton version'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}
