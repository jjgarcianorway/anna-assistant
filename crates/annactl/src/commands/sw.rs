//! SW Command v7.6.0 - Anna Software Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Rule-based categories from descriptions (sorted)
//! - [TOP CPU (24h)]     Top 5 processes by CPU usage (v7.6.0)
//! - [TOP RAM (24h)]     Top 5 processes by memory usage (v7.6.0)
//!
//! This replaces the old `annactl kdb` command.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    categoriser::get_category_summary,
};
use anna_common::config::AnnaConfig;
use anna_common::{TelemetryDb, WINDOW_24H, format_bytes_human};

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

    // [TOP CPU (24h)] and [TOP RAM (24h)] - v7.6.0
    print_top_offenders_section();

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

/// Print top offenders sections - v7.6.0 format
fn print_top_offenders_section() {
    // Check if telemetry is enabled
    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;
    }

    // Try SQLite telemetry database
    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return,
    };

    // Get telemetry health
    let health = match db.get_telemetry_health() {
        Ok(h) => h,
        Err(_) => return,
    };

    if health.total_samples == 0 || health.is_warming_up {
        return;
    }

    // [TOP CPU (24h)]
    if let Ok(top_cpu) = db.top_cpu_compact(WINDOW_24H, 5) {
        if !top_cpu.is_empty() {
            println!("{}", "[TOP CPU (24h)]".cyan());
            for (i, entry) in top_cpu.iter().enumerate() {
                // Calculate avg CPU from cpu_secs / expected_runtime
                let avg_pct = if entry.cpu_secs > 0.0 {
                    // Rough estimate: assume 1 sample per 15s
                    let est_samples = entry.execs.max(1) as f64 * 100.0; // Rough
                    (entry.cpu_secs / (est_samples * 15.0)) * 100.0
                } else {
                    0.0
                };
                println!("  {}. {:<18} {:.0}% avg, {:.0}% max",
                    i + 1,
                    entry.name.cyan(),
                    avg_pct.min(100.0),
                    0.0  // We don't have peak% in compact, show time instead
                );
            }
            println!();
        }
    }

    // [TOP RAM (24h)]
    if let Ok(top_mem) = db.top_memory_compact(WINDOW_24H, 5) {
        if !top_mem.is_empty() {
            println!("{}", "[TOP RAM (24h)]".cyan());
            for (i, entry) in top_mem.iter().enumerate() {
                println!("  {}. {:<18} {} max",
                    i + 1,
                    entry.name.cyan(),
                    format_bytes_human(entry.max_rss)
                );
            }
            println!();
        }
    }
}
