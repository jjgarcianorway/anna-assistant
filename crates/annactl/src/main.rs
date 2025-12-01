//! Anna CLI (annactl) v5.1.0 - Full Inventory
//!
//! Anna is now a paranoid archivist:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions
//! - Tracks ALL systemd services
//! - Detects package installs/removals
//!
//! ## Allowed CLI Commands
//!
//! - annactl status     Quick system and knowledge status + inventory progress
//! - annactl stats      Detailed knowledge statistics + command coverage
//! - annactl knowledge  List all known objects
//! - annactl knowledge <topic>  Details on one object
//! - annactl version    Show version info
//! - annactl help       Show help info
//!
//! Everything else returns a clear message that Q&A is disabled.

#![allow(dead_code)]
#![allow(unused_imports)]

mod client;

use anna_common::{
    init_logger, AnnaConfigV5, logging, LogComponent,
    KnowledgeCategory, KnowledgeObject, KnowledgeStore, TelemetryAggregates,
    KnowledgeBuilder, ObjectType, InventoryPhase,
    count_path_binaries, count_systemd_services,
};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";
const THICK_SEP: &str = "============================================================";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let config = AnnaConfigV5::load();
    init_logger(config.log.clone());
    logging::logger().info(LogComponent::Request, "annactl v5 starting");

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => run_help(),
        [cmd] if cmd.eq_ignore_ascii_case("status") => run_status().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => run_stats().await,
        [cmd] if cmd.eq_ignore_ascii_case("knowledge") => run_knowledge(None).await,
        [cmd, topic] if cmd.eq_ignore_ascii_case("knowledge") => run_knowledge(Some(topic)).await,
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            run_version()
        }
        [flag] if flag.eq_ignore_ascii_case("-h") || flag.eq_ignore_ascii_case("--help") || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        _ => run_disabled_message(),
    }
}

// ============================================================================
// Version Command
// ============================================================================

fn run_version() -> Result<()> {
    println!();
    println!("  annactl v{}", VERSION);
    println!("  Full Inventory - Paranoid Archivist");
    println!();
    Ok(())
}

// ============================================================================
// Help Command
// ============================================================================

