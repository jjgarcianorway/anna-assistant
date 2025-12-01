//! Knowledge Category Command v6.0.0 - Grounded Category Listing
//!
//! Lists objects in a specific category using real system queries.
//!
//! Usage:
//!   annactl knowledge editors
//!   annactl knowledge shells
//!   annactl knowledge terminals
//!   annactl knowledge browsers
//!   annactl knowledge compositors
//!   annactl knowledge services
//!   annactl knowledge tools

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::get_package_info,
    commands::{command_exists, get_command_description},
    services::{list_service_units, get_service_info, ServiceState},
};

const THIN_SEP: &str = "------------------------------------------------------------";

// Category detection patterns
const EDITORS: &[&str] = &["vim", "neovim", "nvim", "nano", "emacs", "helix", "hx", "kate", "gedit", "code", "micro"];
const TERMINALS: &[&str] = &["alacritty", "kitty", "foot", "wezterm", "gnome-terminal", "konsole", "st", "xterm"];
const SHELLS: &[&str] = &["bash", "zsh", "fish", "nushell", "dash", "sh"];
const COMPOSITORS: &[&str] = &["hyprland", "sway", "wayfire", "river", "picom", "weston"];
const BROWSERS: &[&str] = &["firefox", "chromium", "brave", "vivaldi", "qutebrowser", "librewolf", "epiphany"];
const TOOLS: &[&str] = &[
    "git", "curl", "wget", "grep", "awk", "sed", "find", "tar", "gzip", "unzip",
    "htop", "btop", "top", "ps", "kill", "make", "cmake", "gcc", "clang", "rustc",
    "python", "node", "npm", "cargo", "docker", "podman", "ssh", "rsync", "tmux",
    "screen", "less", "more", "cat", "head", "tail", "diff", "patch", "man",
    "df", "du", "free", "uptime", "uname", "whoami", "id", "groups", "env",
    "fastfetch", "neofetch", "ffmpeg", "imagemagick", "jq", "yq", "fzf", "ripgrep"
];

/// Run the knowledge category command
pub async fn run(category_name: &str) -> Result<()> {
    let category_lower = category_name.to_lowercase();

    println!();
    println!("{}", format!("  Anna Knowledge: {}", capitalize(&category_lower)).bold());
    println!("{}", THIN_SEP);
    println!();

    match category_lower.as_str() {
        "editors" | "editor" => print_category("editors", EDITORS),
        "terminals" | "terminal" => print_category("terminals", TERMINALS),
        "shells" | "shell" => print_category("shells", SHELLS),
        "browsers" | "browser" => print_category("browsers", BROWSERS),
        "compositors" | "compositor" => print_category("compositors", COMPOSITORS),
        "tools" | "tool" => print_category("tools", TOOLS),
        "services" | "service" => print_services(),
        _ => {
            println!("  Unknown category: '{}'", category_name);
            println!();
            println!("  Available categories:");
            println!("    editors, terminals, shells, browsers,");
            println!("    compositors, services, tools");
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl knowledge <name>' for full profile.");
    println!();

    Ok(())
}

fn print_category(name: &str, patterns: &[&str]) {
    println!("  {}", format!("(detected via which + pacman -Qi)").dimmed());
    println!();

    let mut found: Vec<(String, String, Option<String>)> = Vec::new();

    for &cmd in patterns {
        if command_exists(cmd) {
            let desc = get_command_description(cmd);
            let version = get_package_info(cmd).map(|p| p.version);
            found.push((cmd.to_string(), desc, version));
        }
    }

    if found.is_empty() {
        println!("  No {} found on this system.", name);
    } else {
        println!("  {} {} installed:", found.len(), name);
        println!();

        for (cmd, desc, version) in &found {
            let ver_str = version.as_ref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();

            let desc_display = if desc.is_empty() {
                "".to_string()
            } else {
                // Truncate and clean description
                let clean = desc.split(" (source:").next().unwrap_or(desc);
                if clean.len() > 40 {
                    format!("{}", &clean[..40])
                } else {
                    clean.to_string()
                }
            };

            println!(
                "  {:<20} {}{}",
                cmd.cyan(),
                desc_display,
                ver_str.dimmed()
            );
        }
    }
}

fn print_services() {
    println!("  {}", "(source: systemctl list-unit-files --type=service)".dimmed());
    println!();

    let units = list_service_units();

    // Filter to show only enabled/running services (not all 260+)
    let mut active_services: Vec<(String, String, String)> = Vec::new();

    for (unit, state) in units.iter().take(100) {
        // Only show enabled services
        if state == "enabled" || state == "static" {
            if let Some(svc) = get_service_info(unit.trim_end_matches(".service")) {
                let state_str = match svc.state {
                    ServiceState::Active => "running".green().to_string(),
                    ServiceState::Inactive => "inactive".to_string(),
                    ServiceState::Failed => "failed".red().to_string(),
                    ServiceState::Unknown => "unknown".to_string(),
                };

                let desc = if svc.description.is_empty() {
                    "".to_string()
                } else {
                    let clean = svc.description.split(" (source:").next().unwrap_or(&svc.description);
                    if clean.len() > 35 {
                        format!("{}...", &clean[..32])
                    } else {
                        clean.to_string()
                    }
                };

                active_services.push((unit.clone(), state_str, desc));
            }
        }
    }

    // Sort by name
    active_services.sort_by(|a, b| a.0.cmp(&b.0));

    if active_services.is_empty() {
        println!("  No enabled services found.");
    } else {
        println!("  {} enabled services (showing first 20):", active_services.len());
        println!();

        for (unit, state, desc) in active_services.iter().take(20) {
            println!(
                "  {:<30} [{}] {}",
                unit.cyan(),
                state,
                desc.dimmed()
            );
        }

        if active_services.len() > 20 {
            println!();
            println!("  ({} more enabled services)", active_services.len() - 20);
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
