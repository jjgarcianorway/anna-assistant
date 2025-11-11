//! CLI command implementations

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, kv, section, Level, Priority};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::rpc_client::RpcClient;

/// Check for updates and show notification banner (non-spammy, once per day)
async fn check_and_notify_updates() {
    // Cache file to track last check
    let cache_file = PathBuf::from("/tmp/anna-update-check");

    // Check if we already checked today
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(last_check) = content.trim().parse::<u64>() {
            // If checked within last 24 hours, skip
            if now - last_check < 86400 {
                return;
            }
        }
    }

    // IMMEDIATELY update cache to prevent spam on failures
    let _ = std::fs::write(&cache_file, now.to_string());

    // Check for updates (silently fail if network issue)
    if let Ok(update_info) = anna_common::updater::check_for_updates().await {
        if update_info.is_update_available {
            // Show update banner
            println!();
            println!("\x1b[38;5;226m╭─────────────────────────────────────────────────────────────╮\x1b[0m");
            println!("\x1b[38;5;226m│\x1b[0m  \x1b[1m📦 Update Available\x1b[0m: {} → {}  \x1b[38;5;226m│\x1b[0m",
                update_info.current_version, update_info.latest_version);
            println!("\x1b[38;5;226m│\x1b[0m  Run \x1b[38;5;159mannactl update --install\x1b[0m to upgrade                 \x1b[38;5;226m│\x1b[0m");
            println!("\x1b[38;5;226m╰─────────────────────────────────────────────────────────────╯\x1b[0m");
        }
    }
    // If check fails, we already updated cache, so won't spam
}

pub async fn status() -> Result<()> {
    // RC.9: Show immediate feedback - don't make users wait in silence
    eprint!("\r{}", beautiful::status(Level::Info, "Checking daemon status..."));
    use std::io::Write;
    let _ = std::io::stderr().flush();

    // Try to connect to daemon
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            // Clear the "Checking..." message
            eprint!("\r\x1b[K");
            let _ = std::io::stderr().flush();

            println!("{}", header("Anna Status"));
            println!();
            println!(
                "{}",
                beautiful::status(Level::Error, "Daemon not running")
            );
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    // RC.9.1: Status focuses ONLY on Anna's health, not recommendations
    // User feedback: "annactl status should show status of anna, not advises"
    // Recommendations moved to: annactl advise
    let status_data = client.call(Method::Status).await?;
    let facts_data = client.call(Method::GetFacts).await?;
    // Phase 0.2d: Get system state detection
    let state_data = client.call(Method::GetState).await?;

    // Clear the "Checking..." message and show header
    eprint!("\r\x1b[K");
    let _ = std::io::stderr().flush();

    println!("{}", header("Anna Status"));
    println!();

    if let (ResponseData::Status(status), ResponseData::Facts(facts))
        = (status_data, facts_data) {

        println!("{}", section("System"));
        println!("  {}", kv("Hostname", &facts.hostname));
        println!("  {}", kv("Kernel", &facts.kernel));

        // Phase 0.2d: Show detected system state
        if let ResponseData::StateDetection(state) = state_data {
            println!("  {}", kv("State", &state.state));
            if let Some(health) = state.details.health_ok {
                let health_str = if health { "OK" } else { "Degraded" };
                println!("  {}", kv("Health", health_str));
            }
        }

        println!();
        println!("{}", section("Anna Daemon"));
        println!("  {}", beautiful::status(Level::Success, "Running"));
        println!("  {}", kv("Version", &status.version));
        println!("  {}", kv("Uptime", &format!("{}s", status.uptime_seconds)));
        println!("  {}", kv("Socket", "/run/anna/anna.sock"));
        println!();

        // Show quick links to other commands
        println!("{}", section("Quick Commands"));
        println!("  {} \x1b[90m# Get recommendations\x1b[0m", "\x1b[96mannactl advise\x1b[0m");
        println!("  {} \x1b[90m# Full system report\x1b[0m", "\x1b[96mannactl report\x1b[0m");
        println!("  {} \x1b[90m# Check Anna health\x1b[0m", "\x1b[96mannactl doctor\x1b[0m");
        println!();
    }

    // Show recent activity from audit log
    if let Ok(entries) = read_recent_audit_entries(10).await {
        if !entries.is_empty() {
            println!("{}", section("Recent Activity"));
            for entry in entries.iter().take(10) {
                let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let status_icon = if entry.success {
                    "\x1b[92m✓\x1b[0m"
                } else {
                    "\x1b[91m✗\x1b[0m"
                };

                // Color the action type based on what it is
                let action_color = match entry.action_type.as_str() {
                    "apply" => "\x1b[96m", // Cyan
                    "install" => "\x1b[92m", // Green
                    "remove" => "\x1b[93m", // Yellow
                    "update" => "\x1b[95m", // Magenta
                    _ => "\x1b[0m", // Default
                };

                println!("  {} \x1b[90m{}\x1b[0m {}{}\x1b[0m",
                    status_icon,
                    time_str,
                    action_color,
                    entry.action_type
                );

                // Show details on next line, indented
                let details = if entry.details.len() > 120 {
                    format!("{}...", &entry.details[..117])
                } else {
                    entry.details.clone()
                };
                println!("      \x1b[90m{}\x1b[0m", details);
            }
            println!();
        }
    }

    // Show recent daemon logs from journalctl (Beta.90)
    if let Ok(logs) = read_recent_daemon_logs(10).await {
        if !logs.is_empty() {
            println!("{}", section("Daemon Logs"));
            for log in logs {
                println!("  {}", log);
            }
            println!();
        }
    }

    // Check for updates (non-spammy, once per day)
    check_and_notify_updates().await;

    Ok(())
}

/// Read recent daemon logs from journalctl (Beta.90)
async fn read_recent_daemon_logs(count: usize) -> Result<Vec<String>> {
    use tokio::process::Command;

    // Use journalctl to get recent annad logs
    // -u annad: only annad service
    // -n count: last N lines
    // --no-pager: don't use pager
    // -o short-iso: ISO timestamp format
    let output = Command::new("journalctl")
        .args(&[
            "-u", "annad",
            "-n", &count.to_string(),
            "--no-pager",
            "-o", "short-iso",
            "--priority=info"  // info and above (info, notice, warning, err, crit)
        ])
        .output()
        .await?;

    if !output.status.success() {
        // journalctl might fail if user doesn't have permission
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut logs = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse journalctl line: timestamp hostname service[pid]: message
        // Example: 2025-11-06T14:00:00+0000 hostname annad[1234]: Connection from UID 1000
        if let Some(message_start) = line.find("annad[") {
            if let Some(colon_pos) = line[message_start..].find(':') {
                let timestamp_end = message_start.min(24); // ISO timestamp is ~24 chars
                let timestamp = &line[..timestamp_end].trim();
                let message = &line[message_start + colon_pos + 1..].trim();

                // Format: [time] message
                // Color code based on log level
                let colored_message = if message.contains("ERROR") || message.contains("error") {
                    format!("\x1b[91m{}\x1b[0m", message) // Red
                } else if message.contains("WARN") || message.contains("warn") {
                    format!("\x1b[93m{}\x1b[0m", message) // Yellow
                } else if message.contains("INFO") || message.contains("info") {
                    format!("\x1b[96m{}\x1b[0m", message) // Cyan
                } else {
                    message.to_string()
                };

                // Truncate long messages
                let display_msg = if colored_message.len() > 100 {
                    format!("{}...", &colored_message[..97])
                } else {
                    colored_message
                };

                logs.push(format!("\x1b[90m{}\x1b[0m {}", timestamp, display_msg));
            }
        }
    }

    Ok(logs)
}

/// Read recent audit entries from the audit log
async fn read_recent_audit_entries(count: usize) -> Result<Vec<anna_common::AuditEntry>> {
    use tokio::fs;

    let audit_path = std::path::Path::new("/var/log/anna/audit.jsonl");

    if !audit_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(audit_path).await?;

    let mut entries: Vec<anna_common::AuditEntry> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Sort by timestamp (newest first)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Take the last N entries
    entries.truncate(count);

    Ok(entries)
}