fn run_help() -> Result<()> {
    println!();
    println!("{}", "ANNA - Full Inventory v5.1.0".bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  Anna is a paranoid archivist that tracks every executable,");
    println!("  package, and service on your Linux system.");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  annactl status           System status + inventory progress");
    println!("  annactl stats            Knowledge stats + command coverage");
    println!("  annactl knowledge        List all known software objects");
    println!("  annactl knowledge <name> Details on one specific object");
    println!("  annactl version          Show version info");
    println!("  annactl help             Show this help");
    println!();
    println!("{}", "TRACKING:".bold());
    println!("  - ALL commands on PATH");
    println!("  - ALL packages with versions");
    println!("  - ALL systemd services");
    println!("  - Package install/remove events");
    println!();
    Ok(())
}

// ============================================================================
// Disabled Q&A Message
// ============================================================================

fn run_disabled_message() -> Result<()> {
    println!();
    println!("{}", "[NOTICE]".yellow());
    println!();
    println!("  Q&A is disabled in Knowledge Core phase.");
    println!();
    println!("  Available commands:");
    println!("    annactl status      - Quick system status");
    println!("    annactl stats       - Detailed statistics");
    println!("    annactl knowledge   - List known software");
    println!("    annactl help        - Show all commands");
    println!();
    Ok(())
}

// ============================================================================
// Status Command (v5.0.0)
// ============================================================================

async fn run_status() -> Result<()> {
    // Load knowledge and telemetry
    let store = KnowledgeStore::load();
    let telemetry = TelemetryAggregates::load();
    let builder = KnowledgeBuilder::new();

    // Check daemon status
    let daemon_status = check_daemon_status().await;

    // Count by category
    let counts = store.count_by_category();
    let (commands, packages, services) = store.count_by_type();

    println!();
    println!("{}", "ANNA STATUS".bold());
    println!("{}", THIN_SEP);

    // Version section
    println!("{}", "[VERSION]".cyan());
    println!("  annactl:    v{}", VERSION);
    println!("  annad:      v{}", VERSION);
    println!();

    // Services section
    println!("{}", "[SERVICES]".cyan());
    if daemon_status.running {
        println!("  Daemon:     {} (up {})", "running".green(), telemetry.uptime_string());
    } else {
        println!("  Daemon:     {}", "not running".red());
    }
    println!();

    // v5.1.0: Inventory section
    println!("{}", "[INVENTORY]".cyan());
    let progress = builder.progress();
    if progress.initial_scan_complete {
        println!("  Status:     {}", "Complete".green());
        println!("  Commands:   {} tracked", commands);
        println!("  Packages:   {} tracked", packages);
        println!("  Services:   {} tracked", services);
    } else {
        // Show current phase and progress
        let status = progress.format_status();
        if progress.phase == InventoryPhase::Idle {
            // Count system totals for context
            let total_bins = count_path_binaries();
            let total_svcs = count_systemd_services();
            println!("  Status:     {} (pending scan)", "Waiting".yellow());
            println!("  PATH cmds:  ~{} (system)", total_bins);
            println!("  Services:   ~{} (system)", total_svcs);
        } else {
            println!("  Status:     {}", status.yellow());
            println!("  Progress:   {}%", progress.percent);
            if let Some(eta) = progress.eta_secs {
                println!("  ETA:        {}s", eta);
            }
        }
    }
    println!();

    // Knowledge section (categorized tools)
    println!("{}", "[KNOWLEDGE]".cyan());
    let total = store.total_objects();
    if total == 0 {
        println!("  No data collected yet. Daemon must be running.");
    } else {
        // Show category breakdown
        let editors = counts.get(&KnowledgeCategory::Editor).unwrap_or(&0);
        let terminals = counts.get(&KnowledgeCategory::Terminal).unwrap_or(&0);
        let shells = counts.get(&KnowledgeCategory::Shell).unwrap_or(&0);
        let wms = counts.get(&KnowledgeCategory::Wm).unwrap_or(&0);
        let compositors = counts.get(&KnowledgeCategory::Compositor).unwrap_or(&0);
        let browsers = counts.get(&KnowledgeCategory::Browser).unwrap_or(&0);

        // Only show categorized (known) tools
        let known = editors + terminals + shells + wms + compositors + browsers;
        println!("  Known:      {} (categorized)", known);
        if *editors > 0 { println!("    Editors:    {}", editors); }
        if *terminals > 0 { println!("    Terminals:  {}", terminals); }
        if *shells > 0 { println!("    Shells:     {}", shells); }
        if *wms > 0 || *compositors > 0 {
            println!("    WMs/Comp:   {}", wms + compositors);
        }
        if *browsers > 0 { println!("    Browsers:   {}", browsers); }

        let with_usage = store.count_with_usage();
        println!("  Observed:   {} (with runs)", with_usage);
    }
    println!();

    // Telemetry section
    println!("{}", "[TELEMETRY]".cyan());
    if telemetry.processes_observed == 0 {
        println!("  No telemetry collected yet.");
    } else {
        println!("  Processes:  {} observed", format_number(telemetry.processes_observed));
        println!("  Commands:   {} unique", telemetry.unique_commands);
        println!("  Samples:    {}", format_number(telemetry.total_samples));
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

// ============================================================================
// Stats Command (v5.0.0)
// ============================================================================

async fn run_stats() -> Result<()> {
    // Load knowledge and telemetry
    let store = KnowledgeStore::load();
    let telemetry = TelemetryAggregates::load();

    println!();
    println!("{}", "ANNA STATS".bold());
    println!("{}", THIN_SEP);

    // v5.1.0: Command Coverage section
    println!("{}", "[COMMAND COVERAGE]".cyan());
    let (commands, packages, services) = store.count_by_type();
    let total_path = count_path_binaries();
    let total_svcs = count_systemd_services();

    if commands > 0 || packages > 0 || services > 0 {
        // Commands coverage
        let cmd_pct = if total_path > 0 { (commands as f64 / total_path as f64) * 100.0 } else { 0.0 };
        println!("  Commands:   {}/{} ({:.0}% of PATH)", commands, total_path, cmd_pct);

        // Packages coverage (always 100% if pacman available)
        println!("  Packages:   {} tracked", packages);

        // Services coverage
        let svc_pct = if total_svcs > 0 { (services as f64 / total_svcs as f64) * 100.0 } else { 0.0 };
        println!("  Services:   {}/{} ({:.0}% of systemd)", services, total_svcs, svc_pct);

        // Observed (with usage)
        let with_usage = store.count_with_usage();
        let usage_pct = if store.total_objects() > 0 {
            (with_usage as f64 / store.total_objects() as f64) * 100.0
        } else { 0.0 };
        println!("  Observed:   {}/{} ({:.0}% with runs)", with_usage, store.total_objects(), usage_pct);
    } else {
        println!("  No inventory data yet. Daemon must be running.");
    }
    println!();

    // Knowledge Coverage section (categorized)
    println!("{}", "[KNOWLEDGE COVERAGE]".cyan());

    let total = store.total_objects();

    if total == 0 {
        println!("  No data collected yet.");
    } else {
        // List by category with names
        for category in &[
            KnowledgeCategory::Editor,
            KnowledgeCategory::Terminal,
            KnowledgeCategory::Shell,
            KnowledgeCategory::Wm,
            KnowledgeCategory::Compositor,
            KnowledgeCategory::Browser,
        ] {
            let objs = store.get_by_category(category);
            if !objs.is_empty() {
                let names: Vec<_> = objs.iter().map(|o| o.name.as_str()).collect();
                println!("  {:12}{} ({})",
                    format!("{}:", category.display_name()),
                    objs.len(),
                    names.join(", ")
                );
            }
        }
    }
    println!();

    // Discovery Latency section
    println!("{}", "[DISCOVERY LATENCY]".cyan());
    if let (Some(first), Some(last)) = (store.first_discovery_at, store.last_discovery_at) {
        let daemon_start = telemetry.daemon_start_at;
        if first >= daemon_start {
            let first_delay = first - daemon_start;
            let last_delay = last - daemon_start;
            println!("  First object discovered:   {}s after daemon start", first_delay);
            println!("  Last new object:           {} after daemon start", format_duration_secs(last_delay));
        } else {
            println!("  First discovery:  historical data");
            println!("  Last discovery:   historical data");
        }
    } else {
        println!("  No discovery events recorded yet.");
    }
    println!();

    // Usage Hotspots section
    println!("{}", "[USAGE HOTSPOTS]".cyan());
    if let Some((cmd, count)) = telemetry.most_used_command() {
        println!("  Most-used command:         {} ({} runs)", cmd, format_number(*count));
    }

    let top_cpu = store.top_by_cpu(1);
    if let Some(obj) = top_cpu.first() {
        if obj.total_cpu_time_ms > 0 {
            println!("  Heaviest CPU process:      {}", obj.name);
        }
    }

    let top_mem = store.top_by_memory(1);
    if let Some(obj) = top_mem.first() {
        if obj.total_mem_bytes_peak > 0 {
            println!("  Heaviest RAM process:      {}", obj.name);
        }
    }
    println!();

    // Top Editors section (example detailed breakdown)
    let editors = store.get_by_category(&KnowledgeCategory::Editor);
    if !editors.is_empty() {
        println!("{}", "[TOP EDITORS]".cyan());
        let mut sorted_editors: Vec<_> = editors.iter().collect();
        sorted_editors.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

        for obj in sorted_editors.iter().take(3) {
            println!("  {}:", obj.name);
            println!("    runs:         {}", obj.usage_count);
            println!("    cpu_time:     {}ms", format_number(obj.total_cpu_time_ms));
            println!("    max_rss:      {}", format_bytes(obj.total_mem_bytes_peak));
        }
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

// ============================================================================
// Knowledge Command (v5.0.0)
// ============================================================================

async fn run_knowledge(topic: Option<&str>) -> Result<()> {
    let store = KnowledgeStore::load();

    match topic {
        None => run_knowledge_list(&store),
        Some(name) => run_knowledge_detail(&store, name),
    }
}

fn run_knowledge_list(store: &KnowledgeStore) -> Result<()> {
    println!();
    println!("{}", "ANNA KNOWLEDGE".bold());
    println!("{}", THIN_SEP);

    if store.total_objects() == 0 {
        println!();
        println!("  No knowledge collected yet.");
        println!("  The daemon must be running to collect data.");
        println!();
        println!("{}", THIN_SEP);
        return Ok(());
    }

    // Group by category
    for category in &[
        KnowledgeCategory::Editor,
        KnowledgeCategory::Terminal,
        KnowledgeCategory::Shell,
        KnowledgeCategory::Wm,
        KnowledgeCategory::Compositor,
        KnowledgeCategory::Browser,
        KnowledgeCategory::Service,
        KnowledgeCategory::Tool,
    ] {
        let objs = store.get_by_category(category);
        if !objs.is_empty() {
            println!("{}", format!("[{}]", category.display_name().to_uppercase()).cyan());

            let mut sorted: Vec<_> = objs.iter().collect();
            sorted.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

            for obj in sorted {
                let installed = if obj.installed { "yes" } else { "no" };
                let wiki = obj.wiki_ref.as_deref().unwrap_or("-");
                println!("  {:<12} installed: {:<3}  runs: {:<6} wiki: {}",
                    obj.name, installed, obj.usage_count, wiki);
            }
            println!();
        }
    }

    println!("{}", THIN_SEP);
    println!("Use 'annactl knowledge <name>' for details on one item.");
    println!();

    Ok(())
}

fn run_knowledge_detail(store: &KnowledgeStore, name: &str) -> Result<()> {
    println!();
    println!("{}", format!("ANNA KNOWLEDGE: {}", name).bold());
    println!("{}", THIN_SEP);

    // Try to find by exact name or lowercase
    let obj = store.get(name)
        .or_else(|| store.get(&name.to_lowercase()));

    match obj {
        Some(obj) => {
            // Identity section
            println!("{}", "[IDENTITY]".cyan());
            println!("  Category:        {}", obj.category.as_str());
            println!("  Name:            {}", obj.name);

            // v5.1.0: Object types
            if !obj.object_types.is_empty() {
                let types: Vec<_> = obj.object_types.iter()
                    .map(|t| t.as_str())
                    .collect();
                println!("  Types:           {}", types.join(", "));
            }

            let detected = match &obj.detected_as {
                anna_common::DetectionSource::Package => "package".to_string(),
                anna_common::DetectionSource::Binary => "binary".to_string(),
                anna_common::DetectionSource::Both => format!(
                    "package '{}', binary",
                    obj.package_name.as_deref().unwrap_or(&obj.name)
                ),
                anna_common::DetectionSource::Unknown => "unknown".to_string(),
            };
            println!("  Detected as:     {}", detected);

            if let Some(wiki) = &obj.wiki_ref {
                println!("  Wiki:            {}", wiki);
            }

            // v5.1.0: Inventory source
            if !obj.inventory_source.is_empty() {
                println!("  Sources:         {}", obj.inventory_source.join(", "));
            }
            println!();

            // Installation section
            println!("{}", "[INSTALLATION]".cyan());
            println!("  Installed:       {}", if obj.installed { "yes" } else { "no" });
            if let Some(pkg) = &obj.package_name {
                println!("  Package:         {}", pkg);
            }
            // v5.1.0: Version
            if let Some(ver) = &obj.package_version {
                println!("  Version:         {}", ver);
            }
            // v5.1.0: Install date
            if let Some(ts) = obj.installed_at {
                println!("  Install date:    {}", format_timestamp(ts));
            }
            // v5.1.0: Removal date
            if let Some(ts) = obj.removed_at {
                println!("  Removed at:      {}", format_timestamp(ts));
            }
            if let Some(path) = &obj.binary_path {
                println!("  Binary path:     {}", path);
            }
            // v5.1.0: All paths
            if obj.paths.len() > 1 {
                println!("  All paths:");
                for p in &obj.paths {
                    println!("    - {}", p);
                }
            }
            println!();

            // v5.1.0: Service section
            if obj.service_unit.is_some() || obj.object_types.contains(&ObjectType::Service) {
                println!("{}", "[SERVICE]".cyan());
                if let Some(unit) = &obj.service_unit {
                    println!("  Unit:            {}", unit);
                }
                if let Some(enabled) = obj.service_enabled {
                    println!("  Enabled:         {}", if enabled { "yes" } else { "no" });
                }
                if let Some(active) = obj.service_active {
                    println!("  Active:          {}", if active { "yes" } else { "no" });
                }
                println!();
            }

            // Usage section
            println!("{}", "[USAGE]".cyan());
            println!("  Runs observed:   {}", obj.usage_count);
            println!("  First seen:      {}", format_timestamp(obj.first_seen_at));
            println!("  Last seen:       {}", format_timestamp(obj.last_seen_at));
            println!("  Total CPU time:  {}ms", format_number(obj.total_cpu_time_ms));
            println!("  Max RSS:         {}", format_bytes(obj.total_mem_bytes_peak));
            println!();

            // Config section
            if !obj.config_paths.is_empty() {
                println!("{}", "[CONFIG]".cyan());
                println!("  Config paths:");
                for path in &obj.config_paths {
                    println!("    - {}", path);
                }
                println!();
            }

            // Notes
            println!("{}", "[NOTES]".cyan());
            println!("  Data collected by anna daemon (v5.1.0 Full Inventory).");
        }
        None => {
            println!();
            println!("  Anna has no knowledge about '{}' yet.", name);
            println!();
            println!("  It might not be installed, or it has not been observed in use.");
        }
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

struct DaemonStatus {
    running: bool,
}

async fn check_daemon_status() -> DaemonStatus {
    // Try to connect to daemon API
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    let result = client
        .get("http://127.0.0.1:7865/v1/health")
        .send()
        .await;

    DaemonStatus {
        running: result.is_ok(),
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn format_timestamp(unix_secs: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(unix_secs as i64, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn format_duration_secs(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", secs)
    }
}

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
