//! Reset Command v0.0.10 - Factory Reset
//!
//! Returns Anna to exactly the same state as a fresh install.
//!
//! Flags:
//!   --dry-run    Show what would be deleted/reset without changing anything
//!   --force, -f  Skip interactive confirmation
//!
//! Safety:
//!   - Requires explicit confirmation (type "I CONFIRM (reset)") unless --force
//!   - Validates all paths before deletion
//!   - Idempotent: running twice produces no errors
//!   - Never deletes outside owned directories
//!   - Runs installer review at end to verify health

use anna_common::helpers::HELPERS_STATE_FILE;
use anna_common::install_state::ReviewResult;
use anna_common::installer_review::{run_and_record_review, InstallerReviewReport};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Directories owned by Anna (allowlist - ONLY these can be deleted)
const OWNED_DIRS: &[&str] = &["/var/lib/anna", "/var/log/anna", "/run/anna", "/etc/anna"];

/// Specific files we own
const OWNED_FILES: &[&str] = &["/etc/anna/config.toml"];

/// Directories to create after reset
const DIRS_TO_CREATE: &[&str] = &[
    "/var/lib/anna",
    "/var/lib/anna/knowledge",
    "/var/lib/anna/telemetry",
    "/var/lib/anna/internal",
    "/var/lib/anna/internal/snapshots",
    "/var/lib/anna/rollback",
    "/var/lib/anna/rollback/files",
    "/var/lib/anna/rollback/logs",
    "/var/lib/anna/kdb",
    "/var/log/anna",
    "/run/anna",
    "/etc/anna",
];

/// Reset options
pub struct ResetOptions {
    pub dry_run: bool,
    pub force: bool,
}

impl Default for ResetOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            force: false,
        }
    }
}

/// Run the reset command
pub async fn run(options: ResetOptions) -> Result<()> {
    println!();
    println!("{}", "  Anna Factory Reset".bold());
    println!("------------------------------------------------------------");
    println!();

    // Check if running as root
    if !is_root() {
        println!("{} This command requires root privileges.", "Error:".red());
        println!("  Run: sudo annactl reset");
        println!();
        return Ok(());
    }

    // Collect what we'll delete
    let mut items_to_delete: Vec<PathBuf> = Vec::new();

    // Add owned directories that exist
    for dir in OWNED_DIRS {
        let path = Path::new(dir);
        if path.exists() {
            // Validate path is safe
            if validate_path_safe(path) {
                items_to_delete.push(path.to_path_buf());
            }
        }
    }

    // Print what will happen
    println!("{}", "This will:".yellow());
    println!("  • Stop the daemon");

    if items_to_delete.is_empty() {
        println!("  • {} (already clean)", "Nothing to delete".dimmed());
    } else {
        for item in &items_to_delete {
            let size = get_dir_size(item);
            if size > 0 {
                println!("  • Delete {} ({})", item.display(), format_size(size));
            } else {
                println!("  • Delete {}", item.display());
            }
        }
    }

    println!("  • Clear helper tracking data");
    println!("  • Recreate directory structure");
    println!("  • Write default configuration");
    println!("  • Start daemon with fresh state");
    println!("  • Run installer review");
    println!();

    // Dry run - just show what would happen
    if options.dry_run {
        println!("{}", "[DRY RUN] No changes made.".cyan().bold());
        println!();
        return Ok(());
    }

    // Confirm unless --force
    if !options.force {
        println!("{}", "This is a destructive operation.".red());
        print!("Type {} to confirm: ", "I CONFIRM (reset)".red().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim() != "I CONFIRM (reset)" {
            println!();
            println!("Reset cancelled.");
            println!();
            return Ok(());
        }
    }

    println!();
    println!("{}", "Resetting...".cyan());

    // 1. Stop daemon
    print!("  Stopping daemon... ");
    io::stdout().flush()?;
    let _ = std::process::Command::new("systemctl")
        .args(["stop", "annad"])
        .output();
    println!("{}", "done".green());

    // 2. Delete owned directories (validated)
    for path in &items_to_delete {
        print!("  Deleting {}... ", path.display());
        io::stdout().flush()?;

        // Double-check path is still safe before deletion
        if !validate_path_safe(path) {
            println!("{} (path validation failed)", "skipped".yellow());
            continue;
        }

        if path.is_dir() {
            match std::fs::remove_dir_all(path) {
                Ok(_) => println!("{}", "done".green()),
                Err(e) => println!("{} ({})", "failed".red(), e),
            }
        } else if path.is_file() {
            match std::fs::remove_file(path) {
                Ok(_) => println!("{}", "done".green()),
                Err(e) => println!("{} ({})", "failed".red(), e),
            }
        }
    }

    // 3. Clear helper tracking
    print!("  Clearing helper tracking... ");
    io::stdout().flush()?;
    let helpers_path = Path::new(HELPERS_STATE_FILE);
    if helpers_path.exists() {
        let _ = std::fs::remove_file(helpers_path);
    }
    println!("{}", "done".green());

    // 4. Recreate directory structure
    print!("  Creating directories... ");
    io::stdout().flush()?;
    for dir in DIRS_TO_CREATE {
        std::fs::create_dir_all(dir)?;
        // Set ownership to root:root and permissions to 0755
        let _ = std::process::Command::new("chown")
            .args(["root:root", dir])
            .output();
        let _ = std::process::Command::new("chmod")
            .args(["0755", dir])
            .output();
    }
    println!("{}", "done".green());

    // 5. Write default config
    print!("  Writing default config... ");
    io::stdout().flush()?;
    write_default_config()?;
    println!("{}", "done".green());

    // 6. Start daemon
    print!("  Starting daemon... ");
    io::stdout().flush()?;
    let _ = std::process::Command::new("systemctl")
        .args(["start", "annad"])
        .output();

    // Wait for daemon to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Check if running
    let is_active = std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if is_active {
        println!("{}", "done".green());
    } else {
        println!("{}", "failed".red());
        println!();
        println!("  Check logs: journalctl -u annad -n 20");
    }

    // 7. Run installer review
    print!("  Running installer review... ");
    io::stdout().flush()?;
    let report = run_and_record_review(true); // auto-repair enabled
    println!("{}", "done".green());

    println!();
    println!("{}", "Factory reset complete.".green().bold());
    println!();

    // Display installer review results
    print_installer_review_summary(&report);

    println!("Anna is now in fresh install state.");
    println!("  annactl status");
    println!();

    Ok(())
}

/// Validate a path is safe to delete
/// Returns false if path is dangerous (/, $HOME, empty, too short, or outside allowlist)
fn validate_path_safe(path: &Path) -> bool {
    // Get canonical path (resolves symlinks)
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(), // Path doesn't exist, use as-is
    };

    let path_str = canonical.to_string_lossy();

    // Never delete root
    if path_str == "/" {
        return false;
    }

    // Never delete home directory
    if let Ok(home) = std::env::var("HOME") {
        if path_str == home || canonical == PathBuf::from(&home) {
            return false;
        }
    }

    // Never delete if path is empty or too short
    if path_str.is_empty() || path_str.len() < 5 {
        return false;
    }

    // Path must start with one of our owned prefixes
    let allowed_prefixes = ["/var/lib/anna", "/var/log/anna", "/run/anna", "/etc/anna"];

    for prefix in &allowed_prefixes {
        if path_str.starts_with(prefix) || path_str == *prefix {
            return true;
        }
    }

    false
}

