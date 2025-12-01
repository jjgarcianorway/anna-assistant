//! KDB Detail Command v7.1.0 - Object Profiles and Category Overviews
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
};

const THIN_SEP: &str = "------------------------------------------------------------";

// Category detection patterns
const EDITORS: &[&str] = &["vim", "nvim", "neovim", "nano", "emacs", "helix", "hx", "kate", "gedit", "code"];
const TERMINALS: &[&str] = &["alacritty", "kitty", "foot", "wezterm", "gnome-terminal", "konsole", "st", "xterm"];
const SHELLS: &[&str] = &["bash", "zsh", "fish", "nushell", "dash", "sh"];
const COMPOSITORS: &[&str] = &["hyprland", "sway", "wayfire", "river", "gnome-shell", "plasmashell", "i3", "bspwm"];
const BROWSERS: &[&str] = &["firefox", "chromium", "brave", "vivaldi", "qutebrowser", "librewolf", "google-chrome-stable"];
const TOOLS: &[&str] = &[
    "git", "curl", "wget", "grep", "awk", "sed", "tar", "gzip", "unzip",
    "htop", "btop", "fastfetch", "neofetch", "ffmpeg", "jq", "fzf", "ripgrep", "rg",
    "make", "cmake", "gcc", "clang", "rustc", "python", "node", "docker", "podman"
];

// ============================================================================
// Category Overview
// ============================================================================

