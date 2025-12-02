//! SW Command v7.41.0 - Snapshot-Only Software Overview
//!
//! ARCHITECTURE RULE: annactl NEVER does heavyweight scanning.
//! This command ONLY reads snapshots written by annad.
//!
//! v7.41.0: Pure snapshot reader
//! - Reads /var/lib/anna/internal/snapshots/sw.json
//! - If snapshot missing, shows "checking..." and waits briefly
//! - If still missing, shows minimal info with guidance
//! - NEVER calls build_sw_cache() or any expensive operations
//!
//! Output modes:
//! - Default (compact): Summary counts + key highlights
//! - --full: All sections with full details
//! - --json: Raw snapshot JSON
//! - --section <name>: Single section only

use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::{self, Write};

use anna_common::snapshots::SwSnapshot;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Output mode for sw command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwOutputMode {
    /// Default compact view
    Compact,
    /// Full detailed view (--full)
    Full,
    /// JSON output (--json)
    Json,
    /// Single section (--section <name>)
    Section(String),
}

/// Run the sw overview command (compact mode)
pub async fn run() -> Result<()> {
    run_with_mode(SwOutputMode::Compact).await
}

/// Run the sw command with full output
pub async fn run_full() -> Result<()> {
    run_with_mode(SwOutputMode::Full).await
}

/// Run the sw command with JSON output
pub async fn run_json() -> Result<()> {
    run_with_mode(SwOutputMode::Json).await
}

/// Run with specified output mode
pub async fn run_with_mode(mode: SwOutputMode) -> Result<()> {
    // Try to load snapshot (NEVER build it - that's daemon's job)
    let snapshot = match SwSnapshot::load() {
        Some(s) => s,
        None => {
            // Snapshot doesn't exist - show "checking..." and wait briefly
            eprint!("  {} ", "checking...".dimmed());
            io::stderr().flush().ok();

            // Wait up to 2 seconds for daemon to create snapshot
            for _ in 0..4 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Some(s) = SwSnapshot::load() {
                    eprintln!();
                    return display_snapshot(&s, &mode);
                }
            }

            // Still no snapshot - show guidance
            eprintln!();
            return show_no_snapshot_message();
        }
    };

    display_snapshot(&snapshot, &mode)
}

