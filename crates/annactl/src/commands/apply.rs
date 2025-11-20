//! Apply command - apply recommendations

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;
use super::utils::parse_number_ranges;

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

/// Apply all advice in a bundle with dependency resolution
async fn apply_bundle(client: &mut RpcClient, bundle_name: &str, dry_run: bool) -> Result<()> {
    println!("{}", header(&format!("Installing Bundle: {}", bundle_name)));
    println!();

    // Get all advice
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok();
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| "bash".to_string())
        .split('/')
        .last()
        .unwrap_or("bash")
        .to_string();

    let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("wayland".to_string())
    } else if std::env::var("DISPLAY").is_ok() {
        Some("x11".to_string())
    } else {
        None
    };

    let result = client
        .call(Method::GetAdviceWithContext {
            username,
            desktop_env,
            shell,
            display_server,
        })
        .await?;

    if let ResponseData::Advice(advice_list) = result {
        // Find all advice in this bundle
        let bundle_advice: Vec<_> = advice_list
            .iter()
            .filter(|a| a.bundle.as_ref().map(|b| b == bundle_name).unwrap_or(false))
            .collect();

        if bundle_advice.is_empty() {
            println!("{}", beautiful::status(Level::Error, &format!("Bundle '{}' not found", bundle_name)));
            println!();
            println!("  Use \x1b[38;5;159mannactl setup\x1b[0m to see available bundles.");
            return Ok(());
        }

        // Sort by dependencies (topological sort)
        let sorted = topological_sort(&bundle_advice);

        println!("{}", section("📦 Bundle Contents"));
        println!();
        println!("  Will install {} item(s) in dependency order:", sorted.len());
        println!();

        for (i, advice) in sorted.iter().enumerate() {
            let num = format!("{}.", i + 1);
            println!("    \x1b[90m{:>3}\x1b[0m  \x1b[97m{}\x1b[0m", num, advice.title);
            if !advice.depends_on.is_empty() {
                println!("         \x1b[90m↳ Depends on: {}\x1b[0m", advice.depends_on.join(", "));
            }
        }
        println!();

        if dry_run {
            println!("{}", beautiful::status(Level::Info, "Dry run - no changes made"));
            return Ok(());
        }

        // Confirmation before installing bundle
        use std::io::{self, Write};
        print!("  \x1b[1;93mProceed with installing this bundle? (y/N):\x1b[0m ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!();
            println!("{}", beautiful::status(Level::Info, "Cancelled by user"));
            return Ok(());
        }

        // Apply each in order
        println!();
        println!("{}", section("🚀 Installing"));
        println!();

        let mut installed_items: Vec<String> = Vec::new();
        let mut installation_status = anna_common::BundleStatus::Completed;

        for (i, advice) in sorted.iter().enumerate() {
            println!("  [{}/{}] \x1b[1m{}\x1b[0m", i + 1, sorted.len(), advice.title);

            // Show command IMMEDIATELY (Beta.110) - user needs to see activity!
            if let Some(ref cmd) = advice.command {
                println!("  \x1b[90m→ Executing: {}\x1b[0m", cmd);
            }
            println!();

            // ENABLE STREAMING - Show EXACTLY what's happening!
            let mut rx = client
                .call_streaming(Method::ApplyAction {
                    advice_id: advice.id.clone(),
                    dry_run: false,
                    stream: true,
                })
                .await?;

            let mut success = false;
            let mut final_message = String::new();

            // Stream live output to user - TRANSPARENCY!
            while let Some(data) = rx.recv().await {
                match data {
                    ResponseData::StreamChunk { chunk_type: _, data: chunk } => {
                        // Show live command output
                        print!("{}", chunk);
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();
                    }
                    ResponseData::StreamEnd { success: s, message } => {
                        success = s;
                        final_message = message;
                        break;
                    }
                    _ => {}
                }
            }

            println!();

            if success {
                println!("         \x1b[92m✓\x1b[0m {}", final_message);
                installed_items.push(advice.id.clone());
            } else {
                println!("         \x1b[91m✗\x1b[0m {}", final_message);
                println!();
                println!("{}", beautiful::status(Level::Error, "Bundle installation failed"));
                println!("  Some items may have been installed before the failure.");
                installation_status = if installed_items.is_empty() {
                    anna_common::BundleStatus::Failed
                } else {
                    anna_common::BundleStatus::Partial
                };
                break;
            }
        }

        // Record installation in bundle history
        let mut history = anna_common::BundleHistory::load().unwrap_or_default();
        let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        history.add_entry(anna_common::BundleHistoryEntry {
            bundle_name: bundle_name.to_string(),
            installed_items: installed_items.clone(),
            installed_at: chrono::Utc::now(),
            installed_by: username,
            status: installation_status,
            rollback_available: installation_status == anna_common::BundleStatus::Completed,
        });

        // Try to save bundle history, but don't show error if permission denied (Beta.109)
        // History file is in /var/lib/anna/ (root-only), this will be fixed in future versions
        let _ = history.save(); // Silently ignore permission errors

        if installation_status == anna_common::BundleStatus::Completed {
            println!();
            println!("{}", beautiful::status(Level::Success, &format!("Bundle '{}' installed successfully!", bundle_name)));
            println!("  {} item(s) installed and tracked for rollback", installed_items.len());
        }

        println!();

    } else {
        println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
    }

    Ok(())
}

