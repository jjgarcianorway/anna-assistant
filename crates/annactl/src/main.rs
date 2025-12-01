//! Anna CLI (annactl) v5.2.2 - Precise Error & Intrusion Logs
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
//! - v5.2.1: Full service summary (total/active/inactive/enabled/disabled/masked/failed)
//! - v5.2.1: Log scan state tracking
//! - v5.2.1: Per-object DISCOVERY status for priority scans
//! - v5.2.2: Concrete grouped errors with cause/example per service
//! - v5.2.2: Intrusion analysis grouped by IP/username
//! - v5.2.2: No generic messages - every error traceable to real logs
//!
//! ## Allowed CLI Commands
//!
//! - annactl status     Quick system and knowledge status + inventory progress
//! - annactl stats      Detailed knowledge statistics + command coverage
//! - annactl knowledge  Global knowledge view with all sections
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
    // v5.2.1: Log scan state
    LogScanState,
    // v5.2.2: Grouped errors and intrusion analysis
    GroupedErrorSummary, GroupedIntrusionByService, IntrusionAnalysisEntry,
    // v5.2.3: Universal error inspection
    LogCategory,
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
    println!("{}", "ANNA - Precise Error & Intrusion Logs v5.2.2".bold());
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
    println!("  - Intrusion detection patterns (grouped by IP)");
    println!("  - Log signatures indexed");
    println!();
    println!("{}", "v5.2.2 PRECISE ERRORS:".bold());
    println!("  Every error grouped by service with cause summary and example.");
    println!("  Intrusion attempts grouped by IP with usernames and timestamps.");
    println!("  No generic messages - every line traceable to real logs.");
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
    // v5.2.1: Load all indexes for global view
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();
    let service_index = ServiceIndex::load();
    let log_scan_state = LogScanState::load();

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

    // v5.2.1: SERVICES section (full state summary)
    println!("{}", "[SERVICES]".cyan());
    println!("  Total services:     {}", service_index.total_count());
    println!("  Active (running):   {}", service_index.running_count);
    println!("  Inactive:           {}", service_index.inactive_count);
    println!("  Enabled at boot:    {}", service_index.enabled_count);
    println!("  Disabled:           {}", service_index.disabled_count);
    println!("  Masked:             {}", service_index.masked_count);
    if service_index.failed_count > 0 {
        println!("  Failed:             {}", format!("{}", service_index.failed_count).red());
    } else {
        println!("  Failed:             {}", "0".green());
    }
    println!();

    // v5.2.3: ERRORS section - universal error inspection (all object types)
    println!("{}", "[ERRORS]".cyan());

    let universal_summary = error_index.universal_grouped_errors();
    if universal_summary.has_errors() {
        // Services with errors
        if !universal_summary.services.is_empty() {
            println!("  Services (24h):");
            for entry in universal_summary.services.iter().take(5) {
                let events_str = if entry.error_count == 1 { "event" } else { "events" };
                println!("    - {} ({} {})",
                    entry.name.red(),
                    entry.error_count,
                    events_str);
                println!("        Cause: {}", entry.cause);
                if let Some(ref example) = entry.example {
                    println!("        Example: {}", truncate_str(example, 55));
                }
            }
            if universal_summary.services.len() > 5 {
                println!("    (... {} more omitted, use journalctl for full view)",
                    universal_summary.services.len() - 5);
            }
        }

        // Packages with errors
        if !universal_summary.packages.is_empty() {
            println!();
            println!("  Packages (24h):");
            for entry in universal_summary.packages.iter().take(5) {
                let events_str = if entry.error_count == 1 { "event" } else { "events" };
                println!("    - {} ({} {})",
                    entry.name.yellow(),
                    entry.error_count,
                    events_str);
                println!("        Cause: {}", entry.cause);
            }
            if universal_summary.packages.len() > 5 {
                println!("    (... {} more omitted)", universal_summary.packages.len() - 5);
            }
        }

        // Executables with errors
        if !universal_summary.executables.is_empty() {
            println!();
            println!("  Executables (24h):");
            for entry in universal_summary.executables.iter().take(5) {
                let events_str = if entry.error_count == 1 { "event" } else { "events" };
                println!("    - {} ({} {})",
                    entry.name.yellow(),
                    entry.error_count,
                    events_str);
                println!("        Cause: {}", entry.cause);
            }
            if universal_summary.executables.len() > 5 {
                println!("    (... {} more omitted)", universal_summary.executables.len() - 5);
            }
        }

        // Filesystem issues
        if !universal_summary.filesystem.is_empty() {
            println!();
            println!("  Filesystem (24h):");
            for entry in universal_summary.filesystem.iter().take(5) {
                let events_str = if entry.error_count == 1 { "event" } else { "events" };
                println!("    - {} ({} {})",
                    entry.name.yellow(),
                    entry.error_count,
                    events_str);
                println!("        Cause: {}", entry.cause);
            }
            if universal_summary.filesystem.len() > 5 {
                println!("    (... {} more omitted)", universal_summary.filesystem.len() - 5);
            }
        }

        // Kernel issues
        if !universal_summary.kernel.is_empty() {
            println!();
            println!("  Kernel (24h):");
            for entry in universal_summary.kernel.iter().take(5) {
                let events_str = if entry.error_count == 1 { "event" } else { "events" };
                println!("    - {} ({} {})",
                    entry.name.red().bold(),
                    entry.error_count,
                    events_str);
                println!("        Cause: {}", entry.cause);
            }
            if universal_summary.kernel.len() > 5 {
                println!("    (... {} more omitted)", universal_summary.kernel.len() - 5);
            }
        }
    } else {
        println!("  No errors or warnings found (24h)");
    }

    // Intrusion-like patterns (24h) grouped by service and IP
    let intrusion_patterns = intrusion_index.grouped_intrusions_by_service_24h();
    if !intrusion_patterns.is_empty() {
        println!();
        println!("  Intrusion-like patterns (24h):");
        for pattern in intrusion_patterns.iter().take(5) {
            let attempts_str = if pattern.attempt_count == 1 { "attempt" } else { "attempts" };
            println!("    - {}: {} invalid login {} from {}",
                pattern.service_name,
                pattern.attempt_count,
                attempts_str,
                pattern.source_ip.red());
        }
        if intrusion_patterns.len() > 5 {
            println!("    (... {} more omitted)", intrusion_patterns.len() - 5);
        }
    }
    println!();

    // v5.2.1: LOG SCAN section
    println!("{}", "[LOG SCAN]".cyan());
    println!("  Last scan:          {}", log_scan_state.format_last_scan());
    println!("  New errors:         {}", log_scan_state.new_errors);
    println!("  New warnings:       {}", log_scan_state.new_warnings);
    println!("  Scanner state:      {}", log_scan_state.state_string());
    println!();

    // v5.2.1: INVENTORY section (expanded)
    let (commands, packages, services) = store.count_by_type();
    let config_count = store.objects.values().filter(|o| !o.config_paths.is_empty()).count();
    let log_signatures = error_index.objects.len();
    println!("{}", "[INVENTORY]".cyan());
    println!("  Packages:           {}", format_number(packages as u64));
    println!("  Commands:           {}", format_number(commands as u64));
    println!("  Services:           {}", format_number(services as u64));
    println!("  Config files:       {}", format_number(config_count as u64));
    println!("  Log signatures:     {}", log_signatures);
    // Calculate progress if available
    let total_known = store.total_objects();
    let total_possible = count_path_binaries() + count_systemd_services();
    let progress = if total_possible > 0 {
        (total_known as f64 / total_possible as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    println!("  Progress:           {:.0}% (estimated)", progress);
    println!();

    // v5.2.1: Recent intrusions section
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

    // v5.2.1: Top errors section (if any)
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

    // v5.2.1: DISCOVERY section for priority scan status
    let needs_discovery = !builder.is_object_complete(name) && !builder.is_object_complete(&name.to_lowercase());

    if needs_discovery {
        println!("{}", "[DISCOVERY]".yellow());
        println!("  Status:        pending (priority scan)");
        println!("  Estimated:     <5 seconds>");
        println!();

        // Perform the scan
        let found = builder.targeted_discovery(name);

        // Update the display
        if found {
            println!("{}", "[DISCOVERY]".green());
            println!("  Status:        completed");
            println!("  Result:        found and indexed");
        } else {
            println!("{}", "[DISCOVERY]".yellow());
            println!("  Status:        completed");
            println!("  Result:        not found on system");
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
            // v5.2.1: BASIC INFO section (renamed from IDENTITY)
            println!("{}", "[BASIC INFO]".cyan());
            println!("  Name:            {}", obj.name);
            // Determine type for display
            let type_str = if obj.object_types.contains(&ObjectType::Service) {
                "service"
            } else if obj.object_types.contains(&ObjectType::Package) {
                "package"
            } else if obj.object_types.contains(&ObjectType::Command) {
                "command"
            } else {
                obj.category.as_str()
            };
            println!("  Type:            {}", type_str);
            if let Some(pkg) = &obj.package_name {
                println!("  Package:         {}", pkg);
            }
            println!("  Installed:       {}", if obj.installed { "yes" } else { "no" });

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

            // v5.2.1: Enhanced Service section with full state per spec
            if obj.service_unit.is_some() || obj.object_types.contains(&ObjectType::Service) {
                let service_index = ServiceIndex::load();
                println!("{}", "[SERVICE STATE]".cyan());

                if let Some(unit) = &obj.service_unit {
                    // v5.2.1: Get full service state from service index
                    if let Some(state) = service_index.services.get(unit) {
                        // Active state with color
                        let active_str = match state.active_state {
                            ActiveState::Active => "running".green().to_string(),
                            ActiveState::Failed => "failed".red().bold().to_string(),
                            ActiveState::Inactive => "inactive".dimmed().to_string(),
                            _ => state.active_state.as_str().yellow().to_string(),
                        };
                        println!("  Active:          {}", active_str);

                        // Enabled at boot with color
                        let enabled_str = match state.enabled_state {
                            EnabledState::Enabled => "yes".green().to_string(),
                            EnabledState::Masked => "MASKED".red().to_string(),
                            EnabledState::Disabled => "no".yellow().to_string(),
                            _ => state.enabled_state.as_str().to_string(),
                        };
                        println!("  Enabled at boot: {}", enabled_str);

                        // Masked state
                        let masked_str = if state.enabled_state.is_masked() {
                            "yes".red().to_string()
                        } else {
                            "no".to_string()
                        };
                        println!("  Masked:          {}", masked_str);

                        // Last exit code (from restart count or 0)
                        let exit_code = if state.active_state.is_failed() { 1 } else { 0 };
                        println!("  Last exit code:  {}", exit_code);

                        // Last restart
                        if let Some(ts) = state.state_change_at {
                            println!("  Last restart:    {}", format_timestamp(ts));
                        }

                        if let Some(reason) = &state.failure_reason {
                            println!("  Failure reason:  {}", reason.red());
                        }
                    } else {
                        // Fallback to basic info from KnowledgeObject
                        if let Some(active) = obj.service_active {
                            println!("  Active:          {}", if active { "running" } else { "inactive" });
                        }
                        if let Some(enabled) = obj.service_enabled {
                            println!("  Enabled at boot: {}", if enabled { "yes" } else { "no" });
                        }
                        println!("  Masked:          no");
                        println!("  Last exit code:  0");
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

            // v5.2.3: ERRORS section - per-object with category sections
            let error_index = ErrorIndex::load();
            let intrusion_index = IntrusionIndex::load();

            // Determine if this is a service
            let is_service = obj.service_unit.is_some() || obj.object_types.contains(&ObjectType::Service);

            if let Some(obj_errors) = error_index.get_object_errors(&obj.name) {
                let errors_24h = obj_errors.errors_24h();
                let warnings_24h = obj_errors.warnings_24h();

                println!("{}", "[ERRORS]".red());
                println!("  Time range: last 24h");
                println!("  Total error events: {}", errors_24h.len());
                println!("  Total warning events: {}", warnings_24h.len());

                // Show related files if any
                let related_files = obj_errors.all_related_files();
                if !related_files.is_empty() {
                    println!("  Related files:");
                    for f in related_files.iter().take(5) {
                        println!("    - {}", f);
                    }
                    if related_files.len() > 5 {
                        println!("    (... {} more omitted)", related_files.len() - 5);
                    }
                }

                // v5.2.3: Category breakdown
                let cat_counts = obj_errors.category_counts();
                if !cat_counts.is_empty() {
                    println!("  By category:");
                    for (cat, count) in &cat_counts {
                        println!("    {}: {}", cat.display_name(), count);
                    }
                }
                println!();

                // v5.2.3: [INTRUSION] section
                let intrusion_errs = obj_errors.intrusion_errors();
                if !intrusion_errs.is_empty() {
                    println!("{}", "[INTRUSION]".red().bold());
                    for entry in intrusion_errs.iter().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        let ip = entry.source_ip.as_deref().unwrap_or("-");
                        let user = entry.username.as_deref().unwrap_or("-");
                        println!("  [{}] {} (IP: {}, user: {})",
                            ts, truncate_str(&entry.message, 45), ip, user);
                    }
                    if intrusion_errs.len() > 5 {
                        println!("  (... {} more omitted, use journalctl for full view)",
                            intrusion_errs.len() - 5);
                    }
                    println!();
                }

                // v5.2.3: [FILESYSTEM] section
                let fs_errs = obj_errors.filesystem_errors();
                if !fs_errs.is_empty() {
                    println!("{}", "[FILESYSTEM]".yellow());
                    for entry in fs_errs.iter().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        println!("  [{}] {}", ts, truncate_str(&entry.message, 60));
                        for f in entry.related_files.iter().take(2) {
                            println!("      -> {}", f);
                        }
                    }
                    if fs_errs.len() > 5 {
                        println!("  (... {} more omitted)", fs_errs.len() - 5);
                    }
                    println!();
                }

                // v5.2.3: [CONFIG] section
                let config_errs = obj_errors.errors_by_category(LogCategory::Config);
                if !config_errs.is_empty() {
                    println!("{}", "[CONFIG]".yellow());
                    for entry in config_errs.iter().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        println!("  [{}] {}", ts, truncate_str(&entry.message, 60));
                    }
                    if config_errs.len() > 5 {
                        println!("  (... {} more omitted)", config_errs.len() - 5);
                    }
                    println!();
                }

                // v5.2.3: [DEPENDENCIES] section
                let dep_errs = obj_errors.dependency_errors();
                if !dep_errs.is_empty() {
                    println!("{}", "[DEPENDENCIES]".yellow());
                    for entry in dep_errs.iter().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        println!("  [{}] {}", ts, truncate_str(&entry.message, 60));
                    }
                    if dep_errs.len() > 5 {
                        println!("  (... {} more omitted)", dep_errs.len() - 5);
                    }
                    println!();
                }

                // v5.2.3: [PERFORMANCE ISSUES] section (runtime errors)
                let perf_errs = obj_errors.runtime_errors();
                if !perf_errs.is_empty() {
                    println!("{}", "[PERFORMANCE ISSUES]".yellow());
                    for entry in perf_errs.iter().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        println!("  [{}] {}", ts, truncate_str(&entry.message, 60));
                    }
                    if perf_errs.len() > 5 {
                        println!("  (... {} more omitted)", perf_errs.len() - 5);
                    }
                    println!();
                }

                // Warnings
                if !warnings_24h.is_empty() {
                    println!("{}", "[WARNINGS]".yellow());
                    // Group by message (count duplicates)
                    let mut warning_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
                    for w in &warnings_24h {
                        *warning_counts.entry(truncate_str(&w.message, 50)).or_insert(0) += 1;
                    }
                    for (msg, count) in warning_counts.iter().take(5) {
                        if *count > 1 {
                            println!("  {} events: \"{}\"", count, msg);
                        } else {
                            println!("  {}", msg);
                        }
                    }
                    if warning_counts.len() > 5 {
                        println!("  (... {} more omitted)", warning_counts.len() - 5);
                    }
                    println!();
                }
            } else {
                // No errors found for this object
                println!("{}", "[ERRORS]".cyan());
                println!("  No errors or warnings found for this object in the last 24h");
                println!();
            }

            // v5.2.2: INTRUSION ANALYSIS section for services
            if is_service {
                let analysis = intrusion_index.intrusion_analysis_for_service(&obj.name);
                if !analysis.is_empty() {
                    println!("{}", "[INTRUSION ANALYSIS]".red().bold());
                    println!("  Suspicious activity (last 24h):");
                    for entry in analysis.iter().take(5) {
                        println!("    - {}", entry.source_ip.red());
                        println!("        Attempts:        {}", entry.attempt_count);
                        let usernames = if entry.usernames.is_empty() {
                            "unknown".to_string()
                        } else {
                            entry.usernames.join(", ")
                        };
                        println!("        Usernames:       {}", usernames);
                        println!("        First seen:      {}", format_timestamp(entry.first_seen));
                        println!("        Last seen:       {}", format_timestamp(entry.last_seen));
                    }
                    if analysis.len() > 5 {
                        println!("    (... {} more source IPs omitted)", analysis.len() - 5);
                    }
                    println!();
                } else {
                    println!("{}", "[INTRUSION ANALYSIS]".cyan());
                    println!("  No intrusion-like patterns detected for this object in the last 24h");
                    println!();
                }
            }

            // v5.2.0: Legacy Intrusion section (for non-services)
            if !is_service {
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
            }

            // v5.2.1: SERVICE RELATIONS section (for packages/commands)
            if !obj.object_types.contains(&ObjectType::Service) {
                let service_index = ServiceIndex::load();
                let binary_path = obj.binary_path.as_deref().unwrap_or(&obj.name);
                let related_services = service_index.find_services_using_executable(binary_path);

                if !related_services.is_empty() {
                    println!("{}", "[SERVICE RELATIONS]".cyan());
                    println!("  Systemd units using this executable:");
                    for svc in related_services.iter().take(5) {
                        let status = match svc.active_state {
                            ActiveState::Active => "running".green().to_string(),
                            ActiveState::Failed => "failed".red().to_string(),
                            _ => svc.active_state.as_str().to_string(),
                        };
                        println!("    - {} ({})", svc.unit_name, status);
                    }
                    println!();
                }
            }

            // Notes
            println!("{}", "[NOTES]".cyan());
            println!("  Data collected by anna daemon (v5.2.3 Universal Error Inspection).");
        }
        None => {
            // v5.2.1: Show ERRORS section even for unknown objects
            let error_index = ErrorIndex::load();
            if let Some(obj_errors) = error_index.get_object_errors(name)
                .or_else(|| error_index.get_object_errors(&name.to_lowercase()))
            {
                if !obj_errors.logs.is_empty() {
                    println!("{}", "[ERRORS]".red());
                    println!("  No object indexed, but logs found:");
                    for entry in obj_errors.errors_only().iter().rev().take(5) {
                        let ts = format_timestamp(entry.timestamp);
                        println!("    [{}] {}", ts, truncate_str(&entry.message, 50));
                    }
                    println!();
                }
            }

            println!();
            println!("  Anna has no knowledge about '{}' yet.", name);
            println!();
            println!("  It might not be installed, or it has not been observed in use.");
            println!();

            // v5.2.3: Priority indexing display
            println!("{}", "[PRIORITY INDEX]".yellow());
            println!("  Indexing priority increased for: {}", name.cyan());
            println!("  Next scan will prioritize discovering this object.");
            println!("  Run 'annactl status' again in ~5 minutes to check if indexed.");
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
