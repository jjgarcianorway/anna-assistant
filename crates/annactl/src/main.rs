//! Anna CLI (annactl) v5.2.0 - Knowledge UX with Full Error Visibility
//!
//! Anna is now a paranoid archivist with full error tracking:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions
//! - Tracks ALL systemd services with full state
//! - Detects package installs/removals
//! - v5.1.1: Priority scans for user-requested objects
//! - v5.2.0: Error indexing from journalctl
//! - v5.2.0: Service state tracking (active/enabled/masked/failed)
//! - v5.2.0: Intrusion detection patterns
//!
//! ## Allowed CLI Commands
//!
//! - annactl status     Quick system and knowledge status + inventory progress
//! - annactl stats      Detailed knowledge statistics + command coverage
//! - annactl knowledge  Global knowledge view with errors/intrusions
//! - annactl knowledge <topic>  Focused view with errors, logs, intrusions
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
    // v5.2.0: Error, Service State, Intrusion
    ErrorIndex, LogSeverity,
    ServiceIndex, ServiceState, ActiveState, EnabledState,
    IntrusionIndex, IntrusionType,
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
    println!("  Priority Knowledge Resolution");
    println!();
    Ok(())
}

// ============================================================================
// Help Command
// ============================================================================

fn run_help() -> Result<()> {
    println!();
    println!("{}", "ANNA - Knowledge UX with Full Error Visibility v5.2.0".bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  Anna is a paranoid archivist that tracks every executable,");
    println!("  package, and service on your Linux system.");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  annactl status           System status + inventory progress");
    println!("  annactl stats            Knowledge stats + command coverage");
    println!("  annactl knowledge        Global knowledge view (all objects)");
    println!("  annactl knowledge <name> Focused view with errors/logs/state");
    println!("  annactl version          Show version info");
    println!("  annactl help             Show this help");
    println!();
    println!("{}", "TRACKING:".bold());
    println!("  - ALL commands on PATH");
    println!("  - ALL packages with versions");
    println!("  - ALL systemd services (active/enabled/masked/failed)");
    println!("  - Package install/remove events");
    println!("  - Errors and warnings from journalctl");
    println!("  - Intrusion detection patterns");
    println!();
    println!("{}", "v5.2.0 ERROR VISIBILITY:".bold());
    println!("  Every object shows its errors, warnings, failures,");
    println!("  and intrusion attempts. No filtering. No guessing.");
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

    // v5.1.0 + v5.1.1: Inventory section
    println!("{}", "[INVENTORY]".cyan());
    let progress = builder.progress();
    if progress.is_priority_scan() {
        // v5.1.1: Priority scan in progress
        let target = progress.priority_target.as_deref().unwrap_or("unknown");
        println!("  State:      {} ({})", "priority_scan".yellow(), target);
        println!("  Progress:   scanning object");
        println!("  ETA:        <1s");
        println!("  Objects:    {} tracked", store.total_objects());
    } else if progress.initial_scan_complete {
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
    println!();

    // v5.2.0: Error/Intrusion summary
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();
    let service_index = ServiceIndex::load();

    println!("{}", "[HEALTH]".cyan());
    // Errors
    if error_index.total_errors > 0 {
        println!("  Errors:     {} indexed", format_number(error_index.total_errors).red());
    } else {
        println!("  Errors:     {} indexed", "0".green());
    }
    // Warnings
    if error_index.total_warnings > 0 {
        println!("  Warnings:   {} indexed", format_number(error_index.total_warnings).yellow());
    } else {
        println!("  Warnings:   0 indexed");
    }
    // Intrusions
    if intrusion_index.total_events > 0 {
        println!("  Intrusions: {} detected", format_number(intrusion_index.total_events).red().bold());
    } else {
        println!("  Intrusions: {} detected", "0".green());
    }
    // Service failures
    if service_index.failed_count > 0 {
        println!("  Failed svc: {}", format!("{}", service_index.failed_count).red());
    } else {
        println!("  Failed svc: {}", "0".green());
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
// Knowledge Command (v5.0.0 + v5.1.1)
// ============================================================================

async fn run_knowledge(topic: Option<&str>) -> Result<()> {
    match topic {
        None => {
            let store = KnowledgeStore::load();
            run_knowledge_list(&store)
        }
        Some(name) => {
            // v5.1.1: Use builder for priority scan support
            let mut builder = KnowledgeBuilder::new();
            run_knowledge_detail_with_builder(&mut builder, name)
        }
    }
}

fn run_knowledge_list(store: &KnowledgeStore) -> Result<()> {
    // v5.2.0: Load error and intrusion indexes for global view
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();
    let service_index = ServiceIndex::load();

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

    // v5.2.0: Summary section
    let (commands, packages, services) = store.count_by_type();
    println!("{}", "[SUMMARY]".cyan());
    println!("  Objects:    {} total", store.total_objects());
    println!("  Commands:   {}", commands);
    println!("  Packages:   {}", packages);
    println!("  Services:   {}", services);
    println!("  Configs:    {}", store.objects.values().filter(|o| !o.config_paths.is_empty()).count());
    println!("  Errors:     {}", error_index.total_errors);
    println!("  Intrusions: {}", intrusion_index.total_events);
    println!();

    // v5.2.0: Failed services section
    if service_index.failed_count > 0 {
        println!("{}", "[FAILED SERVICES]".red().bold());
        for svc in service_index.get_failed() {
            let reason = svc.failure_reason.as_deref().unwrap_or("unknown");
            println!("  {} - {}", svc.unit_name.red(), reason);
        }
        println!();
    }

    // v5.2.0: Recent intrusions section
    let recent_intrusions = intrusion_index.recent_high_severity(3600, 3); // Last hour, severity 3+
    if !recent_intrusions.is_empty() {
        println!("{}", "[RECENT INTRUSIONS (1h)]".red().bold());
        for event in recent_intrusions.iter().take(5) {
            let ts = format_timestamp(event.timestamp);
            let ip = event.source_ip.as_deref().unwrap_or("-");
            println!("  {} [{}] {} (from {})",
                ts, event.intrusion_type.as_str().red(),
                truncate_str(&event.message, 40), ip);
        }
        println!();
    }

    // v5.2.0: Top errors section
    let top_errors = error_index.top_by_errors(5);
    if !top_errors.is_empty() {
        println!("{}", "[TOP ERRORS BY OBJECT]".yellow());
        for (name, count) in &top_errors {
            println!("  {:<20} {} errors", name, count);
        }
        println!();
    }

    // Group by category (existing)
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
                // v5.2.0: Show error count per object
                let err_count = error_index.get_object_errors(&obj.name)
                    .map(|e| e.total_errors())
                    .unwrap_or(0);
                let err_str = if err_count > 0 {
                    format!("{}", err_count).red().to_string()
                } else {
                    "-".to_string()
                };
                println!("  {:<12} inst:{:<3} runs:{:<5} errs:{}",
                    obj.name, installed, obj.usage_count, err_str);
            }
            println!();
        }
    }

    println!("{}", THIN_SEP);
    println!("Use 'annactl knowledge <name>' for full object details.");
    println!();

    Ok(())
}

/// Truncate string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn run_knowledge_detail_with_builder(builder: &mut KnowledgeBuilder, name: &str) -> Result<()> {
    println!();
    println!("{}", format!("ANNA KNOWLEDGE: {}", name).bold());
    println!("{}", THIN_SEP);

    // v5.1.1: Priority scan - ensure object is fresh before answering
    if !builder.is_object_complete(name) && !builder.is_object_complete(&name.to_lowercase()) {
        println!("{}", "[PRIORITY SCAN]".yellow());
        println!("  Scanning for '{}'...", name);
        let found = builder.targeted_discovery(name);
        if found {
            println!("  Found and indexed.");
        } else {
            println!("  Not found on system.");
        }
        println!();
        // Save after discovery
        let _ = builder.save();
    }

    // Try to find by exact name or lowercase
    let store = builder.store();
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

            // v5.2.0: Enhanced Service section with full state
            if obj.service_unit.is_some() || obj.object_types.contains(&ObjectType::Service) {
                let service_index = ServiceIndex::load();
                println!("{}", "[SERVICE STATE]".cyan());

                if let Some(unit) = &obj.service_unit {
                    println!("  Unit:            {}", unit);

                    // v5.2.0: Get full service state from service index
                    if let Some(state) = service_index.services.get(unit) {
                        // Active state with color
                        let active_str = match state.active_state {
                            ActiveState::Active => state.active_state.as_str().green().to_string(),
                            ActiveState::Failed => state.active_state.as_str().red().bold().to_string(),
                            ActiveState::Inactive => state.active_state.as_str().dimmed().to_string(),
                            _ => state.active_state.as_str().yellow().to_string(),
                        };
                        println!("  Active:          {} ({})", active_str, state.sub_state.as_str());

                        // Enabled state with color
                        let enabled_str = match state.enabled_state {
                            EnabledState::Enabled => state.enabled_state.as_str().green().to_string(),
                            EnabledState::Masked => state.enabled_state.as_str().red().to_string(),
                            EnabledState::Disabled => state.enabled_state.as_str().yellow().to_string(),
                            _ => state.enabled_state.as_str().to_string(),
                        };
                        println!("  Enabled:         {}", enabled_str);

                        if let Some(pid) = state.main_pid {
                            println!("  PID:             {}", pid);
                        }
                        println!("  Memory:          {}", state.format_memory());
                        println!("  CPU:             {}", state.format_cpu());
                        println!("  Restarts:        {}", state.restart_count);

                        if let Some(reason) = &state.failure_reason {
                            println!("  Failure:         {}", reason.red());
                        }
                    } else {
                        // Fallback to basic info from KnowledgeObject
                        if let Some(enabled) = obj.service_enabled {
                            println!("  Enabled:         {}", if enabled { "yes" } else { "no" });
                        }
                        if let Some(active) = obj.service_active {
                            println!("  Active:          {}", if active { "yes" } else { "no" });
                        }
                    }
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

            // v5.2.0: Errors section
            let error_index = ErrorIndex::load();
            if let Some(obj_errors) = error_index.get_object_errors(&obj.name) {
                if obj_errors.total_errors() > 0 || obj_errors.warning_count > 0 {
                    println!("{}", "[ERRORS]".red());
                    println!("  Total errors:    {}", obj_errors.total_errors());
                    println!("  Total warnings:  {}", obj_errors.warning_count);

                    // Show error type breakdown
                    if !obj_errors.error_counts.is_empty() {
                        println!("  By type:");
                        for (err_type, count) in &obj_errors.error_counts {
                            println!("    {}: {}", err_type.as_str(), count);
                        }
                    }

                    // Show recent errors (last 5)
                    let errors = obj_errors.errors_only();
                    if !errors.is_empty() {
                        println!("  Recent errors:");
                        for entry in errors.iter().rev().take(5) {
                            let ts = format_timestamp(entry.timestamp);
                            println!("    [{}] {}", ts, truncate_str(&entry.message, 50));
                        }
                    }
                    println!();
                }
            }

            // v5.2.0: Logs section (recent warnings and errors)
            if let Some(obj_errors) = error_index.get_object_errors(&obj.name) {
                if !obj_errors.logs.is_empty() {
                    let recent_logs: Vec<_> = obj_errors.logs.iter()
                        .filter(|l| l.severity >= LogSeverity::Warning)
                        .rev()
                        .take(10)
                        .collect();

                    if !recent_logs.is_empty() {
                        println!("{}", "[LOGS]".yellow());
                        for entry in &recent_logs {
                            let ts = format_timestamp(entry.timestamp);
                            let sev = match entry.severity {
                                LogSeverity::Warning => entry.severity.as_str().yellow().to_string(),
                                LogSeverity::Error => entry.severity.as_str().red().to_string(),
                                LogSeverity::Critical | LogSeverity::Alert | LogSeverity::Emergency => {
                                    entry.severity.as_str().red().bold().to_string()
                                }
                                _ => entry.severity.as_str().to_string(),
                            };
                            println!("  [{}] {} {}", ts, sev, truncate_str(&entry.message, 45));
                        }
                        println!();
                    }
                }
            }

            // v5.2.0: Intrusion section
            let intrusion_index = IntrusionIndex::load();
            if let Some(obj_intrusions) = intrusion_index.get_object_intrusions(&obj.name) {
                if !obj_intrusions.events.is_empty() {
                    println!("{}", "[INTRUSION]".red().bold());
                    println!("  Total events:    {}", obj_intrusions.total_events());
                    println!("  Max severity:    {}", obj_intrusions.max_severity);

                    // Type breakdown
                    if !obj_intrusions.type_counts.is_empty() {
                        println!("  By type:");
                        for (int_type, count) in &obj_intrusions.type_counts {
                            println!("    {}: {}", int_type, count);
                        }
                    }

                    // Recent events (last 5)
                    println!("  Recent events:");
                    for event in obj_intrusions.events.iter().rev().take(5) {
                        let ts = format_timestamp(event.timestamp);
                        let ip = event.source_ip.as_deref().unwrap_or("-");
                        println!("    [{}] {} (from {})", ts, event.intrusion_type.as_str().red(), ip);
                        println!("      {}", truncate_str(&event.message, 55));
                    }
                    println!();
                }
            }

            // Notes
            println!("{}", "[NOTES]".cyan());
            println!("  Data collected by anna daemon (v5.2.0 Full Error Visibility).");
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
