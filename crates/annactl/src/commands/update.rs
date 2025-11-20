//! Update command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

/// Fetch release notes from GitHub API
async fn fetch_release_notes(version: &str) -> Result<String> {
    // GitHub tags have "v" prefix
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };
    let url = format!("https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/tags/{}", tag);

    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .build()?;

    let response = client.get(&url)
        .send()
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&response)?;

    Ok(json["body"].as_str().unwrap_or("").to_string())
}

/// Display formatted release notes
fn display_release_notes(notes: &str) {
    let lines: Vec<&str> = notes.lines().collect();

    for line in lines.iter().take(20) {
        // Headers with emoji
        if line.starts_with("###") {
            println!("  \x1b[1m\x1b[38;5;159m{}\x1b[0m", line);
        }
        // Bold sections
        else if line.starts_with("**") {
            println!("  \x1b[1m{}\x1b[0m", line);
        }
        // Bullet points
        else if line.starts_with("- ") {
            println!("    \x1b[38;5;228m→\x1b[0m \x1b[38;5;250m{}\x1b[0m", &line[2..]);
        }
        // Regular text
        else if !line.is_empty() {
            println!("  \x1b[38;5;250m{}\x1b[0m", line);
        }
    }

    if lines.len() > 20 {
        println!();
        println!("  \x1b[38;5;250m... see full notes at GitHub\x1b[0m");
    }
}

/// Send desktop notification (non-intrusive, no wall spam)
fn send_update_notification(version: &str) {
    // Try to send desktop notification via notify-send (if available)
    use std::process::Command;

    // Only send if notify-send is available and we're in a desktop environment
    if Command::new("which")
        .arg("notify-send")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let _ = Command::new("notify-send")
            .arg("--app-name=Anna Assistant")
            .arg("--icon=system-software-update")
            .arg("Update Complete")
            .arg(&format!("Anna has been updated to {}", version))
            .spawn();
    }
}

/// Check for updates and optionally install them
///
/// Beta.87: Now delegates to daemon via RPC - no sudo required!

