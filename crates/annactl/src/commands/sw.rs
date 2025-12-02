//! SW Command v7.40.0 - Cache-First Software Overview
//!
//! v7.40.0: Cache-first architecture for sub-second response
//! - Uses sw_cache for pre-computed data
//! - Delta detection via pacman.log and PATH mtimes
//! - Falls back to live query only when cache unavailable
//! - Supports --full for detailed view, --json for machine output
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Rule-based categories from descriptions (sorted)
//! - [PLATFORMS]         Steam games and other game platforms
//! - [CONFIG COVERAGE]   Config detection summary
//! - [TOPOLOGY]          Software stack roles and service groups
//! - [IMPACT]            Top resource consumers from telemetry
//! - [HOTSPOTS]          CPU, memory, most started processes

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::sw_cache::{get_sw_cache, SwCache};
use anna_common::grounded::steam::{is_steam_installed, detect_steam_games, format_game_size};
use anna_common::config::AnnaConfig;
use anna_common::impact_view::get_software_impact;
use anna_common::hotspots::{get_software_hotspots, format_software_hotspots_section};

const THIN_SEP: &str = "------------------------------------------------------------";
const MAX_CATEGORY_ITEMS: usize = 10;

/// Output mode for sw command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwOutputMode {
    /// Default compact view
    Compact,
    /// Full detailed view (--full)
    Full,
    /// JSON output (--json)
    Json,
}

