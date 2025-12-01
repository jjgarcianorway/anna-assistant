//! KDB Detail Command v7.3.0 - Object Profiles and Category Overviews
//!
//! Two modes:
//! 1. Single object profile (package/command/service)
//! 2. Category overview (list of objects)
//!
//! For services: includes per-unit logs from journalctl with deduplication.
//! For all objects: includes real [USAGE] telemetry from SQLite.

use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::Command;
use anna_common::TelemetryDb;

use anna_common::grounded::{
    packages::{get_package_info, Package, PackageSource, InstallReason},
    commands::{get_command_info, command_exists, SystemCommand},
    services::{get_service_info, Service, ServiceState, EnabledState},
    config::discover_config_files,
    category::get_category,
    categoriser::{normalize_category, packages_in_category},
};

const THIN_SEP: &str = "------------------------------------------------------------";

// ============================================================================
// Category Overview
// ============================================================================

/// Run category overview (e.g., `annactl kdb editors`)
pub async fn run_category(category: &str) -> Result<()> {
    // Special case for services
    if category.eq_ignore_ascii_case("services") || category.eq_ignore_ascii_case("service") {
        return run_services_category().await;
    }

    // Try rule-based categorisation
    if let Some(cat_name) = normalize_category(category) {
        return run_rule_based_category(&cat_name).await;
    }

    // Unknown category
    eprintln!();
    eprintln!("  {} Unknown category: '{}'", "error:".red(), category);
    eprintln!();
    eprintln!("  Valid categories: Editors, Terminals, Shells, Compositors,");
    eprintln!("                    Browsers, Multimedia, Development, Network,");
    eprintln!("                    System, Power, Tools, Services");
    eprintln!();
    std::process::exit(1);
}

