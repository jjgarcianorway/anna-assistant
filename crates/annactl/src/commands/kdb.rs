//! KDB Command v7.5.0 - Anna KDB Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Rule-based categories from descriptions
//! - [CONFIG HIGHLIGHTS] Config status summary (v7.4.0)
//! - [USAGE HIGHLIGHTS]  Real telemetry from SQLite (v7.5.0 enhanced)
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
    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages known:   {}", pkg_counts.total);
    println!("  Commands known:   {}", cmd_count);
    println!("  Services known:   {}", svc_counts.total);

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
    println!("{}", "[USAGE HIGHLIGHTS]".cyan());
    println!("  {}", "(last 24h)".dimmed());

    // Try to open telemetry database
    match TelemetryDb::open_readonly() {
        Some(db) => {
            let data_status = db.get_data_status();

            match &data_status {
                DataStatus::NoData => {
                    println!("  Telemetry not collected yet.");
                }
                DataStatus::NotEnoughData { minutes } => {
                    println!("  Telemetry warming up ({:.0}m collected, need more data).", minutes);
                }
                DataStatus::PartialWindow { hours } | DataStatus::Ok { hours } => {
                    let horizon = if *hours < 24.0 {
                        format!("since telemetry start ({:.1}h)", hours)
                    } else {
                        "last 24h".to_string()
                    };

                    println!();

                    // Top CPU time (24h) - using new enhanced query
                    if let Ok(top_cpu) = db.top_by_cpu_time(WINDOW_24H, 3) {
                        if !top_cpu.is_empty() {
                            println!("  Top CPU time ({}):", horizon);
                            for entry in &top_cpu {
                                let cpu_str = format!("{} total, peak {:.1}%",
                                    format_cpu_time(entry.cpu_time_secs),
                                    entry.cpu_peak_percent);
                                println!("    {:<14} {}", entry.name.cyan(), cpu_str);
                            }
                            println!();
                        }
                    }

                    // Top memory (RSS peak, 24h)
                    if let Ok(top_mem) = db.top_by_rss_peak(WINDOW_24H, 3) {
                        if !top_mem.is_empty() {
                            println!("  Top memory (RSS peak):");
                            for entry in &top_mem {
                                println!("    {:<14} {}",
                                    entry.name.cyan(),
                                    format_bytes_human(entry.rss_peak_bytes));
                            }
                            println!();
                        }
                    }

                    // Most executed commands (24h)
                    if let Ok(top_exec) = db.top_by_exec_count(WINDOW_24H, 3) {
                        if !top_exec.is_empty() {
                            println!("  Most executed commands:");
                            for entry in &top_exec {
                                println!("    {:<14} {} runs",
                                    entry.name.cyan(),
                                    entry.exec_count);
                            }
                        }
                    }
                }
            }
        }
        None => {
            println!("  Telemetry not collected yet.");
        }
    }

    println!();
}