fn display_snapshot(snapshot: &SwSnapshot, mode: &SwOutputMode) -> Result<()> {
    // JSON mode - just dump the snapshot
    if *mode == SwOutputMode::Json {
        let json = serde_json::to_string_pretty(snapshot)
            .unwrap_or_else(|_| "{}".to_string());
        println!("{}", json);
        return Ok(());
    }

    // Section mode
    if let SwOutputMode::Section(ref section) = mode {
        return display_section(snapshot, section);
    }

    let is_full = *mode == SwOutputMode::Full;

    println!();
    println!("{}", "  Anna Software".bold());
    println!("{}", THIN_SEP);

    // Show snapshot freshness (compact mode only)
    if !is_full {
        println!("  {} {} (scanned in {}ms)",
            "◆".dimmed(),
            snapshot.format_age().dimmed(),
            snapshot.scan_duration_ms);
    }
    println!();

    // [OVERVIEW] - always shown
    print_overview(snapshot);

    // [CATEGORIES] - compact shows top categories, full shows all
    print_categories(snapshot, is_full);

    // [PLATFORMS] - only if Steam present
    if snapshot.platforms.steam_installed {
        print_platforms(snapshot, is_full);
    }

    // [CONFIG COVERAGE] - compact summary, full shows list
    if snapshot.config_coverage.apps_with_config > 0 {
        print_config_coverage(snapshot, is_full);
    }

    // [TOPOLOGY] - compact shows stacks only, full shows service groups too
    if !snapshot.topology.roles.is_empty() {
        print_topology(snapshot, is_full);
    }

    // [SERVICES] - show failed services if any (full mode or if failures exist)
    if is_full || snapshot.services.failed > 0 {
        print_services(snapshot, is_full);
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview(snapshot: &SwSnapshot) {
    println!("{}", "[OVERVIEW]".cyan());
    println!("  Packages:  {} ({} explicit, {} deps, {} AUR)",
        snapshot.packages.total,
        snapshot.packages.explicit,
        snapshot.packages.dependency,
        snapshot.packages.aur);
    println!("  Commands:  {}", snapshot.commands.total);
    println!("  Services:  {} ({} running, {} failed)",
        snapshot.services.total,
        snapshot.services.running,
        snapshot.services.failed);
    println!();
}

fn print_categories(snapshot: &SwSnapshot, is_full: bool) {
    if snapshot.categories.is_empty() {
        return;
    }

    println!("{}", "[CATEGORIES]".cyan());
    println!("  {}", "(from package descriptions)".dimmed());

    // v7.42.4: Show all categories and items - no truncation
    for cat in &snapshot.categories {
        // Skip "Other" in compact mode (usually noise)
        if !is_full && cat.name == "Other" {
            continue;
        }

        let display = cat.packages.join(", ");
        println!("  {:<14} {}", format!("{}:", cat.name), display);
    }

    println!();
}

fn print_platforms(snapshot: &SwSnapshot, _is_full: bool) {
    println!("{}", "[PLATFORMS]".cyan());

    let total_gb = snapshot.platforms.steam_total_size_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    println!("  Steam:  {} games ({:.1} GiB)",
        snapshot.platforms.steam_game_count,
        total_gb);

    // v7.42.4: Show all top games - no truncation
    for game in &snapshot.platforms.steam_top_games {
        let size_gb = game.size_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        println!("    {} ({:.1} GiB)", game.name, size_gb);
    }

    println!();
}

fn print_config_coverage(snapshot: &SwSnapshot, _is_full: bool) {
    let cov = &snapshot.config_coverage;
    let pct = (cov.apps_with_config as f64 / cov.total_apps as f64 * 100.0) as u32;

    println!("{}", "[CONFIG COVERAGE]".cyan());
    println!("  Coverage: {}/{} known apps ({}%)", cov.apps_with_config, cov.total_apps, pct);

    // v7.42.4: Always show detected apps - no truncation
    if !cov.app_names.is_empty() {
        println!("  Detected: {}", cov.app_names.join(", "));
    }

    println!();
}

fn print_topology(snapshot: &SwSnapshot, _is_full: bool) {
    println!("{}", "[TOPOLOGY]".cyan());

    // v7.42.4: Show all components - no truncation
    for role in &snapshot.topology.roles {
        let components = role.components.join(", ");
        println!("  {:<14} {}", role.name.cyan(), components);
    }

    // v7.42.4: Always show service groups if present
    if !snapshot.topology.service_groups.is_empty() {
        println!("  {}", "Service Groups:".dimmed());
        for group in &snapshot.topology.service_groups {
            let services = group.services.join(", ");
            println!("    {:<12} {}", group.name, services);
        }
    }

    println!();
}

fn print_services(snapshot: &SwSnapshot, is_full: bool) {
    // Only show services section if there are failures or in full mode
    if snapshot.services.failed == 0 && !is_full {
        return;
    }

    println!("{}", "[SERVICES]".cyan());

    if snapshot.services.failed > 0 {
        println!("  {} {} failed service{}:",
            "⚠".yellow(),
            snapshot.services.failed,
            if snapshot.services.failed == 1 { "" } else { "s" });
        for svc in &snapshot.services.failed_services {
            println!("    {}", svc.red());
        }
    } else if is_full {
        println!("  {} All services healthy", "✓".green());
    }

    println!();
}

fn display_section(snapshot: &SwSnapshot, section: &str) -> Result<()> {
    match section.to_lowercase().as_str() {
        "overview" => { print_overview(snapshot); }
        "categories" => { print_categories(snapshot, true); }
        "platforms" => { print_platforms(snapshot, true); }
        "config" | "coverage" => { print_config_coverage(snapshot, true); }
        "topology" => { print_topology(snapshot, true); }
        "services" => { print_services(snapshot, true); }
        _ => {
            eprintln!("Unknown section: {}", section);
            eprintln!("Available: overview, categories, platforms, config, topology, services");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn show_no_snapshot_message() -> Result<()> {
    println!();
    println!("{}", "  Anna Software".bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  {} No software snapshot available.", "⚠".yellow());
    println!();
    println!("  The daemon (annad) builds and maintains software snapshots.");
    println!("  To get data:");
    println!();
    println!("    1. Start the daemon:  sudo systemctl start annad");
    println!("    2. Wait a moment for initial scan to complete");
    println!("    3. Run this command again");
    println!();
    println!("  Check daemon status:    annactl status");
    println!("  View daemon logs:       journalctl -u annad -f");
    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}
