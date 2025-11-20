//! Doctor command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn doctor(fix: bool, dry_run: bool, auto: bool) -> Result<()> {
    use std::process::Command;
    use std::io::{self, Write};
    use std::path::Path;

    println!();
    println!("\x1b[1m╭──────────────────────╮\x1b[0m");
    println!("\x1b[1m│  Anna Health Check   │\x1b[0m");
    println!("\x1b[1m╰──────────────────────╯\x1b[0m");
    println!();

    let mut health_score = 100;
    let mut issues: Vec<(String, String, bool)> = Vec::new(); // (issue, fix_command, is_critical)
    let mut critical_issues: Vec<String> = Vec::new();

    // ==================== BINARIES ====================
    println!("{}", section("🔧 Binaries"));

    // Check annad binary
    let annad_path = "/usr/local/bin/annad";
    if Path::new(annad_path).exists() {
        if let Ok(metadata) = std::fs::metadata(annad_path) {
            let permissions = metadata.permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if permissions.mode() & 0o111 != 0 {
                    println!("  {} annad binary exists and is executable", beautiful::status(Level::Success, "✓"));
                } else {
                    println!("  {} annad binary exists but is not executable", beautiful::status(Level::Error, "✗"));
                    issues.push((
                        "annad binary not executable".to_string(),
                        format!("sudo chmod +x {}", annad_path),
                        true
                    ));
                    critical_issues.push("annad binary not executable".to_string());
                    health_score -= 20;
                }
            }
            #[cfg(not(unix))]
            {
                println!("  {} annad binary exists", beautiful::status(Level::Success, "✓"));
            }
        }
    } else {
        println!("  {} annad binary not found at {}", beautiful::status(Level::Error, "✗"), annad_path);
        issues.push((
            "annad binary missing".to_string(),
            "Install Anna Assistant with 'annactl update --install'".to_string(),
            true
        ));
        critical_issues.push("annad binary missing".to_string());
        health_score -= 25;
    }

    // Check annactl binary
    let annactl_path = "/usr/local/bin/annactl";
    if Path::new(annactl_path).exists() {
        if let Ok(metadata) = std::fs::metadata(annactl_path) {
            let permissions = metadata.permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if permissions.mode() & 0o111 != 0 {
                    println!("  {} annactl binary exists and is executable", beautiful::status(Level::Success, "✓"));
                } else {
                    println!("  {} annactl binary exists but is not executable", beautiful::status(Level::Error, "✗"));
                    issues.push((
                        "annactl binary not executable".to_string(),
                        format!("sudo chmod +x {}", annactl_path),
                        true
                    ));
                    critical_issues.push("annactl binary not executable".to_string());
                    health_score -= 20;
                }
            }
            #[cfg(not(unix))]
            {
                println!("  {} annactl binary exists", beautiful::status(Level::Success, "✓"));
            }
        }
    } else {
        println!("  {} annactl binary not found at {}", beautiful::status(Level::Error, "✗"), annactl_path);
        issues.push((
            "annactl binary missing".to_string(),
            "Install Anna Assistant with 'annactl update --install'".to_string(),
            true
        ));
        critical_issues.push("annactl binary missing".to_string());
        health_score -= 25;
    }

    println!();

    // ==================== DEPENDENCIES ====================
    println!("{}", section("📦 Dependencies"));

    // Check curl
    if Command::new("which").arg("curl").output().map(|o| o.status.success()).unwrap_or(false) {
        println!("  {} curl installed", beautiful::status(Level::Success, "✓"));
    } else {
        println!("  {} curl not found", beautiful::status(Level::Error, "✗"));
        issues.push((
            "curl missing".to_string(),
            "sudo pacman -S curl".to_string(),
            false
        ));
        health_score -= 10;
    }

    // Check jq
    if Command::new("which").arg("jq").output().map(|o| o.status.success()).unwrap_or(false) {
        println!("  {} jq installed", beautiful::status(Level::Success, "✓"));
    } else {
        println!("  {} jq not found", beautiful::status(Level::Error, "✗"));
        issues.push((
            "jq missing".to_string(),
            "sudo pacman -S jq".to_string(),
            false
        ));
        health_score -= 10;
    }

    // Check systemctl
    if Command::new("which").arg("systemctl").output().map(|o| o.status.success()).unwrap_or(false) {
        println!("  {} systemctl available", beautiful::status(Level::Success, "✓"));
    } else {
        println!("  {} systemctl not found", beautiful::status(Level::Error, "✗"));
        issues.push((
            "systemctl missing".to_string(),
            "systemd is required".to_string(),
            true
        ));
        critical_issues.push("systemctl missing - systemd required".to_string());
        health_score -= 20;
    }

    println!();

    // ==================== DAEMON SERVICE ====================
    println!("{}", section("🔌 Daemon Service"));

    let service_file = "/etc/systemd/system/annad.service";
    let service_exists = Path::new(service_file).exists();

    if service_exists {
        println!("  {} Service file exists", beautiful::status(Level::Success, "✓"));
    } else {
        println!("  {} Service file not found at {}", beautiful::status(Level::Error, "✗"), service_file);
        issues.push((
            "Service file missing".to_string(),
            "Install Anna Assistant properly with 'annactl update --install'".to_string(),
            true
        ));
        critical_issues.push("Service file missing".to_string());
        health_score -= 20;
    }

    // Check if service is loaded
    if service_exists {
        if let Ok(output) = Command::new("systemctl").args(&["is-enabled", "annad"]).output() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if status == "enabled" || status == "static" || status == "disabled" {
                println!("  {} Service is loaded ({})", beautiful::status(Level::Success, "✓"), status);
            } else {
                println!("  {} Service not loaded properly", beautiful::status(Level::Warning, "!"));
                health_score -= 5;
            }
        }
    }

    // Check service state
    let mut service_running = false;
    if service_exists {
        if let Ok(output) = Command::new("systemctl").args(&["is-active", "annad"]).output() {
            if output.status.success() {
                println!("  {} Service is active", beautiful::status(Level::Success, "✓"));
                service_running = true;
            } else {
                let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("  {} Service is {} (not running)", beautiful::status(Level::Error, "✗"), state);
                println!("     \x1b[90mFix: sudo systemctl start annad\x1b[0m");
                issues.push((
                    "Daemon not running".to_string(),
                    "sudo systemctl start annad".to_string(),
                    true
                ));
                critical_issues.push("Daemon not running".to_string());
                health_score -= 25;
            }
        }
    }

    println!();

    // ==================== SOCKET CONNECTIVITY ====================
    println!("{}", section("🔗 Socket Connectivity"));

    let socket_path = "/run/anna/anna.sock";
    let socket_exists = Path::new(socket_path).exists();

    if socket_exists {
        println!("  {} Socket file exists", beautiful::status(Level::Success, "✓"));

        // Check socket permissions
        if let Ok(metadata) = std::fs::metadata(socket_path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                // Socket should be readable/writable
                if mode & 0o600 != 0 {
                    println!("  {} Socket permissions correct", beautiful::status(Level::Success, "✓"));
                } else {
                    println!("  {} Socket permissions incorrect (mode: {:o})", beautiful::status(Level::Warning, "!"), mode);
                    health_score -= 5;
                }
            }
        }

        // CRITICAL: Test actual RPC connection
        println!("  {} Testing actual RPC connection...", beautiful::status(Level::Info, "ℹ"));
        match RpcClient::connect().await {
            Ok(mut client) => {
                // Try to ping the daemon
                match client.ping().await {
                    Ok(_) => {
                        println!("  {} RPC connection successful (ping OK)", beautiful::status(Level::Success, "✓"));
                    }
                    Err(e) => {
                        println!("  {} RPC ping failed: {}", beautiful::status(Level::Error, "✗"), e);
                        println!("     \x1b[90mSocket exists but daemon is not responding properly\x1b[0m");
                        issues.push((
                            "Daemon not responding to RPC".to_string(),
                            "sudo systemctl restart annad".to_string(),
                            true
                        ));
                        critical_issues.push("Daemon not responding".to_string());
                        health_score -= 20;
                    }
                }
            }
            Err(e) => {
                println!("  {} Cannot connect to daemon: {}", beautiful::status(Level::Error, "✗"), e);
                println!("     \x1b[90mThis usually means the daemon crashed or failed to start.\x1b[0m");
                println!("     \x1b[90mCheck logs: journalctl -u annad -n 50\x1b[0m");
                issues.push((
                    "Cannot connect to daemon socket".to_string(),
                    "journalctl -u annad -n 50".to_string(),
                    true
                ));
                critical_issues.push("Socket not accessible".to_string());
                health_score -= 25;
            }
        }
    } else {
        println!("  {} Socket file does not exist at {}", beautiful::status(Level::Error, "✗"), socket_path);
        println!("  {} Cannot connect to daemon", beautiful::status(Level::Error, "✗"));
        if service_running {
            println!("     \x1b[90mService reports 'active' but socket is missing - daemon may have crashed\x1b[0m");
        } else {
            println!("     \x1b[90mDaemon is not running\x1b[0m");
        }
        println!("     \x1b[90mCheck logs: journalctl -u annad -n 50\x1b[0m");
        issues.push((
            "Socket file missing".to_string(),
            if service_running {
                "sudo systemctl restart annad".to_string()
            } else {
                "sudo systemctl start annad".to_string()
            },
            true
        ));
        critical_issues.push("Socket not accessible".to_string());
        health_score -= 25;
    }

    println!();

    // ==================== DIRECTORIES ====================
    println!("{}", section("📁 Directories"));

    // Check /run/anna/ (daemon runtime directory)
    let run_dir = "/run/anna";
    if Path::new(run_dir).exists() {
        // NOTE: This directory is owned by root (daemon runs as root)
        // Regular users don't need write access - they communicate via socket
        println!("  {} {} exists (daemon runtime)", beautiful::status(Level::Success, "✓"), run_dir);
    } else {
        println!("  {} {} does not exist", beautiful::status(Level::Error, "✗"), run_dir);
        issues.push((
            format!("{} directory missing", run_dir),
            format!("sudo mkdir -p {} && sudo chown root:root {}", run_dir, run_dir),
            true
        ));
        critical_issues.push(format!("{} directory missing", run_dir));
        health_score -= 15;
    }

    // Check config directory
    if let Ok(home) = std::env::var("HOME") {
        let config_dir = format!("{}/.config/anna", home);
        if Path::new(&config_dir).exists() {
            println!("  {} Config directory exists", beautiful::status(Level::Success, "✓"));
        } else {
            println!("  {} Config directory will be created on first use", beautiful::status(Level::Info, "ℹ"));
        }
    }

    println!();

    // ==================== HEALTH SCORE ====================
    let health_color = if health_score >= 90 {
        "\x1b[92m" // Green
    } else if health_score >= 70 {
        "\x1b[93m" // Yellow
    } else {
        "\x1b[91m" // Red
    };

    println!("{}", section("📊 Health Score"));
    println!("  {}{}/100\x1b[0m", health_color, health_score);
    println!();

    // ==================== CRITICAL ISSUES ====================
    if !critical_issues.is_empty() {
        println!("⚠️  \x1b[1mCritical Issues:\x1b[0m");
        for issue in &critical_issues {
            println!("  \x1b[91m•\x1b[0m {}", issue);
        }
        println!();
    }

    // ==================== ALL ISSUES ====================
    if !issues.is_empty() {
        println!("\x1b[1mIssues Found:\x1b[0m");
        for (_i, (issue, fix_cmd, is_critical)) in issues.iter().enumerate() {
            let symbol = if *is_critical { "\x1b[91m✗\x1b[0m" } else { "\x1b[93m!\x1b[0m" };
            println!("  {} {}", symbol, issue);
            if !fix_cmd.is_empty() {
                println!("     \x1b[90mFix: {}\x1b[0m", fix_cmd);
            }
        }
        println!();
    }

    // ==================== FINAL MESSAGE ====================
    if health_score == 100 {
        println!("{}", beautiful::status(Level::Success, "Anna Assistant is healthy and ready!"));
    } else if health_score >= 80 {
        println!("{}", beautiful::status(Level::Success, "Anna Assistant is mostly healthy"));
        if !issues.is_empty() {
            println!();
            println!("Run with \x1b[36m--fix\x1b[0m to attempt automatic fixes.");
        }
    } else {
        println!("{}", beautiful::status(Level::Error, "Anna Assistant needs attention!"));
        println!();
        println!("Run with \x1b[36m--fix\x1b[0m to attempt automatic fixes.");
    }

    // ==================== AUTO-FIX ====================
    if (fix || dry_run) && !issues.is_empty() {
        println!();
        println!("{}", section("🔧 Auto-Fix"));

        let fixable_issues: Vec<_> = issues.iter()
            .filter(|(_, cmd, _)| !cmd.is_empty() && !cmd.starts_with("Install") && !cmd.starts_with("journalctl"))
            .collect();

        if fixable_issues.is_empty() {
            println!("{}", beautiful::status(Level::Info, "No auto-fixable issues found"));
            println!("Manual intervention required for the issues listed above.");
            return Ok(());
        }

        if dry_run {
            println!("{}", beautiful::status(Level::Info, "DRY RUN - showing what would be fixed:"));
            println!();
            for (i, (issue, fix_cmd, _)) in fixable_issues.iter().enumerate() {
                println!("  {}. {}", i + 1, issue);
                println!("     \x1b[36m→ {}\x1b[0m", fix_cmd);
            }
            return Ok(());
        }

        println!("{}", beautiful::status(Level::Info, &format!("Found {} fixable issues", fixable_issues.len())));
        println!();

        let mut fixed_count = 0;
        let mut failed_count = 0;

        for (i, (issue, fix_cmd, _is_critical)) in fixable_issues.iter().enumerate() {
            println!("  [{}] {}", i + 1, issue);

            // RC.9.3: User feedback - "--fix should run by herself" (automatically without confirmation)
            // Always auto-fix when --fix is used. No need for --auto flag anymore.

            println!("  \x1b[36m→ {}\x1b[0m", fix_cmd);

            let fix_result = Command::new("sh")
                .arg("-c")
                .arg(fix_cmd)
                .output();

            match fix_result {
                Ok(output) if output.status.success() => {
                    println!("  {}", beautiful::status(Level::Success, "✓ Fixed successfully"));
                    fixed_count += 1;
                }
                Ok(output) => {
                    println!("  {}", beautiful::status(Level::Error, "✗ Fix failed"));
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("  \x1b[90m{}\x1b[0m", stderr.trim());
                    }
                    failed_count += 1;
                }
                Err(e) => {
                    println!("  {} {}", beautiful::status(Level::Error, "✗"), e);
                    failed_count += 1;
                }
            }
            println!();
        }

        println!("{}", section("📊 Fix Summary"));
        if fixed_count > 0 {
            println!("  {} {} issues fixed", beautiful::status(Level::Success, "✓"), fixed_count);
        }
        if failed_count > 0 {
            println!("  {} {} fixes failed", beautiful::status(Level::Error, "✗"), failed_count);
        }
        println!();

        if fixed_count > 0 {
            println!("{}", beautiful::status(Level::Info, "Run 'annactl doctor' again to verify fixes"));
        }
    }

    Ok(())
}

/// Configuration management
/// NOTE: Removed for v1.0 - not implemented yet
#[allow(dead_code)]
