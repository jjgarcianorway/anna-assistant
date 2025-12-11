//! Follow-up hints for answer enrichment (v0.0.384).
//!
//! Generates contextual suggestions for what the user might want to know next.
//! Based on domain, query patterns, and successful past interactions.

use crate::rpc::SpecialistDomain;

/// A follow-up hint to append to an answer
#[derive(Debug, Clone)]
pub struct FollowupHint {
    /// The suggestion text
    pub hint: String,
    /// Optional command to try
    pub command: Option<String>,
    /// Relevance score (0-100)
    pub relevance: u8,
}

/// Generate follow-up hints based on the query and domain
pub fn generate_followup_hints(
    query: &str,
    domain: SpecialistDomain,
    _answer: &str,
) -> Vec<FollowupHint> {
    let query_lower = query.to_lowercase();
    let mut hints = Vec::new();

    // Domain-specific follow-up suggestions
    match domain {
        SpecialistDomain::Storage => {
            hints.extend(storage_followups(&query_lower));
        }
        SpecialistDomain::System => {
            hints.extend(system_followups(&query_lower));
        }
        SpecialistDomain::Network => {
            hints.extend(network_followups(&query_lower));
        }
        SpecialistDomain::Security => {
            hints.extend(security_followups(&query_lower));
        }
        SpecialistDomain::Packages => {
            hints.extend(package_followups(&query_lower));
        }
        // v0.0.405: New domains - basic hints for now
        SpecialistDomain::Boot => {
            hints.extend(boot_followups(&query_lower));
        }
        SpecialistDomain::Services => {
            hints.extend(services_followups(&query_lower));
        }
        SpecialistDomain::Audio => {
            hints.extend(audio_followups(&query_lower));
        }
        SpecialistDomain::Display => {
            hints.extend(display_followups(&query_lower));
        }
        SpecialistDomain::Desktop => {
            hints.extend(desktop_followups(&query_lower));
        }
    }

    // Sort by relevance and take top 2
    hints.sort_by(|a, b| b.relevance.cmp(&a.relevance));
    hints.truncate(2);
    hints
}

fn storage_followups(query: &str) -> Vec<FollowupHint> {
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

fn system_followups(query: &str) -> Vec<FollowupHint> {
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

fn network_followups(query: &str) -> Vec<FollowupHint> {
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

fn security_followups(query: &str) -> Vec<FollowupHint> {
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

fn package_followups(query: &str) -> Vec<FollowupHint> {
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

fn boot_followups(query: &str) -> Vec<FollowupHint> {
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

fn services_followups(query: &str) -> Vec<FollowupHint> {
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

fn audio_followups(query: &str) -> Vec<FollowupHint> {
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

fn display_followups(query: &str) -> Vec<FollowupHint> {
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

fn desktop_followups(query: &str) -> Vec<FollowupHint> {
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

/// Format hints as a string to append to answer
pub fn format_hints(hints: &[FollowupHint]) -> String {
    if hints.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n\n---\n**Related:**");

    for hint in hints {
        if let Some(cmd) = &hint.command {
            output.push_str(&format!("\n- {} → `{}`", hint.hint, cmd));
        } else {
            output.push_str(&format!("\n- {}", hint.hint));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_followups() {
        let hints = generate_followup_hints("how much disk space do I have", SpecialistDomain::Storage, "");
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|h| h.command.is_some()));
    }

    #[test]
    fn test_system_followups() {
        let hints = generate_followup_hints("what processes are using memory", SpecialistDomain::System, "");
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_format_hints() {
        let hints = vec![
            FollowupHint {
                hint: "Test hint".to_string(),
                command: Some("test cmd".to_string()),
                relevance: 80,
            }
        ];
        let formatted = format_hints(&hints);
        assert!(formatted.contains("Related"));
        assert!(formatted.contains("test cmd"));
    }
}
