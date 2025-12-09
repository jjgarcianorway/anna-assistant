//! Theatre rendering helper functions (v0.0.202).

use anna_shared::teams::Team;
use anna_shared::ui::colors;

/// Get color for reliability score
pub fn reliability_color(score: u8) -> &'static str {
    match score {
        80..=100 => colors::OK,
        50..=79 => colors::WARN,
        _ => colors::ERR,
    }
}

/// Map domain string to Team
pub fn team_from_domain(domain: &str) -> Team {
    match domain {
        "storage" => Team::Storage,
        "memory" => Team::Performance,
        "network" => Team::Network,
        "performance" | "cpu" => Team::Performance,
        "service" | "services" => Team::Services,
        "security" => Team::Security,
        "hardware" | "audio" => Team::Hardware,
        "desktop" | "editor" => Team::Desktop,
        "logs" => Team::Logs,
        _ => Team::General,
    }
}

/// Extract probe ID from command
pub fn probe_id_from_command(command: &str) -> String {
    let cmd = command.to_lowercase();
    if cmd.starts_with("df ") || cmd == "df" {
        return "df".to_string();
    }
    if cmd.starts_with("free ") || cmd == "free" {
        return "free".to_string();
    }
    if cmd.starts_with("lscpu") {
        return "lscpu".to_string();
    }
    if cmd.contains("sensors") {
        return "sensors".to_string();
    }
    if cmd.starts_with("systemctl") {
        return "systemctl".to_string();
    }
    if cmd.contains("journalctl") {
        return "journalctl".to_string();
    }
    if cmd.starts_with("ip ") {
        return "ip".to_string();
    }
    if cmd.contains("lspci") && cmd.contains("audio") {
        return "lspci_audio".to_string();
    }
    if cmd.contains("pactl") {
        return "pactl_cards".to_string();
    }
    if cmd.starts_with("lsblk") {
        return "lsblk".to_string();
    }
    if cmd.starts_with("uname") {
        return "uname".to_string();
    }
    if cmd.contains("command -v") {
        return "command_v".to_string();
    }
    "probe".to_string()
}