pub async fn advise(
    risk_filter: Option<String>,
    mode: String,
    category_filter: Option<String>,
    limit: usize,
) -> Result<()> {
    println!("{}", header("System Recommendations"));
    println!();

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

    // Get advice with user context
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

    let advice_data = client.call(Method::GetAdviceWithContext {
        username,
        desktop_env,
        shell,
        display_server,
    }).await?;

    if let ResponseData::Advice(mut advice_list) = advice_data {
        // Track original total for display
        let total_available = advice_list.len();

        // Apply filtering based on mode
        match mode.as_str() {
            "critical" => {
                // Only show Mandatory priority items
                advice_list.retain(|a| matches!(a.priority, anna_common::Priority::Mandatory));
                println!("{}", beautiful::status(Level::Info, "Showing CRITICAL recommendations only (use --mode=all to see everything)"));
                println!();
            }
            "recommended" => {
                // Show Mandatory + Recommended
                advice_list.retain(|a| matches!(a.priority, anna_common::Priority::Mandatory | anna_common::Priority::Recommended));
                println!("{}", beautiful::status(Level::Info, "Showing CRITICAL + RECOMMENDED (use --mode=all to see everything)"));
                println!();
            }
            "smart" => {
                // Smart mode: Show high priority + limited lower priority
                // RC.9: More aggressive filtering to keep total under 50
                let mandatory: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Mandatory)).cloned().collect();
                let mut recommended: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Recommended)).cloned().collect();
                let mut optional: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Optional)).cloned().collect();
                let mut cosmetic: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Cosmetic)).cloned().collect();

                // RC.9: Limit to prevent overwhelming users (target: <50 total)
                // All Critical, top 20 Recommended, top 10 Optional, top 5 Cosmetic
                recommended.truncate(20);
                optional.truncate(10);
                cosmetic.truncate(5);

                let new_list_size = mandatory.len() + recommended.len() + optional.len() + cosmetic.len();
                let hidden_count = total_available - new_list_size;

                advice_list = mandatory;
                advice_list.extend(recommended);
                advice_list.extend(optional);
                advice_list.extend(cosmetic);

                // Remove verbose messages - user knows they can use "advise all"
            }
            "all" => {
                // Show everything
                println!("{}", beautiful::status(Level::Info, "Showing ALL recommendations"));
                println!();
            }
            _ => {
                println!("{}", beautiful::status(Level::Warning,
                    &format!("Unknown mode '{}', using 'smart'", mode)));
                println!();
            }
        }

        // Filter out dismissed advice (if learning is enabled)
        if let Ok(log) = anna_common::UserFeedbackLog::load() {
            let original_count = advice_list.len();
            advice_list.retain(|a| !log.was_dismissed(&a.id));
            let dismissed_count = original_count - advice_list.len();
            if dismissed_count > 0 {
                println!("{}", beautiful::status(Level::Info,
                    &format!("Hiding {} previously dismissed recommendation(s)", dismissed_count)));
                println!();
            }
        }

        // Apply ignore filters (categories and priorities)
        if let Ok(filters) = anna_common::IgnoreFilters::load() {
            let original_count = advice_list.len();
            advice_list.retain(|a| !filters.should_filter(a));
            let filtered_count = original_count - advice_list.len();
            if filtered_count > 0 {
                println!("{}", beautiful::status(Level::Info,
                    &format!("Hiding {} items by your ignore filters (use 'annactl ignore show' to see)", filtered_count)));
                println!();
            }
        }

        // Filter by risk level if specified
        if let Some(ref risk) = risk_filter {
            advice_list.retain(|a| {
                format!("{:?}", a.risk).to_lowercase() == risk.to_lowercase()
            });
        }

        // Filter by category if specified (unless it's "all")
        if let Some(ref cat) = category_filter {
            if cat.to_lowercase() != "all" {
                advice_list.retain(|a| a.category.to_lowercase() == cat.to_lowercase());
            }
            // If "all", don't filter - show everything
        }

        // Track count before limit
        let count_before_limit = advice_list.len();

        // Apply limit if not 0
        if limit > 0 && advice_list.len() > limit {
            advice_list.truncate(limit);
        }

        if advice_list.is_empty() {
            println!("{}", beautiful::status(Level::Success,
                "Your system looks great! I don't have any suggestions right now."));
            return Ok(());
        }

        // Group by category first
        let mut by_category: std::collections::HashMap<String, Vec<&anna_common::Advice>> =
            std::collections::HashMap::new();

        for advice in &advice_list {
            by_category.entry(advice.category.clone())
                .or_insert_with(Vec::new)
                .push(advice);
        }

        // COMPACT SUMMARY VIEW (default if no category filter)
        // RC.8: Ultra-compact (fits in 15 lines)
        if category_filter.is_none() {
            println!("{}", section("📊 Recommendations"));
            println!();

            // Show category counts (normalized and aligned)
            let mut category_counts: Vec<_> = by_category.iter()
                .map(|(cat, items)| {
                    // Normalize category names to Title Case for consistency
                    let normalized = cat.split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().chain(chars.flat_map(|c| c.to_lowercase())).collect(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    // RC.9.1: Don't truncate - user complained about "Performance & O..."
                    (normalized, items.len())
                })
                .collect();
            category_counts.sort_by(|a, b| b.1.cmp(&a.1));

            // RC.9.3: Show ALL categories (user requested: "advise should list all categories")
            for (category, count) in category_counts.iter() {
                println!("  \x1b[96m{}\x1b[0m \x1b[90m{:>2}\x1b[0m", category, count);
            }
            println!();

            // Compact usage examples (only 2 examples)
            println!("  \x1b[96mannactl advise {}\x1b[0m  \x1b[90m# view category\x1b[0m",
                category_counts[0].0.to_lowercase());
            println!("  \x1b[96mannactl advise all\x1b[0m       \x1b[90m# view all details\x1b[0m");
            println!();

            // Save to cache for apply-by-number (even though we're not showing details)
            let mut ordered_for_cache: Vec<&anna_common::Advice> = advice_list.iter().collect();
            ordered_for_cache.sort_by(|a, b| b.priority.cmp(&a.priority));

            let ids_for_cache: Vec<String> = ordered_for_cache.iter().map(|a| a.id.clone()).collect();
            let _ = anna_common::AdviceDisplayCache::save(ids_for_cache);

            return Ok(());
        }

        // DETAILED VIEW (when category filter is provided)
        println!("{}", beautiful::status(Level::Info,
            &format!("Showing {} recommendation{}", advice_list.len(), if advice_list.len() == 1 { "" } else { "s" })));
        println!();

        // Sort categories by importance (using centralized category order)
        let category_order = anna_common::get_category_order();

        // Build the complete ordered list for display AND cache
        let mut ordered_advice_list: Vec<&anna_common::Advice> = Vec::new();

        for &category in &category_order {
            if let Some(items) = by_category.get(category) {
                if items.is_empty() {
                    continue;
                }

                // Sort items within category by priority, then risk, then popularity
                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then(b.risk.cmp(&a.risk))
                        .then(b.popularity.cmp(&a.popularity))
                });

                ordered_advice_list.extend(sorted_items);
            }
        }

        // Add any remaining categories not in the predefined order
        for (category, items) in &by_category {
            if !category_order.contains(&category.as_str()) && !items.is_empty() {
                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then(b.risk.cmp(&a.risk))
                        .then(b.popularity.cmp(&a.popularity))
                });

                ordered_advice_list.extend(sorted_items);
            }
        }

        // Save the display order to cache for apply command
        let advice_ids: Vec<String> = ordered_advice_list.iter().map(|a| a.id.clone()).collect();
        if let Err(e) = anna_common::AdviceDisplayCache::save(advice_ids) {
            // Non-fatal, just warn
            eprintln!("Warning: Failed to save advice cache: {}", e);
        }

        // Now display everything
        let mut counter = 1;
        let mut current_category = String::new();

        for advice in &ordered_advice_list {
            // Display category header if changed
            if advice.category != current_category {
                current_category = advice.category.clone();

                let (emoji, color_code, title) = get_category_style(&current_category);

                println!();
                println!();
                println!("{}{} \x1b[1m{}\x1b[0m",
                    color_code,
                    emoji,
                    title
                );
                println!("{}", "=".repeat(60));
                println!();
            }

            display_advice_item_enhanced(counter, advice);
            counter += 1;
            println!(); // Extra space between items
        }

        // Summary at the end
        println!();
        println!("{}", "=".repeat(60));
        println!();
        let msg = if advice_list.len() == 1 {
            format!("📋 Found {} recommendation across {} categories",
                advice_list.len(), by_category.len())
        } else {
            format!("📋 Found {} recommendations across {} categories",
                advice_list.len(), by_category.len())
        };
        println!("\x1b[1m{}\x1b[0m", msg);
        println!();

        // Show helpful tips based on mode
        println!("\x1b[90m{}\x1b[0m", "-".repeat(60));
        println!();
        println!("\x1b[1m\x1b[96mQuick Actions:\x1b[0m");
        println!("  annactl apply 1             Apply by number");
        println!("  annactl apply amd-microcode Apply by ID (shown in cyan)");
        println!("  annactl apply 1-5           Apply a range");
        println!("  annactl apply 1,3,5         Apply multiple");
        println!();
        println!("\x1b[1m\x1b[96mFiltering Options:\x1b[0m");
        println!("  annactl advise --mode=critical    Show only critical items");
        println!("  annactl advise --mode=all         Show all recommendations");
        println!("  annactl advise security           Show specific category");
        println!("  annactl advise --limit=10         Limit number of results");
        println!();

        // List available categories with counts (only categories that exist on this system)
        let mut cat_list: Vec<_> = by_category.keys().collect();
        cat_list.sort();
        if !cat_list.is_empty() {
            println!("\x1b[1m\x1b[96mAvailable Categories:\x1b[0m");
            println!();
            for cat in cat_list.iter() {
                let count = by_category.get(*cat).map(|v| v.len()).unwrap_or(0);
                // Get friendly name and description
                if let Some(info) = anna_common::CategoryInfo::get_by_id(cat) {
                    println!("  \x1b[36m{}\x1b[0m ({} items) - {}",
                        info.display_name, count, info.description);
                } else {
                    println!("  \x1b[36m{}\x1b[0m ({})", cat, count);
                }
            }
            println!();
        }
    }

    // Check for updates (non-spammy, once per day)
    check_and_notify_updates().await;

    Ok(())
}

