//! Uninstall Command v0.0.10 - Clean Anna Removal
//!
//! Completely removes Anna from the system.
//!
//! Flags:
//!   --dry-run      Show what would be deleted without changing anything
//!   --force, -f    Skip interactive confirmation
//!   --keep-helpers Don't offer to remove anna-installed helpers
//!
//! Safety:
//!   - Requires explicit confirmation (type "I CONFIRM (uninstall)") unless --force
//!   - Uses install_state.json for accurate removal
//!   - Only removes anna-installed helpers (asks user)
//!   - Never leaves broken permissions

use anna_common::install_state::{InstallState, discover_install_state};
use anna_common::helpers::{get_helper_status_list, InstalledBy, HELPERS_STATE_FILE, HelperState};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Uninstall options
pub struct UninstallOptions {
    pub dry_run: bool,
    pub force: bool,
    pub keep_helpers: bool,
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            force: false,
            keep_helpers: false,
        }
    }
}

/// Run the uninstall command
pub async fn run(options: UninstallOptions) -> Result<()> {
    println!();
    println!("{}", "  Anna Uninstall".bold());
    println!("------------------------------------------------------------");
    println!();

    // Check if running as root
    if !is_root() {
        println!("{} This command requires root privileges.", "Error:".red());
        println!("  Run: sudo annactl uninstall");
        println!();
        return Ok(());
    }

    // Load or discover install state
    let state = InstallState::load().unwrap_or_else(discover_install_state);

    // Get anna-installed helpers
    let anna_helpers: Vec<HelperState> = if options.keep_helpers {
        Vec::new()
    } else {
        get_helper_status_list()
            .into_iter()
            .filter(|h| h.installed_by == InstalledBy::Anna && h.present)
            .collect()
    };

    // Print what will happen
    println!("{}", "This will:".yellow());
    println!("  • Stop and disable annad service");

    // Show systemd unit removal
    if let Some(ref unit) = state.annad_unit {
        println!("  • Remove systemd unit: {}", unit.path.display());
    } else {
        println!("  • Remove systemd unit: /etc/systemd/system/annad.service");
    }

    // Show binary removal
    if let Some(ref bin) = state.annactl {
        println!("  • Remove binary: {}", bin.path.display());
    }
    if let Some(ref bin) = state.annad {
        println!("  • Remove binary: {}", bin.path.display());
    }

    // Show data directory removal
    println!("  • Remove data directories:");
    for dir in &state.data_dirs {
        if dir.anna_internal && Path::new(&dir.path).exists() {
            println!("      {}", dir.path.display());
        }
    }

    // Show config removal
    println!("  • Remove config: /etc/anna/");

    // Show helpers if any
    if !anna_helpers.is_empty() {
        println!();
        println!("  {} Anna-installed helpers found:", anna_helpers.len());
        for helper in &anna_helpers {
            println!("      {} ({})", helper.name, helper.purpose);
        }
    }

    println!();

    // Dry run - just show what would happen
    if options.dry_run {
        println!("{}", "[DRY RUN] No changes made.".cyan().bold());
        println!();
        return Ok(());
    }

    // Confirm unless --force
    if !options.force {
        println!("{}", "This will completely remove Anna from your system.".red());
        print!("Type {} to confirm: ", "I CONFIRM (uninstall)".red().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim() != "I CONFIRM (uninstall)" {
            println!();
            println!("Uninstall cancelled.");
            println!();
            return Ok(());
        }
    }

    // Ask about helpers
    let mut remove_helpers = false;
    if !anna_helpers.is_empty() && !options.force {
        println!();
        print!("Remove Anna-installed helpers? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        remove_helpers = input.trim().to_lowercase() == "y";
    }

    println!();
    println!("{}", "Uninstalling...".cyan());

    // 1. Stop and disable service
    print!("  Stopping annad service... ");
    io::stdout().flush()?;
    let _ = Command::new("systemctl")
        .args(["stop", "annad"])
        .output();
    println!("{}", "done".green());

    print!("  Disabling annad service... ");
    io::stdout().flush()?;
    let _ = Command::new("systemctl")
        .args(["disable", "annad"])
        .output();
    println!("{}", "done".green());

    // 2. Remove systemd unit file
    print!("  Removing systemd unit... ");
    io::stdout().flush()?;
    let unit_path = state.annad_unit
        .as_ref()
        .map(|u| u.path.clone())
        .unwrap_or_else(|| PathBuf::from("/etc/systemd/system/annad.service"));
    if unit_path.exists() {
        match std::fs::remove_file(&unit_path) {
            Ok(_) => println!("{}", "done".green()),
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    } else {
        println!("{}", "not found".dimmed());
    }

    // Reload systemd
    let _ = Command::new("systemctl")
        .args(["daemon-reload"])
        .output();

    // 3. Remove binaries
    print!("  Removing binaries... ");
    io::stdout().flush()?;
    let mut binaries_removed = 0;

    // Remove annactl (but we're running from it, so this is tricky)
    // The binary will be removed but the process will continue running
    if let Some(ref bin) = state.annactl {
        if bin.path.exists() {
            let _ = std::fs::remove_file(&bin.path);
            binaries_removed += 1;
        }
    } else {
        // Try common locations
        for path in &["/usr/bin/annactl", "/usr/local/bin/annactl"] {
            if Path::new(path).exists() {
                let _ = std::fs::remove_file(path);
                binaries_removed += 1;
            }
        }
    }

    // Remove annad
    if let Some(ref bin) = state.annad {
        if bin.path.exists() {
            let _ = std::fs::remove_file(&bin.path);
            binaries_removed += 1;
        }
    } else {
        // Try common locations
        for path in &["/usr/bin/annad", "/usr/local/bin/annad"] {
            if Path::new(path).exists() {
                let _ = std::fs::remove_file(path);
                binaries_removed += 1;
            }
        }
    }
    println!("{} ({} removed)", "done".green(), binaries_removed);

    // 4. Remove data directories
    print!("  Removing data directories... ");
    io::stdout().flush()?;

    // Remove /var/lib/anna (and all contents)
    if Path::new("/var/lib/anna").exists() {
        match std::fs::remove_dir_all("/var/lib/anna") {
            Ok(_) => {}
            Err(e) => {
                println!("{} ({})", "partial".yellow(), e);
            }
        }
    }

    // Remove /var/log/anna
    if Path::new("/var/log/anna").exists() {
        let _ = std::fs::remove_dir_all("/var/log/anna");
    }

    // Remove /run/anna
    if Path::new("/run/anna").exists() {
        let _ = std::fs::remove_dir_all("/run/anna");
    }

    println!("{}", "done".green());

    // 5. Remove config directory
    print!("  Removing config directory... ");
    io::stdout().flush()?;
    if Path::new("/etc/anna").exists() {
        match std::fs::remove_dir_all("/etc/anna") {
            Ok(_) => println!("{}", "done".green()),
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    } else {
        println!("{}", "not found".dimmed());
    }

    // 6. Remove anna-installed helpers if requested
    if remove_helpers && !anna_helpers.is_empty() {
        print!("  Removing helpers... ");
        io::stdout().flush()?;

        let mut removed = 0;
        let mut failed = 0;

        for helper in &anna_helpers {
            let result = Command::new("pacman")
                .args(["-Rns", "--noconfirm", &helper.name])
                .output();

            match result {
                Ok(output) if output.status.success() => removed += 1,
                _ => failed += 1,
            }
        }

        if failed > 0 {
            println!("{} ({} removed, {} failed)", "partial".yellow(), removed, failed);
        } else {
            println!("{} ({} removed)", "done".green(), removed);
        }
    }

    // 7. Clean up helpers.json if it exists (shouldn't but just in case)
    let helpers_path = Path::new(HELPERS_STATE_FILE);
    if helpers_path.exists() {
        let _ = std::fs::remove_file(helpers_path);
    }

    println!();
    println!("{}", "Uninstall complete.".green().bold());
    println!();
    println!("Anna has been removed from your system.");

    if !remove_helpers && !anna_helpers.is_empty() {
        println!();
        println!("Note: The following helpers were kept:");
        for helper in &anna_helpers {
            println!("  - {}", helper.name);
        }
        println!("Remove manually with: sudo pacman -Rns <package>");
    }

    println!();

    Ok(())
}

/// Check if running as root
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninstall_options_default() {
        let opts = UninstallOptions::default();
        assert!(!opts.dry_run);
        assert!(!opts.force);
        assert!(!opts.keep_helpers);
    }
}
