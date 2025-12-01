//! Version Command v5.4.0 - Installation and Version Info
//!
//! Shows:
//! - annactl and annad versions
//! - Install path summary (binaries, config, data, logs)
//! - Anna version history (current <- previous)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the version command
pub fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Version".bold());
    println!("{}", THIN_SEP);
    println!();

    // [VERSION]
    print_version_section();

    // [INSTALL PATHS]
    print_paths_section();

    // [VERSION HISTORY] - from pacman.log if available
    print_history_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());

    // Get annactl version (this binary)
    let annactl_version = VERSION;

    // Try to get annad version from the binary
    let annad_version = get_annad_version().unwrap_or_else(|| VERSION.to_string());

    println!("  annactl:  v{}", annactl_version);
    println!("  annad:    v{}", annad_version);
    println!();
}

fn print_paths_section() {
    println!("{}", "[INSTALL PATHS]".cyan());

    // Binaries
    let annactl_path = which_binary("annactl");
    let annad_path = which_binary("annad");

    println!("  Binaries:");
    println!("    annactl:  {}", format_path_status(&annactl_path));
    println!("    annad:    {}", format_path_status(&annad_path));

    // Config
    let config_path = "/etc/anna/config.toml";
    let config_exists = Path::new(config_path).exists();
    println!("  Config:");
    println!("    {}  {}",
        if config_exists { "✓".green().to_string() } else { "✗".red().to_string() },
        config_path
    );

    // Data
    let data_path = "/var/lib/anna";
    let data_exists = Path::new(data_path).exists();
    println!("  Data:");
    println!("    {}  {}",
        if data_exists { "✓".green().to_string() } else { "✗".red().to_string() },
        data_path
    );

    // Logs (systemd journal)
    println!("  Logs:");
    println!("    ✓  journalctl -u annad");

    println!();
}

fn print_history_section() {
    // Try to get version history from pacman.log
    if let Some(history) = get_pacman_version_history("anna") {
        if !history.is_empty() {
            println!("{}", "[VERSION HISTORY]".cyan());

            // Show current <- previous format
            if history.len() >= 2 {
                println!("  {} <- {}", history[0].green(), history[1].dimmed());
            } else if history.len() == 1 {
                println!("  {} (first install)", history[0].green());
            }

            println!();
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_annad_version() -> Option<String> {
    // Try to run annad --version
    let output = Command::new("annad")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "annad X.Y.Z" or just "X.Y.Z"
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("annad ") {
                return Some(trimmed.strip_prefix("annad ")?.trim().to_string());
            }
            // Try to parse as version directly
            if trimmed.chars().next()?.is_ascii_digit() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

fn which_binary(name: &str) -> Option<String> {
    let output = Command::new("which")
        .arg(name)
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }

    None
}

fn format_path_status(path: &Option<String>) -> String {
    match path {
        Some(p) => format!("{}  {}", "✓".green(), p),
        None => format!("{}  not found", "✗".red()),
    }
}

/// Get version history from pacman.log
/// Returns list of versions, newest first
fn get_pacman_version_history(pkg_prefix: &str) -> Option<Vec<String>> {
    let log_path = "/var/log/pacman.log";
    if !Path::new(log_path).exists() {
        return None;
    }

    // Use grep to find anna-related entries
    let output = Command::new("grep")
        .args(["-E", &format!(r"(upgraded|installed) {}(ctl|d)", pkg_prefix), log_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut versions: Vec<String> = Vec::new();

    for line in stdout.lines().rev() {
        // Parse lines like:
        // [2024-01-15 10:30] [ALPM] upgraded annactl (5.2.0 -> 5.3.0)
        // [2024-01-15 10:30] [ALPM] installed annactl (5.3.0)
        if let Some(ver) = extract_version_from_pacman_line(line) {
            if versions.is_empty() || versions.last() != Some(&ver) {
                versions.push(ver);
            }
            if versions.len() >= 2 {
                break;
            }
        }
    }

    if versions.is_empty() {
        None
    } else {
        Some(versions)
    }
}

fn extract_version_from_pacman_line(line: &str) -> Option<String> {
    // Look for patterns like "(X.Y.Z)" at end or "-> X.Y.Z)"
    if let Some(arrow_pos) = line.find(" -> ") {
        // Upgraded: extract new version after ->
        let after_arrow = &line[arrow_pos + 4..];
        if let Some(paren_pos) = after_arrow.find(')') {
            return Some(after_arrow[..paren_pos].to_string());
        }
    } else if let Some(start) = line.rfind('(') {
        // Installed: extract version in parentheses
        if let Some(end) = line.rfind(')') {
            if end > start {
                return Some(line[start + 1..end].to_string());
            }
        }
    }
    None
}
