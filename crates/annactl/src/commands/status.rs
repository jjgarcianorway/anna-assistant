//! Status Command v7.40.0 - Terminal-Adaptive Cache-Only Status
//!
//! v7.40.0: Improved update scheduler display
//! - Clearer messaging when daemon not running
//! - Shows actual state from update_state.json
//!
//! v7.39.0: Terminal-adaptive rendering, domain status, "checking..." indicator
//! - Compact mode for small terminals (< 24 rows or < 60 cols)
//! - Standard mode for normal terminals
//! - Wide mode for large terminals (> 120 cols)
//! - Shows "checking..." when domain refresh is in progress
//! - Shows domain freshness summary
//!
//! v7.38.0: Cache-only status, NO live probing
//! - Reads status_snapshot.json ONLY (written by daemon every 60s)
//! - Shows last_crash.json when daemon is down
//! - Fast: < 10ms to display
//!
//! Sections (Standard/Wide mode):
//! - [VERSION]             Anna version
//! - [DAEMON]              State from snapshot (or last crash if down)
//! - [HEALTH]              Health from snapshot + self-observation
//! - [DATA]                Knowledge object counts + domain freshness
//! - [TELEMETRY]           Sample counts
//! - [UPDATES]             Update scheduler state
//! - [ALERTS]              Alert counts from snapshot
//! - [PATHS]               Static paths only (skipped in compact)
//!
//! Sections (Compact mode):
//! - [VERSION], [DAEMON], [HEALTH], [ALERTS], [UPDATES] only
//! - One-line summaries for DATA and TELEMETRY

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, UpdateResult, SYSTEM_CONFIG_DIR, DATA_DIR, UpdateMode};
use anna_common::format_duration_secs;
use anna_common::daemon_state::{StatusSnapshot, LastCrash, INTERNAL_DIR};
use anna_common::domain_state::{DomainSummary, RefreshRequest, RefreshResponse, Domain, REQUESTS_DIR};
use anna_common::terminal::{DisplayMode, get_terminal_size, truncate, format_compact_line};
use anna_common::self_observation::SelfObservation;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the status command - v7.39.0: terminal-adaptive, cache-only
pub async fn run() -> Result<()> {
    // Detect display mode
    let mode = DisplayMode::detect();
    let (width, _) = get_terminal_size();

    // Generate separator based on terminal width
    let sep_len = (width as usize).saturating_sub(4).min(60);
    let thin_sep = "-".repeat(sep_len);

    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", thin_sep);
    println!();

    // v7.38.0: Load snapshot (written by daemon every 60s)
    let snapshot = StatusSnapshot::load();
    let last_crash = LastCrash::load();

    // v7.39.0: Load domain summary and self-observation
    let domain_summary = DomainSummary::load();
    let self_obs = SelfObservation::load();

    // v7.39.0: Check for any pending refresh requests (for "checking..." indicator)
    let checking_domains = get_pending_refresh_domains();

    // [VERSION] - always shown
    print_version_section();

    // [DAEMON] - always shown
    print_daemon_section(&snapshot, &last_crash, &checking_domains);

    // [HEALTH] - always shown
    print_health_section(&snapshot, &self_obs);

    // [DATA] - condensed in compact mode
    print_data_section(&snapshot, &domain_summary, &mode);

    // [TELEMETRY] - condensed in compact mode
    if mode != DisplayMode::Compact {
        print_telemetry_section(&snapshot);
    }

    // [UPDATES] - always shown
    print_updates_section(&snapshot);

    // [ALERTS] - always shown
    print_alerts_section(&snapshot);

    // [PATHS] - only in standard/wide mode
    if mode != DisplayMode::Compact {
        print_paths_section();
    }

    println!("{}", thin_sep);
    println!();

    Ok(())
}

/// Get domains that are currently being refreshed
fn get_pending_refresh_domains() -> Vec<String> {
    let mut domains = Vec::new();

    if let Ok(entries) = std::fs::read_dir(REQUESTS_DIR) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(req) = serde_json::from_str::<RefreshRequest>(&content) {
                    if !req.is_expired() {
                        for domain in &req.required_domains {
                            domains.push(domain.as_str().to_string());
                        }
                    }
                }
            }
        }
    }

    domains
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

