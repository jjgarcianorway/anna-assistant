//! KDB Command v7.7.0 - Anna KDB Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services + docs status
//! - [CATEGORIES]        Rule-based categories from descriptions (sorted)
//! - [CONFIG HIGHLIGHTS] Config status summary (v7.4.0)
//! - [USAGE HIGHLIGHTS]  Real telemetry from SQLite (v7.5.0+)
//!
//! v7.6.0: Sorted categories, respects telemetry.enabled
//! v7.7.0: PHASE 23 - Compact CPU/memory highlights, docs status in overview
//!
//! NO journalctl system errors. NO generic host health.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::{ServiceCounts, list_enabled_services},
    categoriser::get_category_summary,
    config::get_config_highlights,
};
use anna_common::config::AnnaConfig;
use anna_common::{TelemetryDb, DataStatus, WINDOW_24H, format_cpu_time, format_bytes_human};

const THIN_SEP: &str = "------------------------------------------------------------";
const MAX_CATEGORY_ITEMS: usize = 10;

/// Run the kdb overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna KDB".bold());
    println!("{}", THIN_SEP);
    println!();

    // [OVERVIEW]
    print_overview_section();

    // [CATEGORIES]
    print_categories_section();

    // [CONFIG HIGHLIGHTS] (v7.4.0)
    print_config_highlights_section();

    // [USAGE HIGHLIGHTS]
    print_usage_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview_section() {
    use anna_common::grounded::config::is_arch_wiki_available;

    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages known:   {}", pkg_counts.total);
    println!("  Commands known:   {}", cmd_count);
    println!("  Services known:   {}", svc_counts.total);

    // Show docs availability status (PHASE 24)
    if is_arch_wiki_available() {
        println!("  Local Arch docs:  {}", "available (used for config paths and categorisation)".dimmed());
    } else {
        println!("  Local Arch docs:  {}", "not detected (using pacman and man only)".dimmed());
    }

    println!();
}

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

            let display: String = if packages.len() <= MAX_CATEGORY_ITEMS {
                packages.join(", ")
            } else {
                format!("{}, ...", packages.iter().take(MAX_CATEGORY_ITEMS).cloned().collect::<Vec<_>>().join(", "))
            };

            // Format category name with padding
            let cat_display = format!("{}:", cat_name);
            println!("  {:<14} {}", cat_display, display);
        }
    }

    println!();
}

fn print_config_highlights_section() {
    println!("{}", "[CONFIG HIGHLIGHTS]".cyan());
    println!("  {}", "(from pacman, man, systemctl)".dimmed());

    // Get a list of important packages to check
    let categories = get_category_summary();
    let mut important_packages: Vec<String> = Vec::new();

    // Collect packages from key categories
    for (cat_name, packages) in &categories {
        if matches!(cat_name.as_str(), "Editors" | "Terminals" | "Shells" | "Compositors" | "Browsers" | "Multimedia" | "Power") {
            important_packages.extend(packages.iter().take(5).cloned());
        }
    }
    important_packages.truncate(30);

    // Get enabled services
    let services: Vec<String> = list_enabled_services()
        .into_iter()
        .filter(|s| !s.contains('@'))
        .take(20)
        .map(|s| s.trim_end_matches(".service").to_string())
        .collect();

    // Get config highlights
    let highlights = get_config_highlights(&important_packages, &services);

    let mut has_any = false;

    // User configs present
    if !highlights.user_configs_present.is_empty() {
        println!("  User configs present:");
        println!("    {}", highlights.user_configs_present.join(", ").cyan());
        has_any = true;
    }

    // Services with overrides
    if !highlights.services_with_overrides.is_empty() {
        if has_any { println!(); }
        println!("  Services with overrides:");
        for (svc, desc) in &highlights.services_with_overrides {
            println!("    {} {}", svc.cyan(), format!("({})", desc).dimmed());
        }
        has_any = true;
    }

    // Default config only
    if !highlights.default_config_only.is_empty() && !has_any {
        println!("  Using default config:");
        println!("    {}", highlights.default_config_only.join(", ").dimmed());
        has_any = true;
    }

    if !has_any {
        println!("  {}", "(no config customizations detected)".dimmed());
    }

    println!();
}

fn print_usage_section() {
    println!("{}", "[USAGE HIGHLIGHTS]  (top by CPU, last 24h)".cyan());

    // Check if telemetry is enabled
    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  Telemetry disabled in config (/etc/anna/config.toml).");
        println!();
        return;
    }

    // Try to open telemetry database
    match TelemetryDb::open_readonly() {
        Some(db) => {
            let data_status = db.get_data_status();

            match &data_status {
                DataStatus::NoData | DataStatus::Disabled => {
                    println!("  Telemetry disabled or insufficient data (less than 1h of samples).");
                }
                DataStatus::NotEnoughData { minutes } => {
                    println!("  Telemetry disabled or insufficient data (less than 1h of samples, {:.0}m).", minutes);
                }
                DataStatus::PartialWindow { .. } | DataStatus::Ok { .. } => {
                    println!();

                    // Top CPU-heavy commands (24h) - using new compact query
                    if let Ok(top_cpu) = db.top_cpu_compact(WINDOW_24H, 3) {
                        if !top_cpu.is_empty() {
                            println!("  Top CPU-heavy commands:");
                            for entry in &top_cpu {
                                println!("    {:<14} cpu={}  execs={}  max_rss={}",
                                    entry.name.cyan(),
                                    format_cpu_time(entry.cpu_secs),
                                    entry.execs,
                                    format_bytes_human(entry.max_rss)
                                );
                            }
                            println!();
                        }
                    }

                    // Top memory-heavy commands (24h)
                    if let Ok(top_mem) = db.top_memory_compact(WINDOW_24H, 3) {
                        if !top_mem.is_empty() {
                            println!("  Top memory-heavy commands:");
                            for entry in &top_mem {
                                println!("    {:<14} cpu={}  execs={}  max_rss={}",
                                    entry.name.cyan(),
                                    format_cpu_time(entry.cpu_secs),
                                    entry.execs,
                                    format_bytes_human(entry.max_rss)
                                );
                            }
                        }
                    }
                }
            }
        }
        None => {
            println!("  Telemetry disabled or insufficient data (less than 1h of samples).");
        }
    }

    println!();
}
