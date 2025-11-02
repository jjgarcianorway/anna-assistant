// Anna v0.12.7 - Reload Command Implementation
// Send SIGHUP to daemon for configuration reload

use anyhow::{Context, Result};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Send SIGHUP to daemon to reload configuration
pub async fn reload_config(verbose: bool) -> Result<()> {
    println!("\n╭─ Anna Configuration Reload ──────────────────────────────────────");
    println!("│");

    // 1. Check if daemon is running
    if verbose {
        println!("│  Checking daemon status...");
    }

    let daemon_check = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .output()
        .context("Failed to check daemon status")?;

    if !daemon_check.status.success() {
        println!("│  ✗ Daemon not running");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Error: annad service is not running");
        println!("Start it with: sudo systemctl start annad");
        println!();
        std::process::exit(1);
    }

    println!("│  ✓ Daemon: annad is running");

    // 2. Get daemon PID
    if verbose {
        println!("│  Getting daemon PID...");
    }

    let pid_output = Command::new("systemctl")
        .args(&["show", "annad", "--property=MainPID", "--value"])
        .output()
        .context("Failed to get daemon PID")?;

    let pid_str = String::from_utf8_lossy(&pid_output.stdout).trim().to_string();
    let pid: u32 = pid_str
        .parse()
        .context("Failed to parse daemon PID")?;

    if pid == 0 {
        println!("│  ✗ Could not determine daemon PID");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Error: Daemon PID is 0 - daemon may not be running correctly");
        println!();
        std::process::exit(1);
    }

    println!("│  ✓ Daemon PID: {}", pid);

    // 3. Get config hash before reload (optional, for comparison)
    if verbose {
        println!("│  Reading current configuration...");
    }

    let config_path = "/etc/anna/config.toml";
    let config_before = if std::path::Path::new(config_path).exists() {
        std::fs::read_to_string(config_path).ok()
    } else {
        None
    };

    if config_before.is_some() {
        println!("│  ✓ Configuration file: {}", config_path);
    } else {
        println!("│  ⚠ Configuration file not found (will use defaults)");
    }

    // 4. Validate config before sending signal
    if verbose {
        println!("│  Validating configuration syntax...");
    }

    if let Some(ref config_content) = config_before {
        match toml::from_str::<toml::Value>(config_content) {
            Ok(_) => {
                println!("│  ✓ Configuration syntax valid");
            }
            Err(e) => {
                println!("│  ✗ Configuration syntax invalid");
                println!("│");
                println!("╰──────────────────────────────────────────────────────────────────");
                println!();
                println!("Error: Configuration file has syntax errors:");
                println!("{}", e);
                println!();
                println!("Fix the configuration file and try again.");
                println!();
                std::process::exit(1);
            }
        }
    }

    // 5. Send SIGHUP
    println!("│  Sending SIGHUP to daemon...");

    let sighup_result = Command::new("kill")
        .args(&["-HUP", &pid.to_string()])
        .status()
        .context("Failed to send SIGHUP")?;

    if !sighup_result.success() {
        println!("│  ✗ Failed to send SIGHUP");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Error: Could not send SIGHUP signal to daemon");
        println!("You may need to run this with sudo.");
        println!();
        std::process::exit(1);
    }

    println!("│  ✓ SIGHUP sent successfully");

    // 6. Wait for daemon to process reload
    println!("│  Waiting for daemon to reload...");

    thread::sleep(Duration::from_millis(500));

    // 7. Verify daemon is still running
    let daemon_check_after = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .output()
        .context("Failed to check daemon status after reload")?;

    if !daemon_check_after.status.success() {
        println!("│  ✗ Daemon stopped after reload");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Error: Daemon crashed during reload!");
        println!("Check logs: sudo journalctl -u annad -n 50");
        println!();
        std::process::exit(1);
    }

    println!("│  ✓ Daemon still running");

    // 8. Try RPC call to verify daemon is responsive
    if verbose {
        println!("│  Checking daemon responsiveness...");
    }

    match crate::rpc_call("status", None).await {
        Ok(_) => {
            println!("│  ✓ Daemon responding to RPC");
        }
        Err(e) => {
            println!("│  ⚠ Daemon not responding to RPC: {}", e);
            println!("│    This may be temporary - check status in a moment");
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();
    println!("✓ Configuration reload complete");
    println!();

    if verbose {
        println!("The daemon has reloaded its configuration.");
        println!("Changes will take effect immediately where applicable.");
        println!();
        println!("Note: Some settings (like socket path) require a full restart.");
        println!();
    }

    println!("Verify with: annactl status");
    println!("View logs:   sudo journalctl -u annad -n 20");
    println!();

    Ok(())
}

/// Dry-run config validation (doesn't send SIGHUP)
pub fn validate_config(config_path: Option<String>, verbose: bool) -> Result<()> {
    let path = config_path.unwrap_or_else(|| "/etc/anna/config.toml".to_string());

    println!("\n╭─ Configuration Validation ───────────────────────────────────────");
    println!("│");
    println!("│  Config file: {}", path);
    println!("│");

    if !std::path::Path::new(&path).exists() {
        println!("│  ✗ File not found");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Error: Configuration file does not exist: {}", path);
        println!();
        std::process::exit(1);
    }

    // Read file
    let content = std::fs::read_to_string(&path)
        .context("Failed to read configuration file")?;

    if verbose {
        println!("│  File size: {} bytes", content.len());
        println!("│");
    }

    // Parse TOML
    match toml::from_str::<toml::Value>(&content) {
        Ok(parsed) => {
            println!("│  ✓ TOML syntax valid");

            if verbose {
                println!("│");
                println!("│  Configuration structure:");

                if let toml::Value::Table(table) = parsed {
                    for (section, _) in table.iter() {
                        println!("│    [{section}]");
                    }
                }
            }
        }
        Err(e) => {
            println!("│  ✗ TOML syntax invalid");
            println!("│");
            println!("╰──────────────────────────────────────────────────────────────────");
            println!();
            println!("Error: Configuration file has syntax errors:");
            println!("{}", e);
            println!();
            std::process::exit(1);
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();
    println!("✓ Configuration file is valid");
    println!();

    Ok(())
}