/// Run category view using rule-based categoriser
async fn run_rule_based_category(category_name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna KDB: {}", category_name).bold());
    println!("{}", THIN_SEP);
    println!();

    let packages = packages_in_category(category_name);

    if packages.is_empty() {
        println!("  No {} found.", category_name.to_lowercase());
    } else {
        println!("  {} {} installed:", packages.len(), category_name.to_lowercase());
        println!();

        for (name, desc, version) in &packages {
            let desc_short = if desc.len() > 50 {
                format!("{}...", &desc[..47])
            } else {
                desc.clone()
            };

            let version_str = if version.is_empty() {
                String::new()
            } else {
                format!(" ({})", version)
            };

            if desc_short.is_empty() {
                println!("  {:<12}{}", name.cyan(), version_str.dimmed());
            } else {
                println!("  {:<12}{}{}", name.cyan(), desc_short, version_str.dimmed());
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

/// Special handling for services category
async fn run_services_category() -> Result<()> {
    use anna_common::grounded::services::list_enabled_services;

    println!();
    println!("{}", "  Anna KDB: Services".bold());
    println!("{}", THIN_SEP);
    println!();

    let enabled_units = list_enabled_services();

    // Filter out template units (contain @) - they're not real running services
    let concrete_units: Vec<_> = enabled_units
        .iter()
        .filter(|u| !u.contains('@'))
        .collect();

    let total = concrete_units.len();
    let display_count = total.min(20);

    println!("  {} enabled services{}:", total,
        if total > 20 { format!(" (showing first {})", display_count) } else { String::new() });
    println!();

    for unit in concrete_units.iter().take(20) {
        let name = unit.trim_end_matches(".service");
        if let Some(svc) = get_service_info(name) {
            let state_str = match svc.state {
                ServiceState::Active => "running".green().to_string(),
                ServiceState::Inactive => "stopped".dimmed().to_string(),
                ServiceState::Failed => "failed".red().to_string(),
                ServiceState::Unknown => "unknown".to_string(),
            };
            let desc_short = if svc.description.len() > 40 {
                format!("{}...", &svc.description[..37])
            } else {
                svc.description.clone()
            };
            println!("  {:<28} [{}] {}", name.cyan(), state_str, desc_short.dimmed());
        } else {
            println!("  {:<28}", name.cyan());
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

// ============================================================================
// Single Object Profile
// ============================================================================

/// Run single object profile (e.g., `annactl kdb vim`)
pub async fn run_object(name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna KDB: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    // Try to find as service first (if name ends with .service or matches a service)
    let service_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    if let Some(svc) = get_service_info(name) {
        print_service_profile(&svc, name);
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // Try as package
    if let Some(pkg) = get_package_info(name) {
        print_package_profile(&pkg);

        // Also show command info if it exists
        if command_exists(name) {
            if let Some(cmd) = get_command_info(name) {
                print_command_section(&cmd);
            }
        }

        // Check if it has a service
        if let Some(svc) = get_service_info(name) {
            print_service_section(&svc);
            print_service_logs(&service_name);
        }

        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // Try as command
    if let Some(cmd) = get_command_info(name) {
        print_command_profile(&cmd);
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // Not found
    println!("{}", "[NOT FOUND]".yellow());
    println!("  '{}' is not a known package, command, or service.", name);
    println!();
    println!("  Checked:");
    println!("    - pacman -Qi {}", name);
    println!("    - which {}", name);
    // Don't double-append .service
    let svc_check = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };
    println!("    - systemctl show {}", svc_check);

    println!();
    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_package_profile(pkg: &Package) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", pkg.name.bold());
    if !pkg.description.is_empty() {
        println!("  Description: {}", pkg.description);
        println!("               {}", "(source: pacman -Qi)".dimmed());
    }

    // Dynamic category from real sources
    if let Some(cat_info) = get_category(&pkg.name) {
        println!("  Category:    {}", cat_info.category);
        println!("               {}", format!("(source: {})", cat_info.source).dimmed());
    }
    println!();

    // [PACKAGE]
    println!("{}", "[PACKAGE]".cyan());
    println!("  {}", "(source: pacman -Qi)".dimmed());
    println!("  Version:     {}", pkg.version);

    let source_str = match pkg.source {
        PackageSource::Official => "official",
        PackageSource::Aur => "AUR",
        PackageSource::Unknown => "unknown",
    };
    println!("  Source:      {}", source_str);

    let reason_str = match pkg.install_reason {
        InstallReason::Explicit => "explicit",
        InstallReason::Dependency => "dependency",
        InstallReason::Unknown => "unknown",
    };
    println!("  Installed:   {}", reason_str);

    if pkg.installed_size > 0 {
        println!("  Size:        {}", format_size(pkg.installed_size));
    }

    if !pkg.install_date.is_empty() {
        println!("  Date:        {}", pkg.install_date);
    }

    println!();

    // [CONFIG] - discovered config files from pacman/man
    print_config_section(&pkg.name);

    // [USAGE] - real telemetry from SQLite
    print_usage_section(&pkg.name);
}

fn print_command_profile(cmd: &SystemCommand) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", cmd.name.bold());
    if !cmd.description.is_empty() {
        println!("  Description: {}", cmd.description);
    }

    // Dynamic category from real sources
    if let Some(cat_info) = get_category(&cmd.name) {
        println!("  Category:    {}", cat_info.category);
        println!("               {}", format!("(source: {})", cat_info.source).dimmed());
    }
    println!();

    // [COMMAND]
    println!("{}", "[COMMAND]".cyan());
    println!("  {}", "(source: which, man -f)".dimmed());
    println!("  Path:        {}", cmd.path);

    if let Some(ref pkg_name) = cmd.owning_package {
        println!("  Package:     {}", pkg_name);
        if let Some(pkg) = get_package_info(pkg_name) {
            println!("  Version:     {}", pkg.version);
        }
    }

    println!();

    // [CONFIG] - discovered config files from pacman/man
    print_config_section(&cmd.name);

    // [USAGE] - real telemetry from SQLite
    print_usage_section(&cmd.name);
}

fn print_command_section(cmd: &SystemCommand) {
    println!("{}", "[COMMAND]".cyan());
    println!("  {}", "(source: which)".dimmed());
    println!("  Path:        {}", cmd.path);
    if !cmd.description.is_empty() {
        println!("  Man:         {}", cmd.description);
    }
    println!();
}

fn print_service_profile(svc: &Service, name: &str) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", svc.name.bold());
    if !svc.description.is_empty() {
        // Description already contains source attribution
        println!("  Description: {}", svc.description);
    }
    println!();

    // [SERVICE]
    print_service_section(svc);

    // [PACKAGE] - if there's an associated package
    let base_name = name.trim_end_matches(".service");
    if let Some(pkg) = get_package_info(base_name) {
        println!("{}", "[PACKAGE]".cyan());
        println!("  {}", "(source: pacman -Qi)".dimmed());
        println!("  Name:        {}", pkg.name);
        println!("  Version:     {}", pkg.version);
        println!();
    }

    // [USAGE] - telemetry for the service
    print_usage_section(base_name);

    // [LOGS]
    let unit_name = if svc.name.ends_with(".service") {
        svc.name.clone()
    } else {
        format!("{}.service", svc.name)
    };
    print_service_logs(&unit_name);
}

fn print_service_section(svc: &Service) {
    println!("{}", "[SERVICE]".cyan());
    println!("  {}", "(source: systemctl)".dimmed());

    let unit_name = if svc.name.ends_with(".service") {
        svc.name.clone()
    } else {
        format!("{}.service", svc.name)
    };
    println!("  Unit:        {}", unit_name);

    let state_str = match svc.state {
        ServiceState::Active => "running".green().to_string(),
        ServiceState::Inactive => "exited".dimmed().to_string(),
        ServiceState::Failed => "failed".red().to_string(),
        ServiceState::Unknown => "unknown".to_string(),
    };
    println!("  State:       {}", state_str);

    let enabled_str = match svc.enabled {
        EnabledState::Enabled => "enabled",
        EnabledState::Disabled => "disabled",
        EnabledState::Static => "static",
        EnabledState::Masked => "masked",
        EnabledState::Unknown => "unknown",
    };
    println!("  Enabled:     {}", enabled_str);

    println!();
}

/// Print service logs with improved deduplication (v7.3.0)
fn print_service_logs(unit_name: &str) {
    println!("{}", "[LOGS]".cyan());
    println!("  {}", format!("(journalctl -b -u {} -p warning..alert)", unit_name).dimmed());

    // Use JSON output for better parsing
    let output = Command::new("journalctl")
        .args([
            "-b",           // Current boot only
            "-u", unit_name,
            "-p", "warning..alert",
            "-n", "100",    // Get more for dedup
            "--no-pager",
            "-o", "json",
            "-q",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let entries: Vec<LogEntry> = stdout
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();

            if entries.is_empty() {
                println!();
                println!("  {}", "✓  no warnings or errors this boot".green());
            } else {
                // Deduplicate by message content
                let deduped = deduplicate_log_entries(&entries);
                let total_unique = deduped.len();
                let total_raw = entries.len();

                // Show summary
                println!();
                if total_raw != total_unique {
                    println!("  {} unique messages ({} total, {} deduplicated)",
                        total_unique, total_raw, total_raw - total_unique);
                } else {
                    println!("  {} messages this boot:", total_unique);
                }
                println!();

                // Display up to 10 most recent unique messages (wide format)
                for entry in deduped.iter().take(10) {
                    // Priority indicator
                    let prio_icon = match entry.priority.as_deref() {
                        Some("3") => "⚠".red().to_string(),    // err
                        Some("4") => "⚠".yellow().to_string(), // warning
                        Some("1") | Some("2") => "✗".red().to_string(), // alert/crit
                        _ => "·".dimmed().to_string(),
                    };

                    // Timestamp (short format)
                    let time_str = entry.timestamp_short();

                    // Message (full, not truncated)
                    let msg = entry.message.as_deref().unwrap_or("(no message)");

                    // Count indicator
                    let count_str = if entry.count > 1 {
                        format!(" {}", format!("(×{})", entry.count).dimmed())
                    } else {
                        String::new()
                    };

                    println!("  {} {} {}{}", prio_icon, time_str.dimmed(), msg, count_str);
                }

                if total_unique > 10 {
                    println!();
                    println!("  {} (and {} more...)", "…".dimmed(), total_unique - 10);
                }
            }
        }
        Ok(_) => {
            println!();
            println!("  {}", "(no logs available)".dimmed());
        }
        Err(_) => {
            println!();
            println!("  {}", "(journalctl not available)".dimmed());
        }
    }

    println!();
}

/// Structured log entry from journalctl JSON
#[derive(Debug, Clone, serde::Deserialize)]
struct LogEntry {
    #[serde(rename = "MESSAGE")]
    message: Option<String>,
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    realtime_timestamp: Option<String>,
    #[serde(skip)]
    count: usize,
}

impl LogEntry {
    /// Get short timestamp (HH:MM:SS)
    fn timestamp_short(&self) -> String {
        if let Some(ref ts_str) = self.realtime_timestamp {
            if let Ok(ts_us) = ts_str.parse::<u64>() {
                let ts_secs = ts_us / 1_000_000;
                use chrono::{DateTime, Utc};
                if let Some(dt) = DateTime::<Utc>::from_timestamp(ts_secs as i64, 0) {
                    return dt.format("%H:%M:%S").to_string();
                }
            }
        }
        "??:??:??".to_string()
    }

    /// Get message key for deduplication
    fn dedup_key(&self) -> String {
        self.message.clone().unwrap_or_default()
    }
}

/// Deduplicate log entries by message content, preserving most recent
fn deduplicate_log_entries(entries: &[LogEntry]) -> Vec<LogEntry> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<LogEntry> = Vec::new();

    // Process in order (most recent first from journalctl)
    for entry in entries {
        let key = entry.dedup_key();
        if let Some(idx) = seen.get(&key) {
            // Already seen - increment count on existing entry
            result[*idx].count += 1;
        } else {
            // First occurrence - add to result
            seen.insert(key, result.len());
            let mut new_entry = entry.clone();
            new_entry.count = 1;
            result.push(new_entry);
        }
    }

    result
}

fn format_size(bytes: u64) -> String {
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

/// Print [CONFIG] section with config files discovered from pacman/man/Arch Wiki
fn print_config_section(name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let configs = discover_config_files(name);

    if configs.is_empty() {
        println!("  {}", "(no config files discovered from pacman, man, or Arch Wiki)".dimmed());
    } else {
        // Separate system and user configs
        let system_configs: Vec<_> = configs.iter().filter(|c| !c.is_user_config).collect();
        let user_configs: Vec<_> = configs.iter().filter(|c| c.is_user_config).collect();

        // Show system configs
        if !system_configs.is_empty() {
            println!("  System:");
            for cfg in system_configs {
                let status_colored = if cfg.exists {
                    String::new()
                } else {
                    format!(" {}", "(missing)".yellow())
                };
                println!("    {}  {}{}", cfg.path, format!("({})", cfg.source).dimmed(), status_colored);
            }
        }

        // Show user configs
        if !user_configs.is_empty() {
            println!("  User:");
            for cfg in user_configs {
                let status_colored = if cfg.exists {
                    String::new()
                } else {
                    format!(" {}", "(missing)".yellow())
                };
                println!("    {}  {}{}", cfg.path, format!("({})", cfg.source).dimmed(), status_colored);
            }
        }
    }

    println!();
}

/// Print [USAGE] section with real telemetry from SQLite (v7.3.0 - multi-window)
fn print_usage_section(name: &str) {
    println!("{}", "[USAGE]".cyan());

    // Try to open telemetry database (read-only for CLI)
    match TelemetryDb::open_readonly() {
        Some(db) => {
            // Get windowed stats (1h, 24h, 7d, 30d)
            match db.get_windowed_stats(name) {
                Ok(stats) => {
                    // Check if we have any data
                    let total_samples = stats.last_30d.sample_count;
                    if total_samples == 0 {
                        println!("  {}", "(source: /var/lib/anna/telemetry.db)".dimmed());
                        println!("  {}", "(no telemetry recorded for this object)".dimmed());
                    } else {
                        println!("  {}", "(source: /var/lib/anna/telemetry.db)".dimmed());

                        // Get launch counts too
                        let launches = db.get_windowed_launches(name).unwrap_or_default();

                        // Sample counts per window
                        println!();
                        println!("  {}  {:>8}  {:>8}  {:>8}  {:>8}",
                            "Window".dimmed(), "1h".dimmed(), "24h".dimmed(), "7d".dimmed(), "30d".dimmed());
                        println!("  {}",
                            "─".repeat(50).dimmed());

                        // Launches (distinct PIDs)
                        println!("  {:<8}  {:>8}  {:>8}  {:>8}  {:>8}",
                            "Launches",
                            format_count(launches.last_1h),
                            format_count(launches.last_24h),
                            format_count(launches.last_7d),
                            format_count(launches.last_30d));

                        // Samples
                        println!("  {:<8}  {:>8}  {:>8}  {:>8}  {:>8}",
                            "Samples",
                            format_count(stats.last_1h.sample_count),
                            format_count(stats.last_24h.sample_count),
                            format_count(stats.last_7d.sample_count),
                            format_count(stats.last_30d.sample_count));

                        // CPU average
                        println!("  {:<8}  {:>7}%  {:>7}%  {:>7}%  {:>7}%",
                            "Avg CPU",
                            format_cpu(stats.last_1h.avg_cpu_percent),
                            format_cpu(stats.last_24h.avg_cpu_percent),
                            format_cpu(stats.last_7d.avg_cpu_percent),
                            format_cpu(stats.last_30d.avg_cpu_percent));

                        // CPU peak
                        println!("  {:<8}  {:>7}%  {:>7}%  {:>7}%  {:>7}%",
                            "Peak CPU",
                            format_cpu(stats.last_1h.peak_cpu_percent),
                            format_cpu(stats.last_24h.peak_cpu_percent),
                            format_cpu(stats.last_7d.peak_cpu_percent),
                            format_cpu(stats.last_30d.peak_cpu_percent));

                        // Memory average
                        println!("  {:<8}  {:>8}  {:>8}  {:>8}  {:>8}",
                            "Avg Mem",
                            format_size_compact(stats.last_1h.avg_mem_bytes),
                            format_size_compact(stats.last_24h.avg_mem_bytes),
                            format_size_compact(stats.last_7d.avg_mem_bytes),
                            format_size_compact(stats.last_30d.avg_mem_bytes));

                        // Memory peak
                        println!("  {:<8}  {:>8}  {:>8}  {:>8}  {:>8}",
                            "Peak Mem",
                            format_size_compact(stats.last_1h.peak_mem_bytes),
                            format_size_compact(stats.last_24h.peak_mem_bytes),
                            format_size_compact(stats.last_7d.peak_mem_bytes),
                            format_size_compact(stats.last_30d.peak_mem_bytes));

                        // Data quality note
                        if !stats.last_24h.has_enough_data && stats.last_24h.sample_count > 0 {
                            println!();
                            println!("  {}", "(limited 24h data - less than 10 min observed)".yellow());
                        }
                    }
                }
                Err(_) => {
                    println!("  {}", "(failed to query telemetry)".dimmed());
                }
            }
        }
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
        }
    }

    println!();
}

/// Format count for display (- if zero)
fn format_count(n: u64) -> String {
    if n == 0 { "-".to_string() } else { n.to_string() }
}

/// Format CPU percentage (- if zero)
fn format_cpu(pct: f32) -> String {
    if pct < 0.05 { "-".to_string() } else { format!("{:.1}", pct) }
}

/// Format size compactly for table (e.g., "1.2G", "512M")
fn format_size_compact(bytes: u64) -> String {
    if bytes == 0 {
        "-".to_string()
    } else if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.0}M", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
