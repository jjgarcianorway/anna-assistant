//! Knowledge Stats Command v5.2.6 - Coverage and Quality Metrics
//!
//! Shows knowledge coverage and quality statistics.
//! Focuses on what percentage of the system Anna understands.
//!
//! v5.2.6: Every metric has explicit scope/units.
//!
//! Sections:
//! - [COVERAGE] Object coverage (indexed / total on system)
//! - [QUALITY] Metadata completeness for installed objects
//! - [TIMELINE] When objects were first discovered

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeStore,
    count_path_binaries, count_systemd_services,
    format_time_ago, format_percent, get_description,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    let store = KnowledgeStore::load();

    // [COVERAGE]
    print_coverage_section(&store);

    // [QUALITY]
    print_quality_section(&store);

    // [TIMELINE]
    print_timeline_section(&store);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_coverage_section(store: &KnowledgeStore) {
    println!("{}", "[COVERAGE]".cyan());
    println!("  {}", "(indexed / total on system)".dimmed());

    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();

    // Coverage percentages
    let cmd_coverage = (commands as f64 / total_path_cmds.max(1) as f64) * 100.0;
    let svc_coverage = (services as f64 / total_services.max(1) as f64) * 100.0;

    println!(
        "  Commands:    {}/{} {}",
        commands,
        total_path_cmds,
        format_coverage_bar(cmd_coverage)
    );
    println!(
        "  Services:    {}/{} {}",
        services,
        total_services,
        format_coverage_bar(svc_coverage)
    );
    println!("  Packages:    {} (from pacman)", packages);

    // Overall coverage
    let total_possible = total_path_cmds + total_services;
    let total_known = store.total_objects();
    let overall = (total_known as f64 / total_possible.max(1) as f64 * 100.0).min(100.0);
    println!("  Overall:     {}", format_percent(overall));

    println!();
}

fn print_quality_section(store: &KnowledgeStore) {
    println!("{}", "[QUALITY]".cyan());
    println!("  {}", "(metadata completeness for installed objects)".dimmed());

    let installed = store.objects.values().filter(|o| o.installed).count();
    if installed == 0 {
        println!("  No installed objects to measure");
        println!();
        return;
    }

    // Count objects with various metadata
    let mut with_description = 0;
    let mut with_version = 0;
    let mut with_config = 0;
    let mut with_usage = 0;

    for obj in store.objects.values() {
        if !obj.installed {
            continue;
        }

        if get_description(&obj.name).is_some() {
            with_description += 1;
        }
        if obj.package_version.is_some() {
            with_version += 1;
        }
        if !obj.config_paths.is_empty() {
            with_config += 1;
        }
        if obj.usage_count > 0 {
            with_usage += 1;
        }
    }

    let desc_pct = (with_description as f64 / installed as f64) * 100.0;
    let ver_pct = (with_version as f64 / installed as f64) * 100.0;
    let cfg_pct = (with_config as f64 / installed as f64) * 100.0;
    let usage_pct = (with_usage as f64 / installed as f64) * 100.0;

    println!(
        "  Descriptions:  {}/{} {}",
        with_description,
        installed,
        format_coverage_bar(desc_pct)
    );
    println!(
        "  Versions:      {}/{} {}",
        with_version,
        installed,
        format_coverage_bar(ver_pct)
    );
    println!(
        "  Config paths:  {}/{} {}",
        with_config,
        installed,
        format_coverage_bar(cfg_pct)
    );
    println!(
        "  Usage data:    {}/{} {}",
        with_usage,
        installed,
        format_coverage_bar(usage_pct)
    );

    // Overall quality score (average of metrics)
    let quality_score = (desc_pct + ver_pct + cfg_pct + usage_pct) / 4.0;
    let quality_label = if quality_score >= 80.0 {
        "excellent".green().to_string()
    } else if quality_score >= 60.0 {
        "good".to_string()
    } else if quality_score >= 40.0 {
        "fair".yellow().to_string()
    } else {
        "needs work".red().to_string()
    };

    println!(
        "  Quality:       {} ({})",
        format_percent(quality_score),
        quality_label
    );

    println!();
}

fn print_timeline_section(store: &KnowledgeStore) {
    println!("{}", "[TIMELINE]".cyan());
    println!("  {}", "(when objects were first discovered)".dimmed());

    // First and last discovery times
    let first_seen = store
        .objects
        .values()
        .filter(|o| o.first_seen_at > 0)
        .map(|o| o.first_seen_at)
        .min();

    let last_seen = store
        .objects
        .values()
        .filter(|o| o.first_seen_at > 0)
        .map(|o| o.first_seen_at)
        .max();

    // Objects with usage data
    let with_usage = store.objects.values().filter(|o| o.usage_count > 0).count();

    if let Some(first) = first_seen {
        println!("  First indexed: {}", format_time_ago(first));
    } else {
        println!("  First indexed: n/a");
    }

    if let Some(last) = last_seen {
        println!("  Last indexed:  {}", format_time_ago(last));
    } else {
        println!("  Last indexed:  n/a");
    }

    println!("  With usage:    {} objects have been observed running", with_usage);

    println!();
}

/// Format a coverage percentage as a visual bar
fn format_coverage_bar(pct: f64) -> String {
    let filled = ((pct / 100.0) * 10.0).round() as usize;
    let empty = 10 - filled;

    let bar = format!(
        "[{}{}]",
        "#".repeat(filled),
        "-".repeat(empty)
    );

    let pct_str = format_percent(pct);

    if pct >= 80.0 {
        format!("{} {}", bar.green(), pct_str)
    } else if pct >= 50.0 {
        format!("{} {}", bar, pct_str)
    } else {
        format!("{} {}", bar.yellow(), pct_str)
    }
}