pub async fn update(install: bool, check_only: bool) -> Result<()> {
    println!("{}", header("Anna Update"));
    println!();

    // Show current version first
    let current = anna_common::updater::get_current_version().unwrap_or_else(|_| "unknown".to_string());
    println!("  {}", kv("Installed version", &current));
    println!();

    // Try to connect to daemon for update operations
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            // RC.7: Fallback to direct update if daemon unavailable
            if install {
                println!("{}", beautiful::status(Level::Warning, "Cannot connect to daemon"));
                println!("{}", beautiful::status(Level::Info, "Using direct update method as fallback..."));
                println!();
                return direct_update_fallback(&current).await;
            } else {
                println!("{}", beautiful::status(Level::Error, "Cannot connect to daemon"));
                println!();
                println!("{}", beautiful::status(Level::Info, "Start daemon with: sudo systemctl start annad"));
                println!("{}", beautiful::status(Level::Info, "Or use --install flag for direct update"));
                return Ok(());
            }
        }
    };

    // Check for updates via daemon (no sudo needed!)
    println!("{}", beautiful::status(Level::Info, "Checking for updates via daemon..."));
    println!();

    match client.call(Method::CheckUpdate).await {
        Ok(ResponseData::UpdateCheck {
            current_version,
            latest_version,
            is_update_available,
            download_url: _,
            release_notes,
        }) => {
            if !is_update_available {
                println!("{}", beautiful::status(
                    Level::Success,
                    &format!("Already on latest version: {}", current_version)
                ));
                println!();

                // Show release notes for current version when --install is used
                if install {
                    println!("{}", section("Current Version Information"));
                    println!("  {}", kv("Version", &current_version));
                    println!();

                    println!("{}", section("What's in This Version"));
                    if let Ok(notes) = fetch_release_notes(&current_version).await {
                        display_release_notes(&notes);
                    } else if let Some(ref url) = release_notes {
                        println!("  \x1b[38;5;159m{}\x1b[0m", url);
                    }
                    println!();
                }

                return Ok(());
            }

            // Update available
            println!("{}", beautiful::status(
                Level::Warning,
                &format!("Update available: {} → {}",
                    current_version,
                    latest_version)
            ));
            println!();

            if !check_only {
                println!("{}", section("📦 Release Information"));
                println!("  {}", kv("Current", &current_version));
                println!("  {}", kv("Latest", &latest_version));
                println!();

                // FETCH AND DISPLAY actual release notes, not just URL
                println!("{}", section("📝 What's New in This Release"));
                if let Ok(notes) = fetch_release_notes(&latest_version).await {
                    display_release_notes(&notes);
                } else if let Some(ref url) = release_notes {
                    // Fallback: show URL if fetching failed
                    println!("  \x1b[38;5;159mRelease notes: {}\x1b[0m", url);
                }
                println!();
            }

            if install {
                // Ask for confirmation before updating
                use std::io::{self, Write};
                print!("  \x1b[1;93mProceed with update? (y/N):\x1b[0m ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "y" && input != "yes" {
                    println!();
                    println!("{}", beautiful::status(Level::Info, "Update cancelled by user"));
                    return Ok(());
                }

                // Perform the update via daemon (no sudo needed!)
                println!();
                println!("{}", beautiful::status(Level::Info, "🔐 Delegating update to daemon (no sudo required!)"));
                println!("{}", beautiful::status(Level::Info, "Starting update process..."));
                println!();

                // Call daemon to perform update
                match client.call(Method::PerformUpdate { restart_only: false }).await {
                    Ok(ResponseData::UpdateResult {
                        success,
                        message: _,
                        old_version,
                        new_version,
                    }) if success => {
                        println!();

                        // Beautiful update success banner
                        println!("\x1b[38;5;120m╭{}╮\x1b[0m", "─".repeat(60));
                        println!("\x1b[38;5;120m│\x1b[0m \x1b[1m\x1b[38;5;159m🎉 Update Successful!\x1b[0m");
                        println!("\x1b[38;5;120m│\x1b[0m");
                        println!("\x1b[38;5;120m│\x1b[0m   Version: \x1b[1m{}\x1b[0m → \x1b[1m\x1b[38;5;159m{}\x1b[0m",
                            old_version,
                            new_version);
                        println!("\x1b[38;5;120m│\x1b[0m   Method: Daemon delegation (no sudo!)");
                        println!("\x1b[38;5;120m│\x1b[0m");
                        println!("\x1b[38;5;120m╰{}╯\x1b[0m", "─".repeat(60));
                        println!();

                        println!("{}", section("What's New"));

                        // Fetch and display release notes
                        if let Ok(notes) = fetch_release_notes(&new_version).await {
                            display_release_notes(&notes);
                        } else if let Some(ref url) = release_notes {
                            println!("  \x1b[38;5;159m{}\x1b[0m", url);
                        }
                        println!();

                        println!("{}", beautiful::status(Level::Info, "Daemon has been restarted"));
                        println!();

                        // Send desktop notification (non-intrusive)
                        send_update_notification(&new_version);
                    }
                    Ok(ResponseData::UpdateResult {
                        success: false,
                        message,
                        ..
                    }) => {
                        println!();
                        println!("{}", beautiful::status(
                            Level::Warning,
                            &message
                        ));
                        println!();
                    }
                    Err(e) => {
                        println!();
                        println!("{}", beautiful::status(
                            Level::Error,
                            &format!("Update failed: {}", e)
                        ));
                        println!();
                        println!("{}", beautiful::status(
                            Level::Info,
                            "Your previous version has been backed up to /var/lib/anna/backup/"
                        ));
                    }
                    _ => {
                        println!();
                        println!("{}", beautiful::status(
                            Level::Error,
                            "Unexpected response from daemon"
                        ));
                    }
                }
            } else {
                // Prompt user to install
                println!("{}", section("💡 Next Steps"));
                println!();
                println!("  Run \x1b[38;5;159mannactl update --install\x1b[0m to install the update");
                println!("  \x1b[1m(No sudo required - daemon handles update as root!)\x1b[0m");
                if let Some(ref url) = release_notes {
                    println!("  Or visit \x1b[38;5;159m{}\x1b[0m to see what's new", url);
                }
                println!();
            }
        }
        Err(e) => {
            // Check if it's just "no release assets" vs a real network error
            let err_msg = e.to_string();
            if err_msg.contains("binary not found in release") || err_msg.contains("No assets") {
                println!("{}", beautiful::status(
                    Level::Success,
                    "No updates available - you're on the latest development version!"
                ));
                println!();
                println!("{}", beautiful::status(
                    Level::Info,
                    "Check https://github.com/jjgarcianorway/anna-assistant/releases for new releases"
                ));
            } else {
                println!("{}", beautiful::status(
                    Level::Error,
                    &format!("Could not check for updates: {}", e)
                ));
                println!();
                println!("{}", beautiful::status(
                    Level::Info,
                    "Check your internet connection and try again"
                ));
            }
        }
        _ => {
            println!("{}", beautiful::status(
                Level::Error,
                "Unexpected response from daemon"
            ));
        }
    }

    Ok(())
}

