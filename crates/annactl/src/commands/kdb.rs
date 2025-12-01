//! KDB Command v7.2.0 - Anna KDB Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Rule-based categories from descriptions
//! - [USAGE HIGHLIGHTS]  Real telemetry from SQLite
//!
//! NO journalctl system errors. NO generic host health.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    categoriser::get_category_summary,
};
use anna_common::{TelemetryDb, DataStatus};

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

fn print_usage_section() {
    println!("{}", "[USAGE HIGHLIGHTS]".cyan());

    // Try to open telemetry database
    match TelemetryDb::open_readonly() {
        Some(db) => {
            let data_status = db.get_data_status();

            match &data_status {
                DataStatus::NoData => {
                    println!("  Telemetry:    {}", "no data".yellow());
                    println!("  {}", "(daemon needs to collect samples)".dimmed());
                }
                DataStatus::NotEnoughData { minutes } => {
                    println!("  Telemetry:    {} ({:.0}m collected)", "not enough data".yellow(), minutes);
                    println!("  {}", "(need at least 10 minutes of data)".dimmed());
                }
                DataStatus::PartialWindow { hours } | DataStatus::Ok { hours } => {
                    // Show telemetry status
                    let status_str = if matches!(data_status, DataStatus::PartialWindow { .. }) {
                        format!("{} ({:.1}h)", "partial".yellow(), hours)
                    } else {
                        format!("{} ({:.1}h)", "OK".green(), hours)
                    };
                    println!("  Telemetry:    {}", status_str);

                    // Get stats
                    if let Ok(stats) = db.get_stats() {
                        println!("  Samples:      {} total", stats.total_samples);

                        // Data window
                        if stats.first_sample_at > 0 && stats.last_sample_at > 0 {
                            let first = TelemetryDb::format_timestamp(stats.first_sample_at);
                            let last = TelemetryDb::format_timestamp(stats.last_sample_at);
                            println!("  Window:       {} → {}", first.dimmed(), last.dimmed());
                        }
                    }

                    println!();

                    // Top launches (24h)
                    if let Ok(top_launches) = db.top_by_launches_24h(5) {
                        if !top_launches.is_empty() {
                            println!("  Top activity (24h):");
                            for (i, (name, count)) in top_launches.iter().enumerate() {
                                println!("    {}) {:<12} {}", i + 1, name.cyan(), count);
                            }
                            println!();
                        }
                    }

                    // Top CPU (avg, 24h)
                    if let Ok(top_cpu) = db.top_by_avg_cpu_24h(5) {
                        if !top_cpu.is_empty() {
                            println!("  Top CPU avg (24h):");
                            for (i, (name, avg)) in top_cpu.iter().enumerate() {
                                println!("    {}) {:<12} {:.1}%", i + 1, name.cyan(), avg);
                            }
                            println!();
                        }
                    }

                    // Top memory (avg RSS, 24h)
                    if let Ok(top_mem) = db.top_by_avg_memory_24h(5) {
                        if !top_mem.is_empty() {
                            println!("  Top memory avg (24h):");
                            for (i, (name, bytes)) in top_mem.iter().enumerate() {
                                println!("    {}) {:<12} {}", i + 1, name.cyan(), format_bytes(*bytes));
                            }
                        }
                    }
                }
            }
        }
        None => {
            println!("  Telemetry:    {}", "unavailable".dimmed());
            println!("  {}", "(daemon not collecting telemetry)".dimmed());
        }
    }

    println!();
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