/// Get category styling (emoji, ANSI color code, display title) - updated for standardized names
fn get_category_style(category: &str) -> (&'static str, &'static str, String) {
    match category {
        "Security & Privacy" => ("🔒", "\x1b[91m", "SECURITY & PRIVACY".to_string()), // Bright red
        "Hardware Support" => ("🔌", "\x1b[93m", "HARDWARE SUPPORT".to_string()), // Bright yellow
        "System Maintenance" => ("🔧", "\x1b[96m", "SYSTEM MAINTENANCE".to_string()), // Bright cyan
        "Performance & Optimization" => ("⚡", "\x1b[93m", "PERFORMANCE & OPTIMIZATION".to_string()), // Bright yellow
        "Power Management" => ("🔋", "\x1b[33m", "POWER MANAGEMENT".to_string()), // Yellow
        "Development Tools" => ("💻", "\x1b[95m", "DEVELOPMENT TOOLS".to_string()), // Bright magenta
        "Desktop Environment" => ("🖥️", "\x1b[34m", "DESKTOP ENVIRONMENT".to_string()), // Blue
        "Gaming & Entertainment" => ("🎮", "\x1b[95m", "GAMING & ENTERTAINMENT".to_string()), // Bright magenta
        "Multimedia & Graphics" => ("🎬", "\x1b[35m", "MULTIMEDIA & GRAPHICS".to_string()), // Magenta
        "Network Configuration" => ("📡", "\x1b[96m", "NETWORK CONFIGURATION".to_string()), // Bright cyan
        "Utilities" => ("🛠️", "\x1b[36m", "UTILITIES".to_string()), // Cyan
        "System Configuration" => ("⚙️", "\x1b[94m", "SYSTEM CONFIGURATION".to_string()), // Bright blue
        "Productivity" => ("📊", "\x1b[92m", "PRODUCTIVITY".to_string()), // Bright green
        "Terminal & CLI Tools" => ("🐚", "\x1b[96m", "TERMINAL & CLI TOOLS".to_string()), // Bright cyan
        "Communication" => ("💬", "\x1b[94m", "COMMUNICATION".to_string()), // Bright blue
        "Engineering & CAD" => ("📐", "\x1b[95m", "ENGINEERING & CAD".to_string()), // Bright magenta
        "Desktop Customization" => ("🎨", "\x1b[95m", "DESKTOP CUSTOMIZATION".to_string()), // Bright magenta
        _ => ("💡", "\x1b[36m", category.to_uppercase()), // Cyan
    }
}

/// Enhanced display for a single advice item
fn display_advice_item_enhanced(number: usize, advice: &anna_common::Advice) {
    // Priority and risk badges using ANSI codes
    let priority_badge = match advice.priority {
        anna_common::Priority::Mandatory => "\x1b[101m\x1b[97m\x1b[1m  CRITICAL  \x1b[0m", // Bright red bg, white text
        anna_common::Priority::Recommended => "\x1b[103m\x1b[30m\x1b[1m RECOMMENDED \x1b[0m", // Bright yellow bg, black text
        anna_common::Priority::Optional => "\x1b[104m\x1b[97m\x1b[1m  OPTIONAL  \x1b[0m", // Bright blue bg, white text
        anna_common::Priority::Cosmetic => "\x1b[100m\x1b[97m\x1b[1m  COSMETIC  \x1b[0m", // Bright black bg, white text
    };

    let risk_badge = match advice.risk {
        anna_common::RiskLevel::High => "\x1b[41m\x1b[97m\x1b[1m HIGH RISK \x1b[0m", // Red bg, white text
        anna_common::RiskLevel::Medium => "\x1b[43m\x1b[30m\x1b[1m MED RISK \x1b[0m", // Yellow bg, black text
        anna_common::RiskLevel::Low => "\x1b[42m\x1b[97m\x1b[1m LOW RISK \x1b[0m", // Green bg, white text
    };

    // RC.9.3: Show ID alongside number so users can use either
    // Format: [1] amd-microcode  Enable AMD microcode updates
    println!("\x1b[90m\x1b[1m[{}]\x1b[0m \x1b[36m{}\x1b[0m  \x1b[1m\x1b[97m{}\x1b[0m",
        number,
        advice.id,
        advice.title);

    // Badges
    let popularity_stars = advice.popularity_stars();
    let popularity_label = advice.popularity_label();
    println!("    {} {}  \x1b[93m{}\x1b[0m \x1b[90m({})\x1b[0m",
        priority_badge, risk_badge, popularity_stars, popularity_label);
    println!();

    // Reason - wrapped with proper indentation
    let reason = wrap_text(&advice.reason, 72, "    ");
    println!("\x1b[90m{}\x1b[0m", reason);

    // Command if available
    if let Some(ref cmd) = advice.command {
        println!();
        println!("    \x1b[96m\x1b[1mAction:\x1b[0m");
        println!("    \x1b[92m❯\x1b[0m \x1b[97m{}\x1b[0m", cmd);
    }

    // Show alternatives if available
    if !advice.alternatives.is_empty() {
        println!();
        println!("    \x1b[96m\x1b[1mAlternatives:\x1b[0m");
        for (i, alt) in advice.alternatives.iter().enumerate() {
            let marker = if i == 0 { "\x1b[92m★\x1b[0m" } else { "\x1b[90m○\x1b[0m" };
            println!("    {} \x1b[97m{}\x1b[0m", marker, alt.name);
            let desc_wrapped = wrap_text(&alt.description, 68, "      ");
            println!("\x1b[90m{}\x1b[0m", desc_wrapped);
            println!("      \x1b[90m\x1b[3m{}\x1b[0m", alt.install_command);
            if i < advice.alternatives.len() - 1 {
                println!();
            }
        }
    }

    // Wiki references if available
    if !advice.wiki_refs.is_empty() {
        println!();
        println!("    \x1b[96m\x1b[1m📚 Learn More:\x1b[0m");
        for wiki_ref in &advice.wiki_refs {
            println!("    \x1b[94m\x1b[3m{}\x1b[0m", wiki_ref);
        }
    }

    // ID for applying (smaller, less prominent)
    println!();
    println!("    \x1b[90m\x1b[3mID: {}\x1b[0m", advice.id);
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

/// Parse number ranges like "1", "1-5", "1,3,5-7" into a list of indices
fn parse_number_ranges(input: &str) -> Result<Vec<usize>> {
    let mut result = Vec::new();

    for part in input.split(',') {
        let part = part.trim();

        if part.contains('-') {
            // Range like "1-5"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                anyhow::bail!("Invalid range format: {}", part);
            }

            let start: usize = range_parts[0].trim().parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", range_parts[0]))?;
            let end: usize = range_parts[1].trim().parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", range_parts[1]))?;

            if start > end {
                anyhow::bail!("Invalid range: {} > {}", start, end);
            }

            for i in start..=end {
                result.push(i);
            }
        } else {
            // Single number
            let num: usize = part.parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", part))?;
            result.push(num);
        }
    }

    // Remove duplicates and sort
    result.sort_unstable();
    result.dedup();

    Ok(result)
}

