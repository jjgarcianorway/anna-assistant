//! Domain-specific follow-up hint generators.

use super::types::FollowupHint;

pub fn storage_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("disk") || query.contains("space") || query.contains("full") {
        hints.push(FollowupHint {
            hint: "Want to find what's using the most space?".to_string(),
            command: Some("du -sh /* 2>/dev/null | sort -hr | head -10".to_string()),
            relevance: 85,
        });
    }

    if query.contains("mount") || query.contains("drive") {
        hints.push(FollowupHint {
            hint: "To check disk health and SMART status".to_string(),
            command: Some("sudo smartctl -a /dev/sda".to_string()),
            relevance: 70,
        });
    }

    if query.contains("partition") || query.contains("format") {
        hints.push(FollowupHint {
            hint: "View detailed partition layout".to_string(),
            command: Some("lsblk -f".to_string()),
            relevance: 80,
        });
    }

    hints
}

pub fn system_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("memory") || query.contains("ram") {
        hints.push(FollowupHint {
            hint: "Want to see what's using the most memory?".to_string(),
            command: Some("ps aux --sort=-%mem | head -10".to_string()),
            relevance: 85,
        });
    }

    if query.contains("cpu") || query.contains("process") || query.contains("slow") {
        hints.push(FollowupHint {
            hint: "Check for CPU-heavy processes".to_string(),
            command: Some("ps aux --sort=-%cpu | head -10".to_string()),
            relevance: 85,
        });
    }

    if query.contains("service") || query.contains("systemd") {
        hints.push(FollowupHint {
            hint: "View recent service logs".to_string(),
            command: Some("journalctl -xe --no-pager | tail -30".to_string()),
            relevance: 75,
        });
    }

    if query.contains("boot") || query.contains("startup") {
        hints.push(FollowupHint {
            hint: "Analyze boot performance".to_string(),
            command: Some("systemd-analyze blame | head -15".to_string()),
            relevance: 80,
        });
    }

    hints
}

pub fn network_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("ip") || query.contains("address") || query.contains("interface") {
        hints.push(FollowupHint {
            hint: "Check network connectivity".to_string(),
            command: Some("ping -c 3 8.8.8.8".to_string()),
            relevance: 75,
        });
    }

    if query.contains("dns") || query.contains("resolve") {
        hints.push(FollowupHint {
            hint: "Test DNS resolution".to_string(),
            command: Some("dig google.com +short".to_string()),
            relevance: 85,
        });
    }

    if query.contains("port") || query.contains("listen") || query.contains("connection") {
        hints.push(FollowupHint {
            hint: "See what's listening on all ports".to_string(),
            command: Some("ss -tulpn".to_string()),
            relevance: 80,
        });
    }

    if query.contains("wifi") || query.contains("wireless") {
        hints.push(FollowupHint {
            hint: "Check WiFi signal strength".to_string(),
            command: Some("iwconfig 2>/dev/null || nmcli dev wifi".to_string()),
            relevance: 80,
        });
    }

    hints
}

pub fn security_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("permission") || query.contains("chmod") || query.contains("access") {
        hints.push(FollowupHint {
            hint: "Find files with unusual permissions".to_string(),
            command: Some("find /home -perm /go+w -type f 2>/dev/null | head -10".to_string()),
            relevance: 75,
        });
    }

    if query.contains("firewall") || query.contains("port") {
        hints.push(FollowupHint {
            hint: "List current firewall rules".to_string(),
            command: Some("sudo iptables -L -n || sudo nft list ruleset".to_string()),
            relevance: 80,
        });
    }

    if query.contains("ssh") || query.contains("login") {
        hints.push(FollowupHint {
            hint: "Check recent login attempts".to_string(),
            command: Some("last -10 && lastb -10 2>/dev/null".to_string()),
            relevance: 85,
        });
    }

    hints
}

pub fn package_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("install") || query.contains("update") {
        hints.push(FollowupHint {
            hint: "Check for available updates".to_string(),
            command: None, // Distro-specific, handled by prompts.rs
            relevance: 75,
        });
    }

    if query.contains("remove") || query.contains("uninstall") {
        hints.push(FollowupHint {
            hint: "Clean up unused dependencies afterwards".to_string(),
            command: None, // Distro-specific
            relevance: 70,
        });
    }

    if query.contains("broken") || query.contains("dependency") {
        hints.push(FollowupHint {
            hint: "Try checking for broken packages".to_string(),
            command: None, // Distro-specific
            relevance: 85,
        });
    }

    hints
}

// v0.0.405: New domain followup functions

pub fn boot_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("slow") || query.contains("time") || query.contains("long") {
        hints.push(FollowupHint {
            hint: "See what's slowing down boot".to_string(),
            command: Some("systemd-analyze blame | head -10".to_string()),
            relevance: 90,
        });
    }

    if query.contains("fail") || query.contains("error") {
        hints.push(FollowupHint {
            hint: "Check for boot errors".to_string(),
            command: Some("journalctl -b -p err".to_string()),
            relevance: 85,
        });
    }

    hints
}

pub fn services_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("fail") || query.contains("error") {
        hints.push(FollowupHint {
            hint: "View logs for failed services".to_string(),
            command: Some("journalctl -xe --no-pager | tail -30".to_string()),
            relevance: 85,
        });
    }

    if query.contains("start") || query.contains("enable") {
        hints.push(FollowupHint {
            hint: "Enable service to start on boot".to_string(),
            command: None, // Service-specific
            relevance: 75,
        });
    }

    hints
}

pub fn audio_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("no sound") || query.contains("mute") || query.contains("silent") {
        hints.push(FollowupHint {
            hint: "Check if output is muted".to_string(),
            command: Some("wpctl status || pactl list sinks".to_string()),
            relevance: 90,
        });
    }

    if query.contains("device") || query.contains("speaker") || query.contains("headphone") {
        hints.push(FollowupHint {
            hint: "List all audio devices".to_string(),
            command: Some("pactl list sinks short".to_string()),
            relevance: 80,
        });
    }

    hints
}

pub fn display_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("resolution") || query.contains("screen") || query.contains("monitor") {
        hints.push(FollowupHint {
            hint: "List available resolutions".to_string(),
            command: Some("xrandr 2>/dev/null || wlr-randr".to_string()),
            relevance: 85,
        });
    }

    if query.contains("driver") || query.contains("gpu") || query.contains("graphics") {
        hints.push(FollowupHint {
            hint: "Check GPU driver in use".to_string(),
            command: Some("glxinfo | grep -i 'renderer\\|vendor'".to_string()),
            relevance: 85,
        });
    }

    hints
}

pub fn desktop_followups(query: &str) -> Vec<FollowupHint> {
    let mut hints = Vec::new();

    if query.contains("config") || query.contains("setting") {
        hints.push(FollowupHint {
            hint: "Reload config without restart".to_string(),
            command: None, // DE-specific
            relevance: 75,
        });
    }

    if query.contains("hyprland") || query.contains("hypr") {
        hints.push(FollowupHint {
            hint: "Check Hyprland config syntax".to_string(),
            command: Some("hyprctl reload".to_string()),
            relevance: 80,
        });
    }

    hints
}