/// Run the sw overview command
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
    // Get cached data (fast path) or build cache (slow path)
    let cache = get_sw_cache();

    if mode == SwOutputMode::Json {
        // JSON output
        let json = serde_json::to_string_pretty(&cache)
            .unwrap_or_else(|_| "{}".to_string());
        println!("{}", json);
        return Ok(());
    }

    println!();
    println!("{}", "  Anna Software".bold());
    println!("{}", THIN_SEP);

    // Show cache status in compact mode
    if mode == SwOutputMode::Compact {
        if let Some(age) = cache.updated_at {
            let age_str = cache.format_age();
            let build_ms = cache.build_duration_ms;
            println!("  {} cached {} (built in {}ms)", "◆".dimmed(), age_str.dimmed(), build_ms);
        }
    }
    println!();

    // [OVERVIEW]
    print_overview_section(&cache);

    // [CATEGORIES]
    print_categories_section(&cache, mode);

    // [PLATFORMS] - only if Steam installed
    print_platforms_section();

    // [CONFIG COVERAGE]
    print_config_coverage_section(&cache, mode);

    // [TOPOLOGY]
    print_topology_section(&cache, mode);

    // Full mode: show impact and hotspots
    if mode == SwOutputMode::Full {
        // [IMPACT]
        print_impact_section();

        // [HOTSPOTS]
        print_hotspots_section();
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview_section(cache: &SwCache) {
    println!("{}", "[OVERVIEW]".cyan());

    println!("  Packages known:   {}", cache.package_counts.total);
    println!("  Commands known:   {}", cache.command_count);
    println!("  Services known:   {}", cache.service_counts.total);

    println!();
}

fn print_categories_section(cache: &SwCache, mode: SwOutputMode) {
    println!("{}", "[CATEGORIES]".cyan());
    println!("  {}", "(from pacman descriptions)".dimmed());

    if cache.categories.is_empty() {
        println!("  {}", "(no categories detected)".dimmed());
    } else {
        let max_items = if mode == SwOutputMode::Full { usize::MAX } else { MAX_CATEGORY_ITEMS };

        for cat in &cache.categories {
            // Skip "Other" in overview
            if cat.name == "Other" {
                continue;
            }

            let display: String = if cat.packages.len() <= max_items {
                cat.packages.join(", ")
            } else {
                format!("{} (and {} more)",
                    cat.packages.iter().take(max_items).cloned().collect::<Vec<_>>().join(", "),
                    cat.packages.len() - max_items)
            };

            let cat_display = format!("{}:", cat.name);
            println!("  {:<14} {}", cat_display, display);
        }
    }

    println!();
}

/// Print [PLATFORMS] section
/// Shows game platforms like Steam with installed games
fn print_platforms_section() {
    // Only show if Steam is installed
    if !is_steam_installed() {
        return;
    }

    let games = detect_steam_games();
    if games.is_empty() {
        return;
    }

    println!("{}", "[PLATFORMS]".cyan());
    println!("  {}", "(game platforms with local manifests)".dimmed());

    // Steam section
    let total_size: u64 = games.iter().filter_map(|g| g.size_on_disk).sum();
    println!("  Steam:        {} games ({})", games.len(), format_game_size(total_size));

    // Show top games by size (up to 5)
    let mut sorted_games = games.clone();
    sorted_games.sort_by(|a, b| b.size_on_disk.cmp(&a.size_on_disk));

    let game_names: Vec<String> = sorted_games.iter()
        .take(5)
        .map(|g| {
            let size = g.size_on_disk.map(|s| format!(" ({})", format_game_size(s))).unwrap_or_default();
            format!("{}{}", g.name, size)
        })
        .collect();

    if !game_names.is_empty() {
        println!("    Largest:    {}", game_names[0]);
        for name in game_names.iter().skip(1).take(4) {
            println!("                {}", name);
        }
        if games.len() > 5 {
            println!("                (+{} more)", games.len() - 5);
        }
    }

    println!();
}

fn print_config_coverage_section(cache: &SwCache, mode: SwOutputMode) {
    let coverage = &cache.config_coverage;

    // Only show if we have any coverage
    if coverage.apps_with_config == 0 {
        return;
    }

    println!("{}", "[CONFIG COVERAGE]".cyan());
    println!("  {}", "(apps with detected config files)".dimmed());

    // Show coverage ratio
    let pct = (coverage.apps_with_config as f64 / coverage.total_apps as f64 * 100.0) as u32;
    println!("  Coverage: {}/{} known apps ({}%)", coverage.apps_with_config, coverage.total_apps, pct);

    // List apps with config
    if !coverage.app_names.is_empty() {
        let max_items = if mode == SwOutputMode::Full { usize::MAX } else { 10 };
        let display: String = if coverage.app_names.len() <= max_items {
            coverage.app_names.join(", ")
        } else {
            format!("{} (+{} more)",
                coverage.app_names.iter().take(max_items).cloned().collect::<Vec<_>>().join(", "),
                coverage.app_names.len() - max_items)
        };
        println!("  Detected: {}", display);
    }

    println!();
}

fn print_topology_section(cache: &SwCache, mode: SwOutputMode) {
    let topology = &cache.topology;

    // Stack roles
    if !topology.roles.is_empty() {
        println!("{}", "[TOPOLOGY]".cyan());
        println!("  {}", "(from package descriptions and deps)".dimmed());

        println!("  Stacks:");
        let max_components = if mode == SwOutputMode::Full { usize::MAX } else { 5 };

        for role in &topology.roles {
            let components: String = role.components.iter()
                .take(max_components)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = if role.components.len() > max_components {
                format!(", +{}", role.components.len() - max_components)
            } else {
                String::new()
            };
            println!("    {:<12} {}{}", role.name.cyan(), components, suffix);
        }

        // Service groups
        if !topology.service_groups.is_empty() {
            println!("  Service Groups:");
            let max_services = if mode == SwOutputMode::Full { usize::MAX } else { 4 };

            for group in &topology.service_groups {
                let services: String = group.services.iter()
                    .take(max_services)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if group.services.len() > max_services {
                    format!(", +{}", group.services.len() - max_services)
                } else {
                    String::new()
                };
                println!("    {:<12} {}{}", group.name.cyan(), services, suffix);
            }
        }

        println!();
    }
}

/// Print [IMPACT] section (full mode only)
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

/// Print [HOTSPOTS] section (full mode only)
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
