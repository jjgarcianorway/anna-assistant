//! SW Detail Command v7.18.0 - Software Profiles and Category Overviews
//!
//! Two modes:
//! 1. Single object profile (package/command/service)
//! 2. Category overview (list of objects)
//!
//! Sections per profile:
//! - [IDENTITY]     Name, Type, Description with source
//! - [PACKAGE]      Version, source, size, date
//! - [COMMAND]      Path, man description
//! - [SERVICE]      Unit, state, enabled
//! - [SERVICE LIFECYCLE] Restarts, exit codes, activation failures (v7.16.0)
//! - [DEPENDENCIES] Package deps and service relations (v7.13.0)
//! - [CONFIG]       Primary/Secondary/Notes + Sanity notes (v7.14.0)
//! - [CONFIG GRAPH] Ownership and consumers of config files (v7.17.0)
//! - [HISTORY]      Package lifecycle and config changes (v7.18.0)
//! - [LOGS]         Boot-anchored patterns with novelty (v7.18.0)
//! - [TELEMETRY]    Real windows with peak/trend summaries (v7.14.0)
//! - Cross notes:   Links between logs, telemetry, deps, config (v7.14.0)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::Command;

use anna_common::grounded::{
    packages::{get_package_info, Package, PackageSource, InstallReason},
    commands::{get_command_info, command_exists, SystemCommand},
    services::{get_service_info, Service, ServiceState, EnabledState},
    config::{discover_config_info, discover_service_config},
    category::get_category,
    categoriser::{normalize_category, packages_in_category},
    deps::{get_package_deps, get_service_deps},
    log_patterns::{extract_patterns_for_unit, extract_patterns_with_history, LogPatternSummary, LogHistorySummary, format_time_short},
    config_graph::get_config_graph_for_software,
};
use anna_common::ServiceLifecycle;
use anna_common::change_journal::{get_package_history, get_config_history};

const THIN_SEP: &str = "------------------------------------------------------------";

// ============================================================================
// Category Overview
// ============================================================================

/// Run category overview (e.g., `annactl sw editors`)
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
    println!("{}", format!("  Anna SW: {}", category_name).bold());
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
    println!("{}", "  Anna SW: Services".bold());
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

