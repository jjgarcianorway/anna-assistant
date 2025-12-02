//! Doctor Command v7.42.0 - Diagnostic Tool
//!
//! Minimal diagnostic output for troubleshooting the daemon/snapshot contract.
//!
//! Exit codes:
//! - 0:  OK (daemon running, snapshot available)
//! - 10: Daemon unreachable
//! - 11: Snapshot missing
//! - 12: Snapshot stale
//! - 13: Permission problem

use anyhow::Result;
use owo_colors::OwoColorize;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use anna_common::daemon_state::{
    StatusSnapshot, INTERNAL_DIR, SNAPSHOTS_DIR, STATUS_SNAPSHOT_PATH, STALE_THRESHOLD_SECS,
};
use anna_common::control_socket::{
    check_daemon_health, DaemonHealth, socket_connectable,
    check_systemd_active, get_systemd_pid, SOCKET_PATH,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit codes
pub const EXIT_OK: i32 = 0;
pub const EXIT_DAEMON_UNREACHABLE: i32 = 10;
pub const EXIT_SNAPSHOT_MISSING: i32 = 11;
pub const EXIT_SNAPSHOT_STALE: i32 = 12;
pub const EXIT_PERMISSION_PROBLEM: i32 = 13;

/// Run the doctor command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "Anna Doctor v7.42.0".bold());
    println!("────────────────────────────────────────");
    println!();

    let mut issues: Vec<(i32, String)> = Vec::new();

    // 1. Systemd state
    print_systemd_state(&mut issues);

    // 2. Control socket state
    print_socket_state(&mut issues);

    // 3. Snapshot state
    print_snapshot_state(&mut issues);

    // 4. Directory permissions
    print_directory_state(&mut issues);

    // 5. Summary
    println!();
    println!("────────────────────────────────────────");

    if issues.is_empty() {
        println!("{}", "✓ All checks passed".green());
        std::process::exit(EXIT_OK);
    } else {
        println!("{} {} issue(s) found:", "✗".red(), issues.len());
        for (code, msg) in &issues {
            println!("  {} (exit code {})", msg.red(), code);
        }

        // Exit with the first (most severe) issue code
        let exit_code = issues.first().map(|(c, _)| *c).unwrap_or(1);
        std::process::exit(exit_code);
    }
}

fn print_systemd_state(issues: &mut Vec<(i32, String)>) {
    println!("{}", "[SYSTEMD]".cyan());

    match check_systemd_active() {
        Ok(true) => {
            println!("  State:      {}", "active".green());
            if let Some(pid) = get_systemd_pid() {
                println!("  MainPID:    {}", pid);
            }
        }
        Ok(false) => {
            println!("  State:      {}", "inactive".red());
            issues.push((EXIT_DAEMON_UNREACHABLE, "Daemon not running (systemd inactive)".to_string()));
        }
        Err(e) => {
            println!("  State:      {} ({})", "unknown".yellow(), e.dimmed());
        }
    }

    println!();
}

fn print_socket_state(issues: &mut Vec<(i32, String)>) {
    println!("{}", "[SOCKET]".cyan());

    let socket_path = Path::new(SOCKET_PATH);
    println!("  Path:       {}", SOCKET_PATH);
    println!("  Exists:     {}", if socket_path.exists() { "yes".green().to_string() } else { "no".red().to_string() });

    if socket_path.exists() {
        let connectable = socket_connectable();
        println!("  Connect:    {}", if connectable { "ok".green().to_string() } else { "failed".red().to_string() });

        if !connectable {
            // Not an issue if systemd says daemon is running - socket might just be slow
        }
    } else {
        println!("  {}", "(socket may be created by daemon or require /run/anna)".dimmed());
    }

    println!();
}

fn print_snapshot_state(issues: &mut Vec<(i32, String)>) {
    println!("{}", "[SNAPSHOT]".cyan());

    println!("  Path:       {}", STATUS_SNAPSHOT_PATH);

    let snapshot_path = Path::new(STATUS_SNAPSHOT_PATH);
    if !snapshot_path.exists() {
        println!("  Exists:     {}", "no".red());

        // Also check legacy path
        let legacy_path = format!("{}/status_snapshot.json", INTERNAL_DIR);
        if Path::new(&legacy_path).exists() {
            println!("  Legacy:     {} (found at {})", "yes".yellow(), legacy_path);
        }

        issues.push((EXIT_SNAPSHOT_MISSING, format!("Snapshot missing: {}", STATUS_SNAPSHOT_PATH)));
    } else {
        println!("  Exists:     {}", "yes".green());

        // Check metadata
        if let Ok(meta) = std::fs::metadata(snapshot_path) {
            println!("  Size:       {} bytes", meta.len());
            println!("  Mode:       {:o}", meta.mode() & 0o777);

            if let Ok(mtime) = meta.modified() {
                if let Ok(age) = std::time::SystemTime::now().duration_since(mtime) {
                    let age_secs = age.as_secs();
                    let age_str = if age_secs < 60 {
                        format!("{}s ago", age_secs)
                    } else if age_secs < 3600 {
                        format!("{}m ago", age_secs / 60)
                    } else {
                        format!("{}h ago", age_secs / 3600)
                    };
                    println!("  Modified:   {}", age_str);

                    if age_secs > STALE_THRESHOLD_SECS {
                        issues.push((EXIT_SNAPSHOT_STALE, format!("Snapshot stale ({}s old, threshold {}s)", age_secs, STALE_THRESHOLD_SECS)));
                    }
                }
            }
        }

        // Try to parse it
        match StatusSnapshot::load() {
            Some(s) => {
                println!("  Parse:      {}", "ok".green());
                println!("  Version:    v{}", s.version);
                println!("  Seq:        {}", s.seq);
                if let Some(gen) = s.generated_at {
                    println!("  Generated:  {}", gen.format("%Y-%m-%d %H:%M:%S UTC"));
                }
            }
            None => {
                println!("  Parse:      {} (file exists but couldn't parse)", "failed".red());
            }
        }
    }

    println!();
}

fn print_directory_state(issues: &mut Vec<(i32, String)>) {
    println!("{}", "[DIRECTORIES]".cyan());

    let dirs = [
        ("/var/lib/anna", "Data directory"),
        (INTERNAL_DIR, "Internal directory"),
        (SNAPSHOTS_DIR, "Snapshots directory"),
        ("/run/anna", "Runtime directory (for socket)"),
    ];

    for (path, desc) in &dirs {
        let p = Path::new(path);
        if p.exists() {
            let writable = check_writable(path);
            let status = if writable { "writable".green().to_string() } else { "not writable".red().to_string() };
            println!("  {:24} {} ({})", format!("{}:", desc), path, status);

            if !writable && *path == SNAPSHOTS_DIR {
                issues.push((EXIT_PERMISSION_PROBLEM, format!("Cannot write to {}", path)));
            }
        } else {
            println!("  {:24} {} ({})", format!("{}:", desc), path, "missing".yellow());
        }
    }

    println!();
}

fn check_writable(path: &str) -> bool {
    let test_file = format!("{}/.anna_write_test", path);
    if std::fs::write(&test_file, "test").is_ok() {
        let _ = std::fs::remove_file(&test_file);
        true
    } else {
        false
    }
}
