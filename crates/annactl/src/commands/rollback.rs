//! Rollback command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn rollback_bundle(bundle_identifier: &str, dry_run: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    // Load bundle history
    let history = match anna_common::BundleHistory::load() {
        Ok(h) => h,
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Failed to load bundle history: {}", e)));
            return Ok(());
        }
    };

    // Check if identifier is a number (#1, #2, etc.)
    let entry = if bundle_identifier.starts_with('#') {
        // Parse the number
        let num_str = &bundle_identifier[1..];
        match num_str.parse::<usize>() {
            Ok(num) if num > 0 => {
                // Get installed bundles in order
                let installed: Vec<_> = history.entries.iter()
                    .filter(|e| e.status == anna_common::BundleStatus::Completed && e.rollback_available)
                    .collect();

                if num > installed.len() {
                    println!("{}", beautiful::status(Level::Error, &format!("Bundle #{} not found", num)));
                    println!();
                    println!("  Use \x1b[38;5;159mannactl setup\x1b[0m to see installed bundles.");
                    return Ok(());
                }

                installed[num - 1]
            }
            _ => {
                println!("{}", beautiful::status(Level::Error, &format!("Invalid bundle number: {}", bundle_identifier)));
                println!();
                println!("  Use \x1b[38;5;159mannactl setup\x1b[0m to see installed bundles with numbers.");
                return Ok(());
            }
        }
    } else {
        // Look up by name
        match history.get_latest(bundle_identifier) {
            Some(e) => e,
            None => {
                println!("{}", beautiful::status(Level::Error, &format!("No installation history found for bundle '{}'", bundle_identifier)));
                println!();
                println!("  This bundle was never installed or the history was cleared.");
                println!("  Use \x1b[38;5;159mannactl setup\x1b[0m to see available bundles.");
                return Ok(());
            }
        }
    };

    println!("{}", header(&format!("Rolling Back Bundle: {}", entry.bundle_name)));
    println!();

    if !entry.rollback_available {
        println!("{}", beautiful::status(Level::Warning, "This bundle installation cannot be rolled back"));
        println!("  The installation was incomplete or failed.");
        return Ok(());
    }

    println!("{}", section("📋 Bundle Information"));
    println!();
    println!("  Bundle: \x1b[1m{}\x1b[0m", entry.bundle_name);
    println!("  Installed: {} by {}", entry.installed_at.format("%Y-%m-%d %H:%M:%S"), entry.installed_by);
    println!("  Items: {} package(s)", entry.installed_items.len());
    println!();

    println!("{}", section("🗑️  Items to Remove"));
    println!();
    println!("  Will remove {} item(s) in reverse order:", entry.installed_items.len());
    println!();

    // Display items in reverse order
    for (i, item_id) in entry.installed_items.iter().rev().enumerate() {
        let num = format!("{}.", i + 1);
        println!("    \x1b[90m{:>3}\x1b[0m  \x1b[97m{}\x1b[0m", num, item_id);
    }
    println!();

    if dry_run {
        println!("{}", beautiful::status(Level::Info, "Dry run - no changes made"));
        return Ok(());
    }

    // Warning prompt
    println!("{}", beautiful::status(Level::Warning, "This will remove all packages installed by this bundle"));
    println!("  Press Enter to continue or Ctrl+C to cancel...");
    println!();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Connect to daemon (not currently used but validates daemon is running)
    let _client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Daemon not running"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    println!("{}", section("🧹 Removing"));
    println!();

    let mut removed_count = 0;

    // Remove in reverse order
    for (i, item_id) in entry.installed_items.iter().rev().enumerate() {
        println!("  [{}/{}] Removing \x1b[1m{}\x1b[0m...", i + 1, entry.installed_items.len(), item_id);

        // For now, we need to figure out the package name from the advice ID
        // Most advice IDs follow the pattern like "python-lsp" or "docker-install"
        // We'll need to query the daemon for the actual package name

        // This is a simplified removal - in reality we'd need to track the exact
        // package names that were installed
        let package_name = item_id.trim_end_matches("-install");

        let remove_result = std::process::Command::new("sudo")
            .args(&["pacman", "-R", "--noconfirm", package_name])
            .output();

        match remove_result {
            Ok(output) if output.status.success() => {
                println!("         \x1b[92m✓\x1b[0m Removed");
                removed_count += 1;
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("not found") {
                    println!("         \x1b[93m⊘\x1b[0m Already removed or not installed");
                } else {
                    println!("         \x1b[91m✗\x1b[0m Failed: {}", stderr.trim());
                }
            }
            Err(e) => {
                println!("         \x1b[91m✗\x1b[0m Error: {}", e);
            }
        }
    }

    println!();
    if removed_count > 0 {
        println!("{}", beautiful::status(Level::Success, &format!("Rolled back '{}' - {} item(s) removed", entry.bundle_name, removed_count)));
    } else {
        println!("{}", beautiful::status(Level::Info, "No items were removed"));
    }
    println!();

    Ok(())
}

/// Topological sort for dependency resolution
fn topological_sort(advice: &[&anna_common::Advice]) -> Vec<anna_common::Advice> {
    use std::collections::{HashMap, VecDeque};

    // Build adjacency list and in-degree map
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut id_to_advice: HashMap<String, anna_common::Advice> = HashMap::new();

    for item in advice {
        id_to_advice.insert(item.id.clone(), (*item).clone());
        graph.entry(item.id.clone()).or_insert_with(Vec::new);
        in_degree.entry(item.id.clone()).or_insert(0);

        for dep in &item.depends_on {
            graph.entry(dep.clone()).or_insert_with(Vec::new).push(item.id.clone());
            *in_degree.entry(item.id.clone()).or_insert(0) += 1;
        }
    }

    // Kahn's algorithm for topological sort
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut result: Vec<anna_common::Advice> = Vec::new();

    // Find all nodes with in-degree 0
    for (id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(id.clone());
        }
    }

    while let Some(id) = queue.pop_front() {
        if let Some(advice) = id_to_advice.get(&id) {
            result.push(advice.clone());
        }

        if let Some(neighbors) = graph.get(&id) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    result
}

// Additional rollback functions (lines 3636-3800)
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