/// Run single object profile (e.g., `annactl sw vim`)
/// Resolution order:
/// 1. Exact match on package name (pacman -Qi)
/// 2. Exact match on command name (in PATH)
/// 3. Exact match on systemd unit name
/// 4. Category names (handled in main.rs before this)
/// If NAME ends with .service, prefer service over package
pub async fn run_object(name: &str) -> Result<()> {
    // Canonical name (may differ in case from input)
    let canonical_name: String;
    let input_name = name;

    // Service name for systemd lookup
    let service_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    // If name ends with .service, prefer service resolution
    if name.ends_with(".service") {
        if let Some(svc) = get_service_info(name) {
            canonical_name = name.to_string();
            println!();
            println!("{}", format!("  Anna SW: {}", canonical_name).bold());
            println!("{}", THIN_SEP);
            println!();
            print_service_profile(&svc, &canonical_name);
            println!("{}", THIN_SEP);
            println!();
            return Ok(());
        }
    }

    // 1. Try exact match on package name first
    if let Some(pkg) = get_package_info(name) {
        canonical_name = pkg.name.clone();
        println!();
        println!("{}", format!("  Anna SW: {}", canonical_name).bold());
        println!("{}", THIN_SEP);
        println!();

        print_package_profile(&pkg);

        // Also show command info if it exists
        if command_exists(&canonical_name) {
            if let Some(cmd) = get_command_info(&canonical_name) {
                print_command_section(&cmd);
            }
        }

        // Check if it has a related service
        if let Some(svc) = get_service_info(&canonical_name) {
            print_service_section(&svc);
            let log_summary = print_service_logs(&service_name);
            print_logs_health_note(&log_summary);
        }

        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // 2. Try exact match on command name
    if let Some(cmd) = get_command_info(name) {
        canonical_name = cmd.name.clone();
        println!();
        println!("{}", format!("  Anna SW: {}", canonical_name).bold());
        println!("{}", THIN_SEP);
        println!();
        print_command_profile(&cmd);
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // 3. Try as service (without .service suffix)
    if let Some(svc) = get_service_info(name) {
        canonical_name = name.to_string();
        println!();
        println!("{}", format!("  Anna SW: {}", canonical_name).bold());
        println!("{}", THIN_SEP);
        println!();
        print_service_profile(&svc, &canonical_name);
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // 4. Try case-insensitive match on package name
    if let Some(pkg) = try_case_insensitive_package(name) {
        canonical_name = pkg.name.clone();
        println!();
        println!("{}", format!("  Anna SW: {}", canonical_name).bold());
        println!("{}", THIN_SEP);
        println!();

        print_package_profile(&pkg);

        // Also show command info if it exists
        if command_exists(&canonical_name) {
            if let Some(cmd) = get_command_info(&canonical_name) {
                print_command_section(&cmd);
            }
        }

        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // Not found
    println!();
    println!("{}", format!("  Anna SW: {}", input_name).bold());
    println!("{}", THIN_SEP);
    println!();
    println!("{}", "[NOT FOUND]".yellow());
    println!("  '{}' is not a known package, command, or service.", input_name);
    println!();
    println!("  Checked:");
    println!("    - pacman -Qi {}", input_name);
    println!("    - which {}", input_name);
    let svc_check = if input_name.ends_with(".service") {
        input_name.to_string()
    } else {
        format!("{}.service", input_name)
    };
    println!("    - systemctl show {}", svc_check);

    println!();
    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

/// Try case-insensitive package lookup
fn try_case_insensitive_package(name: &str) -> Option<Package> {
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let name_lower = name.to_lowercase();

    for line in stdout.lines() {
        if line.to_lowercase() == name_lower && line != name {
            return get_package_info(line);
        }
    }

    None
}

/// Determine object type string
fn get_object_type(name: &str) -> String {
    let is_pkg = get_package_info(name).is_some();
    let is_cmd = command_exists(name);
    let is_svc = get_service_info(name).is_some();

    let mut types = Vec::new();
    if is_pkg { types.push("package"); }
    if is_cmd { types.push("command"); }
    if is_svc { types.push("service"); }

    if types.is_empty() {
        "unknown".to_string()
    } else {
        types.join(" + ")
    }
}

fn print_package_profile(pkg: &Package) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", pkg.name.bold());
    println!("  Type:        {}", get_object_type(&pkg.name));
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

    // [DEPENDENCIES] - v7.13.0
    print_package_dependencies_section(&pkg.name);

    // [CONFIG] - discovered config files
    print_config_section(&pkg.name);

    // [CONFIG GRAPH] - v7.17.0: ownership and consumers
    print_config_graph_section(&pkg.name);

    // [HISTORY] - v7.18.0: package and config history
    print_history_section(&pkg.name);

    // [USAGE] - real telemetry
    print_telemetry_section(&pkg.name);
}

fn print_command_profile(cmd: &SystemCommand) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", cmd.name.bold());
    println!("  Type:        {}", get_object_type(&cmd.name));
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

    // [DEPENDENCIES] - v7.13.0 (use package name if available)
    let deps_name = cmd.owning_package.as_deref().unwrap_or(&cmd.name);
    print_package_dependencies_section(deps_name);

    // [CONFIG]
    print_config_section(&cmd.name);

    // [CONFIG GRAPH] - v7.17.0
    print_config_graph_section(&cmd.name);

    // [HISTORY] - v7.18.0: use package name for history if available
    let history_name = cmd.owning_package.as_deref().unwrap_or(&cmd.name);
    print_history_section(history_name);

    // [USAGE]
    print_telemetry_section(&cmd.name);
}

fn print_command_section(cmd: &SystemCommand) {
    println!("{}", "[COMMAND]".cyan());
    println!("  {}", "(source: which)".dimmed());
    println!("  Path:        {}", cmd.path);
    if !cmd.description.is_empty() {
        println!("  Man:         {} {}", cmd.description, "(source: man -f)".dimmed());
    }
    println!();
}

fn print_service_profile(svc: &Service, name: &str) {
    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", svc.name.bold());
    println!("  Type:        service");
    if !svc.description.is_empty() {
        println!("  Description: {} {}", svc.description, "(source: systemctl show)".dimmed());
    }
    println!();

    // [SERVICE]
    print_service_section(svc);

    // [SERVICE LIFECYCLE] - v7.16.0
    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };
    print_service_lifecycle_section(&unit_name);

    // [PACKAGE] - if there's an associated package
    let base_name = name.trim_end_matches(".service");
    if let Some(pkg) = get_package_info(base_name) {
        println!("{}", "[PACKAGE]".cyan());
        println!("  {}", "(source: pacman -Qi)".dimmed());
        println!("  Name:        {}", pkg.name);
        println!("  Version:     {}", pkg.version);
        println!();
    }

    // [DEPENDENCIES] - v7.13.0
    print_service_dependencies_section(name);

    // [CONFIG]
    print_service_config_section(name);

    // [HISTORY] - v7.18.0: package and config history
    print_history_section(base_name);

    // [LOGS] - v7.18.0: boot-anchored patterns with novelty
    let log_summary = print_service_logs_v718(&unit_name);

    // [USAGE]
    print_telemetry_section(base_name);

    // v7.14.0: Cross notes - link logs, telemetry, deps, config
    print_cross_notes_sw_v716(&log_summary, base_name);
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

/// Print service logs with pattern-based grouping - v7.14.0
/// Shows patterns with counts and time hints, not raw log lines
fn print_service_logs(unit_name: &str) -> LogPatternSummary {
    println!("{}", "[LOGS]".cyan());

    // v7.14.0: Use pattern extraction module
    let summary = extract_patterns_for_unit(unit_name);

    if summary.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component in the current boot.");
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    // v7.14.0: Pattern summary header
    println!();
    println!("  Patterns (this boot):");
    println!("    Total warnings/errors: {} ({} patterns)",
             summary.total_count.to_string().yellow(),
             summary.pattern_count);
    println!();

    // v7.14.0: Show top 3 patterns with counts and time hints
    for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
        let time_hint = format_time_short(&pattern.last_seen);

        // Truncate pattern for display if too long
        let display_pattern = if pattern.pattern.len() > 60 {
            format!("{}...", &pattern.pattern[..57])
        } else {
            pattern.pattern.clone()
        };

        let count_str = if pattern.count == 1 {
            "seen 1 time".to_string()
        } else {
            format!("seen {} times", pattern.count)
        };

        println!("    {}) \"{}\"", i + 1, display_pattern);
        println!("       {} ({}, last at {})",
                 "",
                 count_str.dimmed(),
                 time_hint);
    }

    // Show if there are more patterns
    if summary.pattern_count > 3 {
        println!();
        println!("    {} ({} more patterns not shown)",
                 "...".dimmed(),
                 summary.pattern_count - 3);
    }

    println!();
    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

/// Print health note based on log patterns - v7.14.0
fn print_logs_health_note(summary: &LogPatternSummary) {
    if summary.total_count > 10 {
        // Check for repeated patterns
        if let Some(top) = summary.patterns.first() {
            if top.count > 5 {
                println!();
                println!("  Health note: {} repeated errors (most common: {} occurrences).",
                         summary.total_count, top.count);
            }
        }
    }
}

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
    /// v7.10.0: Formatted timestamp string for display
    #[serde(skip)]
    timestamp: Option<String>,
}

impl LogEntry {
    #[allow(dead_code)]
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

    /// v7.10.0: Format timestamp for display (YYYY-MM-DD HH:MM:SS)
    fn timestamp_v710(&self) -> String {
        if let Some(ref ts_str) = self.realtime_timestamp {
            if let Ok(ts_us) = ts_str.parse::<u64>() {
                let ts_secs = ts_us / 1_000_000;
                use chrono::{DateTime, Utc, Local};
                if let Some(dt) = DateTime::<Utc>::from_timestamp(ts_secs as i64, 0) {
                    let local: DateTime<Local> = dt.into();
                    return local.format("%Y-%m-%d %H:%M:%S").to_string();
                }
            }
        }
        "????-??-?? ??:??:??".to_string()
    }

    fn dedup_key(&self) -> String {
        self.message.clone().unwrap_or_default()
    }
}

#[allow(dead_code)]
fn deduplicate_log_entries(entries: &[LogEntry]) -> Vec<LogEntry> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<LogEntry> = Vec::new();

    for entry in entries {
        let key = entry.dedup_key();
        if let Some(idx) = seen.get(&key) {
            result[*idx].count += 1;
        } else {
            seen.insert(key, result.len());
            let mut new_entry = entry.clone();
            new_entry.count = 1;
            result.push(new_entry);
        }
    }

    result
}

/// Deduplicate log entries v7.12.0 format
/// Keeps first timestamp for each unique message, tracks total count
/// Designed for clarity: shows when an error first appeared + how many times
#[allow(dead_code)]
fn deduplicate_log_entries_v712(entries: &[LogEntry]) -> Vec<LogEntry> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<LogEntry> = Vec::new();

    for entry in entries {
        let key = entry.dedup_key();
        if let Some(idx) = seen.get(&key) {
            // v7.12.0: Just increment count, keep FIRST timestamp
            result[*idx].count += 1;
        } else {
            seen.insert(key, result.len());
            let mut new_entry = entry.clone();
            new_entry.count = 1;
            new_entry.timestamp = Some(entry.timestamp_v710());
            result.push(new_entry);
        }
    }

    result
}

