//! KDB Command v7.0.0 - Knowledge Database Overview
//!
//! Sections:
//! - [OVERVIEW]          Counts of packages, commands, services
//! - [CATEGORIES]        Grouped by category with member names
//! - [USAGE HIGHLIGHTS]  (when telemetry exists) Most used, never used, etc.
//! - [WARNINGS]          KDB-related issues only
//!
//! NO journalctl system errors. NO generic host health.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
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

/// Run the kdb overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge Database".bold());
    println!("{}", THIN_SEP);
    println!();

    // [OVERVIEW]
    print_overview_section();

    // [CATEGORIES]
    print_categories_section();

    // [USAGE HIGHLIGHTS]
    print_usage_section();

    // [WARNINGS] - only if there are any
    print_warnings_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview_section() {
    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages known:   {}", pkg_counts.total);
    println!("  Commands known:   {}", cmd_count);
    println!("  Services known:   {}", svc_counts.total);

    println!();
}

fn print_categories_section() {
    println!("{}", "[CATEGORIES]".cyan());

    use anna_common::grounded::commands::command_exists;

    // Editors
    let editors: Vec<&str> = EDITORS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !editors.is_empty() {
        println!("  Editors:          {}", editors.join(", "));
    }

    // Terminals
    let terminals: Vec<&str> = TERMINALS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !terminals.is_empty() {
        println!("  Terminals:        {}", terminals.join(", "));
    }

    // Shells
    let shells: Vec<&str> = SHELLS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !shells.is_empty() {
        println!("  Shells:           {}", shells.join(", "));
    }

    // Compositors
    let compositors: Vec<&str> = COMPOSITORS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !compositors.is_empty() {
        println!("  Compositors:      {}", compositors.join(", "));
    }

    // Browsers
    let browsers: Vec<&str> = BROWSERS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !browsers.is_empty() {
        println!("  Browsers:         {}", browsers.join(", "));
    }

    // Tools - show first 10 with "..." if more
    let tools: Vec<&str> = TOOLS.iter().filter(|&&cmd| command_exists(cmd)).copied().collect();
    if !tools.is_empty() {
        if tools.len() <= 10 {
            println!("  Tools:            {}", tools.join(", "));
        } else {
            let display: Vec<&str> = tools.iter().take(10).copied().collect();
            println!("  Tools:            {}, ...", display.join(", "));
        }
    }

    println!();
}

fn print_usage_section() {
    println!("{}", "[USAGE HIGHLIGHTS]".cyan());

    // v7.0.0: Telemetry not yet implemented
    // When implemented, show:
    // - Most used commands (7d)
    // - Never used commands
    // - Most CPU hungry (7d)
    // - Most RAM hungry (7d)

    println!("  {}", "Usage telemetry: not collected yet".dimmed());

    println!();
}

fn print_warnings_section() {
    // Only show if there are actual KDB-related warnings
    // For now, we don't have any real warnings to show
    // Future: check for missing pacman data, systemctl issues, etc.

    // Example of what this would look like:
    // println!("{}", "[WARNINGS]".cyan());
    // println!("  Missing pacman data for 3 commands in PATH.");
    // println!();
}