/// [DAEMON] section - shows daemon state from snapshot or last crash
fn print_daemon_section(
    snapshot: &Option<StatusSnapshot>,
    last_crash: &Option<LastCrash>,
    checking_domains: &[String],
) {
    println!("{}", "[DAEMON]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            // Daemon is running (snapshot is recent)
            println!("  Status:     {}", "running".green());
            println!("  Uptime:     {}", format_duration_secs(s.uptime_secs));
            println!("  PID:        {}", s.pid);
            println!("  Snapshot:   {}", s.format_age().dimmed());

            // v7.39.0: Show "checking..." if refresh is in progress
            if !checking_domains.is_empty() {
                let domains_str = checking_domains.join(", ");
                println!("  Refresh:    {} ({})", "checking...".yellow(), domains_str.dimmed());
            }
        }
        Some(s) => {
            // Snapshot exists but is stale (daemon probably crashed)
            println!("  Status:     {} (snapshot stale: {})", "stopped".red(), s.format_age());
            println!("  Last PID:   {}", s.pid);

            // Show last crash info if available
            if let Some(crash) = last_crash {
                println!();
                println!("  Last crash: {}", crash.format_summary().yellow());
            }
        }
        None => {
            // No snapshot at all
            println!("  Status:     {} (no status snapshot found)", "stopped".red());

            // Show last crash info if available
            if let Some(crash) = last_crash {
                println!();
                println!("  Last crash: {}", crash.format_summary().yellow());
            } else {
                println!("  {}", "(daemon may have never run)".dimmed());
            }
        }
    }

    println!();
}

/// [HEALTH] section - from snapshot + self-observation
fn print_health_section(snapshot: &Option<StatusSnapshot>, self_obs: &SelfObservation) {
    println!("{}", "[HEALTH]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            if s.healthy {
                println!("  Overall:    {} all systems nominal", "✓".green());
            } else {
                // Count alerts
                let total_alerts = s.alerts_critical + s.alerts_warning;
                if s.alerts_critical > 0 {
                    println!("  Overall:    {} {} critical, {} warning",
                        "✗".red(),
                        s.alerts_critical.to_string().red(),
                        s.alerts_warning);
                } else if total_alerts > 0 {
                    println!("  Overall:    {} {} warning(s)",
                        "⚠".yellow(),
                        s.alerts_warning.to_string().yellow());
                } else {
                    println!("  Overall:    {} (snapshot indicates unhealthy)", "⚠".yellow());
                }
            }

            // v7.39.0: Self-observation warning
            if let Some(ref warning) = self_obs.warning {
                println!("  Anna:       {}", self_obs.format_summary().yellow());
            } else {
                println!("  Anna:       {}", self_obs.format_summary().dimmed());
            }
        }
        _ => {
            println!("  Overall:    {} daemon not running", "✗".red());
        }
    }

    println!();
}

/// [DATA] section - knowledge object counts from snapshot + domain freshness
fn print_data_section(
    snapshot: &Option<StatusSnapshot>,
    domain_summary: &Option<DomainSummary>,
    mode: &DisplayMode,
) {
    println!("{}", "[DATA]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Knowledge:  {} objects", s.knowledge_objects);

            // Last scan info
            if let Some(scan_at) = s.last_scan_at {
                let scan_age = (chrono::Utc::now() - scan_at).num_seconds().max(0) as u64;
                let age_str = if scan_age < 60 {
                    format!("{}s ago", scan_age)
                } else if scan_age < 3600 {
                    format!("{}m ago", scan_age / 60)
                } else {
                    format!("{}h ago", scan_age / 3600)
                };
                println!("  Last scan:  {} (took {}ms)", age_str.dimmed(), s.last_scan_duration_ms);
            } else {
                println!("  Last scan:  {}", "pending".dimmed());
            }

            // v7.39.0: Domain freshness summary (not in compact mode)
            if *mode != DisplayMode::Compact {
                if let Some(ds) = domain_summary {
                    let total_domains = ds.fresh_domains + ds.stale_domains + ds.missing_domains;
                    if ds.stale_domains > 0 || ds.missing_domains > 0 {
                        println!("  Domains:    {}/{} fresh, {} stale, {} missing",
                            ds.fresh_domains, total_domains,
                            ds.stale_domains.to_string().yellow(),
                            ds.missing_domains);
                    } else {
                        println!("  Domains:    {}/{} fresh", ds.fresh_domains.to_string().green(), total_domains);
                    }
                }
            }
        }
        _ => {
            println!("  Knowledge:  {} (daemon not running)", "-".dimmed());
            println!("  Last scan:  -");
        }
    }

    println!();
}