/// Deduplicate log entries v7.10.0 format (legacy)
/// Keeps last timestamp for each unique message, tracks count
#[allow(dead_code)]
fn deduplicate_log_entries_v710(entries: &[LogEntry]) -> Vec<LogEntry> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<LogEntry> = Vec::new();

    for entry in entries {
        let key = entry.dedup_key();
        if let Some(idx) = seen.get(&key) {
            result[*idx].count += 1;
            // v7.10.0: Keep the LAST timestamp (most recent occurrence)
            result[*idx].timestamp = Some(entry.timestamp_v710());
        } else {
            seen.insert(key, result.len());
            let mut new_entry = entry.clone();
            new_entry.count = 1;
            new_entry.timestamp = Some(entry.timestamp_v710());
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

/// Print [CONFIG] section - v7.12.0 format
/// Structure: Primary (active configs), Secondary (templates/examples), Notes (precedence)
fn print_config_section(name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let info = discover_config_info(name);

    if !info.has_configs {
        println!("  No specific configuration paths discovered for this software.");
        println!("  {}", format!("Source: {}", info.source_description).dimmed());
        println!();
        return;
    }

    // v7.12.0: Separate into Primary (active) and Secondary (templates/examples)
    // Primary: /etc configs and ~/.config user configs that are main active locations
    // Secondary: /usr/share templates, examples, defaults
    let mut primary_configs: Vec<_> = Vec::new();
    let mut secondary_configs: Vec<_> = Vec::new();

    // System configs (/etc) go to Primary
    for cfg in &info.system_configs {
        if cfg.path.starts_with("/etc/") {
            primary_configs.push(cfg);
        } else {
            secondary_configs.push(cfg);
        }
    }

    // User configs (~/) go to Primary
    for cfg in &info.user_configs {
        primary_configs.push(cfg);
    }

    // Other configs (templates, examples) go to Secondary
    for cfg in &info.other_configs {
        secondary_configs.push(cfg);
    }

    // Primary section - v7.12.0
    println!("  Primary:");
    if primary_configs.is_empty() {
        println!("    {}", "(no active config locations found)".dimmed());
    } else {
        for cfg in &primary_configs {
            print_config_line_v712(cfg);
        }
    }

    // Secondary section - v7.12.0
    if !secondary_configs.is_empty() {
        println!("  Secondary:");
        for cfg in &secondary_configs {
            print_config_line_v712(cfg);
        }
    }

    // Notes section - v7.12.0: max 3 lines of useful info
    println!("  Notes:");

    // Check for override situation (both user and system present)
    let has_system_present = primary_configs.iter().any(|c| c.exists && c.path.starts_with("/etc/"));
    let has_user_present = primary_configs.iter().any(|c| c.exists && c.is_user_config);

    if has_user_present && has_system_present {
        println!("    - User config overrides system config when both exist.");
    } else if has_user_present {
        println!("    - User config is active.");
    } else if has_system_present {
        println!("    - System config is active (no user override).");
    }

    // XDG hint if applicable
    let has_xdg_config = primary_configs.iter().any(|c| c.path.contains("/.config/"));
    if has_xdg_config {
        println!("    - XDG paths take precedence when documented.");
    }

    println!("    {}", format!("Source: {}", info.source_description).dimmed());

    // v7.14.0: Sanity notes section
    print_config_sanity_notes(&primary_configs, &secondary_configs);

    println!();
}

/// Print [CONFIG GRAPH] section - v7.17.0
/// Shows which configs a software reads and shared configs
fn print_config_graph_section(name: &str) {
    let graph = get_config_graph_for_software(name);

    // Skip if no configs found
    if graph.reads.is_empty() && graph.shared.is_empty() {
        return;
    }

    println!("{}", "[CONFIG GRAPH]".cyan());
    println!("  {}", format!("(source: {})", graph.source).dimmed());

    // Configs this software reads
    if !graph.reads.is_empty() {
        println!("  Reads:");
        for cfg in &graph.reads {
            let status = if cfg.exists {
                "[present]".green().to_string()
            } else {
                "[not present]".dimmed().to_string()
            };
            println!("    {:<40} {}  {}", cfg.path, status, format!("({})", cfg.evidence).dimmed());
        }
    }

    // Shared configs (PAM, NSS, etc.)
    if !graph.shared.is_empty() {
        println!("  Shared:");
        for cfg in &graph.shared {
            let status = if cfg.exists {
                "[present]".green().to_string()
            } else {
                "[not present]".dimmed().to_string()
            };
            println!("    {:<40} {}  {}", cfg.path, status, format!("({})", cfg.evidence).dimmed());
        }
    }

    println!();
}

/// Print [HISTORY] section - v7.18.0
/// Shows package lifecycle events and config file changes
fn print_history_section(name: &str) {
    use chrono::{DateTime, Local};

    let pkg_history = get_package_history(name);
    let config_history = get_config_history(name);

    // Skip if no history
    if pkg_history.is_empty() && config_history.is_empty() {
        return;
    }

    println!("{}", "[HISTORY]".cyan());
    println!("  {}", "(source: pacman.log, change journal)".dimmed());

    // Package events
    if !pkg_history.is_empty() {
        println!("  Package:");
        for event in pkg_history.iter().take(5) {
            let ts = DateTime::from_timestamp(event.timestamp as i64, 0)
                .map(|dt| {
                    let local: DateTime<Local> = dt.into();
                    local.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());

            let action = event.change_type.as_str();
            let details = event.details.as_ref()
                .map(|d| {
                    if let Some(ref new_ver) = d.new_version {
                        if let Some(ref old_ver) = d.old_version {
                            format!("{} -> {}", old_ver, new_ver)
                        } else {
                            new_ver.clone()
                        }
                    } else if let Some(ref ver) = d.version {
                        ver.clone()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();

            if details.is_empty() {
                println!("    {}  {:<12} {}", ts, action, name);
            } else {
                println!("    {}  {:<12} {}  {}", ts, action, name, details.dimmed());
            }
        }
        if pkg_history.len() > 5 {
            println!("    {} ({} more events)", "...".dimmed(), pkg_history.len() - 5);
        }
    }

    // Config changes
    if !config_history.is_empty() {
        println!("  Config:");
        for event in config_history.iter().take(3) {
            let ts = DateTime::from_timestamp(event.timestamp as i64, 0)
                .map(|dt| {
                    let local: DateTime<Local> = dt.into();
                    local.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());

            let path = event.details.as_ref()
                .and_then(|d| d.config_path.as_ref())
                .map(|p| p.as_str())
                .unwrap_or(&event.subject);

            println!("    {}  {} modified", ts, path);
        }
        if config_history.len() > 3 {
            println!("    {} ({} more changes)", "...".dimmed(), config_history.len() - 3);
        }
    }

    println!();
}

/// Print a single config line - v7.12.0 format
/// Format: path [present]/[not present] (source)
fn print_config_line_v712(cfg: &anna_common::grounded::config::ConfigFile) {
    // Format status - v7.12.0 uses [present] / [not present]
    let status_str = if cfg.exists {
        "[present]".green().to_string()
    } else {
        "[not present]".dimmed().to_string()
    };

    // Format source (abbreviated)
    let source_short = abbreviate_source(&cfg.source);

    // Print with alignment - show path dim if not present
    if cfg.exists {
        println!("    {:<45} {}   {}", cfg.path, status_str, format!("({})", source_short).dimmed());
    } else {
        println!("    {:<45} {} {}", cfg.path.dimmed(), status_str, format!("({})", source_short).dimmed());
    }
}

/// Print a single config line - v7.10.0 format (legacy)
#[allow(dead_code)]
fn print_config_line_v710(cfg: &anna_common::grounded::config::ConfigFile) {
    // Format status - v7.10.0 uses [present] / [not present]
    let status_str = if cfg.exists {
        "[present]".green().to_string()
    } else {
        "[not present]".dimmed().to_string()
    };

    // Format source (abbreviated)
    let source_short = abbreviate_source(&cfg.source);

    // Print with alignment - show path dim if not present
    if cfg.exists {
        println!("    {:<45} {}   {}", cfg.path, status_str, format!("({})", source_short).dimmed());
    } else {
        println!("    {:<45} {} {}", cfg.path.dimmed(), status_str, format!("({})", source_short).dimmed());
    }
}

/// Print a single config line with status and source - v7.8.0 (legacy)
#[allow(dead_code)]
fn print_config_line(cfg: &anna_common::grounded::config::ConfigFile, is_user: bool) {
    use anna_common::grounded::config::ConfigStatus;

    let path = if is_user {
        resolve_user_path_display(&cfg.path)
    } else {
        cfg.path.clone()
    };

    // Format status
    let status_str = match cfg.status {
        ConfigStatus::Present => "[present]".green().to_string(),
        ConfigStatus::Missing => "[missing]".dimmed().to_string(),
        ConfigStatus::Recommended => "[recommended]".yellow().to_string(),
    };

    // Format source (abbreviated)
    let source_short = abbreviate_source(&cfg.source);

    // Print with alignment
    if cfg.exists {
        println!("    {:<45} {}    {}", path, status_str, format!("({})", source_short).dimmed());
    } else {
        println!("    {:<45} {}    {}", path.dimmed(), status_str, format!("({})", source_short).dimmed());
    }
}

/// Abbreviate source for display - v7.8.0
fn abbreviate_source(source: &str) -> String {
    // If multiple sources, just show first one
    if source.contains(", ") {
        if let Some(first) = source.split(", ").next() {
            return abbreviate_single_source(first);
        }
    }
    abbreviate_single_source(source)
}

/// Abbreviate a single source string
fn abbreviate_single_source(source: &str) -> String {
    if source.starts_with("pacman -Ql") {
        return source.replace("pacman -Ql ", "pacman -Ql ");
    }
    if source.starts_with("man ") {
        return source.to_string();
    }
    if source == "Arch Wiki" {
        return "Arch Wiki".to_string();
    }
    if source == "filesystem" {
        return "filesystem".to_string();
    }
    source.to_string()
}

/// Print config sanity notes - v7.14.0
/// Checks: empty files, unreadable, unexpected symlinks, basic metadata
fn print_config_sanity_notes(
    primary: &[&anna_common::grounded::config::ConfigFile],
    _secondary: &[&anna_common::grounded::config::ConfigFile],
) {
    let mut sanity_notes: Vec<String> = Vec::new();

    // Check each primary config that exists
    for cfg in primary.iter().filter(|c| c.exists) {
        let path = resolve_user_path_display(&cfg.path);

        // Check if file is empty
        if let Ok(metadata) = std::fs::metadata(&path) {
            if metadata.is_file() && metadata.len() == 0 {
                sanity_notes.push(format!("{} exists but is empty (0 bytes).", cfg.path));
            }

            // Check if it's a symlink (might be unexpected)
            if metadata.file_type().is_symlink() {
                if let Ok(target) = std::fs::read_link(&path) {
                    sanity_notes.push(format!(
                        "{} is a symlink to {}.",
                        cfg.path,
                        target.display()
                    ));
                }
            }
        }

        // Check readability by current user
        if std::fs::read(&path).is_err() && !std::fs::read_dir(&path).is_ok() {
            sanity_notes.push(format!("{} exists but is not readable.", cfg.path));
        }
    }

    // Check for any missing primary configs that were expected
    let missing_count = primary.iter().filter(|c| !c.exists).count();
    if missing_count > 0 && sanity_notes.is_empty() {
        // Only note if there are no other issues
        let present_count = primary.iter().filter(|c| c.exists).count();
        if present_count > 0 {
            // Some configs exist, some don't - that's fine
        }
    }

    // Print sanity notes section
    if sanity_notes.is_empty() {
        println!("  Sanity notes:");
        println!("    - No obvious issues detected with primary config paths.");
    } else {
        println!("  Sanity notes:");
        for note in sanity_notes.iter().take(3) {
            println!("    - {}", note.yellow());
        }
        if sanity_notes.len() > 3 {
            println!("    - {} ({} more issues not shown)",
                     "...".dimmed(),
                     sanity_notes.len() - 3);
        }
    }
}

/// Print Cross notes section - v7.14.0
/// Links observations from logs, telemetry, deps, and config
#[allow(dead_code)]
fn print_cross_notes_sw(log_summary: &LogPatternSummary, _name: &str) {
    // Only show if there's something interesting to note
    let mut notes: Vec<String> = Vec::new();

    // Check log patterns vs telemetry
    if log_summary.total_count > 20 {
        // High error count
        notes.push(format!(
            "Frequent log activity ({} warnings/errors this boot).",
            log_summary.total_count
        ));
    } else if log_summary.total_count == 0 {
        notes.push("No warnings or errors recorded this boot.".to_string());
    }

    // Check for recurring patterns
    if let Some(top) = log_summary.patterns.first() {
        if top.count > 10 && top.count_last_hour > 0 {
            notes.push(format!(
                "Most common pattern seen {} times ({} in last hour).",
                top.count, top.count_last_hour
            ));
        }
    }

    // Only print if we have 1-2 notes
    if !notes.is_empty() && notes.len() <= 2 {
        println!();
        println!("{}", "Cross notes:".cyan());
        for note in notes.iter().take(2) {
            println!("  - {}", note);
        }
    }
}

/// Resolve user paths (~/) to actual home directory for display
fn resolve_user_path_display(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

/// Print [CONFIG] section for a service - v7.6.0 enhanced layout
fn print_service_config_section(svc_name: &str) {
    println!("{}", "[CONFIG]".cyan());
    println!("  {}", "(sources: systemctl show, pacman -Ql, filesystem)".dimmed());

    let info = discover_service_config(svc_name);

    // Unit file section
    if let Some(ref unit) = info.unit_file {
        println!();
        println!("  Unit file:");
        println!("    - {}", unit.path);
    }

    // Overrides section (v7.6.0)
    println!();
    println!("  Overrides:");

    let has_override = info.override_unit.as_ref().map(|u| u.exists).unwrap_or(false);
    let has_dropins = !info.drop_in_files.is_empty();

    if has_override || has_dropins {
        println!("    Present:");
        if let Some(ref override_unit) = info.override_unit {
            if override_unit.exists {
                println!("      - {}", override_unit.path);
            }
        }
        for file in &info.drop_in_files {
            println!("      - {}", file.path);
        }
    }

    // Show declared override locations even if not present
    if let Some(ref drop_in) = info.drop_in_dir {
        if !drop_in.exists && info.drop_in_files.is_empty() {
            println!("    Missing:");
            println!("      - {:<44} {}", drop_in.path, "(declared, not present)".dimmed());
        }
    }

    if !has_override && !has_dropins && info.drop_in_dir.as_ref().map(|d| d.exists).unwrap_or(true) {
        println!("    (none)");
    }

    // Related configs (EnvironmentFile, etc.)
    if !info.related_configs.is_empty() {
        println!();
        println!("  Related configs:");
        for cfg in &info.related_configs {
            let status = if cfg.exists {
                "[exists]".green().to_string()
            } else {
                "[missing]".yellow().to_string()
            };
            println!("    - {:<46} {}", cfg.path, status);
        }
    }

    // Also show package configs if available
    if info.package_configs.has_configs {
        println!();
        println!("  Package configs:");
        for cfg in &info.package_configs.system_configs {
            let status = if cfg.exists {
                "[exists]".green().to_string()
            } else {
                "[missing]".yellow().to_string()
            };
            println!("    - {:<46} {}", cfg.path, status);
        }
    }

    println!();
}

/// Print [TELEMETRY] section with v7.12.0 format: State summary + windows + trends + health notes
fn print_telemetry_section(name: &str) {
    use anna_common::config::AnnaConfig;
    use anna_common::{TelemetryDb, format_bytes_human,
                      WINDOW_1H, WINDOW_24H, WINDOW_7D, WINDOW_30D};

    println!("{}", "[TELEMETRY]".cyan());
    println!("  {}", "(source: Anna daemon, sampling every 30s)".dimmed());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!();
        println!("  Telemetry disabled in config.");
        println!();
        return;
    }

    // Try to open SQLite database
    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => {
            println!();
            println!("  State (24h):     not enough data yet");
            println!("  No telemetry samples collected for this identity yet.");
            println!();
            return;
        }
    };

    // Check if we have data for this object
    if !db.has_key(name) {
        println!();
        println!("  State (24h):     not enough data yet");
        println!("  No telemetry samples collected for this identity yet.");
        println!();
        return;
    }

    // Get stats for each window
    let stats_1h = db.get_usage_stats_window(name, WINDOW_1H).ok();
    let stats_24h = db.get_usage_stats_window(name, WINDOW_24H).ok();
    let stats_7d = db.get_usage_stats_window(name, WINDOW_7D).ok();
    let stats_30d = db.get_usage_stats_window(name, WINDOW_30D).ok();

    // Check if we have very few samples
    let total_samples = stats_30d.as_ref().map(|s| s.sample_count).unwrap_or(0);
    if total_samples == 0 {
        println!();
        println!("  State (24h):     not enough data yet");
        println!("  No telemetry samples collected for this identity yet.");
        println!();
        return;
    }

    println!();

    // v7.12.0: State summary line at the top
    let state_desc = derive_telemetry_state(&stats_24h);
    println!("  State (24h):     {}", state_desc);
    println!();

    // v7.12.0: Show key metrics in compact form
    if let Some(ref s) = stats_1h {
        if s.sample_count > 0 {
            println!("  CPU avg (1h):    {:.1} %    (max {:.1} %)",
                     s.avg_cpu_percent, s.peak_cpu_percent);
            println!("  RAM avg (1h):    {}  (max {})",
                     format_bytes_human(s.avg_mem_bytes), format_bytes_human(s.peak_mem_bytes));
        }
    }
    println!();

    // Activity windows section
    println!("  Activity windows:");

    // Helper to format a window line
    let format_window = |label: &str, stats: &Option<anna_common::UsageStats>| {
        if let Some(s) = stats {
            if s.sample_count > 0 {
                format!("    {}:   {} samples, avg CPU {:.1}%, peak {:.1}%, avg RSS {}, peak {}",
                    label,
                    s.sample_count,
                    s.avg_cpu_percent,
                    s.peak_cpu_percent,
                    format_bytes_human(s.avg_mem_bytes),
                    format_bytes_human(s.peak_mem_bytes))
            } else {
                format!("    {}:   no samples", label)
            }
        } else {
            format!("    {}:   no data", label)
        }
    };

    println!("{}", format_window("Last 1h", &stats_1h));
    println!("{}", format_window("Last 24h", &stats_24h));
    println!("{}", format_window("Last 7d", &stats_7d));
    println!("{}", format_window("Last 30d", &stats_30d));
    println!();

    // Trend section (24h vs 7d)
    let mut has_trend = false;
    if let Ok(trend) = db.get_trend(name) {
        if trend.has_enough_data {
            println!("  Trend:");
            if let Some(cpu_trend) = trend.cpu_trend {
                println!("    CPU:    {} (24h vs 7d)", cpu_trend.as_str());
                has_trend = true;
            }
            if let Some(mem_trend) = trend.memory_trend {
                println!("    Memory: {} (24h vs 7d)", mem_trend.as_str());
                has_trend = true;
            }
            if has_trend {
                println!();
            }
        }
    }

    // v7.11.0: Health notes section - link telemetry with logs
    print_telemetry_health_notes(name, &stats_24h);
}

/// v7.12.0: Derive telemetry state summary based on thresholds
/// CPU thresholds: <5% light, 5-30% moderate, >30% high
/// RAM thresholds: <256MiB low, 256MiB-2GiB moderate, >2GiB high
fn derive_telemetry_state(stats_24h: &Option<anna_common::UsageStats>) -> String {
    let Some(stats) = stats_24h else {
        return "not enough data yet".to_string();
    };

    if stats.sample_count == 0 {
        return "not enough data yet".to_string();
    }

    // Classify CPU usage
    let cpu_state = if stats.avg_cpu_percent < 5.0 && stats.peak_cpu_percent < 50.0 {
        "light CPU"
    } else if stats.avg_cpu_percent <= 30.0 {
        "moderate CPU"
    } else {
        "high CPU"
    };

    // Classify RAM usage
    let avg_ram_mib = stats.avg_mem_bytes as f64 / (1024.0 * 1024.0);
    let ram_state = if avg_ram_mib < 256.0 {
        "low RAM"
    } else if avg_ram_mib <= 2048.0 {
        "moderate RAM"
    } else {
        "high RAM"
    };

    // Check for activity
    let activity = if stats.sample_count > 10 {
        "active"
    } else {
        "mostly idle"
    };

    format!("{}, {}, {}", activity, cpu_state, ram_state)
}

/// v7.11.0: Print health notes linking telemetry with recent logs
fn print_telemetry_health_notes(name: &str, stats_24h: &Option<anna_common::UsageStats>) {
    let mut notes = Vec::new();

    // Check for high CPU usage
    if let Some(stats) = stats_24h {
        if stats.peak_cpu_percent > 90.0 {
            notes.push(format!("High CPU usage detected (peak {:.0}%) - check for runaway processes", stats.peak_cpu_percent));
        }
        if stats.peak_mem_bytes > 4 * 1024 * 1024 * 1024 { // 4 GiB
            let gb = stats.peak_mem_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            notes.push(format!("High memory usage (peak {:.1} GiB) - consider memory limits", gb));
        }
    }

    // Check for service errors in journalctl
    let unit_name = format!("{}.service", name);
    let error_count = get_service_error_count(&unit_name);
    if error_count > 0 {
        notes.push(format!("{} error(s) in logs this boot - see [LOGS] section above", error_count));
    }

    // Only show section if we have notes
    if !notes.is_empty() {
        println!("  Notes:");
        for note in &notes {
            println!("    ⚠  {}", note.yellow());
        }
        println!();
    }
}

/// Get count of errors for a service unit in current boot
fn get_service_error_count(unit_name: &str) -> usize {
    let output = Command::new("journalctl")
        .args([
            "-u", unit_name,
            "-p", "err..alert",
            "-b",
            "--no-pager",
            "-q",
            "-o", "short",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.lines().count();
        }
    }

    0
}

// ============================================================================
// v7.13.0: Dependency Section
// ============================================================================

/// Print [DEPENDENCIES] section for packages - v7.13.0
fn print_package_dependencies_section(name: &str) {
    let pkg_deps = get_package_deps(name);
    let svc_deps = get_service_deps(name);

    // Only show if we have some data
    if !pkg_deps.has_data() && !svc_deps.has_data() {
        return;
    }

    println!("{}", "[DEPENDENCIES]".cyan());

    // Package dependencies
    if pkg_deps.has_data() {
        println!("  Direct package deps:");
        let deps_display = if pkg_deps.direct.is_empty() {
            "none".to_string()
        } else {
            pkg_deps.direct.join(", ")
        };
        println!("    {}", deps_display);

        // Show optional deps if any (limited)
        if !pkg_deps.optional.is_empty() {
            println!("    {} {}", "optional:".dimmed(), pkg_deps.optional.join(", ").dimmed());
        }
    }

    // Service relations (if this package has a service)
    if svc_deps.has_data() {
        println!();
        println!("  Service relations:");

        if !svc_deps.requires.is_empty() {
            println!("    Requires:  {}", svc_deps.requires.join(", "));
        }
        if !svc_deps.wants.is_empty() {
            println!("    Wants:     {}", svc_deps.wants.join(", "));
        }
        if !svc_deps.part_of.is_empty() {
            println!("    Part of:   {}", svc_deps.part_of.join(", "));
        }
        if !svc_deps.wanted_by.is_empty() {
            println!("    WantedBy:  {}", svc_deps.wanted_by.join(", "));
        }
    }

    println!();
    println!("  Notes:");
    println!("    - Package dependencies from pacman.");
    if svc_deps.has_data() {
        println!("    - Service dependencies from systemctl show.");
    }
    println!("    {}", format!("Source: {}", pkg_deps.source).dimmed());

    println!();
}

/// Print [DEPENDENCIES] section for services - v7.13.0
fn print_service_dependencies_section(name: &str) {
    let base_name = name.trim_end_matches(".service");
    let svc_deps = get_service_deps(name);
    let pkg_deps = get_package_deps(base_name);

    // Only show if we have some data
    if !svc_deps.has_data() && !pkg_deps.has_data() {
        return;
    }

    println!("{}", "[DEPENDENCIES]".cyan());

    // Package dependencies (if there's an associated package)
    if pkg_deps.has_data() {
        println!("  Direct package deps:");
        let deps_display = if pkg_deps.direct.is_empty() {
            "none".to_string()
        } else {
            pkg_deps.direct.join(", ")
        };
        println!("    {}", deps_display);
        println!();
    }

    // Service relations
    if svc_deps.has_data() {
        println!("  Service relations:");

        if !svc_deps.requires.is_empty() {
            println!("    Requires:  {}", svc_deps.requires.join(", "));
        }
        if !svc_deps.wants.is_empty() {
            println!("    Wants:     {}", svc_deps.wants.join(", "));
        }
        if !svc_deps.part_of.is_empty() {
            println!("    Part of:   {}", svc_deps.part_of.join(", "));
        }
        if !svc_deps.wanted_by.is_empty() {
            println!("    WantedBy:  {}", svc_deps.wanted_by.join(", "));
        }
        if !svc_deps.required_by.is_empty() {
            println!("    RequiredBy: {}", svc_deps.required_by.join(", "));
        }
    }

    println!();
    println!("  Notes:");
    if pkg_deps.has_data() {
        println!("    - Package dependencies from pacman.");
    }
    println!("    - Service dependencies from systemctl show.");
    println!("    {}", format!("Source: {}", svc_deps.source).dimmed());

    println!();
}

// ============================================================================
// v7.16.0: Service Lifecycle Section
// ============================================================================

/// Print [SERVICE LIFECYCLE] section - v7.16.0
/// Shows restarts, exit codes, activation failures over time windows
fn print_service_lifecycle_section(unit_name: &str) {
    let lifecycle = ServiceLifecycle::query(unit_name);

    if !lifecycle.exists {
        return;
    }

    // Skip for static units (no restart semantics)
    if lifecycle.is_static {
        return;
    }

    println!("{}", "[SERVICE LIFECYCLE]".cyan());
    println!("  {}", format!("(source: {})", lifecycle.source).dimmed());
    println!();

    // State
    println!("  State:       {}", lifecycle.format_state());

    // Restarts
    println!("  Restarts:    {}", lifecycle.format_restarts());

    // Last exit
    println!("  Last exit:   {}", lifecycle.format_last_exit());

    // Activation failures
    println!("  Failures:");
    println!("    last 24h:  {}", if lifecycle.failures_24h == 0 {
        "0".green().to_string()
    } else {
        lifecycle.failures_24h.to_string().yellow().to_string()
    });
    println!("    last 7d:   {}", if lifecycle.failures_7d == 0 {
        "0".green().to_string()
    } else {
        lifecycle.failures_7d.to_string().yellow().to_string()
    });

    println!();
}

// ============================================================================
// v7.16.0: Multi-Window Log History
// ============================================================================

/// Print [LOGS] section with v7.16.0 multi-window history
fn print_service_logs_v716(unit_name: &str) -> LogHistorySummary {
    println!("{}", "[LOGS]".cyan());

    let summary = extract_patterns_with_history(unit_name);

    if summary.is_empty_this_boot() && summary.patterns.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component.");
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    println!();

    // v7.16.0: Severity breakdown for this boot
    println!("  This boot:");
    let total_this_boot = summary.total_this_boot();
    if total_this_boot == 0 {
        println!("    {} {} warnings or errors", "✓".green(), "No".green());
    } else {
        if summary.this_boot_critical > 0 {
            println!("    Critical: {}", summary.this_boot_critical.to_string().red().bold());
        }
        if summary.this_boot_error > 0 {
            println!("    Errors:   {}", summary.this_boot_error.to_string().red());
        }
        if summary.this_boot_warning > 0 {
            println!("    Warnings: {}", summary.this_boot_warning.to_string().yellow());
        }
    }
    println!();

    // v7.16.0: Top patterns with history
    if !summary.patterns.is_empty() {
        println!("  Top patterns:");
        for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
            // Truncate pattern for display
            let display_pattern = if pattern.pattern.len() > 55 {
                format!("{}...", &pattern.pattern[..52])
            } else {
                pattern.pattern.clone()
            };

            // Build history string
            let mut history_parts = Vec::new();
            if pattern.count_this_boot > 0 {
                history_parts.push(format!("boot: {}", pattern.count_this_boot));
            }
            if pattern.count_24h > pattern.count_this_boot {
                history_parts.push(format!("24h: {}", pattern.count_24h));
            }
            if pattern.count_7d > pattern.count_24h {
                history_parts.push(format!("7d: {}", pattern.count_7d));
            }

            let history_str = if history_parts.is_empty() {
                "no history".to_string()
            } else {
                history_parts.join(", ")
            };

            println!("    {}) \"{}\"", i + 1, display_pattern);
            println!("       {} ({})", pattern.priority.dimmed(), history_str.dimmed());
        }

        if summary.patterns.len() > 3 {
            println!();
            println!("    {} ({} more patterns not shown)",
                     "...".dimmed(),
                     summary.patterns.len() - 3);
        }
    }

    // v7.16.0: Show patterns with history beyond this boot
    let history_patterns = summary.patterns_with_history();
    if !history_patterns.is_empty() {
        println!();
        println!("  Recurring patterns (seen in previous boots):");
        for pattern in history_patterns.iter().take(2) {
            let display_pattern = if pattern.pattern.len() > 50 {
                format!("{}...", &pattern.pattern[..47])
            } else {
                pattern.pattern.clone()
            };
            println!("    - \"{}\" ({} boots, {} total in 7d)",
                     display_pattern.dimmed(),
                     pattern.boots_seen,
                     pattern.count_7d);
        }
    }

    println!();
    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

// ============================================================================
// v7.18.0: Boot-Anchored Logs with Pattern IDs and Novelty
// ============================================================================

/// Print [LOGS] section with v7.18.0 boot-anchored patterns and novelty detection
fn print_service_logs_v718(unit_name: &str) -> LogHistorySummary {
    use anna_common::log_patterns_enhanced::LogPatternAnalyzer;

    println!("{}", "[LOGS]".cyan());

    // First get the basic history summary for compatibility
    let summary = extract_patterns_with_history(unit_name);

    // Get enhanced patterns with novelty detection
    let service_name = unit_name.trim_end_matches(".service");
    let analyzer = LogPatternAnalyzer::new();
    let pattern_summary = analyzer.get_patterns_for_service(service_name);

    if summary.is_empty_this_boot() && pattern_summary.current_boot.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component.");
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    println!();

    // v7.18.0: Boot-anchored view with novelty
    println!("  Boot 0 (current):");
    let total_this_boot = summary.total_this_boot();
    if total_this_boot == 0 {
        println!("    {} {} warnings or errors", "✓".green(), "No".green());
    } else {
        if summary.this_boot_critical > 0 {
            println!("    Critical: {}", summary.this_boot_critical.to_string().red().bold());
        }
        if summary.this_boot_error > 0 {
            println!("    Errors:   {}", summary.this_boot_error.to_string().red());
        }
        if summary.this_boot_warning > 0 {
            println!("    Warnings: {}", summary.this_boot_warning.to_string().yellow());
        }
    }
    println!();

    // v7.18.0: Show new patterns (not seen before this boot)
    if !pattern_summary.new_this_boot.is_empty() {
        println!("  {} (first seen this boot):", "New patterns".yellow());
        for occurrence in pattern_summary.new_this_boot.iter().take(3) {
            let display = if occurrence.pattern.template.len() > 50 {
                format!("{}...", &occurrence.pattern.template[..47])
            } else {
                occurrence.pattern.template.clone()
            };
            println!("    [{}] \"{}\"",
                     occurrence.pattern.short_id().yellow(),
                     display);
            println!("           {} (count: {})",
                     occurrence.pattern.priority.dimmed(),
                     occurrence.count_this_boot);
        }
        if pattern_summary.new_this_boot.len() > 3 {
            println!("    {} ({} more new patterns)",
                     "...".dimmed(),
                     pattern_summary.new_this_boot.len() - 3);
        }
        println!();
    }

    // v7.18.0: Show known patterns with history
    if !pattern_summary.known_patterns.is_empty() {
        println!("  Known patterns:");
        for occurrence in pattern_summary.known_patterns.iter().take(3) {
            let display = if occurrence.pattern.template.len() > 50 {
                format!("{}...", &occurrence.pattern.template[..47])
            } else {
                occurrence.pattern.template.clone()
            };

            let history = format!("boot: {}, 7d: {}, {} boots",
                                  occurrence.count_this_boot,
                                  occurrence.count_7d,
                                  occurrence.boots_seen);

            println!("    [{}] \"{}\"",
                     occurrence.pattern.short_id().dimmed(),
                     display);
            println!("           {} ({})",
                     occurrence.pattern.priority.dimmed(),
                     history.dimmed());
        }
        if pattern_summary.known_patterns.len() > 3 {
            println!("    {} ({} more known patterns)",
                     "...".dimmed(),
                     pattern_summary.known_patterns.len() - 3);
        }
        println!();
    }

    // v7.18.0: Previous boot summary (if available)
    if !pattern_summary.previous_boot.is_empty() {
        let prev_count: u32 = pattern_summary.previous_boot.iter()
            .map(|p| p.count_24h)
            .sum();
        println!("  Boot -1 (previous):");
        println!("    {} patterns, {} total events", pattern_summary.previous_boot.len(), prev_count);
        println!();
    }

    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

/// v7.16.0: Cross notes based on log history
fn print_cross_notes_sw_v716(log_summary: &LogHistorySummary, _name: &str) {
    let mut notes: Vec<String> = Vec::new();

    // Check for high error counts
    let total = log_summary.total_this_boot();
    if total > 20 {
        notes.push(format!(
            "Frequent log activity ({} warnings/errors this boot).",
            total
        ));
    } else if total == 0 {
        notes.push("No warnings or errors recorded this boot.".to_string());
    }

    // Check for recurring patterns
    let recurring = log_summary.patterns_with_history();
    if !recurring.is_empty() {
        let top = &recurring[0];
        if top.boots_seen > 2 {
            notes.push(format!(
                "Recurring issue seen in {} boots - may need attention.",
                top.boots_seen
            ));
        }
    }

    // Check for critical errors
    if log_summary.this_boot_critical > 0 {
        notes.push(format!(
            "{} critical error(s) this boot - requires attention.",
            log_summary.this_boot_critical
        ));
    }

    // Only print if we have 1-3 notes
    if !notes.is_empty() && notes.len() <= 3 {
        println!();
        println!("{}", "Cross notes:".cyan());
        for note in notes.iter().take(3) {
            println!("  - {}", note);
        }
    }
}