pub async fn report(category: Option<String>) -> Result<()> {
    let title = if let Some(ref cat) = category {
        format!("📊 System Health Report - {}", cat)
    } else {
        "📊 System Health Report".to_string()
    };
    println!("{}", header(&title));
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

    // Get status, facts, and advice
    let status_data = client.call(Method::Status).await?;
    let facts_data = client.call(Method::GetFacts).await?;
    let advice_data = client.call(Method::GetAdvice).await?;

    let status = if let ResponseData::Status(s) = status_data {
        s
    } else {
        println!("{}", beautiful::status(Level::Error, "Failed to get status"));
        return Ok(());
    };

    let facts = if let ResponseData::Facts(f) = facts_data {
        f
    } else {
        println!("{}", beautiful::status(Level::Error, "Failed to get facts"));
        return Ok(());
    };

    let mut advice_list = if let ResponseData::Advice(a) = advice_data {
        a
    } else {
        Vec::new()
    };

    // Filter by category if specified
    if let Some(ref cat) = category {
        advice_list.retain(|a| a.category.to_lowercase() == cat.to_lowercase());

        if advice_list.is_empty() {
            println!("{}", beautiful::status(Level::Info, &format!("No recommendations found for category '{}'", cat)));
            println!();
            println!("  Available categories: security, performance, updates, gaming, desktop,");
            println!("                       multimedia, hardware, development, beautification");
            println!();
            return Ok(());
        }
    }

    // Apply ignore filters
    if let Ok(filters) = anna_common::IgnoreFilters::load() {
        let original_count = advice_list.len();
        advice_list.retain(|a| !filters.should_filter(a));
        let filtered_count = original_count - advice_list.len();
        if filtered_count > 0 {
            println!("{}", beautiful::status(Level::Info,
                &format!("Hiding {} items by your ignore filters", filtered_count)));
            println!();
        }
    }

    // Generate plain English summary
    generate_plain_english_report(&status, &facts, &advice_list);

    // Check for updates (non-spammy, once per day)
    check_and_notify_updates().await;

    Ok(())
}

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
pub async fn config(set: Option<String>) -> Result<()> {
    use anna_common::Config;

    println!("{}", header("Anna Configuration"));
    println!();

    // Load config
    let mut config = Config::load()?;

    // If setting a value
    if let Some(set_expr) = set {
        let parts: Vec<&str> = set_expr.splitn(2, '=').collect();
        if parts.len() != 2 {
            println!("{}", beautiful::status(Level::Error, "Invalid format. Use: key=value"));
            return Ok(());
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        // Handle different configuration keys
        match key {
            "autonomy_tier" => {
                let tier: u8 = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid autonomy tier. Use 0-3"))?;
                config.autonomy.tier = match tier {
                    0 => anna_common::AutonomyTier::AdviseOnly,
                    1 => anna_common::AutonomyTier::SafeAutoApply,
                    2 => anna_common::AutonomyTier::SemiAutonomous,
                    3 => anna_common::AutonomyTier::FullyAutonomous,
                    _ => anyhow::bail!("Autonomy tier must be 0-3"),
                };
                println!("{}", beautiful::status(Level::Success,
                    &format!("Set autonomy tier to {}", tier)));
            }
            "snapshots_enabled" => {
                config.snapshots.enabled = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Snapshots {}", if config.snapshots.enabled { "enabled" } else { "disabled" })));
            }
            "snapshot_method" => {
                if !["btrfs", "timeshift", "rsync", "none"].contains(&value) {
                    anyhow::bail!("Snapshot method must be: btrfs, timeshift, rsync, or none");
                }
                config.snapshots.method = value.to_string();
                println!("{}", beautiful::status(Level::Success,
                    &format!("Snapshot method set to {}", value)));
            }
            "learning_enabled" => {
                config.learning.enabled = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Learning {}", if config.learning.enabled { "enabled" } else { "disabled" })));
            }
            "desktop_notifications" => {
                config.notifications.desktop_notifications = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Desktop notifications {}",
                        if config.notifications.desktop_notifications { "enabled" } else { "disabled" })));
            }
            "refresh_interval" => {
                let seconds: u64 = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid number"))?;
                if seconds < 60 {
                    anyhow::bail!("Refresh interval must be at least 60 seconds");
                }
                config.general.refresh_interval_seconds = seconds;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Refresh interval set to {} seconds", seconds)));
            }
            _ => {
                println!("{}", beautiful::status(Level::Error,
                    &format!("Unknown configuration key: {}", key)));
                println!();
                println!("{}", beautiful::status(Level::Info, "Available keys:"));
                println!("  autonomy_tier (0-3)");
                println!("  snapshots_enabled (true/false)");
                println!("  snapshot_method (btrfs/timeshift/rsync/none)");
                println!("  learning_enabled (true/false)");
                println!("  desktop_notifications (true/false)");
                println!("  refresh_interval (seconds)");
                return Ok(());
            }
        }

        // Save updated config
        config.save()?;
        println!();
        println!("{}", beautiful::status(Level::Info,
            &format!("Configuration saved to {}",
                Config::default_path()?.display())));

        return Ok(());
    }

    // Display current config
    println!("{}", section("🎛️  General"));
    println!("  {}", kv("Refresh Interval",
        &format!("{} seconds", config.general.refresh_interval_seconds)));
    println!("  {}", kv("Verbosity", &config.general.verbosity.to_string()));
    println!("  {}", kv("Emoji", if config.general.enable_emoji { "enabled" } else { "disabled" }));
    println!();

    println!("{}", section("🤖 Autonomy"));
    let tier_name = match config.autonomy.tier {
        anna_common::AutonomyTier::AdviseOnly => "0 (Advise Only)",
        anna_common::AutonomyTier::SafeAutoApply => "1 (Safe Auto-Apply)",
        anna_common::AutonomyTier::SemiAutonomous => "2 (Semi-Autonomous)",
        anna_common::AutonomyTier::FullyAutonomous => "3 (Fully Autonomous)",
    };
    println!("  {}", kv("Tier", tier_name));
    println!("  {}", kv("Confirm High Risk",
        if config.autonomy.confirm_high_risk { "yes" } else { "no" }));
    println!("  {}", kv("Snapshot Before Apply",
        if config.autonomy.snapshot_before_apply { "yes" } else { "no" }));
    println!();

    println!("{}", section("📸 Snapshots"));
    println!("  {}", kv("Enabled", if config.snapshots.enabled { "yes" } else { "no" }));
    println!("  {}", kv("Method", &config.snapshots.method));
    println!("  {}", kv("Max Snapshots", &config.snapshots.max_snapshots.to_string()));
    println!();

    println!("{}", section("🧠 Learning"));
    println!("  {}", kv("Enabled", if config.learning.enabled { "yes" } else { "no" }));
    println!("  {}", kv("Track Dismissed", if config.learning.track_dismissed { "yes" } else { "no" }));
    println!("  {}", kv("History Days", &config.learning.command_history_days.to_string()));
    println!();

    println!("{}", section("🔔 Notifications"));
    println!("  {}", kv("Desktop", if config.notifications.desktop_notifications { "enabled" } else { "disabled" }));
    println!("  {}", kv("On Critical", if config.notifications.notify_on_critical { "yes" } else { "no" }));
    println!("  {}", kv("On Auto-Apply", if config.notifications.notify_on_auto_apply { "yes" } else { "no" }));
    println!();

    println!("{}", beautiful::status(Level::Info, "Use --set key=value to change settings"));
    println!();
    println!("{}", beautiful::status(Level::Info, "Examples:"));
    println!("  annactl config --set autonomy_tier=1");
    println!("  annactl config --set snapshots_enabled=true");
    println!("  annactl config --set learning_enabled=true");

    Ok(())
}

/// New config interface supporting get/set/TUI
/// Simplified config command - no action parameter needed
/// Examples:
///   annactl config                  -> show all
///   annactl config autonomy_tier    -> get value
///   annactl config autonomy_tier 1  -> set value
/// NOTE: Removed for v1.0 - not implemented yet
#[allow(dead_code)]
pub async fn config_simple(
    key: Option<String>,
    value: Option<String>,
) -> Result<()> {
    match (key, value) {
        // No key, no value -> show all config
        (None, None) => {
            config(None).await
        }
        // Key but no value -> get that key
        (Some(k), None) => {
            // Connect to daemon to get current config
            let mut client = match RpcClient::connect().await {
                Ok(c) => c,
                Err(_) => {
                    println!("{}", beautiful::status(Level::Error, "Daemon not running"));
                    println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
                    return Ok(());
                }
            };

            let config_data = client.call(Method::GetConfig).await?;
            if let ResponseData::Config(config) = config_data {
                // Match the key and return the value
                let val = match k.as_str() {
                    "autonomy_tier" => config.autonomy_tier.to_string(),
                    "auto_update_check" => config.auto_update_check.to_string(),
                    "wiki_cache_path" => config.wiki_cache_path.clone(),
                    _ => {
                        println!("{}", beautiful::status(Level::Error, &format!("Unknown config key: {}", k)));
                        println!();
                        println!("Available keys:");
                        println!("  autonomy_tier       - Current autonomy tier (0-3)");
                        println!("  auto_update_check   - Enable automatic update checking");
                        println!("  wiki_cache_path     - Path to Arch Wiki cache directory");
                        println!();
                        println!("To see all settings: \x1b[96mannactl config\x1b[0m");
                        return Ok(());
                    }
                };
                println!("{} = {}", k, val);
            } else {
                println!("{}", beautiful::status(Level::Error, "Failed to get configuration"));
            }
            Ok(())
        }
        // Key and value -> set that key
        (Some(k), Some(v)) => {
            config(Some(format!("{}={}", k, v))).await
        }
        // Value without key makes no sense
        (None, Some(_)) => {
            println!("{}", beautiful::status(Level::Error, "Cannot set value without key"));
            println!();
            println!("Usage: annactl config <key> <value>");
            Ok(())
        }
    }
}