/// Get approximate size of a directory
fn get_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    // Use du for accurate size
    let output = std::process::Command::new("du")
        .args(["-sb", &path.to_string_lossy()])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        }
        Err(_) => 0,
    }
}

/// Format bytes to human readable
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Write default configuration file
fn write_default_config() -> Result<()> {
    let config_file = "/etc/anna/config.toml";

    let default_config = format!(
        r#"# Anna v{} Configuration
# Generated by factory reset

[core]
mode = "normal"

[telemetry]
enabled = true
sample_interval_secs = 15
log_scan_interval_secs = 60
max_events_per_log = 100000
retention_days = 30

[updates]
mode = "auto"
interval_secs = 600

[log]
level = "info"
"#,
        VERSION
    );

    std::fs::write(config_file, default_config)?;

    // Set permissions
    let _ = std::process::Command::new("chown")
        .args(["root:root", config_file])
        .output();
    let _ = std::process::Command::new("chmod")
        .args(["0644", config_file])
        .output();

    Ok(())
}

/// Print installer review summary
fn print_installer_review_summary(report: &InstallerReviewReport) {
    match &report.overall {
        ReviewResult::Healthy => {
            println!("{}", "INSTALLER REVIEW: System is healthy".green().bold());
            println!("  {}", report.format_summary());
        }
        ReviewResult::Repaired { fixes } => {
            println!(
                "{}",
                "INSTALLER REVIEW: System is healthy (auto-repaired)"
                    .green()
                    .bold()
            );
            println!("  {}", report.format_summary());
            for fix in fixes {
                println!("    - {}", fix);
            }
        }
        ReviewResult::NeedsAttention { issues } => {
            println!("{}", "INSTALLER REVIEW: Needs attention".yellow().bold());
            println!("  {}", report.format_summary());
            for issue in issues {
                println!("    - {}", issue.yellow());
            }
        }
        ReviewResult::Failed { reason } => {
            println!("{}", "INSTALLER REVIEW: Failed".red().bold());
            println!("  {}", reason.red());
        }
    }
    println!();
}

/// Check if running as root
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_safe() {
        // Safe paths
        assert!(validate_path_safe(Path::new("/var/lib/anna")));
        assert!(validate_path_safe(Path::new("/var/lib/anna/knowledge")));
        assert!(validate_path_safe(Path::new("/var/log/anna")));
        assert!(validate_path_safe(Path::new("/etc/anna")));
        assert!(validate_path_safe(Path::new("/run/anna")));

        // Dangerous paths - must be rejected
        assert!(!validate_path_safe(Path::new("/")));
        assert!(!validate_path_safe(Path::new("")));
        assert!(!validate_path_safe(Path::new("/var")));
        assert!(!validate_path_safe(Path::new("/etc")));
        assert!(!validate_path_safe(Path::new("/home")));
        assert!(!validate_path_safe(Path::new("/usr")));
        assert!(!validate_path_safe(Path::new("/tmp")));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
