//! Ignore command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn ignore(action: crate::IgnoreAction) -> Result<()> {
    use anna_common::beautiful::{header, section};
    use crate::IgnoreAction;

    match action {
        IgnoreAction::Show => {
            println!("{}", header("Ignore Filters"));
            println!();

            let filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            println!("{}", section("Current Filters"));
            println!();
            println!("{}", filters.get_ignored_summary());
            println!();

            if !filters.ignored_categories.is_empty() || !filters.ignored_priorities.is_empty() {
                println!("{}", section("Commands"));
                println!();
                println!("  annactl ignore list-hidden                # Show hidden recommendations");
                println!("  annactl ignore unignore category <name>   # Remove category filter");
                println!("  annactl ignore unignore priority <level>  # Remove priority filter");
                println!("  annactl ignore reset                       # Clear all filters");
                println!();
            }
        }

        IgnoreAction::ListHidden => {
            println!("{}", header("Hidden Recommendations"));
            println!();

            // Load ignore filters
            let filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            if filters.ignored_categories.is_empty() && filters.ignored_priorities.is_empty() {
                println!("{}", beautiful::status(Level::Info, "No ignore filters active"));
                println!();
                println!("Use 'annactl ignore category <name>' or 'annactl ignore priority <level>' to ignore items");
                return Ok(());
            }

            // Connect to daemon
            let mut client = match RpcClient::connect().await {
                Ok(c) => c,
                Err(_) => {
                    println!("{}", beautiful::status(Level::Error, "Daemon not running"));
                    println!();
                    println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
                    return Ok(());
                }
            };

            // Get all advice
            let advice_data = client.call(Method::GetAdvice).await?;
            if let ResponseData::Advice(advice_list) = advice_data {
                // Filter to only show items that ARE filtered (inverse logic)
                let hidden_items: Vec<_> = advice_list.iter()
                    .filter(|a| filters.should_filter(a))
                    .collect();

                if hidden_items.is_empty() {
                    println!("{}", beautiful::status(Level::Info, "No recommendations are currently hidden"));
                    println!();
                    return Ok(());
                }

                println!("{}", beautiful::status(Level::Info,
                    &format!("{} recommendation{} currently hidden by your filters",
                        hidden_items.len(),
                        if hidden_items.len() == 1 { " is" } else { "s are" })));
                println!();

                // Group by category
                let mut by_category: std::collections::HashMap<String, Vec<&anna_common::Advice>> =
                    std::collections::HashMap::new();

                for advice in &hidden_items {
                    by_category.entry(advice.category.clone())
                        .or_insert_with(Vec::new)
                        .push(advice);
                }

                // Display by category
                for (category, items) in by_category.iter() {
                    let category_emoji = anna_common::get_category_emoji(category);
                    println!("{}", section(&format!("{} {}", category_emoji, category)));

                    for advice in items.iter().take(5) {
                        let priority_color = match advice.priority {
                            anna_common::Priority::Mandatory => "\x1b[91m",
                            anna_common::Priority::Recommended => "\x1b[93m",
                            anna_common::Priority::Optional => "\x1b[96m",
                            anna_common::Priority::Cosmetic => "\x1b[90m",
                        };

                        println!("  {} {}{:?}\x1b[0m - {}",
                            "•",
                            priority_color,
                            advice.priority,
                            advice.title);
                    }

                    if items.len() > 5 {
                        println!("  \x1b[90m... and {} more\x1b[0m", items.len() - 5);
                    }
                    println!();
                }

                // Show un-ignore commands
                println!("{}", section("To Un-ignore"));

                if !filters.ignored_categories.is_empty() {
                    println!("Categories:");
                    for cat in &filters.ignored_categories {
                        println!("  annactl ignore unignore category \"{}\"", cat);
                    }
                    println!();
                }

                if !filters.ignored_priorities.is_empty() {
                    println!("Priorities:");
                    for pri in &filters.ignored_priorities {
                        println!("  annactl ignore unignore priority {:?}", pri);
                    }
                    println!();
                }

                println!("Or reset all filters:");
                println!("  annactl ignore reset");
                println!();
            }
        }

        IgnoreAction::Category { name } => {
            let mut filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            filters.ignore_category(&name);
            filters.save()?;

            println!("{}", beautiful::status(Level::Success,
                &format!("Category '{}' is now ignored", name)));
            println!();
            println!("Run 'annactl advise' to see updated recommendations");
        }

        IgnoreAction::Priority { level } => {
            use anna_common::Priority;

            let priority = match level.to_lowercase().as_str() {
                "mandatory" => Priority::Mandatory,
                "recommended" => Priority::Recommended,
                "optional" => Priority::Optional,
                "cosmetic" => Priority::Cosmetic,
                _ => {
                    println!("{}", beautiful::status(Level::Error,
                        &format!("Unknown priority level: {}", level)));
                    println!();
                    println!("Valid levels: Mandatory, Recommended, Optional, Cosmetic");
                    return Ok(());
                }
            };

            let mut filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            filters.ignore_priority(priority);
            filters.save()?;

            println!("{}", beautiful::status(Level::Success,
                &format!("Priority '{}' is now ignored", level)));
            println!();
            println!("Run 'annactl advise' to see updated recommendations");
        }

        IgnoreAction::Unignore { filter_type, value } => {
            let mut filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            match filter_type.to_lowercase().as_str() {
                "category" => {
                    filters.unignore_category(&value);
                    filters.save()?;

                    println!("{}", beautiful::status(Level::Success,
                        &format!("Category '{}' is no longer ignored", value)));
                }
                "priority" => {
                    use anna_common::Priority;

                    let priority = match value.to_lowercase().as_str() {
                        "mandatory" => Priority::Mandatory,
                        "recommended" => Priority::Recommended,
                        "optional" => Priority::Optional,
                        "cosmetic" => Priority::Cosmetic,
                        _ => {
                            println!("{}", beautiful::status(Level::Error,
                                &format!("Unknown priority level: {}", value)));
                            return Ok(());
                        }
                    };

                    filters.unignore_priority(priority);
                    filters.save()?;

                    println!("{}", beautiful::status(Level::Success,
                        &format!("Priority '{}' is no longer ignored", value)));
                }
                _ => {
                    println!("{}", beautiful::status(Level::Error,
                        &format!("Unknown filter type: {}", filter_type)));
                    println!();
                    println!("Valid types: category, priority");
                    return Ok(());
                }
            }

            println!();
            println!("Run 'annactl advise' to see updated recommendations");
        }

        IgnoreAction::Reset => {
            let mut filters = anna_common::IgnoreFilters::load().unwrap_or_default();

            filters.reset_all();
            filters.save()?;

            println!("{}", beautiful::status(Level::Success,
                "All ignore filters have been cleared"));
            println!();
            println!("Run 'annactl advise' to see all recommendations");
        }
    }

    Ok(())
}

/// Direct update fallback when daemon is unavailable (RC.7)
/// Uses curl/wget to download and install binaries directly
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