/// Wrap text at specified width with indentation
fn wrap_text(text: &str, width: usize, indent: &str) -> String {
    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    
    for word in text.split_whitespace() {
        let word_len = console::measure_text_width(word);
        
        if current_width + word_len + 1 > width && !current_line.is_empty() {
            result.push(format!("{}{}", indent, current_line.trim()));
            current_line.clear();
            current_width = 0;
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }
        
        current_line.push_str(word);
        current_width += word_len;
    }
    
    if !current_line.is_empty() {
        result.push(format!("{}{}", indent, current_line.trim()));
    }
    
    result.join("\n")
}

/// Generate a plain English system health summary with sysadmin-level insights
fn generate_plain_english_report(_status: &anna_common::ipc::StatusData, facts: &anna_common::SystemFacts, advice: &[anna_common::Advice]) {

    println!("{}", section("📊 System Report"));
    println!();

    // Calculate scores for narrative
    let score = facts.performance_score;
    let tier_desc = if score >= 75 {
        ("high-performance", "powerful")
    } else if score >= 45 {
        ("mid-range", "capable")
    } else {
        ("modest", "efficient")
    };

    let gpu_desc = if facts.is_nvidia {
        "NVIDIA GPU"
    } else if facts.is_amd_gpu {
        "AMD GPU"
    } else if facts.is_intel_gpu {
        "Intel integrated graphics"
    } else {
        "basic graphics"
    };

    let de_info = facts.desktop_environment.as_ref()
        .map(|de| format!("running {}", de))
        .unwrap_or_else(|| "no desktop environment detected".to_string());

    let ds_info = facts.display_server.as_ref()
        .map(|ds| format!(" on {}", ds))
        .unwrap_or_default();

    // Executive Summary (like a sysadmin report)
    println!("   This is a {} system with {:.0}GB RAM and {}, {}{}.",
        tier_desc.0, facts.total_memory_gb, gpu_desc, de_info, ds_info);

    println!("   Based on the hardware profile (capability score: \x1b[1m{}/100\x1b[0m), Anna is configured", score);
    println!("   to recommend {} tools appropriate for your system's capabilities.",
        tier_desc.1);
    println!();

    // System Health Narrative
    let mut health_notes = Vec::new();

    // Boot performance
    if let Some(boot_time) = facts.boot_time_seconds {
        if boot_time < 20.0 {
            health_notes.push(format!("Boot time is excellent ({:.1}s)", boot_time));
        } else if boot_time < 40.0 {
            health_notes.push(format!("Boot time is acceptable ({:.1}s)", boot_time));
        } else {
            health_notes.push(format!("Boot time is slow ({:.1}s)", boot_time));
            if !facts.slow_services.is_empty() {
                let slowest = &facts.slow_services[0];
                health_notes.push(format!("   {} is taking {:.1}s to start",
                    slowest.name, slowest.time_seconds));
            }
        }
    }

    // Storage analysis
    if let Some(root_device) = facts.storage_devices.iter().find(|d| d.mount_point == "/") {
        let used_percent = if root_device.size_gb > 0.0 {
            (root_device.used_gb / root_device.size_gb * 100.0) as u8
        } else {
            0
        };
        if used_percent > 90 {
            health_notes.push(format!("Root partition is \x1b[91m{}% full\x1b[0m - needs attention soon", used_percent));
        } else if used_percent > 70 {
            health_notes.push(format!("Root partition is {}% full - no immediate concern", used_percent));
        }
    }

    // Orphaned packages
    if !facts.orphan_packages.is_empty() {
        health_notes.push(format!("Found {} orphaned packages that can be cleaned",
            facts.orphan_packages.len()));
    }

    // System health issues
    if !facts.failed_services.is_empty() {
        health_notes.push(format!("\x1b[91m{} service(s) have failed\x1b[0m", facts.failed_services.len()));
    }

    if health_notes.is_empty() {
        println!("   The system is generally healthy. No significant issues detected.");
    } else {
        println!("   \x1b[1mSystem Health:\x1b[0m");
        for note in &health_notes {
            println!("   {}", note);
        }
    }
    println!();

    // Issues Found - Narrative Summary
    println!("   \x1b[1mIssues Found:\x1b[0m");
    println!();

    // Overall assessment - use Priority (not RiskLevel) to be consistent with TUI
    let critical_issues: Vec<_> = advice.iter().filter(|a| matches!(a.priority, Priority::Mandatory)).collect();
    let critical = critical_issues.len();
    let recommended = advice.iter().filter(|a| matches!(a.priority, Priority::Recommended)).count();
    let optional = advice.iter().filter(|a| matches!(a.priority, Priority::Optional)).count();

    if critical == 0 && recommended == 0 && optional == 0 {
        println!("   Your system is in great shape! Everything looks secure and well-maintained.");
        println!("   I don't have any urgent recommendations right now.");
    } else if critical > 0 {
        println!("   I found \x1b[1;91m{} critical {}\x1b[0m that need your attention right away!",
            critical, if critical == 1 { "issue" } else { "issues" });
        println!("   These affect your system's security or stability.");
        println!();

        // Show critical issue details
        println!("   \x1b[1;91m🚨 Critical Issues:\x1b[0m");
        for (i, issue) in critical_issues.iter().take(5).enumerate() {
            println!("   \x1b[91m  {}. {}\x1b[0m", i + 1, issue.title);
            println!("      \x1b[2m{}\x1b[0m", issue.reason.lines().next().unwrap_or(""));
            if let Some(cmd) = &issue.command {
                println!("      \x1b[38;5;159m→ {}\x1b[0m", cmd);
            }
            println!();
        }
        if critical > 5 {
            println!("   \x1b[2m  ... and {} more critical issues\x1b[0m", critical - 5);
            println!();
        }
    } else if recommended > 5 {
        println!("   Your system is working fine, but I have {} recommendations that could", recommended);
        println!("   make things better. Nothing urgent, but worth looking at when you have time.");
    } else if recommended > 0 {
        println!("   Your system looks pretty good! I have {} suggestion{} that might be helpful.",
            recommended, if recommended == 1 { "" } else { "s" });
    } else {
        println!("   Your system is running well! I only have some optional suggestions for you.");
    }
    println!();
    
    // System info
    println!("{}", section("📋 System Overview"));
    println!();
    println!("   You're running Arch Linux with {} packages installed.", facts.installed_packages);
    println!("   Your kernel is version {} on {}.", facts.kernel, facts.cpu_model);

    // Storage - get info from first storage device if available
    if let Some(storage) = facts.storage_devices.first() {
        if storage.filesystem.contains("btrfs") {
            println!("   You're using Btrfs for your filesystem - great choice for modern features!");
        } else if storage.filesystem.contains("ext4") {
            println!("   You're using ext4 - solid and reliable!");
        }

        // Disk usage - calculate percentage
        let used_percent = if storage.size_gb > 0.0 {
            (storage.used_gb / storage.size_gb * 100.0) as u8
        } else {
            0
        };

        if used_percent > 90 {
            println!("   ⚠️  Your disk is {}% full - you might want to free up some space soon.", used_percent);
        } else if used_percent > 70 {
            println!("   Your disk is {}% full - still plenty of room.", used_percent);
        } else {
            println!("   You have plenty of disk space ({}% used).", used_percent);
        }
    }
    println!();
    
    // Recommendations breakdown
    if !advice.is_empty() {
        println!("{}", section("🎯 Recommendations Summary"));
        println!();
        
        if critical > 0 {
            println!("   🚨 {} Critical - These need immediate attention", critical);
        }
        if recommended > 0 {
            println!("   🔧 {} Recommended - Would improve your system", recommended);
        }
        if optional > 0 {
            println!("   ✨ {} Optional - Nice to have enhancements", optional);
        }
        println!();
        
        // Category breakdown
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for a in advice {
            *categories.entry(a.category.clone()).or_insert(0) += 1;
        }
        
        let mut sorted_cats: Vec<_> = categories.iter().collect();
        sorted_cats.sort_by(|a, b| b.1.cmp(a.1));
        
        if !sorted_cats.is_empty() {
            println!("   \x1b[38;5;240mBy category:\x1b[0m");
            for (cat, count) in sorted_cats {
                // Get friendly category name from CategoryInfo
                let cat_display = if let Some(info) = anna_common::CategoryInfo::get_by_id(cat) {
                    info.display_name
                } else {
                    cat.clone()
                };
                println!("   \x1b[38;5;240m  • {} suggestions about {}\x1b[0m", count, cat_display);
            }
            println!();
        }
    }
    
    // Call to action
    println!("{}", section("🚀 Next Steps"));
    println!();
    if critical > 0 {
        println!("   Run \x1b[38;5;159mannactl advise\x1b[0m to see the critical issues that need fixing.");
    } else if recommended > 0 || optional > 0 {
        println!("   Run \x1b[38;5;159mannactl advise\x1b[0m to see all recommendations.");
        println!("   Use \x1b[38;5;159mannactl apply --id <id>\x1b[0m to apply specific suggestions.");
    } else {
        println!("   Keep doing what you're doing - your system is well maintained!");
    }
    println!();
}
/// List available workflow bundles
pub async fn setup(desktop: Option<&str>, preview: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    // If no desktop specified, show available options
    if desktop.is_none() {
        println!("{}", header("Desktop Environment Setup"));
        println!();
        println!("  Anna can install and configure complete desktop environments for you.");
        println!("  Everything ready to use: window manager, terminal, launcher, bar, theme, etc.");
        println!();
        println!("{}", section("Available Setups"));
        println!();
        println!("  \x1b[1mhyprland\x1b[0m - Modern Wayland compositor with animations");
        println!("    • Automatic hardware detection (NVIDIA/AMD/Intel)");
        println!("    • Optimized for your system's capabilities");
        println!("    • Complete working environment out of the box");
        println!();
        println!("{}", section("Usage"));
        println!();
        println!("  \x1b[38;5;159mannactl setup hyprland\x1b[0m           # Install Hyprland environment");
        println!("  \x1b[38;5;159mannactl setup hyprland --preview\x1b[0m # Show what would be installed");
        println!();
        return Ok(());
    }

    let desktop_name = desktop.unwrap();

    // For now, only Hyprland is supported
    if desktop_name != "hyprland" {
        println!("{}", header("Desktop Setup"));
        println!();
        println!("  \x1b[91m✗\x1b[0m Desktop '{}' not available", desktop_name);
        println!();
        println!("  Currently supported:");
        println!("    • hyprland");
        println!();
        println!("  Run \x1b[38;5;159mannactl setup\x1b[0m to see all options");
        println!();
        return Ok(());
    }

    println!("{}", header(&format!("Setup: {}", desktop_name)));
    println!();

    // Show installed bundles first if any exist
    if let Ok(history) = anna_common::BundleHistory::load() {
        if !history.entries.is_empty() {
            println!("{}", section("📦 Installed Bundles"));
            println!();

            let installed: Vec<_> = history.entries.iter()
                .filter(|e| e.status == anna_common::BundleStatus::Completed && e.rollback_available)
                .collect();

            if !installed.is_empty() {
                for (i, entry) in installed.iter().enumerate() {
                    let rollback_id = i + 1;
                    println!("  \x1b[1;96m[#{}]\x1b[0m \x1b[1m{}\x1b[0m", rollback_id, entry.bundle_name);
                    println!("      Installed: {} by {}",
                        entry.installed_at.format("%Y-%m-%d %H:%M:%S"),
                        entry.installed_by
                    );
                    println!("      Items: {} package(s)", entry.installed_items.len());
                    println!();
                }
                println!("  \x1b[90mTo rollback:\x1b[0m annactl rollback <bundle-name> \x1b[90mor\x1b[0m annactl rollback #<number>");
                println!();
            }
        }
    }


    // Connect to daemon to get advice
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
        // Group by bundle
        let mut bundles: std::collections::HashMap<String, Vec<anna_common::Advice>> = std::collections::HashMap::new();

        for advice in advice_list {
            if let Some(ref bundle_name) = advice.bundle {
                bundles.entry(bundle_name.clone()).or_insert_with(Vec::new).push(advice);
            }
        }

        if bundles.is_empty() {
            println!("  \x1b[93m⚠\x1b[0m  No workflow bundles available for your system right now.");
            println!("     Install some base tools (Docker, Python, Rust) to see bundle suggestions!");
            println!();
            return Ok(());
        }

        println!("{}", section("📦 Available Bundles"));
        println!();

        for (bundle_name, items) in bundles.iter() {
            println!("  \x1b[1m{}\x1b[0m", bundle_name);
            println!("  \x1b[90m{} recommendation(s)\x1b[0m", items.len());
            println!();

            // Show what's included
            for (i, item) in items.iter().enumerate() {
                let marker = if !item.depends_on.is_empty() {
                    "  ├─"
                } else {
                    "  •"
                };
                println!("    {} \x1b[38;5;159m{}\x1b[0m", marker, item.title);

                if i == items.len() - 1 {
                    println!();
                }
            }

            // Show install command
            println!("    \x1b[96m❯\x1b[0m  annactl apply --bundle \"{}\"", bundle_name);
            println!();
        }

        println!("{}", section("💡 Tips"));
        println!();
        println!("  • Bundles install tools in dependency order");
        println!("  • Use \x1b[38;5;159m--dry-run\x1b[0m to see what will be installed");
        println!("  • Dependencies are automatically installed first");
        println!();

    } else {
        println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
    }

    Ok(())
}

