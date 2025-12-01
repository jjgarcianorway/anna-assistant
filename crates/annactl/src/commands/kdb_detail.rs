//! KDB Detail Command v7.5.0 - Object Profiles and Category Overviews
//!
//! Two modes:
//! 1. Single object profile (package/command/service)
//! 2. Category overview (list of objects)
//!
//! For services: includes per-unit logs from journalctl with deduplication.
//! For all objects: includes real [USAGE] telemetry from SQLite.
//! v7.4.0: Enhanced [CONFIG] sections with precedence rules.
//! v7.5.0: Enhanced [USAGE] with exec counts, CPU time totals per window.
//!         Enhanced [LOGS] with severity breakdown and local timestamps.

use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::Command;
use anna_common::TelemetryDb;

use anna_common::grounded::{
    packages::{get_package_info, Package, PackageSource, InstallReason},
    commands::{get_command_info, command_exists, SystemCommand},
    services::{get_service_info, Service, ServiceState, EnabledState},
    config::{discover_config_info, discover_service_config},
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

    // [CONFIG] - service unit files and related configs (v7.4.0)
    print_service_config_section(name);

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

/// Print service logs with improved deduplication (v7.5.0)
fn print_service_logs(unit_name: &str) {
    println!("{}", "[LOGS]".cyan());
    println!("  {}", format!("(journalctl -b -u {} -p warning..alert)", unit_name).dimmed());

    // Use JSON output for better parsing
    let output = Command::new("journalctl")
        .args([
            "-b",           // Current boot only
            "-u", unit_name,
            "-p", "warning..alert",
            "-n", "200",    // Get more for dedup
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

                // Count by severity
                let err_count = deduped.iter().filter(|e| e.priority.as_deref() == Some("3")).map(|e| e.count).sum::<usize>();
                let warn_count = deduped.iter().filter(|e| e.priority.as_deref() == Some("4")).map(|e| e.count).sum::<usize>();
                let crit_count = deduped.iter().filter(|e| matches!(e.priority.as_deref(), Some("1") | Some("2"))).map(|e| e.count).sum::<usize>();

                // Show summary with severity breakdown
                println!();
                let mut summary_parts = Vec::new();
                if crit_count > 0 { summary_parts.push(format!("{} critical", crit_count).red().to_string()); }
                if err_count > 0 { summary_parts.push(format!("{} errors", err_count).red().to_string()); }
                if warn_count > 0 { summary_parts.push(format!("{} warnings", warn_count).yellow().to_string()); }

                if total_raw != total_unique {
                    println!("  {} unique ({} total): {}",
                        total_unique, total_raw, summary_parts.join(", "));
                } else {
                    println!("  {} messages: {}", total_unique, summary_parts.join(", "));
                }
                println!();

                // Display up to 10 most recent unique messages (wide format)
                for entry in deduped.iter().take(10) {
                    // Priority indicator with label
                    let (prio_icon, prio_label) = match entry.priority.as_deref() {
                        Some("1") => ("✗".red().to_string(), "ALERT".red().to_string()),
                        Some("2") => ("✗".red().to_string(), "CRIT".red().to_string()),
                        Some("3") => ("⚠".red().to_string(), "ERR".red().to_string()),
                        Some("4") => ("·".yellow().to_string(), "WARN".yellow().to_string()),
                        _ => ("·".dimmed().to_string(), "INFO".dimmed().to_string()),
                    };

                    // Timestamp (local time, short format)
                    let time_str = entry.timestamp_local();

                    // Message (full, not truncated)
                    let msg = entry.message.as_deref().unwrap_or("(no message)");

                    // Count indicator
                    let count_str = if entry.count > 1 {
                        format!(" {}", format!("(×{})", entry.count).dimmed())
                    } else {
                        String::new()
                    };

                    println!("  {} [{:<4}] {} {}{}", prio_icon, prio_label, time_str.dimmed(), msg, count_str);
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
    /// Get short timestamp in local time (HH:MM:SS)
    fn timestamp_local(&self) -> String {
        if let Some(ref ts_str) = self.realtime_timestamp {
            if let Ok(ts_us) = ts_str.parse::<u64>() {
                let ts_secs = ts_us / 1_000_000;
                use chrono::{DateTime, Utc, Local};
                if let Some(dt) = DateTime::<Utc>::from_timestamp(ts_secs as i64, 0) {
                    let local: DateTime<Local> = dt.into();
                    return local.format("%H:%M:%S").to_string();
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

/// Print [CONFIG] section with config files discovered from pacman/man/Arch Wiki (v7.4.0)
fn print_config_section(name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let info = discover_config_info(name);

    if !info.has_configs {
        println!("  No configuration files documented by pacman, man pages, or Arch Wiki.");
    } else {
        // Show system configs
        if !info.system_configs.is_empty() {
            println!("  System:");
            for cfg in &info.system_configs {
                let status = if cfg.exists {
                    "[present]".green().to_string()
                } else {
                    "[not present]".yellow().to_string()
                };
                println!("    {:<40} {} {}", cfg.path, status, format!("({})", cfg.source).dimmed());
            }
        }

        // Show user configs
        if !info.user_configs.is_empty() {
            println!("  User:");
            for cfg in &info.user_configs {
                let status = if cfg.exists {
                    "[present]".green().to_string()
                } else {
                    "[not present]".yellow().to_string()
                };
                println!("    {:<40} {} {}", cfg.path, status, format!("({})", cfg.source).dimmed());
            }
        }

        // Show precedence notes if any
        if !info.precedence_rules.is_empty() {
            println!("  Notes:");
            for rule in &info.precedence_rules {
                let prefix = if rule.is_conventional { "Conventional" } else { "Precedence" };
                println!("    {}: {} {}", prefix, rule.description, format!("({})", rule.source).dimmed());
            }
        }
    }

    println!();
}

/// Print [CONFIG] section for a service (v7.4.0)
fn print_service_config_section(svc_name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let info = discover_service_config(svc_name);

    let mut has_any = false;

    // Unit file section
    if info.unit_file.is_some() || info.override_unit.is_some() {
        println!("  Unit file:");
        if let Some(ref unit) = info.unit_file {
            let status = if unit.exists {
                "[present]".green().to_string()
            } else {
                "[not present]".yellow().to_string()
            };
            println!("    {:<50} {} {}", unit.path, status, format!("({})", unit.source).dimmed());
            has_any = true;
        }
        if let Some(ref override_unit) = info.override_unit {
            let status = if override_unit.exists {
                "[present]".green().to_string()
            } else {
                "[not present]".yellow().to_string()
            };
            println!("    {:<50} {} (user override)", override_unit.path, status);
            has_any = true;
        }
    }

    // Drop-in section
    if let Some(ref drop_in) = info.drop_in_dir {
        println!("  Drop-in:");
        let status = if drop_in.exists {
            "[present]".green().to_string()
        } else {
            "[not present]".yellow().to_string()
        };
        println!("    {:<50} {}", drop_in.path, status);
        has_any = has_any || drop_in.exists;

        for file in &info.drop_in_files {
            // Show just the filename for drop-in files
            let file_name = std::path::Path::new(&file.path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.clone());
            println!("      {:<48} {}", file_name, "[present]".green());
        }
    }

    // Related configs (EnvironmentFile, etc)
    if !info.related_configs.is_empty() {
        println!("  Related:");
        for cfg in &info.related_configs {
            let status = if cfg.exists {
                "[present]".green().to_string()
            } else {
                "[not present]".yellow().to_string()
            };
            println!("    {:<50} {} {}", cfg.path, status, format!("({})", cfg.source).dimmed());
            has_any = true;
        }
    }

    // Package configs if any
    if info.package_configs.has_configs {
        if !info.package_configs.system_configs.is_empty() {
            println!("  Package config:");
            for cfg in &info.package_configs.system_configs {
                let status = if cfg.exists {
                    "[present]".green().to_string()
                } else {
                    "[not present]".yellow().to_string()
                };
                println!("    {:<50} {} {}", cfg.path, status, format!("({})", cfg.source).dimmed());
                has_any = true;
            }
        }
    }

    if !has_any {
        println!("  No configuration files documented.");
    }

    println!();
}

/// Print [USAGE] section with real telemetry from SQLite (v7.5.0 - enhanced format)
fn print_usage_section(name: &str) {
    use anna_common::{format_cpu_time, format_bytes_human};

    println!("{}", "[USAGE]".cyan());

    // Try to open telemetry database (read-only for CLI)
    match TelemetryDb::open_readonly() {
        Some(db) => {
            // Get enhanced windowed stats (1h, 24h, 7d, 30d)
            match db.get_enhanced_windowed_stats(name) {
                Ok(stats) => {
                    // Check if we have any data
                    if !stats.last_30d.has_data {
                        println!("  Telemetry not collected yet (daemon uptime too short or feature disabled).");
                    } else {
                        // Print each window that has data
                        let windows = [
                            ("1h", &stats.last_1h),
                            ("24h", &stats.last_24h),
                            ("7d", &stats.last_7d),
                            ("30d", &stats.last_30d),
                        ];

                        for (label, window) in &windows {
                            if window.has_data {
                                println!("  {}:", label);
                                println!("    Execs:       {}", window.exec_count);
                                println!("    CPU total:   {}", format_cpu_time(window.cpu_time_total_secs));
                                println!("    CPU peak:    {:.1}%", window.cpu_peak_percent);
                                println!("    RSS peak:    {}", format_bytes_human(window.rss_peak_bytes));
                                if window.last_seen_ts > 0 {
                                    println!("    Last exec:   {}", TelemetryDb::format_timestamp(window.last_seen_ts));
                                }
                                println!();
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("  {}", "(failed to query telemetry)".dimmed());
                }
            }
        }
        None => {
            println!("  Telemetry not collected yet (daemon uptime too short or feature disabled).");
        }
    }

    println!();
}