pub async fn apply(id: Option<String>, nums: Option<String>, bundle: Option<String>, auto: bool, dry_run: bool) -> Result<()> {
    println!("{}", header("Apply Recommendations"));
    println!();

    // Connect to daemon
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!(
                "{}",
                beautiful::status(Level::Error, "Daemon not running")
            );
            println!();
            println!(
                "{}",
                beautiful::status(Level::Info, "Start with: sudo systemctl start annad")
            );
            return Ok(());
        }
    };

    // If bundle provided, apply all advice in the bundle
    if let Some(bundle_name) = bundle {
        return apply_bundle(&mut client, &bundle_name, dry_run).await;
    }

    // If specific ID provided, apply that one
    if let Some(advice_id) = id {
        // Get advice details first to show what will be applied
        let advice_data = client.call(Method::GetAdvice).await?;
        if let ResponseData::Advice(advice_list) = advice_data {
            if let Some(advice) = advice_list.iter().find(|a| a.id == advice_id) {
                // Show preview
                println!("{}", beautiful::status(Level::Info, "Preview:"));
                println!();
                println!("  \x1b[1m{}\x1b[0m", advice.title);
                println!("  {}", advice.reason);
                if let Some(cmd) = &advice.command {
                    println!();
                    println!("  \x1b[90mCommand: {}\x1b[0m", cmd);
                }
                println!("  \x1b[90mRisk: {:?} | Category: {}\x1b[0m", advice.risk, advice.category);
                println!();

                // Confirmation (unless auto or dry-run)
                if !auto && !dry_run {
                    use std::io::{self, Write};
                    print!("  \x1b[1;93mProceed? (y/N):\x1b[0m ");
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let input = input.trim().to_lowercase();

                    if input != "y" && input != "yes" {
                        println!();
                        println!("{}", beautiful::status(Level::Info, "Cancelled by user"));
                        return Ok(());
                    }
                }
                println!();
            }
        }

        println!(
            "{}",
            beautiful::status(
                Level::Info,
                &format!("Applying advice: {}", advice_id)
            )
        );

        let result = client
            .call(Method::ApplyAction {
                advice_id: advice_id.clone(),
                dry_run,
                stream: false,
            })
            .await?;

        if let ResponseData::ActionResult { success, message } = result {
            if success {
                println!("{}", beautiful::status(Level::Success, &message));

                // Record application in feedback log
                let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
                // Get advice details to know the category
                let advice_data = client.call(Method::GetAdvice).await?;
                if let ResponseData::Advice(advice_list) = advice_data {
                    if let Some(advice) = advice_list.iter().find(|a| a.id == advice_id) {
                        let mut log = anna_common::UserFeedbackLog::load().unwrap_or_default();
                        log.record(anna_common::FeedbackEvent {
                            advice_id: advice_id.clone(),
                            advice_category: advice.category.clone(),
                            event_type: anna_common::FeedbackType::Applied,
                            timestamp: chrono::Utc::now(),
                            username,
                        });
                        let _ = log.save(); // Ignore errors
                    }
                }

                // Invalidate cache after successful apply
                if !dry_run {
                    let _ = anna_common::AdviceDisplayCache::invalidate();
                    println!();
                    println!("{}", beautiful::status(Level::Info,
                        "Tip: Run 'annactl advise' to see updated recommendations"));
                }
            } else {
                println!("{}", beautiful::status(Level::Error, &message));
            }
        }

        return Ok(());
    }

    // If nums provided, apply by index OR ID
    if let Some(nums_str) = nums {
        // RC.9.3: Check if input is an ID (non-numeric) or number(s)
        // If it looks like an ID (contains letters), route to ID path
        // NOTE: Check for alphabetic FIRST - "1-5" is a range, "amd-microcode" is an ID
        if nums_str.contains(|c: char| c.is_alphabetic()) {
            // It's an ID like "amd-microcode" - route to ID handler (use Box::pin to avoid infinite size)
            return Box::pin(apply(Some(nums_str), None, None, auto, dry_run)).await;
        }

        println!("{}", beautiful::status(Level::Info,
            "Applying by number - loading cache..."));
        println!();

        // Load the cached display order
        let cache = match anna_common::AdviceDisplayCache::load() {
            Ok(c) => c,
            Err(e) => {
                println!("{}", beautiful::status(Level::Error, &format!("Cache error: {}", e)));
                println!();
                println!("Run '\x1b[38;5;159mannactl advise\x1b[0m' first to refresh the list.");
                return Ok(());
            }
        };

        // Parse the nums string (e.g., "1,3,5-7")
        let indices = parse_number_ranges(&nums_str)?;

        // Show what WILL be applied
        println!("{}", beautiful::status(Level::Info, "Items to be applied:"));
        println!();

        let mut valid_ids = Vec::new();
        for idx in &indices {
            if let Some(advice_id) = cache.get_id_by_number(*idx) {
                println!("   \x1b[1m{}. {}\x1b[0m", idx, advice_id);
                valid_ids.push(advice_id.to_string());
            } else {
                println!("   \x1b[91m❌ Number {} is out of range (1-{})\x1b[0m", idx, cache.len());
            }
        }
        if valid_ids.is_empty() {
            println!("{}", beautiful::status(Level::Error, "No valid items to apply"));
            return Ok(());
        }

        // Require confirmation unless dry-run
        if !dry_run {
            println!();
            use std::io::{self, Write};
            print!("   \x1b[1;93mProceed with applying these items? (y/N):\x1b[0m ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                println!();
                println!("{}", beautiful::status(Level::Info, "Cancelled by user"));
                return Ok(());
            }
            println!();
        }

        // Apply each advice by ID
        let mut success_count = 0;
        let mut fail_count = 0;

        for advice_id in valid_ids {
            println!("{}", beautiful::status(Level::Info,
                &format!("Applying: {}", advice_id)));

            let result = client.call(Method::ApplyAction {
                advice_id: advice_id.clone(),
                dry_run,
                stream: false,
            }).await?;

            if let ResponseData::ActionResult { success, message } = result {
                if success {
                    println!("   {}", beautiful::status(Level::Success, &message));
                    success_count += 1;

                    // Record application in feedback log
                    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
                    // Get advice details to know the category
                    let advice_data = client.call(Method::GetAdvice).await?;
                    if let ResponseData::Advice(advice_list) = advice_data {
                        if let Some(advice) = advice_list.iter().find(|a| a.id == advice_id) {
                            let mut log = anna_common::UserFeedbackLog::load().unwrap_or_default();
                            log.record(anna_common::FeedbackEvent {
                                advice_id: advice_id.clone(),
                                advice_category: advice.category.clone(),
                                event_type: anna_common::FeedbackType::Applied,
                                timestamp: chrono::Utc::now(),
                                username,
                            });
                            let _ = log.save(); // Ignore errors
                        }
                    }
                } else {
                    println!("   {}", beautiful::status(Level::Error, &message));
                    fail_count += 1;
                }
            }
            println!();
        }

        println!();
        println!("{}", beautiful::status(Level::Info,
            &format!("Applied {} successfully, {} failed", success_count, fail_count)));

        // Invalidate cache after applying to force regeneration on next advise
        if success_count > 0 && !dry_run {
            let _ = anna_common::AdviceDisplayCache::invalidate();
            println!();
            println!("{}", beautiful::status(Level::Info,
                "Tip: Run 'annactl advise' to see updated recommendations"));
        }

        return Ok(());
    }

    // Auto mode not yet implemented
    if auto {
        println!(
            "{}",
            beautiful::status(Level::Warning, "Auto-apply not yet implemented")
        );
        println!(
            "{}",
            beautiful::status(
                Level::Info,
                "Use --id <advice-id> or --nums <numbers> to apply recommendations"
            )
        );
        return Ok(());
    }

    // Show usage
    println!(
        "{}",
        beautiful::status(
            Level::Info,
            "Use --id <advice-id> to apply a specific recommendation by ID"
        )
    );
    println!(
        "{}",
        beautiful::status(Level::Info, "Use --nums <range> to apply by number (e.g., 1, 1-5, 1,3,5-7)")
    );
    println!(
        "{}",
        beautiful::status(Level::Info, "Use --dry-run to see what would happen")
    );
    println!();
    println!("{}", beautiful::status(Level::Info, "Examples:"));
    println!("  annactl apply --id orphan-packages --dry-run");
    println!("  annactl apply --nums 1");
    println!("  annactl apply --nums 1-5");
    println!("  annactl apply --nums 1,3,5-7");

    Ok(())
}