/// Apply all advice in a bundle with dependency resolution
async fn apply_bundle(client: &mut RpcClient, bundle_name: &str, dry_run: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

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

/// Rollback a workflow bundle (by name or #number)
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

/// Display recent autonomous actions
/// NOTE: Removed for v1.0 - Anna doesn't have autonomous features yet
#[allow(dead_code)]
pub async fn autonomy(limit: usize) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Autonomous Actions Log"));
    println!();

    // Load autonomy log
    let log = match anna_common::AutonomyLog::load() {
        Ok(l) => l,
        Err(_) => {
            println!("{}", beautiful::status(Level::Info, "No autonomous actions have been performed yet"));
            println!();
            println!("  Autonomous maintenance runs based on your autonomy tier setting.");
            println!("  Use \x1b[38;5;159mannactl config --set autonomy_tier=1\x1b[0m to enable safe auto-apply.");
            println!();
            return Ok(());
        }
    };

    let recent = log.recent(limit);

    if recent.is_empty() {
        println!("{}", beautiful::status(Level::Info, "No autonomous actions recorded"));
        println!();
        return Ok(());
    }

    println!("{}", section(&format!("🤖 Recent {} Actions", recent.len())));
    println!();

    for (i, action) in recent.iter().enumerate() {
        // Success/failure indicator
        let status_icon = if action.success {
            "\x1b[92m✓\x1b[0m"
        } else {
            "\x1b[91m✗\x1b[0m"
        };

        // Action type badge
        let type_badge = match action.action_type.as_str() {
            "clean_orphans" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_cache" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_journal" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_tmp" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "remove_old_kernels" => "\x1b[43m\x1b[30m\x1b[1m MAINT \x1b[0m",
            "update_mirrorlist" => "\x1b[45m\x1b[97m\x1b[1m UPDATE \x1b[0m",
            _ => "\x1b[100m\x1b[97m\x1b[1m OTHER \x1b[0m",
        };

        println!("  \x1b[90m[{}]\x1b[0m  {} {}", i + 1, status_icon, type_badge);
        println!();
        println!("      \x1b[1m{}\x1b[0m", action.description);
        println!("      \x1b[90mExecuted: {}\x1b[0m", action.executed_at.format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("      \x1b[96mCommand:\x1b[0m");
        println!("      \x1b[90m❯\x1b[0m \x1b[97m{}\x1b[0m", action.command_run);

        if !action.output.is_empty() {
            let trimmed_output = action.output.trim();
            if !trimmed_output.is_empty() {
                println!();
                println!("      \x1b[96mOutput:\x1b[0m");
                // Show first 3 lines of output
                for (idx, line) in trimmed_output.lines().take(3).enumerate() {
                    if idx < 3 {
                        println!("      \x1b[90m{}\x1b[0m", line);
                    }
                }
                if trimmed_output.lines().count() > 3 {
                    println!("      \x1b[90m... ({} more lines)\x1b[0m", trimmed_output.lines().count() - 3);
                }
            }
        }

        if action.can_undo {
            if let Some(ref undo) = action.undo_command {
                println!();
                println!("      \x1b[93m⟲ Can undo:\x1b[0m \x1b[90m{}\x1b[0m", undo);
            }
        }

        if i < recent.len() - 1 {
            println!();
            println!("  \x1b[90m{}\x1b[0m", "─".repeat(78));
            println!();
        }
    }

    println!();
    println!("{}", section("ℹ️  Information"));
    println!();
    println!("  Total actions logged: {}", log.actions.len());
    println!("  Showing most recent: {}", recent.len());
    println!();
    println!("  Use \x1b[38;5;159mannactl autonomy --limit=<n>\x1b[0m to change the number of actions shown.");
    println!();

    Ok(())
}

/// Update Arch Wiki cache
/// NOTE: Removed for v1.0 - wiki cache now maintained automatically by daemon
#[allow(dead_code)]
pub async fn wiki_cache(force: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Arch Wiki Cache"));
    println!();

    // Check if we need to refresh
    let needs_refresh = anna_common::WikiCache::load()
        .map(|cache| cache.needs_refresh())
        .unwrap_or(true);

    if !force && !needs_refresh {
        println!("{}", beautiful::status(Level::Info, "Wiki cache is up to date"));
        println!();
        println!("  Use \x1b[38;5;159m--force\x1b[0m to refresh anyway.");
        println!();
        return Ok(());
    }

    println!("{}", section("📥 Updating Cache"));
    println!();

    if force {
        println!("{}", beautiful::status(Level::Info, "Forcing cache refresh..."));
    } else {
        println!("{}", beautiful::status(Level::Info, "Cache is stale, refreshing..."));
    }
    println!();

    // Connect to daemon to request wiki cache update
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Daemon not running"));
            println!("  The wiki cache update requires the daemon to be running.");
            println!("  Please start the daemon: \x1b[38;5;159msudo systemctl start annad\x1b[0m");
            println!();
            return Ok(());
        }
    };

    // Request cache update via RPC
    println!("{}", beautiful::status(Level::Info, "Updating Arch Wiki cache..."));
    println!("  This will download \x1b[1m88+ essential Arch Wiki pages\x1b[0m for offline access.");
    println!("  Progress details are logged by the daemon.");
    println!();
    println!("  \x1b[2mTip: Watch progress in another terminal:\x1b[0m");
    println!("       \x1b[38;5;159mjournalctl -u annad -f\x1b[0m");
    println!();

    use std::io::{self, Write};
    print!("  \x1b[38;5;226m⏳\x1b[0m Downloading wiki pages");
    io::stdout().flush()?;

    // Spawn progress animation
    let animation_handle = tokio::spawn(async {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        loop {
            print!("\r  \x1b[38;5;226m{}\x1b[0m Downloading wiki pages... ", frames[i % frames.len()]);
            io::stdout().flush().ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
            i += 1;
        }
    });

    let response = client
        .call(Method::UpdateWikiCache)
        .await
        .context("Failed to update wiki cache")?;

    // Stop animation
    animation_handle.abort();
    print!("\r\x1b[K"); // Clear line
    io::stdout().flush()?;

    match response {
        ResponseData::ActionResult { success, message } => {
            if success {
                println!("{}", beautiful::status(Level::Success, "Wiki cache updated successfully!"));
                println!("  {}", message);
                println!();
                println!("  \x1b[2m88 essential pages cached for offline use\x1b[0m");
            } else {
                println!("{}", beautiful::status(Level::Error, "Failed to update cache"));
                println!("  {}", message);
            }
        }
        _ => {
            println!("{}", beautiful::status(Level::Warning, "Unexpected response from daemon"));
        }
    }
    println!();

    Ok(())
}

/// Display system health score
/// NOTE: Removed for v1.0 - merged into status command
#[allow(dead_code)]
pub async fn health() -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("System Health Score"));
    println!();

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

    // Get facts and advice to calculate score
    let facts_data = client.call(Method::GetFacts).await?;
    let advice_data = client.call(Method::GetAdvice).await?;

    let facts = if let ResponseData::Facts(f) = facts_data {
        f
    } else {
        println!("{}", beautiful::status(Level::Error, "Failed to get system facts"));
        return Ok(());
    };

    let mut advice_list = if let ResponseData::Advice(a) = advice_data {
        a
    } else {
        Vec::new()
    };

    // Apply ignore filters before calculating health score
    if let Ok(filters) = anna_common::IgnoreFilters::load() {
        advice_list.retain(|a| !filters.should_filter(a));
    }

    // Calculate health score
    let score = anna_common::SystemHealthScore::calculate(&facts, &advice_list);

    println!("{}", section("📊 Overall Health"));
    println!();

    // Large score display
    let color = score.get_color_code();
    let grade = score.get_grade();

    println!("     {}{}/100{}\x1b[0m  \x1b[1m{}\x1b[0m",
        color,
        score.overall_score,
        color,
        grade
    );
    println!();

    // Score bar visualization
    let bar_width = 50;
    let filled = (score.overall_score as f64 / 100.0 * bar_width as f64) as usize;
    let empty = bar_width - filled;

    println!("     {}{}{}{}",
        color,
        "█".repeat(filled),
        "\x1b[90m",
        "░".repeat(empty)
    );
    println!("\x1b[0m");

    // Trend indicator
    let trend_icon = match score.health_trend {
        anna_common::HealthTrend::Improving => "\x1b[92m↗ Improving\x1b[0m",
        anna_common::HealthTrend::Stable => "\x1b[93m→ Stable\x1b[0m",
        anna_common::HealthTrend::Declining => "\x1b[91m↘ Declining\x1b[0m",
    };
    println!("     Trend: {}", trend_icon);
    println!();

    // Detailed scores
    println!("{}", section("📈 Score Breakdown"));
    println!();

    let format_score_with_details = |name: &str, score_val: u8, details: &[String]| {
        let color = if score_val >= 90 { "\x1b[92m" } else if score_val >= 70 { "\x1b[93m" } else { "\x1b[91m" };
        println!("  {:<20} {}{}{}  \x1b[90m{}\x1b[0m",
            name,
            color,
            score_val,
            "\x1b[0m",
            "█".repeat((score_val as f64 / 100.0 * 20.0) as usize)
        );
        for detail in details {
            println!("    \x1b[90m{}\x1b[0m", detail);
        }
        println!();
    };

    format_score_with_details("Security", score.security_score, &score.security_details);
    format_score_with_details("Performance", score.performance_score, &score.performance_details);
    format_score_with_details("Maintenance", score.maintenance_score, &score.maintenance_details);

    // Issues summary
    println!("{}", section("⚠️  Issues Summary"));
    println!();
    println!("  Total recommendations: \x1b[93m{}\x1b[0m", score.issues_count);
    if score.critical_issues > 0 {
        println!("  Critical issues: \x1b[91m{}\x1b[0m", score.critical_issues);
    } else {
        println!("  Critical issues: \x1b[92m0\x1b[0m");
    }
    println!();

    // Health interpretation
    println!("{}", section("💭 What This Means"));
    println!();
    match score.overall_score {
        95..=100 => {
            println!("  Your system is in excellent condition! Everything is well-maintained");
            println!("  and secure. Keep up the good work!");
        }
        85..=94 => {
            println!("  Your system is in very good shape. There are a few minor things");
            println!("  to address, but nothing urgent.");
        }
        70..=84 => {
            println!("  Your system is generally healthy, but there are some recommendations");
            println!("  you should look at when you have time.");
        }
        50..=69 => {
            println!("  Your system needs some attention. Please review the recommendations");
            println!("  to improve security and performance.");
        }
        _ => {
            println!("  Your system has significant issues that need immediate attention.");
            println!("  Please run \x1b[38;5;159mannactl advise\x1b[0m to see what needs to be fixed.");
        }
    }
    println!();

    // Call to action
    println!("{}", section("🎯 Next Steps"));
    println!();
    if score.critical_issues > 0 {
        println!("  1. Run \x1b[38;5;159mannactl advise --mode=critical\x1b[0m to see critical issues");
        println!("  2. Apply fixes with \x1b[38;5;159mannactl apply --nums <number>\x1b[0m");
    } else if score.issues_count > 0 {
        println!("  Run \x1b[38;5;159mannactl advise\x1b[0m to see all recommendations");
    } else {
        println!("  Your system is healthy! Run \x1b[38;5;159mannactl status\x1b[0m to monitor.");
    }
    println!();

    Ok(())
}

/// Dismiss a recommendation
pub async fn dismiss(id: Option<String>, num: Option<usize>) -> Result<()> {
    use anna_common::beautiful::{header};

    println!("{}", header("Dismiss Recommendation"));
    println!();

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

    // Get the advice ID
    let advice_id = if let Some(id) = id {
        id
    } else if let Some(num) = num {
        // Get all advice and find by number
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

        let advice_data = client.call(Method::GetAdviceWithContext {
            username,
            desktop_env,
            shell,
            display_server,
        }).await?;

        if let ResponseData::Advice(advice_list) = advice_data {
            if num < 1 || num > advice_list.len() {
                println!("{}", beautiful::status(Level::Error,
                    &format!("Number {} out of range (1-{})", num, advice_list.len())));
                return Ok(());
            }
            advice_list[num - 1].id.clone()
        } else {
            println!("{}", beautiful::status(Level::Error, "Failed to get advice list"));
            return Ok(());
        }
    } else {
        println!("{}", beautiful::status(Level::Error, "Please specify either --id or --num"));
        println!();
        println!("  Examples:");
        println!("    annactl dismiss --id orphan-packages");
        println!("    annactl dismiss --num 5");
        return Ok(());
    };

    // Record dismissal in feedback log
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    // Get advice details to know the category
    let advice_data = client.call(Method::GetAdvice).await?;
    if let ResponseData::Advice(advice_list) = advice_data {
        if let Some(advice) = advice_list.iter().find(|a| a.id == advice_id) {
            let mut log = anna_common::UserFeedbackLog::load().unwrap_or_default();
            log.record(anna_common::FeedbackEvent {
                advice_id: advice_id.clone(),
                advice_category: advice.category.clone(),
                event_type: anna_common::FeedbackType::Dismissed,
                timestamp: chrono::Utc::now(),
                username,
            });

            if let Err(e) = log.save() {
                println!("{}", beautiful::status(Level::Warning,
                    &format!("Failed to save feedback: {}", e)));
            } else {
                println!("{}", beautiful::status(Level::Success,
                    &format!("Dismissed: {}", advice.title)));
                println!();
                println!("  This recommendation won't be shown again.");
                println!("  Anna will learn from your preferences over time.");
            }
        } else {
            println!("{}", beautiful::status(Level::Error,
                &format!("Advice '{}' not found", advice_id)));
        }
    }

    println!();
    Ok(())
}

/// Show dismissed recommendations and optionally un-dismiss
pub async fn dismissed(undismiss_num: Option<usize>) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Dismissed Recommendations"));
    println!();

    // Load feedback log
    let mut log = match anna_common::UserFeedbackLog::load() {
        Ok(l) => l,
        Err(_) => {
            println!("{}", beautiful::status(Level::Info, "No dismissed recommendations"));
            println!();
            return Ok(());
        }
    };

    // Get dismissed events
    let dismissed: Vec<_> = log.events.iter()
        .filter(|e| matches!(e.event_type, anna_common::FeedbackType::Dismissed))
        .collect();

    if dismissed.is_empty() {
        println!("{}", beautiful::status(Level::Info, "No dismissed recommendations"));
        println!();
        println!("Use 'annactl dismiss <number>' to dismiss recommendations from the advise list");
        return Ok(());
    }

    // If undismiss requested
    if let Some(num) = undismiss_num {
        if num < 1 || num > dismissed.len() {
            println!("{}", beautiful::status(Level::Error,
                &format!("Number {} out of range (1-{})", num, dismissed.len())));
            return Ok(());
        }

        let event = dismissed[num - 1];
        let advice_id_to_remove = event.advice_id.clone();

        // Remove from log
        log.events.retain(|e| e.advice_id != advice_id_to_remove);
        log.save()?;

        println!("{}", beautiful::status(Level::Success,
            &format!("Un-dismissed: {}", advice_id_to_remove)));
        println!();
        println!("Run 'annactl advise' to see this recommendation again");
        println!();
        return Ok(());
    }

    // Show all dismissed items
    println!("{}", beautiful::status(Level::Info,
        &format!("{} dismissed recommendation{}", dismissed.len(), if dismissed.len() == 1 { "" } else { "s" })));
    println!();

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&anna_common::FeedbackEvent>> =
        std::collections::HashMap::new();

    for event in &dismissed {
        by_category.entry(event.advice_category.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    // Display by category
    for (category, events) in by_category.iter() {
        let category_emoji = anna_common::get_category_emoji(category);
        println!("{}", section(&format!("{} {}", category_emoji, category)));

        for event in events {
            let time_ago = {
                let duration = chrono::Utc::now().signed_duration_since(event.timestamp);
                if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} minutes ago", duration.num_minutes())
                } else {
                    "just now".to_string()
                }
            };

            println!("  • {} \x1b[90m({})\x1b[0m",
                event.advice_id.replace('-', " "),
                time_ago);
        }
        println!();
    }

    // Show commands
    println!("{}", section("Commands"));
    println!();
    println!("  annactl dismissed --undismiss <number>  # Restore a dismissed item");
    println!("  annactl advise                          # View current recommendations");
    println!();

    Ok(())
}

