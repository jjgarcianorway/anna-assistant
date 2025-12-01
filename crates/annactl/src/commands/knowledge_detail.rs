//! Knowledge Detail Command v6.0.0 - Grounded Object Details
//!
//! v6.0.0: Complete rewrite with real data sources
//! - Package info from pacman -Qi
//! - Command info from which + man -f
//! - Service info from systemctl
//!
//! Every piece of information has a source attribution.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::{get_package_info, Package},
    commands::{get_command_info, command_exists, SystemCommand},
    services::{get_service_info, Service, ServiceState, EnabledState},
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge detail command for a specific object
pub async fn run(name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna Knowledge: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    // Try to find as package first
    if let Some(pkg) = get_package_info(name) {
        print_package_detail(&pkg);
    } else if let Some(cmd) = get_command_info(name) {
        // It's a command on PATH
        print_command_detail(&cmd);
    } else if let Some(svc) = get_service_info(name) {
        // It's a systemd service
        print_service_detail(&svc);
    } else {
        // Not found
        print_not_found(name);
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_package_detail(pkg: &Package) {
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", pkg.name.bold());
    if !pkg.description.is_empty() {
        println!("  Description: {}", pkg.description);
        println!("               {}", "(source: pacman -Qi)".dimmed());
    }

    println!();
    println!("{}", "[PACKAGE]".cyan());
    println!("  {}", "(source: pacman -Qi)".dimmed());
    println!("  Version:     {}", pkg.version);

    let source_str = match pkg.source {
        anna_common::grounded::packages::PackageSource::Official => "official",
        anna_common::grounded::packages::PackageSource::Aur => "AUR",
        anna_common::grounded::packages::PackageSource::Unknown => "unknown",
    };
    println!("  Source:      {}", source_str);

    let reason_str = match pkg.install_reason {
        anna_common::grounded::packages::InstallReason::Explicit => "explicit".green().to_string(),
        anna_common::grounded::packages::InstallReason::Dependency => "dependency".to_string(),
        anna_common::grounded::packages::InstallReason::Unknown => "unknown".to_string(),
    };
    println!("  Installed:   {}", reason_str);

    if pkg.installed_size > 0 {
        println!("  Size:        {}", format_size(pkg.installed_size));
    }

    if !pkg.install_date.is_empty() {
        println!("  Date:        {}", pkg.install_date);
    }

    // Show config files from package
    if !pkg.config_files.is_empty() {
        println!();
        println!("{}", "[CONFIG FILES]".cyan());
        println!("  {}", "(source: pacman -Ql | grep /etc/)".dimmed());
        for path in pkg.config_files.iter().take(5) {
            println!("  {}", path);
        }
        if pkg.config_files.len() > 5 {
            println!("  ({} more)", pkg.config_files.len() - 5);
        }
    }

    // Check if it provides a command
    if command_exists(&pkg.name) {
        if let Some(cmd) = get_command_info(&pkg.name) {
            println!();
            println!("{}", "[COMMAND]".cyan());
            println!("  {}", "(source: which)".dimmed());
            println!("  Path:        {}", cmd.path);
            if !cmd.description.is_empty() {
                println!("  Man:         {}", cmd.description);
            }
        }
    }

    println!();
}

fn print_command_detail(cmd: &SystemCommand) {
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", cmd.name.bold());
    if !cmd.description.is_empty() {
        println!("  Description: {}", cmd.description);
    }

    println!();
    println!("{}", "[COMMAND]".cyan());
    println!("  {}", "(source: which, man -f)".dimmed());
    println!("  Path:        {}", cmd.path);

    // Show owning package if known
    if let Some(pkg_name) = &cmd.owning_package {
        println!("  Package:     {}", pkg_name);

        // Get more package details
        if let Some(pkg) = get_package_info(pkg_name) {
            println!("  Version:     {}", pkg.version);
        }
    }

    println!();
}

fn print_service_detail(svc: &Service) {
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", svc.name.bold());
    if !svc.description.is_empty() {
        println!("  Description: {}", svc.description);
    }

    println!();
    println!("{}", "[SERVICE]".cyan());
    println!("  {}", "(source: systemctl)".dimmed());

    // State
    let state_str = match svc.state {
        ServiceState::Active => "running".green().to_string(),
        ServiceState::Inactive => "inactive".dimmed().to_string(),
        ServiceState::Failed => "failed".red().to_string(),
        ServiceState::Unknown => "unknown".to_string(),
    };
    println!("  State:       {}", state_str);

    // Enabled
    let enabled_str = match svc.enabled {
        EnabledState::Enabled => "enabled".green().to_string(),
        EnabledState::Disabled => "disabled".dimmed().to_string(),
        EnabledState::Static => "static".to_string(),
        EnabledState::Masked => "masked".yellow().to_string(),
        EnabledState::Unknown => "unknown".to_string(),
    };
    println!("  Enabled:     {}", enabled_str);

    // Check if there's a package for this service
    let base_name = svc.name.trim_end_matches(".service");
    if let Some(pkg) = get_package_info(base_name) {
        println!();
        println!("{}", "[PACKAGE]".cyan());
        println!("  {}", "(source: pacman -Qi)".dimmed());
        println!("  Name:        {}", pkg.name);
        println!("  Version:     {}", pkg.version);
    }

    println!();
}

fn print_not_found(name: &str) {
    println!("{}", "[STATUS]".cyan());
    println!("  Not found on this system");
    println!();

    // Give hints about what was checked
    println!("  {}", "Checked:".dimmed());
    println!("    - pacman -Qi {} (not installed)", name);
    println!("    - which {} (not in PATH)", name);
    println!("    - systemctl cat {}.service (not a service)", name);

    println!();
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