/// Run category overview (e.g., `annactl kdb editors`)
pub async fn run_category(category: &str) -> Result<()> {
    let category_lower = category.to_lowercase();

    let (category_name, patterns): (&str, &[&str]) = match category_lower.as_str() {
        "editors" | "editor" => ("Editors", EDITORS),
        "terminals" | "terminal" => ("Terminals", TERMINALS),
        "shells" | "shell" => ("Shells", SHELLS),
        "compositors" | "compositor" => ("Compositors", COMPOSITORS),
        "browsers" | "browser" => ("Browsers", BROWSERS),
        "tools" | "tool" => ("Tools", TOOLS),
        "services" | "service" => {
            return run_services_category().await;
        }
        _ => {
            eprintln!();
            eprintln!("  {} Unknown category: '{}'", "error:".red(), category);
            eprintln!();
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", format!("  Anna KDB: {}", category_name).bold());
    println!("{}", THIN_SEP);
    println!();

    // Find installed members
    let mut installed: Vec<(&str, String, String)> = Vec::new();
    for &name in patterns {
        if command_exists(name) {
            let desc = get_package_info(name)
                .map(|p| p.description.clone())
                .or_else(|| get_command_info(name).map(|c| c.description.clone()))
                .unwrap_or_default();

            let version = get_package_info(name)
                .map(|p| p.version.clone())
                .unwrap_or_default();

            installed.push((name, desc, version));
        }
    }

    if installed.is_empty() {
        println!("  No {} installed.", category_name.to_lowercase());
    } else {
        println!("  {} {} installed:", installed.len(), category_name.to_lowercase());
        println!();

        for (name, desc, version) in &installed {
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
        println!("  Description: {}", svc.description);
        println!("               {}", "(source: systemctl show)".dimmed());
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

/// Print service logs with deduplication
fn print_service_logs(unit_name: &str) {
    println!("{}", "[LOGS]".cyan());
    println!("  {}", format!("(journalctl -b -u {} -p warning..alert -n 10)", unit_name).dimmed());

    let output = Command::new("journalctl")
        .args([
            "-b",           // Current boot only
            "-u", unit_name,
            "-p", "warning..alert",
            "-n", "50",     // Get more for dedup, display 10
            "--no-pager",
            "-o", "short",
            "-q",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

            if lines.is_empty() {
                println!();
                println!("  {}", "(no warnings or errors this boot)".green());
            } else {
                // Deduplicate messages
                let deduped = deduplicate_logs(&lines);

                println!();
                for (line, count) in deduped.iter().take(10) {
                    if *count > 1 {
                        println!("  {} {}", line, format!("(seen {} times this boot)", count).dimmed());
                    } else {
                        println!("  {}", line);
                    }
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

/// Deduplicate log messages, preserving order of first occurrence
fn deduplicate_logs(lines: &[&str]) -> Vec<(String, usize)> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for line in lines {
        // Extract the message part (after timestamp and priority)
        // Format: "Dec 01 14:20:31 host unit[pid]: message"
        let key = extract_message_key(line);

        if let Some(count) = seen.get_mut(&key) {
            *count += 1;
        } else {
            seen.insert(key.clone(), 1);
            order.push(line.to_string());
        }
    }

    order.into_iter()
        .map(|line| {
            let key = extract_message_key(&line);
            let count = seen.get(&key).copied().unwrap_or(1);
            (line, count)
        })
        .collect()
}

/// Extract a key for deduplication (unit + message, ignoring timestamp)
fn extract_message_key(line: &str) -> String {
    // Try to skip the timestamp prefix
    // Format: "Dec 01 14:20:31 host unit[pid]: message"
    if let Some(pos) = line.find("]: ") {
        // Include unit name and message
        if let Some(unit_start) = line[..pos].rfind(' ') {
            return line[unit_start..].to_string();
        }
    }
    line.to_string()
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

/// Print [CONFIG] section with config files discovered from pacman/man
fn print_config_section(name: &str) {
    println!("{}", "[CONFIG]".cyan());

    let configs = discover_config_files(name);

    if configs.is_empty() {
        println!("  {}", "(no config files discovered)".dimmed());
    } else {
        // Group by source for cleaner output
        let mut by_source: HashMap<String, Vec<&anna_common::grounded::config::ConfigFile>> = HashMap::new();
        for cfg in &configs {
            by_source.entry(cfg.source.clone()).or_default().push(cfg);
        }

        for (source, files) in &by_source {
            println!("  {}", format!("(source: {})", source).dimmed());
            for cfg in files {
                if cfg.exists {
                    println!("  {}", cfg.path);
                } else {
                    println!("  {} {}", cfg.path, "(missing)".yellow());
                }
            }
        }
    }

    println!();
}

/// Print [USAGE] section with real telemetry from SQLite
fn print_usage_section(name: &str) {
    println!("{}", "[USAGE]".cyan());

    // Try to open telemetry database
    match TelemetryDb::open() {
        Ok(db) => {
            match db.get_object_telemetry(name) {
                Ok(telemetry) => {
                    if telemetry.total_samples == 0 {
                        println!("  {}", "(source: /var/lib/anna/telemetry.db)".dimmed());
                        println!("  {}", "(no telemetry recorded for this object)".dimmed());
                    } else {
                        println!("  {}", "(source: /var/lib/anna/telemetry.db)".dimmed());
                        println!("  Samples:     {}", telemetry.total_samples);

                        // Peak CPU
                        if telemetry.peak_cpu_percent > 0.0 {
                            println!("  Peak CPU:    {:.1}%", telemetry.peak_cpu_percent);
                        }

                        // Peak memory
                        if telemetry.peak_mem_bytes > 0 {
                            println!("  Peak memory: {}", format_size(telemetry.peak_mem_bytes));
                        }

                        // Average memory
                        if telemetry.avg_mem_bytes > 0 {
                            println!("  Avg memory:  {}", format_size(telemetry.avg_mem_bytes));
                        }

                        // Total CPU time
                        if telemetry.total_cpu_time_ms > 0 {
                            let cpu_secs = telemetry.total_cpu_time_ms / 1000;
                            let cpu_str = if cpu_secs < 60 {
                                format!("{}s", cpu_secs)
                            } else if cpu_secs < 3600 {
                                format!("{}m {}s", cpu_secs / 60, cpu_secs % 60)
                            } else {
                                format!("{}h {}m", cpu_secs / 3600, (cpu_secs % 3600) / 60)
                            };
                            println!("  CPU time:    {}", cpu_str);
                        }

                        // Coverage period
                        if telemetry.coverage_hours > 0.0 {
                            let coverage_str = if telemetry.coverage_hours < 1.0 {
                                format!("{:.0}m", telemetry.coverage_hours * 60.0)
                            } else if telemetry.coverage_hours < 24.0 {
                                format!("{:.1}h", telemetry.coverage_hours)
                            } else {
                                format!("{:.1}d", telemetry.coverage_hours / 24.0)
                            };
                            println!("  Observed:    {}", coverage_str);
                        }
                    }
                }
                Err(_) => {
                    println!("  {}", "(failed to query telemetry)".dimmed());
                }
            }
        }
        Err(_) => {
            println!("  {}", "(telemetry DB not available)".dimmed());
        }
    }

    println!();
}