/// List all rollbackable actions (Beta.91)
pub async fn rollback_list() -> Result<()> {
    use anna_common::beautiful::{header, section, kv};
    use anna_common::ipc::{Method, ResponseData};

    println!("{}", header("Rollbackable Actions"));
    println!();

    let mut client = match crate::rpc_client::RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Cannot connect to daemon"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start daemon with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    match client.call(Method::ListRollbackable).await {
        Ok(ResponseData::RollbackableActions(actions)) => {
            if actions.is_empty() {
                println!("{}", beautiful::status(Level::Info, "No rollbackable actions found"));
                println!();
                println!("  Actions become rollbackable after you apply advice.");
                println!("  Use \x1b[38;5;159mannactl apply\x1b[0m to apply recommendations.");
                return Ok(());
            }

            println!("{}", section(&format!("Found {} Rollbackable Actions", actions.len())));
            println!();

            for (i, action) in actions.iter().enumerate() {
                println!("  \x1b[1m{}. {}\x1b[0m", i + 1, action.title);
                println!("     {}", kv("ID", &action.advice_id));
                println!("     {}", kv("Executed", &action.executed_at));
                println!("     {}", kv("Command", &action.command));
                if let Some(ref rollback_cmd) = action.rollback_command {
                    println!("     \x1b[38;5;120m✓ Rollback:\x1b[0m {}", rollback_cmd);
                } else {
                    println!("     \x1b[38;5;196m✗ Cannot rollback:\x1b[0m {}",
                        action.rollback_unavailable_reason.as_ref().unwrap_or(&"Unknown".to_string()));
                }
                println!();
            }

            println!("{}", section("How to Rollback"));
            println!();
            println!("  \x1b[38;5;159mannactl rollback action <advice-id>\x1b[0m");
            println!("  \x1b[38;5;159mannactl rollback last [N]\x1b[0m");
            println!();
        }
        Ok(_) => {
            println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
        }
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Failed to list rollbackable actions: {}", e)));
        }
    }

    Ok(())
}

/// Rollback a specific action by advice ID (Beta.91)
pub async fn rollback_action(advice_id: &str, dry_run: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};
    use anna_common::ipc::{Method, ResponseData};

    println!("{}", header(&format!("Rollback: {}", advice_id)));
    println!();

    let mut client = match crate::rpc_client::RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Cannot connect to daemon"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start daemon with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    if dry_run {
        println!("{}", beautiful::status(Level::Info, "DRY RUN - No changes will be made"));
        println!();
    }

    match client.call(Method::RollbackAction {
        advice_id: advice_id.to_string(),
        dry_run,
    }).await {
        Ok(ResponseData::RollbackResult { success, message, actions_reversed }) => {
            if success {
                println!("{}", beautiful::status(Level::Success, &message));
                println!();
                if !dry_run && !actions_reversed.is_empty() {
                    println!("{}", section("Actions Rolled Back"));
                    for id in actions_reversed {
                        println!("  \x1b[38;5;120m✓\x1b[0m {}", id);
                    }
                    println!();
                }
            } else {
                println!("{}", beautiful::status(Level::Error, &message));
            }
        }
        Ok(_) => {
            println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
        }
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Rollback failed: {}", e)));
        }
    }

    Ok(())
}

/// Rollback last N actions (Beta.91)
pub async fn rollback_last(count: usize, dry_run: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};
    use anna_common::ipc::{Method, ResponseData};

    println!("{}", header(&format!("Rollback Last {} Action{}", count, if count == 1 { "" } else { "s" })));
    println!();

    let mut client = match crate::rpc_client::RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Cannot connect to daemon"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start daemon with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    if dry_run {
        println!("{}", beautiful::status(Level::Info, "DRY RUN - No changes will be made"));
        println!();
    }

    match client.call(Method::RollbackLast { count, dry_run }).await {
        Ok(ResponseData::RollbackResult { success, message, actions_reversed }) => {
            if success {
                println!("{}", beautiful::status(Level::Success, &message));
                println!();
                if !dry_run && !actions_reversed.is_empty() {
                    println!("{}", section("Actions Rolled Back"));
                    for id in actions_reversed {
                        println!("  \x1b[38;5;120m✓\x1b[0m {}", id);
                    }
                    println!();
                }
            } else {
                println!("{}", beautiful::status(Level::Error, &message));
            }
        }
        Ok(_) => {
            println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
        }
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Rollback failed: {}", e)));
        }
    }

    Ok(())
}

