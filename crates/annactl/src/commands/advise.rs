//! Advise command - show system recommendations

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;
use super::utils::{check_and_notify_updates, wrap_text};

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

                advice_list = mandatory;
                advice_list.extend(recommended);
                advice_list.extend(optional);
                advice_list.extend(cosmetic);
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
        }

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
        if category_filter.is_none() {
            println!("{}", section("📊 Recommendations"));
            println!();

            // Show category counts
            let mut category_counts: Vec<_> = by_category.iter()
                .map(|(cat, items)| {
                    // Normalize category names to Title Case
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
                    (normalized, items.len())
                })
                .collect();
            category_counts.sort_by(|a, b| b.1.cmp(&a.1));

            for (category, count) in category_counts.iter() {
                println!("  \x1b[96m{}\x1b[0m \x1b[90m{:>2}\x1b[0m", category, count);
            }
            println!();

            // Compact usage examples
            println!("  \x1b[96mannactl advise {}\x1b[0m  \x1b[90m# view category\x1b[0m",
                category_counts[0].0.to_lowercase());
            println!("  \x1b[96mannactl advise all\x1b[0m       \x1b[90m# view all details\x1b[0m");
            println!();

            // Save to cache for apply-by-number
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

        // Build ordered list for display
        let category_order = anna_common::get_category_order();
        let mut ordered_advice_list: Vec<&anna_common::Advice> = Vec::new();

        for &category in &category_order {
            if let Some(items) = by_category.get(category) {
                if items.is_empty() {
                    continue;
                }

                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then(b.risk.cmp(&a.risk))
                        .then(b.popularity.cmp(&a.popularity))
                });

                ordered_advice_list.extend(sorted_items);
            }
        }

        // Add remaining categories
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

        // Save display order to cache
        let advice_ids: Vec<String> = ordered_advice_list.iter().map(|a| a.id.clone()).collect();
        if let Err(e) = anna_common::AdviceDisplayCache::save(advice_ids) {
            eprintln!("Warning: Failed to save advice cache: {}", e);
        }

        // Display everything
        let mut counter = 1;
        let mut current_category = String::new();

        for advice in &ordered_advice_list {
            // Display category header if changed
            if advice.category != current_category {
                current_category = advice.category.clone();
                let (emoji, color_code, title) = get_category_style(&current_category);

                println!();
                println!();
                println!("{}{} \x1b[1m{}\x1b[0m", color_code, emoji, title);
                println!("{}", "=".repeat(60));
                println!();
            }

            display_advice_item_enhanced(counter, advice);
            counter += 1;
            println!();
        }

        // Summary
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

        // Show helpful tips
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

        // List available categories
        let mut cat_list: Vec<_> = by_category.keys().collect();
        cat_list.sort();
        if !cat_list.is_empty() {
            println!("\x1b[1m\x1b[96mAvailable Categories:\x1b[0m");
            println!();
            for cat in cat_list.iter() {
                let count = by_category.get(*cat).map(|v| v.len()).unwrap_or(0);
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

    check_and_notify_updates().await;
    Ok(())
}