/// [TELEMETRY] section - sample counts from snapshot
fn print_telemetry_section(snapshot: &Option<StatusSnapshot>) {
    let config = AnnaConfig::load();

    println!("{}", "[TELEMETRY]".cyan());

    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config.".dimmed());
        println!();
        return;
    }

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Samples (24h): {}", s.telemetry_samples_24h);

            // Memory usage
            if s.memory_mb > 0 {
                println!("  Memory:        {} MB", s.memory_mb);
            }

            // CPU alert if any
            if let Some(ref cpu_alert) = s.cpu_alert {
                println!("  CPU:           {}", cpu_alert.yellow());
            }

            // Disk info
            if let Some(ref disk) = s.disk_info {
                println!("  Disk:          {}", disk);
            }

            // Network IO
            if let Some(ref net) = s.network_io {
                println!("  Network:       {}", net);
            }
        }
        _ => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

/// [UPDATES] section - update scheduler state (v7.40.0: improved clarity)
fn print_updates_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[UPDATES]".cyan());

    let state = UpdateState::load();
    let daemon_running = snapshot.as_ref().map(|s| !s.is_stale()).unwrap_or(false);

    // Mode
    println!("  Mode:       {}", state.format_mode());

    // Interval
    println!("  Interval:   {}", state.format_interval());

    // v7.40.0: Last check - show actual timestamp or clear status
    let last_check_str = if state.last_check_at == 0 {
        if state.mode != UpdateMode::Auto {
            "n/a (auto-updates disabled)".dimmed().to_string()
        } else if daemon_running {
            "pending (first check soon)".to_string()
        } else {
            "pending (start daemon)".dimmed().to_string()
        }
    } else {
        state.format_last_check()
    };
    println!("  Last check: {}", last_check_str);

    // Result - only show if there's a result
    if state.last_check_at > 0 || state.last_result != UpdateResult::Pending {
        println!("  Result:     {}", state.format_last_result());
    }

    // v7.40.0: Next check - clearer messaging
    if state.mode == UpdateMode::Auto {
        let next_str = if !daemon_running {
            "paused (daemon not running)".dimmed().to_string()
        } else if let Some(s) = snapshot {
            if let Some(next) = s.update_next_check {
                let delta = (next - chrono::Utc::now()).num_seconds();
                if delta > 0 {
                    format!("in {}m {}s", delta / 60, delta % 60)
                } else {
                    "now".to_string()
                }
            } else if state.next_check_at > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let delta = state.next_check_at.saturating_sub(now);
                if delta > 0 {
                    format!("in {}m {}s", delta / 60, delta % 60)
                } else {
                    "now".to_string()
                }
            } else {
                "soon".to_string()
            }
        } else {
            "soon".to_string()
        };
        println!("  Next check: {}", next_str);
    }

    // Show available version if update is available
    if let Some(ref ver) = state.last_checked_version_available {
        if !state.last_checked_version_installed.is_empty() {
            println!("  Available:  {} -> {}",
                state.last_checked_version_installed,
                ver.green());
        }
    }

    println!();
}

/// [ALERTS] section - alert counts from snapshot
fn print_alerts_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[ALERTS]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Critical:   {}", if s.alerts_critical > 0 {
                s.alerts_critical.to_string().red().to_string()
            } else {
                "0".green().to_string()
            });
            println!("  Warnings:   {}", if s.alerts_warning > 0 {
                s.alerts_warning.to_string().yellow().to_string()
            } else {
                "0".green().to_string()
            });

            // Instrumentation count
            if s.instrumentation_count > 0 {
                println!("  Installed:  {} tool(s) by Anna", s.instrumentation_count);
            }
        }
        _ => {
            println!("  {}", "(daemon not running - no live data)".dimmed());
        }
    }

    println!();
}

/// [PATHS] section - static paths only, no probing
fn print_paths_section() {
    println!("{}", "[PATHS]".cyan());

    // Config path
    let config_path = format!("{}/config.toml", SYSTEM_CONFIG_DIR);
    let config_exists = Path::new(&config_path).exists();
    if config_exists {
        println!("  Config:     {}", config_path);
    } else {
        println!("  Config:     {} {}", config_path, "(missing)".yellow());
    }

    // Data path
    let data_exists = Path::new(DATA_DIR).exists();
    if data_exists {
        println!("  Data:       {}", DATA_DIR);
    } else {
        println!("  Data:       {} {}", DATA_DIR, "(missing)".yellow());
    }

    // Internal dir
    let internal_exists = Path::new(INTERNAL_DIR).exists();
    if internal_exists {
        println!("  Internal:   {}", INTERNAL_DIR);
    } else {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(will create on daemon start)".dimmed());
    }

    // Logs hint
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    println!();
}