/// Manage ignore filters
async fn direct_update_fallback(current_version: &str) -> Result<()> {
    use std::process::Command;
    
    println!("{}", section("🔄 Direct Update Mode"));
    println!();
    println!("  This will download and install Anna directly without the daemon.");
    println!();
    
    // Check for updates via GitHub API
    println!("{}", beautiful::status(Level::Info, "Checking GitHub for latest release..."));
    
    let update_info = match anna_common::updater::check_for_updates().await {
        Ok(info) => info,
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Failed to check for updates: {}", e)));
            return Ok(());
        }
    };
    
    if !update_info.is_update_available {
        println!("{}", beautiful::status(Level::Success, &format!("Already on latest version: {}", current_version)));
        return Ok(());
    }
    
    println!();
    println!("{}", beautiful::status(Level::Warning, &format!("Update available: {} → {}", 
        update_info.current_version, update_info.latest_version)));
    println!();
    
    // Show release info
    println!("{}", section("📦 Release Information"));
    println!("  {}", kv("Current", &update_info.current_version));
    println!("  {}", kv("Latest", &update_info.latest_version));
    if !update_info.release_notes_url.is_empty() {
        println!("  {}", kv("Release Page", &update_info.release_notes_url));
    }
    println!();
    
    // Confirmation
    use std::io::{self, Write};
    print!("  \x1b[1;93mProceed with direct update? (y/N):\x1b[0m ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input != "y" && input != "yes" {
        println!();
        println!("{}", beautiful::status(Level::Info, "Update cancelled"));
        return Ok(());
    }
    
    println!();
    println!("{}", beautiful::status(Level::Info, "Downloading binaries..."));

    // Download URLs (GitHub tags have "v" prefix)
    let base_url = format!("https://github.com/jjgarcianorway/anna-assistant/releases/download/v{}",
        update_info.latest_version);
    let annad_url = format!("{}/annad-x86_64-linux", base_url);
    let annactl_url = format!("{}/annactl-x86_64-linux", base_url);
    
    // Download to /tmp
    let tmp_annad = "/tmp/annad-new";
    let tmp_annactl = "/tmp/annactl-new";
    
    // Use curl if available, otherwise wget
    let download_cmd = if Command::new("curl").arg("--version").output().is_ok() {
        "curl"
    } else if Command::new("wget").arg("--version").output().is_ok() {
        "wget"
    } else {
        println!("{}", beautiful::status(Level::Error, "Neither curl nor wget found. Please install one."));
        return Ok(());
    };
    
    // Download annad
    let download_result = if download_cmd == "curl" {
        Command::new("curl")
            .args(&["-fsSL", "-o", tmp_annad, &annad_url])
            .status()
    } else {
        Command::new("wget")
            .args(&["-q", "-O", tmp_annad, &annad_url])
            .status()
    };
    
    if !download_result?.success() {
        println!("{}", beautiful::status(Level::Error, "Failed to download annad"));
        return Ok(());
    }
    
    // Download annactl
    let download_result = if download_cmd == "curl" {
        Command::new("curl")
            .args(&["-fsSL", "-o", tmp_annactl, &annactl_url])
            .status()
    } else {
        Command::new("wget")
            .args(&["-q", "-O", tmp_annactl, &annactl_url])
            .status()
    };
    
    if !download_result?.success() {
        println!("{}", beautiful::status(Level::Error, "Failed to download annactl"));
        return Ok(());
    }
    
    // Make executable
    Command::new("chmod")
        .args(&["+x", tmp_annad, tmp_annactl])
        .status()?;
    
    println!("{}", beautiful::status(Level::Success, "Downloaded successfully"));
    println!();
    println!("{}", beautiful::status(Level::Info, "Installing (requires sudo)..."));
    println!();
    
    // Install with sudo
    let install_result = Command::new("sudo")
        .args(&["mv", tmp_annad, "/usr/local/bin/annad"])
        .status()?;
    
    if !install_result.success() {
        println!("{}", beautiful::status(Level::Error, "Failed to install annad"));
        return Ok(());
    }
    
    let install_result = Command::new("sudo")
        .args(&["mv", tmp_annactl, "/usr/local/bin/annactl"])
        .status()?;
    
    if !install_result.success() {
        println!("{}", beautiful::status(Level::Error, "Failed to install annactl"));
        return Ok(());
    }
    
    println!("{}", beautiful::status(Level::Success, "✓ Update installed successfully!"));
    println!();
    println!("{}", section("🎉 Update Complete"));
    println!();
    println!("  Restart the daemon to use the new version:");
    println!("  \x1b[96msudo systemctl restart annad\x1b[0m");
    println!();
    println!("  Verify the update:");
    println!("  \x1b[96mannactl --version\x1b[0m");
    println!();
    
    Ok(())
}
