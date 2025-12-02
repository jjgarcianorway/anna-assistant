//! SW Detail Command v7.28.0 - Zero Truncation & Config Reorganization
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
//! - [CONFIG - DETECTED]        Existing config paths (v7.28.0)
//! - [CONFIG - COMMON LOCATIONS] From man pages, package files (v7.28.0)
//! - [CONFIG - PRECEDENCE]      Load order for existing configs (v7.28.0)
//! - [CONFIG GRAPH] Config relationships: reads, shared (v7.17.0)
//! - [HISTORY]      Package lifecycle and config changes (v7.18.0)
//! - [RELATIONSHIPS] Services, processes, hardware touched (v7.24.0)
//! - [LOGS]         Boot-anchored patterns with baseline tags (v7.20.0)
//! - [USAGE]       Time-anchored trends with percentage+range display (v7.23.0)
//! - Cross notes:   Links between logs, telemetry, deps, config (v7.14.0)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::Command;

use anna_common::grounded::{
    packages::{get_package_info, Package, PackageSource, InstallReason},
    commands::{get_command_info, command_exists, SystemCommand},
    services::{get_service_info, Service, ServiceState, EnabledState},
    config::discover_service_config,
    category::get_category,
    categoriser::{normalize_category, packages_in_category},
    deps::{get_package_deps, get_service_deps},
    log_patterns::{extract_patterns_for_unit, extract_patterns_with_history, LogPatternSummary, LogHistorySummary, format_time_short},
    config_graph::get_config_graph_for_software,
};
use anna_common::ServiceLifecycle;
use anna_common::change_journal::{get_package_history, get_config_history};
use anna_common::config_atlas::{build_config_atlas, ConfigStatus};
// v7.22.0: Software scenario lenses
use anna_common::sw_lens::{
    is_sw_category, get_sw_category,
    NetworkSwLens, DisplaySwLens, AudioSwLens, PowerSwLens,
};
// v7.24.0: Relationships
use anna_common::relationships::{
    get_software_relationships, format_software_relationships_section,
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

    // v7.22.0: Check for scenario lens categories
    if is_sw_category(category) {
        return run_scenario_lens_category(category).await;
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
    eprintln!("                    System, Power, Tools, Services, Display, Audio");
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
            // v7.29.0: No truncation - show full description
            let version_str = if version.is_empty() {
                String::new()
            } else {
                format!(" ({})", version)
            };

            if desc.is_empty() {
                println!("  {:<12}{}", name.cyan(), version_str.dimmed());
            } else {
                println!("  {:<12}{}{}", name.cyan(), desc, version_str.dimmed());
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
            // v7.29.0: No truncation - show full description
            println!("  {:<28} [{}] {}", name.cyan(), state_str, svc.description.dimmed());
        } else {
            println!("  {:<28}", name.cyan());
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

/// v7.22.0: Run scenario lens view for network, display, audio, power
async fn run_scenario_lens_category(category: &str) -> Result<()> {
    let cat_type = get_sw_category(category).unwrap_or("unknown");

    match cat_type {
        "network" => run_network_sw_lens().await,
        "display" => run_display_sw_lens().await,
        "audio" => run_audio_sw_lens().await,
        "power" => run_power_sw_lens().await,
        _ => {
            eprintln!("  Unknown scenario lens category: {}", category);
            Ok(())
        }
    }
}

/// Network software lens
async fn run_network_sw_lens() -> Result<()> {
    println!();
    println!("{}", "  Anna SW: network".bold());
    println!("{}", THIN_SEP);
    println!();

    let lens = NetworkSwLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let service_names: Vec<_> = lens.services.iter()
        .filter(|s| s.active)
        .map(|s| s.name.as_str())
        .collect();
    println!("  Category:    network");
    println!("  Components:  {}", if service_names.is_empty() {
        "none active".to_string()
    } else {
        service_names.join(", ")
    });
    println!();

    // [TOPOLOGY]
    println!("{}", "[TOPOLOGY]".cyan());
    println!("  Core services:");
    for svc in &lens.services {
        let status = if svc.active {
            "[running]".green().to_string()
        } else if svc.status == "enabled" {
            "[enabled]".dimmed().to_string()
        } else {
            format!("[{}]", svc.status).dimmed().to_string()
        };
        println!("    {:<40} {}", svc.unit, status);
    }
    println!();

    // [CONFIG]
    println!("{}", "[CONFIG]".cyan());
    println!("  Key config files:");
    for cfg in &lens.configs {
        let status = if cfg.exists {
            "[present]".green().to_string()
        } else {
            "[missing]".dimmed().to_string()
        };
        println!("    {:<50} {}", cfg.path, status);
    }
    println!();

    // [CONFIG GRAPH]
    println!("{}", "[CONFIG GRAPH]".cyan());
    println!("  Precedence hints:");
    for hint in &lens.precedence_hints {
        println!("    {}", hint);
    }
    println!();

    // [TELEMETRY]
    if !lens.telemetry.is_empty() {
        println!("{}", "[TELEMETRY]".cyan());
        println!("  Service CPU and memory (avg last 24h):");
        for (name, tel) in &lens.telemetry {
            if tel.cpu_avg_24h > 0.0 || tel.memory_rss_avg_24h > 0 {
                println!("    {:<20} cpu {} percent, rss {} MiB",
                    format!("{}:", name),
                    tel.cpu_avg_24h as u32,
                    tel.memory_rss_avg_24h / (1024 * 1024)
                );
            }
        }
        println!();
    }

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Patterns (current boot, warning and above):");
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            // v7.29.0: No truncation - show full message
            println!(
                "    [{}] {} ({}x)",
                id,
                msg,
                count
            );
        }
        println!();
    }

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Display software lens
async fn run_display_sw_lens() -> Result<()> {
    println!();
    println!("{}", "  Anna SW: display".bold());
    println!("{}", THIN_SEP);
    println!();

    let lens = DisplaySwLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let active_services: Vec<_> = lens.services.iter()
        .filter(|s| s.active)
        .map(|s| s.name.as_str())
        .collect();
    println!("  Category:    display");
    println!("  Components:  {}", if active_services.is_empty() {
        "none detected".to_string()
    } else {
        active_services.join(", ")
    });
    println!();

    // [TOPOLOGY]
    if !lens.services.is_empty() {
        println!("{}", "[TOPOLOGY]".cyan());
        println!("  Display stack:");
        for svc in &lens.services {
            let status = if svc.active {
                "[running]".green().to_string()
            } else {
                format!("[{}]", svc.status).dimmed().to_string()
            };
            println!("    {:<40} {}", svc.unit, status);
        }
        println!();
    }

    // [CONFIG]
    if !lens.configs.is_empty() {
        println!("{}", "[CONFIG]".cyan());
        println!("  Config files:");
        for cfg in &lens.configs {
            if cfg.exists {
                println!("    {} {}", cfg.path, "[present]".green());
            }
        }
        println!();
    }

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Patterns (current boot, warning and above):");
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            // v7.29.0: No truncation - show full message
            println!(
                "    [{}] {} ({}x)",
                id, msg, count
            );
        }
        println!();
    }

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Audio software lens
async fn run_audio_sw_lens() -> Result<()> {
    println!();
    println!("{}", "  Anna SW: audio".bold());
    println!("{}", THIN_SEP);
    println!();

    let lens = AudioSwLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let active_services: Vec<_> = lens.services.iter()
        .filter(|s| s.active)
        .map(|s| s.name.as_str())
        .collect();
    println!("  Category:    audio");
    println!("  Components:  {}", if active_services.is_empty() {
        "none detected".to_string()
    } else {
        active_services.join(", ")
    });
    println!();

    // [TOPOLOGY]
    if !lens.services.is_empty() {
        println!("{}", "[TOPOLOGY]".cyan());
        println!("  Audio stack:");
        for svc in &lens.services {
            let status = if svc.active {
                "[running]".green().to_string()
            } else {
                format!("[{}]", svc.status).dimmed().to_string()
            };
            println!("    {:<40} {}", svc.unit, status);
        }
        println!();
    }

    // [CONFIG]
    println!("{}", "[CONFIG]".cyan());
    println!("  Config files:");
    for cfg in &lens.configs {
        let status = if cfg.exists {
            "[present]".green().to_string()
        } else {
            "[missing]".dimmed().to_string()
        };
        println!("    {:<50} {}", cfg.path, status);
    }
    println!();

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Patterns (current boot, warning and above):");
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            // v7.29.0: No truncation - show full message
            println!(
                "    [{}] {} ({}x)",
                id, msg, count
            );
        }
        println!();
    }

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Power software lens
async fn run_power_sw_lens() -> Result<()> {
    println!();
    println!("{}", "  Anna SW: power".bold());
    println!("{}", THIN_SEP);
    println!();

    let lens = PowerSwLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let active_services: Vec<_> = lens.services.iter()
        .filter(|s| s.active)
        .map(|s| s.name.as_str())
        .collect();
    println!("  Category:    power");
    println!("  Components:  {}", if active_services.is_empty() {
        "none detected".to_string()
    } else {
        active_services.join(", ")
    });
    println!();

    // [TOPOLOGY]
    if !lens.services.is_empty() {
        println!("{}", "[TOPOLOGY]".cyan());
        println!("  Power management:");
        for svc in &lens.services {
            let status = if svc.active {
                "[running]".green().to_string()
            } else {
                format!("[{}]", svc.status).dimmed().to_string()
            };
            println!("    {:<40} {}", svc.unit, status);
        }
        println!();
    }

    // [CONFIG]
    println!("{}", "[CONFIG]".cyan());
    println!("  Config files:");
    for cfg in &lens.configs {
        let status = if cfg.exists {
            "[present]".green().to_string()
        } else {
            "[missing]".dimmed().to_string()
        };
        println!("    {:<50} {}", cfg.path, status);
    }
    println!();

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Patterns (current boot, warning and above):");
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            // v7.29.0: No truncation - show full message
            println!(
                "    [{}] {} ({}x)",
                id, msg, count
            );
        }
        println!();
    }

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

    // [RELATIONSHIPS] - v7.24.0: services, processes, hardware
    print_relationships_section(&pkg.name);

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

    // [RELATIONSHIPS] - v7.24.0: services, processes, hardware
    print_relationships_section(base_name);

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
    // v7.29.0: No truncation - show full patterns
    for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
        let time_hint = format_time_short(&pattern.last_seen);

        let count_str = if pattern.count == 1 {
            "1x".to_string()
        } else {
            format!("{}x", pattern.count)
        };

        println!("    {}) \"{}\"", i + 1, pattern.pattern);
        println!("       ({}, last at {})",
                 count_str.dimmed(),
                 time_hint);
    }

    // Show if there are more patterns
    if summary.pattern_count > 3 {
        println!();
        println!("    (and {} more patterns)",
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

/// Print [CONFIG] section - v7.28.0 format with ConfigAtlas
/// v7.28.0: Three subsections: DETECTED, COMMON LOCATIONS, PRECEDENCE
fn print_config_section(name: &str) {
    let atlas = build_config_atlas(name);

    // v7.28.0: Collect only existing configs for "DETECTED"
    let detected: Vec<_> = atlas.existing_configs.iter()
        .filter(|c| c.status == ConfigStatus::Present)
        .collect();

    // v7.28.0: Collect documented locations (from man, pacman) for "COMMON LOCATIONS"
    let common_locations: Vec<_> = atlas.recommended_defaults.iter()
        .filter(|path| {
            // Only include if not already in detected
            !detected.iter().any(|c| &c.path == *path)
        })
        .collect();

    // v7.28.0: Precedence order (existing files only)
    let precedence: Vec<_> = atlas.precedence.iter()
        .filter(|e| e.status == ConfigStatus::Present)
        .collect();

    // Skip entire section if nothing to show
    if detected.is_empty() && common_locations.is_empty() && precedence.is_empty() {
        return;
    }

    // v7.28.0: [CONFIG - DETECTED] - only paths that exist on this machine
    if !detected.is_empty() {
        println!("{}", "[CONFIG - DETECTED]".cyan());
        if !atlas.sources.is_empty() {
            println!("  {}", format!("(sources: {})", atlas.sources.join(", ")).dimmed());
        }
        for cfg in &detected {
            let category = format!("({})", cfg.category.label());
            println!("  {}  {}", cfg.path, category.dimmed());
        }
        println!();
    }

    // v7.28.0: [CONFIG - COMMON LOCATIONS] - from man pages or package files
    if !common_locations.is_empty() {
        println!("{}", "[CONFIG - COMMON LOCATIONS]".cyan());
        println!("  {}", "(from man pages, package files)".dimmed());
        for path in &common_locations {
            println!("  {}", path);
        }
        println!();
    }

    // v7.28.0: [CONFIG - PRECEDENCE] - load order for existing configs
    if precedence.len() > 1 {
        println!("{}", "[CONFIG - PRECEDENCE]".cyan());
        println!("  {}", "(first match wins)".dimmed());
        for (i, entry) in precedence.iter().enumerate() {
            let rank = format!("{}.", i + 1);
            println!("  {:<3} {}", rank, entry.path);
        }
        println!();
    }
}

/// Format config file age from mtime
fn format_config_age(mtime: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if mtime == 0 || mtime > now {
        return "unknown".to_string();
    }

    let age_secs = now - mtime;
    if age_secs < 3600 {
        format!("{}m ago", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{}h ago", age_secs / 3600)
    } else if age_secs < 604800 {
        format!("{}d ago", age_secs / 86400)
    } else {
        format!("{}w ago", age_secs / 604800)
    }
}

/// Print [CONFIG GRAPH] section - v7.28.0
/// Shows config relationships (reads, shared) - precedence moved to CONFIG - PRECEDENCE
fn print_config_graph_section(name: &str) {
    let graph = get_config_graph_for_software(name);

    // Skip if no graph data
    if graph.reads.is_empty() && graph.shared.is_empty() {
        return;
    }

    println!("{}", "[CONFIG GRAPH]".cyan());

    // v7.28.0: Only show existing config reads
    let existing_reads: Vec<_> = graph.reads.iter().filter(|c| c.exists).collect();
    if !existing_reads.is_empty() {
        println!("  Reads:");
        for cfg in existing_reads {
            println!("    {}  {}", cfg.path, format!("({})", cfg.evidence).dimmed());
        }
    }

    // v7.28.0: Only show existing shared configs
    let existing_shared: Vec<_> = graph.shared.iter().filter(|c| c.exists).collect();
    if !existing_shared.is_empty() {
        println!("  Shared:");
        for cfg in existing_shared {
            println!("    {}  {}", cfg.path, format!("({})", cfg.evidence).dimmed());
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

    // Package events - v7.29.0: Show all events, no truncation
    if !pkg_history.is_empty() {
        println!("  Package:");
        for event in pkg_history.iter() {
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
        // v7.29.0: Show all events (no truncation)
    }

    // Config changes - v7.29.0: Show all changes, no truncation
    if !config_history.is_empty() {
        println!("  Config:");
        for event in config_history.iter() {
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
        // v7.29.0: Show all config changes (no truncation)
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

    // Print sanity notes section - v7.29.0: Show all notes, no truncation
    if sanity_notes.is_empty() {
        println!("  Sanity notes:");
        println!("    - No obvious issues detected with primary config paths.");
    } else {
        println!("  Sanity notes:");
        for note in &sanity_notes {
            println!("    - {}", note.yellow());
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

/// Print [RELATIONSHIPS] section - v7.24.0: services, processes, hardware touched
fn print_relationships_section(name: &str) {
    let rels = get_software_relationships(name);
    let lines = format_software_relationships_section(&rels);

    for line in lines {
        if line.starts_with("[RELATIONSHIPS]") {
            println!("{}", line.cyan());
        } else {
            println!("{}", line);
        }
    }
    println!();
}

/// Print [USAGE] section with v7.23.0 format: Time-anchored trends with percentage+range
fn print_telemetry_section(name: &str) {
    use anna_common::config::AnnaConfig;
    use anna_common::{get_usage_trends, format_cpu_percent_with_range,
                      timeline_format_memory, TrendLabel};

    println!("{}", "[USAGE]".cyan());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  Telemetry: disabled in config");
        println!();
        return;
    }

    // Get time-anchored usage trends
    let trends = get_usage_trends(name);

    println!("  Source: {}", trends.source.dimmed());
    println!();

    if !trends.has_any_data() {
        println!("  Telemetry: not collected yet");
        println!();
        return;
    }

    // CPU section with percentage+range format
    println!("  CPU avg:");
    if let Some(ref w) = trends.cpu_1h {
        if w.is_valid() {
            println!("    last 1h:    {}",
                     format_cpu_percent_with_range(w.avg, trends.logical_cores));
        }
    }
    if let Some(ref w) = trends.cpu_24h {
        if w.is_valid() {
            println!("    last 24h:   {}",
                     format_cpu_percent_with_range(w.avg, trends.logical_cores));
        }
    } else if trends.cpu_1h.is_some() {
        println!("    last 24h:   {}", "n/a (insufficient data)".dimmed());
    }
    if let Some(ref w) = trends.cpu_7d {
        if w.is_valid() {
            println!("    last 7d:    {}",
                     format_cpu_percent_with_range(w.avg, trends.logical_cores));
        }
    } else if trends.cpu_24h.is_some() {
        println!("    last 7d:    {}", "n/a (insufficient data)".dimmed());
    }
    // CPU trend
    let cpu_trend_str = match &trends.cpu_trend {
        TrendLabel::Stable { delta } => format!("stable ({})", delta).green().to_string(),
        TrendLabel::Rising { delta } => format!("rising ({})", delta).yellow().to_string(),
        TrendLabel::Falling { delta } => format!("falling ({})", delta).cyan().to_string(),
        TrendLabel::InsufficientData => "n/a (insufficient data)".dimmed().to_string(),
    };
    println!("    trend:      {}", cpu_trend_str);
    println!();

    // Memory RSS section
    println!("  Memory RSS avg:");
    if let Some(ref w) = trends.mem_1h {
        if w.is_valid() {
            println!("    last 1h:    {}", timeline_format_memory(w.avg as u64));
        }
    }
    if let Some(ref w) = trends.mem_24h {
        if w.is_valid() {
            println!("    last 24h:   {}", timeline_format_memory(w.avg as u64));
        }
    } else if trends.mem_1h.is_some() {
        println!("    last 24h:   {}", "n/a (insufficient data)".dimmed());
    }
    if let Some(ref w) = trends.mem_7d {
        if w.is_valid() {
            println!("    last 7d:    {}", timeline_format_memory(w.avg as u64));
        }
    } else if trends.mem_24h.is_some() {
        println!("    last 7d:    {}", "n/a (insufficient data)".dimmed());
    }
    // Memory trend
    let mem_trend_str = match &trends.mem_trend {
        TrendLabel::Stable { delta } => format!("stable ({})", delta).green().to_string(),
        TrendLabel::Rising { delta } => format!("rising ({})", delta).yellow().to_string(),
        TrendLabel::Falling { delta } => format!("falling ({})", delta).cyan().to_string(),
        TrendLabel::InsufficientData => "n/a (insufficient data)".dimmed().to_string(),
    };
    println!("    trend:      {}", mem_trend_str);
    println!();

    // Starts section (if any)
    if trends.starts_24h > 0 || trends.starts_7d > 0 || trends.starts_30d > 0 {
        println!("  Starts:");
        println!("    last 24h:   {}", trends.starts_24h);
        println!("    last 7d:    {}", trends.starts_7d);
        println!("    last 30d:   {}", trends.starts_30d);
        println!();
    }
}

/// Format trend label with color - v7.20.0
fn format_trend_label(label: &str, is_increasing: bool) -> String {
    use owo_colors::OwoColorize;
    match label {
        "stable" => label.green().to_string(),
        "slightly higher" | "slightly lower" => label.to_string(),
        "higher" | "lower" => label.yellow().to_string(),
        "much higher" | "much lower" => {
            if is_increasing {
                label.red().to_string()
            } else {
                label.yellow().to_string()
            }
        }
        _ => label.dimmed().to_string(),
    }
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

    // v7.19.0: Cross-reference to related hardware
    let related_hw = get_service_related_hardware(base_name);
    if !related_hw.is_empty() {
        println!();
        println!("  Related hardware:");
        for hw in &related_hw {
            println!("    → See: annactl hw {}", hw.dimmed());
        }
    }

    println!();
}

/// Get related hardware for a service - v7.19.0
fn get_service_related_hardware(service_name: &str) -> Vec<&'static str> {
    let name_lower = service_name.to_lowercase();
    let mut related = Vec::new();

    // Network services -> network hardware
    if name_lower.contains("networkmanager") || name_lower.contains("network")
        || name_lower.contains("wpa_supplicant") {
        related.push("wifi");
        related.push("ethernet");
    }

    // Bluetooth services -> bluetooth hardware
    if name_lower.contains("bluetooth") || name_lower.contains("bluez") {
        related.push("bluetooth");
    }

    // Audio services -> audio hardware
    if name_lower.contains("pipewire") || name_lower.contains("pulseaudio")
        || name_lower.contains("audio") {
        related.push("audio");
    }

    // Power services -> power hardware
    if name_lower.contains("upower") || name_lower.contains("power") {
        related.push("power");
    }

    // GPU services -> gpu hardware
    if name_lower.contains("nvidia") || name_lower.contains("gpu") {
        related.push("gpu");
    }

    related
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
    // v7.29.0: No truncation - show full patterns
    if !summary.patterns.is_empty() {
        println!("  Top patterns:");
        for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
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

            // v7.29.0: No truncation - show full pattern
            println!("    {}) \"{}\"", i + 1, pattern.pattern);
            println!("       {} ({})", pattern.priority.dimmed(), history_str.dimmed());
        }

        if summary.patterns.len() > 3 {
            println!();
            println!("    (and {} more patterns)",
                     summary.patterns.len() - 3);
        }
    }

    // v7.16.0: Show patterns with history beyond this boot
    // v7.29.0: No truncation - show full patterns
    let history_patterns = summary.patterns_with_history();
    if !history_patterns.is_empty() {
        println!();
        println!("  Recurring patterns (seen in previous boots):");
        for pattern in history_patterns.iter().take(2) {
            // v7.29.0: Show full pattern without truncation
            println!("    - \"{}\" ({} boots, {} total in 7d)",
                     pattern.pattern.dimmed(),
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

/// Print [LOGS] section with v7.20.0 boot-anchored patterns and baseline tags
fn print_service_logs_v718(unit_name: &str) -> LogHistorySummary {
    use anna_common::log_patterns_enhanced::LogPatternAnalyzer;
    use anna_common::{find_or_create_service_baseline, tag_pattern, normalize_message};

    println!("{}", "[LOGS]".cyan());

    // First get the basic history summary for compatibility
    let summary = extract_patterns_with_history(unit_name);

    // Get enhanced patterns with novelty detection
    let service_name = unit_name.trim_end_matches(".service");
    let analyzer = LogPatternAnalyzer::new();
    let pattern_summary = analyzer.get_patterns_for_service(service_name);

    // v7.20.0: Try to get or create baseline for this service
    let baseline = find_or_create_service_baseline(unit_name, 5);

    if summary.is_empty_this_boot() && pattern_summary.current_boot.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component.");
        if baseline.is_some() {
            println!("  {}", "(baseline established)".dimmed());
        }
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    println!();

    // v7.20.0: Boot-anchored view with baseline info
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

    // v7.28.0: Show new patterns with dedupe count - no truncation, up to 20 unique
    if !pattern_summary.new_this_boot.is_empty() {
        println!("  {} (first seen this boot):", "New patterns".yellow());
        for occurrence in pattern_summary.new_this_boot.iter().take(20) {
            // v7.27.0: No truncation per spec
            let display = &occurrence.pattern.template;

            // v7.20.0: Get baseline tag for this pattern
            let normalized = normalize_message(&occurrence.pattern.template);
            let baseline_tag = tag_pattern(unit_name, &normalized);

            // v7.27.0: Format "(seen N times this boot)" per spec
            let seen_str = if occurrence.count_this_boot == 1 {
                "(seen 1 time this boot)".to_string()
            } else {
                format!("(seen {} times this boot)", occurrence.count_this_boot)
            };

            println!("    [{}] \"{}\" {}",
                     occurrence.pattern.short_id().yellow(),
                     display,
                     baseline_tag.format().yellow());
            println!("           {} {}",
                     occurrence.pattern.priority.dimmed(),
                     seen_str.dimmed());
        }
        if pattern_summary.new_this_boot.len() > 20 {
            // v7.29.0: No ellipsis
            println!("    (and {} more new patterns)",
                     pattern_summary.new_this_boot.len() - 20);
        }
        println!();
    }

    // v7.28.0: Show known patterns with dedupe - no truncation, up to 20 unique
    if !pattern_summary.known_patterns.is_empty() {
        println!("  Known patterns:");
        for occurrence in pattern_summary.known_patterns.iter().take(20) {
            // v7.27.0: No truncation per spec
            let display = &occurrence.pattern.template;

            // v7.20.0: Get baseline tag for this pattern
            let normalized = normalize_message(&occurrence.pattern.template);
            let baseline_tag = tag_pattern(unit_name, &normalized);
            let tag_str = baseline_tag.format();

            // v7.27.0: Format "(seen N times this boot)" per spec
            let seen_str = if occurrence.count_this_boot == 1 {
                format!("(seen 1 time this boot, {} in 7d, {} boots)", occurrence.count_7d, occurrence.boots_seen)
            } else {
                format!("(seen {} times this boot, {} in 7d, {} boots)", occurrence.count_this_boot, occurrence.count_7d, occurrence.boots_seen)
            };

            if tag_str.is_empty() {
                println!("    [{}] \"{}\"",
                         occurrence.pattern.short_id().dimmed(),
                         display);
            } else {
                println!("    [{}] \"{}\" {}",
                         occurrence.pattern.short_id().dimmed(),
                         display,
                         tag_str.dimmed());
            }
            println!("           {} {}",
                     occurrence.pattern.priority.dimmed(),
                     seen_str.dimmed());
        }
        if pattern_summary.known_patterns.len() > 20 {
            // v7.29.0: No ellipsis
            println!("    (and {} more known patterns)",
                     pattern_summary.known_patterns.len() - 20);
        }
        println!();
    }

    // v7.20.0: Baseline info
    if let Some(ref bl) = baseline {
        println!("  Baseline:");
        println!("    Boot: -{}, {} known warning patterns",
                 bl.boot_id.abs(), bl.warning_count);
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

