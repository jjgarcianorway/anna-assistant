//! Knowledge Command v6.0.0 - Grounded System Knowledge
//!
//! v6.0.0: Complete rewrite with real data sources
//! - Packages from pacman -Q
//! - Commands from $PATH
//! - Services from systemctl
//! - No invented categories, no fake metrics
//!
//! Shows what's ACTUALLY on the system, not what Anna "thinks" is there.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Known categories with their package detection patterns
const EDITORS: &[&str] = &["vim", "neovim", "nvim", "nano", "emacs", "helix", "hx", "kate", "gedit", "code"];
const TERMINALS: &[&str] = &["alacritty", "kitty", "foot", "wezterm", "gnome-terminal", "konsole", "st"];
const SHELLS: &[&str] = &["bash", "zsh", "fish", "nushell"];
const COMPOSITORS: &[&str] = &["hyprland", "sway", "wayfire", "river", "picom"];
const BROWSERS: &[&str] = &["firefox", "chromium", "brave", "vivaldi", "qutebrowser", "librewolf"];

/// Run the knowledge overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge".bold());
    println!("{}", THIN_SEP);
    println!();

    // [OVERVIEW] - real counts from real sources
    print_overview_section();

    // [BY CATEGORY] - only show what's actually installed
    print_categories_section();

    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl knowledge <name>' for details on a specific package.");
    println!();

    Ok(())
}

fn print_overview_section() {
    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages:   {} (source: pacman -Q)", pkg_counts.total);
    println!("  Commands:   {} (source: $PATH)", cmd_count);
    println!("  Services:   {} (source: systemctl)", svc_counts.total);

    println!();
}

fn print_categories_section() {
    println!("{}", "[BY CATEGORY]".cyan());
    println!("  {}", "(detected from installed packages)".dimmed());

    // Check which packages are installed and categorize them
    let installed = get_installed_by_category();

    if !installed.editors.is_empty() {
        println!(
            "  Editors:    {} ({})",
            installed.editors.len(),
            installed.editors.join(", ")
        );
    }

    if !installed.terminals.is_empty() {
        println!(
            "  Terminals:  {} ({})",
            installed.terminals.len(),
            installed.terminals.join(", ")
        );
    }

    if !installed.shells.is_empty() {
        println!(
            "  Shells:     {} ({})",
            installed.shells.len(),
            installed.shells.join(", ")
        );
    }

    if !installed.compositors.is_empty() {
        println!(
            "  Compositors: {} ({})",
            installed.compositors.len(),
            installed.compositors.join(", ")
        );
    }

    if !installed.browsers.is_empty() {
        println!(
            "  Browsers:   {} ({})",
            installed.browsers.len(),
            installed.browsers.join(", ")
        );
    }

    println!();
}

struct InstalledByCategory {
    editors: Vec<String>,
    terminals: Vec<String>,
    shells: Vec<String>,
    compositors: Vec<String>,
    browsers: Vec<String>,
}

fn get_installed_by_category() -> InstalledByCategory {
    use anna_common::grounded::commands::command_exists;

    let mut result = InstalledByCategory {
        editors: Vec::new(),
        terminals: Vec::new(),
        shells: Vec::new(),
        compositors: Vec::new(),
        browsers: Vec::new(),
    };

    // Check editors
    for &name in EDITORS {
        if command_exists(name) {
            result.editors.push(name.to_string());
        }
    }

    // Check terminals
    for &name in TERMINALS {
        if command_exists(name) {
            result.terminals.push(name.to_string());
        }
    }

    // Check shells
    for &name in SHELLS {
        if command_exists(name) {
            result.shells.push(name.to_string());
        }
    }

    // Check compositors
    for &name in COMPOSITORS {
        if command_exists(name) {
            result.compositors.push(name.to_string());
        }
    }

    // Check browsers
    for &name in BROWSERS {
        if command_exists(name) {
            result.browsers.push(name.to_string());
        }
    }

    result
}
