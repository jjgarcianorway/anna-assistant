//! SW Command v7.30.0 - Anna Software Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Rule-based categories from descriptions (sorted)
//! - [CONFIG COVERAGE]   Config detection summary (v7.30.0)
//! - [TOPOLOGY]          Software stack roles and service groups (v7.21.0)
//! - [IMPACT]            Top resource consumers from telemetry (v7.21.0)
//! - [HOTSPOTS]          CPU, memory, most started processes (v7.24.0)

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    categoriser::get_category_summary,
};
use anna_common::config::AnnaConfig;
use anna_common::topology_map::build_software_topology;
use anna_common::impact_view::get_software_impact;
use anna_common::hotspots::{get_software_hotspots, format_software_hotspots_section};
use anna_common::config_locator::discover_config;

const THIN_SEP: &str = "------------------------------------------------------------";
const MAX_CATEGORY_ITEMS: usize = 10;

/// Run the sw overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Software".bold());
    println!("{}", THIN_SEP);
    println!();

    // [OVERVIEW]
    print_overview_section();

    // [CATEGORIES]
    print_categories_section();

    // [CONFIG COVERAGE] - v7.30.0
    print_config_coverage_section();

    // [TOPOLOGY] - v7.21.0
    print_topology_section();

    // [IMPACT] - v7.21.0
    print_impact_section();

    // [HOTSPOTS] - v7.24.0
    print_hotspots_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview_section() {
    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages known:   {}", pkg_counts.total);
    println!("  Commands known:   {}", cmd_count);
    println!("  Services known:   {}", svc_counts.total);

    println!();
}

/// Known apps to check for config coverage
/// Limited to: Editors, Terminals, Compositors, Multimedia, Network, Power, System
const CONFIG_COVERAGE_APPS: &[&str] = &[
    // Editors
    "vim", "nvim", "neovim", "emacs", "nano", "helix", "kakoune", "micro",
    // Terminals
    "alacritty", "kitty", "foot", "wezterm", "termite", "st", "urxvt", "xterm",
    // Compositors
    "hyprland", "sway", "wayfire", "river", "dwl", "i3", "bspwm", "openbox",
    // Multimedia
    "mpv", "vlc", "mplayer", "mpd", "ncmpcpp", "cmus", "pipewire", "pulseaudio",
    // Network
    "networkmanager", "iwd", "wpa_supplicant", "systemd-networkd", "connman",
    // Power
    "tlp", "powertop", "acpid", "thermald", "auto-cpufreq",
    // System
    "systemd", "grub", "mkinitcpio", "pacman", "yay", "paru",
];

fn print_categories_section() {
    println!("{}", "[CATEGORIES]".cyan());
    println!("  {}", "(from pacman descriptions)".dimmed());

    let categories = get_category_summary();

    if categories.is_empty() {
        println!("  {}", "(no categories detected)".dimmed());
    } else {
        for (cat_name, packages) in categories {
            // Skip "Other" in overview
            if cat_name == "Other" {
                continue;
            }

            // v7.29.0: No truncation - show all items, or indicate count
            let display: String = if packages.len() <= MAX_CATEGORY_ITEMS {
                packages.join(", ")
            } else {
                format!("{} (and {} more)",
                    packages.iter().take(MAX_CATEGORY_ITEMS).cloned().collect::<Vec<_>>().join(", "),
                    packages.len() - MAX_CATEGORY_ITEMS)
            };

            // Format category name with padding
            let cat_display = format!("{}:", cat_name);
            println!("  {:<14} {}", cat_display, display);
        }
    }

    println!();
}

/// Print [TOPOLOGY] section - v7.21.0
/// Shows software stack roles and service groups
fn print_topology_section() {
    let topology = build_software_topology();

    // Stack roles (display, network, audio, etc.)
    if !topology.roles.is_empty() {
        println!("{}", "[TOPOLOGY]".cyan());
        println!("  {}", "(from package descriptions and deps)".dimmed());

        println!("  Stacks:");
        for role in &topology.roles {
            let components: String = role.components.iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = if role.components.len() > 5 {
                format!(", +{}", role.components.len() - 5)
            } else {
                String::new()
            };
            println!("    {:<12} {}{}", role.name.cyan(), components, suffix);
        }

        // Service groups
        if !topology.service_groups.is_empty() {
            println!("  Service Groups:");
            for group in &topology.service_groups {
                let services: String = group.services.iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if group.services.len() > 4 {
                    format!(", +{}", group.services.len() - 4)
                } else {
                    String::new()
                };
                println!("    {:<12} {}{}", group.name.cyan(), services, suffix);
            }
        }

        println!();
    }
}

/// Print [IMPACT] section - v7.21.0
/// Shows top resource consumers from telemetry
fn print_impact_section() {
    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;
    }

    let impact = get_software_impact(5);
    if !impact.has_data {
        return;
    }

    println!("{}", "[IMPACT]".cyan());
    println!("  {}", "(from telemetry, last 24h)".dimmed());

    // Top CPU consumers
    if !impact.cpu_consumers.is_empty() {
        println!("  CPU:");
        for (i, entry) in impact.cpu_consumers.iter().take(5).enumerate() {
            println!("    {}. {:<18} {}% avg", i + 1, entry.name.cyan(), entry.formatted);
        }
    }

    // Top memory consumers
    if !impact.memory_consumers.is_empty() {
        println!("  Memory:");
        for (i, entry) in impact.memory_consumers.iter().take(5).enumerate() {
            println!("    {}. {:<18} {}", i + 1, entry.name.cyan(), entry.formatted);
        }
    }

    println!();
}

/// Print [HOTSPOTS] section - v7.24.0
/// Shows CPU, memory, and most started processes from telemetry
fn print_hotspots_section() {
    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;
    }

    let hotspots = get_software_hotspots();
    if !hotspots.has_data {
        return;
    }

    let lines = format_software_hotspots_section(&hotspots);
    for line in lines {
        if line.starts_with("[HOTSPOTS]") {
            println!("{}", line.cyan());
        } else {
            println!("{}", line);
        }
    }
    println!();
}

/// Print [CONFIG COVERAGE] section - v7.30.0
/// Shows how many known apps have detected config files
fn print_config_coverage_section() {
    let mut with_config = 0;
    let mut apps_with_config: Vec<&str> = Vec::new();

    for &app in CONFIG_COVERAGE_APPS {
        let discovery = discover_config(app);
        if !discovery.detected.is_empty() {
            with_config += 1;
            apps_with_config.push(app);
        }
    }

    let total = CONFIG_COVERAGE_APPS.len();

    // Only show if we have any coverage
    if with_config == 0 {
        return;
    }

    println!("{}", "[CONFIG COVERAGE]".cyan());
    println!("  {}", "(apps with detected config files)".dimmed());

    // Show coverage ratio
    let pct = (with_config as f64 / total as f64 * 100.0) as u32;
    println!("  Coverage: {}/{} known apps ({}%)", with_config, total, pct);

    // List apps with config (up to 10)
    if !apps_with_config.is_empty() {
        let display: String = if apps_with_config.len() <= 10 {
            apps_with_config.join(", ")
        } else {
            format!("{} (+{} more)",
                apps_with_config.iter().take(10).cloned().collect::<Vec<_>>().join(", "),
                apps_with_config.len() - 10)
        };
        println!("  Detected: {}", display);
    }

    println!();
}
