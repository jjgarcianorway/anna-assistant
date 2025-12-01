//! SW Detail Command v7.10.0 - Software Profiles and Category Overviews
//!
//! Two modes:
//! 1. Single object profile (package/command/service)
//! 2. Category overview (list of objects)
//!
//! Sections per profile:
//! - [IDENTITY]   Name, Type, Description with source
//! - [PACKAGE]    Version, source, size, date
//! - [COMMAND]    Path, man description
//! - [SERVICE]    Unit, state, enabled
//! - [CONFIG]     System/User layout with [present]/[not present] markers (v7.10.0)
//! - [LOGS]       Severity-grouped, deduplicated logs with -p warning..alert (v7.10.0)
//! - [TELEMETRY]  Real windows (1h, 24h, 7d, 30d) with trends (v7.9.0)

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
};

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
            print_service_logs(&service_name);
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

    // [CONFIG] - discovered config files
    print_config_section(&pkg.name);

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

    // [CONFIG]
    print_config_section(&cmd.name);

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

    // [PACKAGE] - if there's an associated package
    let base_name = name.trim_end_matches(".service");
    if let Some(pkg) = get_package_info(base_name) {
        println!("{}", "[PACKAGE]".cyan());
        println!("  {}", "(source: pacman -Qi)".dimmed());
        println!("  Name:        {}", pkg.name);
        println!("  Version:     {}", pkg.version);
        println!();
    }

    // [CONFIG]
    print_service_config_section(name);

    // [LOGS]
    let unit_name = if svc.name.ends_with(".service") {
        svc.name.clone()
    } else {
        format!("{}.service", svc.name)
    };
    print_service_logs(&unit_name);

    // [USAGE]
    print_telemetry_section(base_name);
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

/// Print service logs with severity-based grouping and deduplication - v7.10.0
/// Format: timestamp "message" (seen N times)
fn print_service_logs(unit_name: &str) {
    println!("{}", "[LOGS]".cyan());
    println!("  {}", format!("(journalctl -b -u {} -p warning..alert, current boot)", unit_name).dimmed());

    // v7.10.0: Use -p warning..alert (priorities 4-1: warning, err, crit, alert)
    let output = Command::new("journalctl")
        .args([
            "-b",
            "-u", unit_name,
            "-p", "warning..alert",  // v7.10.0: only warning and above
            "-n", "50",  // v7.10.0: up to 50 messages
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
                println!("  No warnings or errors for this unit in the current boot.");
            } else {
                // v7.10.0: Deduplicate all entries together (no category separation for warning..alert)
                let all_dedup = deduplicate_log_entries_v710(&entries);

                println!();

                // v7.10.0 format: timestamp "message" (seen N times)
                for entry in all_dedup.iter() {
                    let msg = entry.message.as_deref().unwrap_or("(no message)");
                    let timestamp = entry.timestamp.as_deref().unwrap_or("");
                    let count_str = if entry.count > 1 {
                        format!("(seen {} times)", entry.count)
                    } else {
                        "(seen 1 time)".to_string()
                    };
                    println!("  {}  \"{}\"          {}", timestamp, msg, count_str.dimmed());
                }

                println!();
                println!("  No other warnings or errors this boot.");
            }
        }
        Ok(_) => {
            println!();
            println!("  {}", "(no logs available)".dimmed());
            println!();
        }
        Err(_) => {
            println!();
            println!("  {}", "(journalctl not available)".dimmed());
            println!();
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

/// Deduplicate log entries v7.10.0 format
/// Keeps last timestamp for each unique message, tracks count
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

/// Print [CONFIG] section - v7.10.0 format
/// Format: path [present/not present] (source)
fn print_config_section(name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let info = discover_config_info(name);

    if !info.has_configs {
        println!("  No specific config files detected for this software.");
        println!("  {}", format!("Source: {}", info.source_description).dimmed());
        println!();
        return;
    }

    println!();

    // System section - v7.10.0 format
    println!("  System:");
    if info.system_configs.is_empty() {
        println!("    {}", "(none from pacman -Ql)".dimmed());
    } else {
        for cfg in &info.system_configs {
            print_config_line_v710(cfg);
        }
    }
    println!();

    // User section - v7.10.0 format
    println!("  User:");
    if info.user_configs.is_empty() {
        println!("    {}", "(none documented)".dimmed());
    } else {
        for cfg in &info.user_configs {
            print_config_line_v710(cfg);
        }
    }
    println!();

    // Notes section - v7.10.0: always show precedence and source
    println!("  Notes:");
    println!("    Precedence: user configs in ~/.config usually override system configs in /etc or /usr/share.");
    println!("    {}", format!("Source: {}", info.source_description).dimmed());
    println!();
}

/// Print a single config line - v7.10.0 format
/// Format: path [present]/[not present] (source)
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

/// Print [TELEMETRY] section with v7.9.0 format: per-identity windows with trends
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
            println!("  No telemetry samples collected for this identity yet.");
            println!();
            return;
        }
    };

    // Check if we have data for this object
    if !db.has_key(name) {
        println!();
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
        println!("  No telemetry samples collected for this identity yet.");
        println!();
        return;
    }

    println!();

    // Activity windows section (v7.9.0 format)
    println!("  Activity windows:");

    // Helper to format a window line
    let format_window = |label: &str, stats: &Option<anna_common::UsageStats>| {
        if let Some(s) = stats {
            if s.sample_count > 0 {
                format!("    {}:   {} samples active, avg CPU {:.1} percent, peak {:.1} percent, avg RSS {}, peak {}",
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

    // Trend section (24h vs 7d, v7.9.0 spec)
    if let Ok(trend) = db.get_trend(name) {
        if trend.has_enough_data {
            println!("  Trend:");
            if let Some(cpu_trend) = trend.cpu_trend {
                println!("    CPU:    {} (24h vs 7d)", cpu_trend.as_str());
            }
            if let Some(mem_trend) = trend.memory_trend {
                println!("    Memory: {} (24h vs 7d)", mem_trend.as_str());
            }
            println!();
        } else if total_samples < 10 {
            println!("  Telemetry still warming up for this identity (very few samples available).");
            println!();
        }
    }
}