/// View application history and analytics
pub async fn history(days: i64, detailed: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Application History & Analytics"));
    println!();

    // Load application history
    let history = match anna_common::ApplicationHistory::load() {
        Ok(h) => h,
        Err(_) => {
            println!("{}", beautiful::status(Level::Info, "No application history yet"));
            println!();
            println!("  Start applying recommendations with \x1b[38;5;159mannactl apply\x1b[0m");
            println!("  History will be tracked automatically.");
            println!();
            return Ok(());
        }
    };

    if history.entries.is_empty() {
        println!("{}", beautiful::status(Level::Info, "No applications recorded yet"));
        println!();
        return Ok(());
    }

    // Get statistics for the period
    let stats = history.period_stats(days);

    println!("{}", section(&format!("📊 Last {} Days", days)));
    println!();
    println!("  Total Applications:  \x1b[1m{}\x1b[0m", stats.total_applications);
    println!("  Successful:          \x1b[92m{}\x1b[0m", stats.successful_applications);
    if stats.failed_applications > 0 {
        println!("  Failed:              \x1b[91m{}\x1b[0m", stats.failed_applications);
    }
    println!();

    // Success rate with visual bar
    let success_color = if stats.success_rate >= 90.0 {
        "\x1b[92m"
    } else if stats.success_rate >= 70.0 {
        "\x1b[93m"
    } else {
        "\x1b[91m"
    };

    println!("  Success Rate:        {}{:.1}%\x1b[0m", success_color, stats.success_rate);
    let bar_width = 40;
    let filled = (stats.success_rate / 100.0 * bar_width as f64) as usize;
    let empty = bar_width - filled;
    println!("  {}{}{}", 
        success_color,
        "█".repeat(filled),
        "\x1b[90m".to_string() + &"░".repeat(empty)
    );
    println!("\x1b[0m");

    // Top category
    if let Some((category, count)) = &stats.top_category {
        println!("  Most Active Category: \x1b[96m{}\x1b[0m ({} applications)", category, count);
        println!();
    }

    // Overall statistics
    println!("{}", section("📈 Overall Statistics"));
    println!();

    let all_time_success = history.success_rate();
    println!("  All-Time Success Rate: \x1b[1m{:.1}%\x1b[0m", all_time_success);
    println!("  Total Applications Ever: \x1b[1m{}\x1b[0m", history.entries.len());
    println!();

    // Top categories all time
    let top_cats = history.top_categories(5);
    if !top_cats.is_empty() {
        println!("  Top Categories (All Time):");
        for (i, (cat, count)) in top_cats.iter().enumerate() {
            let bar_len = (*count as f64 / top_cats[0].1 as f64 * 20.0) as usize;
            println!("    {:>2}. {:<20} \x1b[96m{}\x1b[0m \x1b[90m{}\x1b[0m",
                i + 1,
                cat,
                count,
                "█".repeat(bar_len)
            );
        }
        println!();
    }

    // Health improvement
    if let Some(avg_improvement) = history.average_health_improvement() {
        if avg_improvement > 0.0 {
            println!("  Average Health Improvement: \x1b[92m+{:.1} points\x1b[0m", avg_improvement);
        } else {
            println!("  Average Health Improvement: {:.1} points", avg_improvement);
        }
        println!();
    }

    // Recent applications
    if detailed {
        println!("{}", section("📜 Recent Applications"));
        println!();

        let recent = history.recent(10);
        for (i, entry) in recent.iter().enumerate() {
            let rollback_num = i + 1;
            let status_icon = if entry.success {
                "\x1b[92m✓\x1b[0m"
            } else {
                "\x1b[91m✗\x1b[0m"
            };

            println!("  \x1b[1;96m[#{}]\x1b[0m {} \x1b[1m{}\x1b[0m",
                rollback_num,
                status_icon,
                entry.advice_title
            );
            println!("       \x1b[90mID:\x1b[0m       \x1b[38;5;159m{}\x1b[0m", entry.advice_id);
            println!("       Category: \x1b[96m{}\x1b[0m", entry.category);
            println!("       Applied:  \x1b[90m{}\x1b[0m", entry.applied_at.format("%Y-%m-%d %H:%M:%S"));
            println!("       By:       \x1b[90m{}\x1b[0m", entry.applied_by);

            if let (Some(before), Some(after)) = (entry.health_score_before, entry.health_score_after) {
                let diff = after as i16 - before as i16;
                if diff > 0 {
                    println!("       Health:   {} → {} \x1b[92m(+{})\x1b[0m", before, after, diff);
                } else if diff < 0 {
                    println!("       Health:   {} → {} \x1b[91m({})\x1b[0m", before, after, diff);
                } else {
                    println!("       Health:   {} → {}", before, after);
                }
            }

            if i < recent.len() - 1 {
                println!();
            }
        }
        println!();
    }

    // Call to action
    println!("{}", section("💡 Tips"));
    println!();
    println!("  • Use \x1b[38;5;159m--detailed\x1b[0m to see full application history");
    println!("  • Use \x1b[38;5;159m--days=7\x1b[0m to view just the last week");
    println!("  • Track your progress with \x1b[38;5;159mannactl health\x1b[0m");
    println!();

    Ok(())
}

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
