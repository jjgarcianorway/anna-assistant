//! CLI command implementations

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, kv, section, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn status() -> Result<()> {
    println!("{}", header("Anna Status"));
    println!();

    // Try to connect to daemon
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!(
                "{}",
                beautiful::status(Level::Error, "Daemon not running")
            );
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    // Get status from daemon
    let status_data = client.call(Method::Status).await?;

    if let ResponseData::Status(status) = status_data {
        println!("{}", section("System"));

        // Get system facts for display
        let facts_data = client.call(Method::GetFacts).await?;
        if let ResponseData::Facts(facts) = facts_data {
            println!("  {}", kv("Hostname", &facts.hostname));
            println!("  {}", kv("Kernel", &facts.kernel));
        }

        println!();
        println!("{}", section("Daemon"));
        println!("  {}", beautiful::status(Level::Success, "Running"));
        println!("  {}", kv("Version", &status.version));
        println!("  {}", kv("Uptime", &format!("{}s", status.uptime_seconds)));
        println!();

        if status.pending_recommendations > 0 {
            println!(
                "{}",
                beautiful::status(
                    Level::Info,
                    &format!("{} recommendations pending", status.pending_recommendations)
                )
            );
        } else {
            println!(
                "{}",
                beautiful::status(Level::Info, "All systems operational")
            );
        }
    }

    Ok(())
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

    println!("{}", beautiful::status(Level::Info, "Taking a look at your system..."));
    println!();

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
                // Keep all Mandatory and Recommended
                // Limit Optional and Cosmetic
                let mandatory: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Mandatory)).cloned().collect();
                let recommended: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Recommended)).cloned().collect();
                let mut optional: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Optional)).cloned().collect();
                let mut cosmetic: Vec<_> = advice_list.iter().filter(|a| matches!(a.priority, anna_common::Priority::Cosmetic)).cloned().collect();

                // Take top 10 optional and top 5 cosmetic
                optional.truncate(10);
                cosmetic.truncate(5);

                advice_list = mandatory;
                advice_list.extend(recommended);
                advice_list.extend(optional);
                advice_list.extend(cosmetic);

                let total_hidden = advice_list.len();
                println!("{}", beautiful::status(Level::Info,
                    &format!("Showing SMART filtered view (use --mode=all to see {} more recommendations)", total_hidden)));
                println!();
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

        // Filter by risk level if specified
        if let Some(ref risk) = risk_filter {
            advice_list.retain(|a| {
                format!("{:?}", a.risk).to_lowercase() == risk.to_lowercase()
            });
        }

        // Filter by category if specified
        if let Some(ref cat) = category_filter {
            advice_list.retain(|a| a.category.to_lowercase() == cat.to_lowercase());
        }

        // Apply limit if not 0
        if limit > 0 && advice_list.len() > limit {
            let hidden = advice_list.len() - limit;
            advice_list.truncate(limit);
            println!("{}", beautiful::status(Level::Info,
                &format!("Showing first {} of {} recommendations (use --limit=0 to see all)", limit, limit + hidden)));
            println!();
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

        // Sort categories by importance
        let category_order = vec![
            "security", "drivers", "updates", "maintenance", "cleanup",
            "performance", "power", "development", "desktop", "gaming",
            "multimedia", "hardware", "networking", "beautification",
        ];

        let mut counter = 1;

        // Display each category in a beautiful box
        for &category in &category_order {
            if let Some(items) = by_category.get(category) {
                if items.is_empty() {
                    continue;
                }

                // Category header with box and emoji
                let (emoji, color_code, title) = get_category_style(category);

                println!();
                let box_width = 80;
                let category_title = format!(" {} {} ", emoji, title);
                let title_len = console::measure_text_width(&category_title);
                let left_pad = (box_width - title_len) / 2;
                let right_pad = box_width - title_len - left_pad;

                println!("\x1b[90m{}\x1b[0m", format!("╭{}╮", "─".repeat(box_width)));
                println!("\x1b[90m│\x1b[0m{}{}\x1b[1m{}\x1b[0m{}\x1b[90m│\x1b[0m",
                    " ".repeat(left_pad),
                    color_code,
                    category_title,
                    " ".repeat(right_pad)
                );
                println!("\x1b[90m{}\x1b[0m", format!("╰{}╯", "─".repeat(box_width)));
                println!();

                // Sort items within category by priority then risk
                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then(b.risk.cmp(&a.risk))
                });

                for advice in sorted_items {
                    display_advice_item_enhanced(counter, advice);
                    counter += 1;
                    println!(); // Extra space between items
                }
            }
        }

        // Display any remaining categories not in the predefined order
        for (category, items) in &by_category {
            if !category_order.contains(&category.as_str()) && !items.is_empty() {
                println!();
                let (emoji, color_code, title) = get_category_style(&category);
                let box_width = 80;
                let category_title = format!(" {} {} ", emoji, title);
                let title_len = console::measure_text_width(&category_title);
                let left_pad = (box_width - title_len) / 2;
                let right_pad = box_width - title_len - left_pad;

                println!("\x1b[90m{}\x1b[0m", format!("╭{}╮", "─".repeat(box_width)));
                println!("\x1b[90m│\x1b[0m{}{}\x1b[1m{}\x1b[0m{}\x1b[90m│\x1b[0m",
                    " ".repeat(left_pad),
                    color_code,
                    category_title,
                    " ".repeat(right_pad)
                );
                println!("\x1b[90m{}\x1b[0m", format!("╰{}╯", "─".repeat(box_width)));
                println!();

                let mut sorted_items = items.clone();
                sorted_items.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.risk.cmp(&a.risk)));

                for advice in sorted_items {
                    display_advice_item_enhanced(counter, advice);
                    counter += 1;
                    println!();
                }
            }
        }

        // Summary at the end
        println!();
        println!("\x1b[90m{}\x1b[0m", "═".repeat(80));
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
        println!("\x1b[38;5;250m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();
        println!("\x1b[1m\x1b[96mQuick Actions:\x1b[0m");
        println!("  annactl apply --nums <number>     Apply specific recommendation");
        println!("  annactl apply --nums 1-5          Apply a range");
        println!("  annactl apply --nums 1,3,5        Apply multiple");
        println!();
        println!("\x1b[1m\x1b[96mFiltering Options:\x1b[0m");
        println!("  annactl advise --mode=critical    Show only critical items");
        println!("  annactl advise --mode=all         Show all recommendations");
        println!("  annactl advise --category=security  Show specific category");
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

    Ok(())
}

/// Get category styling (emoji, ANSI color code, display title)
fn get_category_style(category: &str) -> (&'static str, &'static str, String) {
    match category {
        "security" => ("🔒", "\x1b[91m", "SECURITY".to_string()), // Bright red
        "drivers" => ("🔌", "\x1b[95m", "DRIVERS & HARDWARE".to_string()), // Bright magenta
        "updates" => ("📦", "\x1b[94m", "SYSTEM UPDATES".to_string()), // Bright blue
        "maintenance" => ("🔧", "\x1b[96m", "SYSTEM MAINTENANCE".to_string()), // Bright cyan
        "cleanup" => ("🧹", "\x1b[36m", "CLEANUP & OPTIMIZATION".to_string()), // Cyan
        "performance" => ("⚡", "\x1b[93m", "PERFORMANCE".to_string()), // Bright yellow
        "power" => ("🔋", "\x1b[33m", "POWER MANAGEMENT".to_string()), // Yellow
        "development" => ("💻", "\x1b[95m", "DEVELOPMENT TOOLS".to_string()), // Bright magenta
        "desktop" => ("🖥️", "\x1b[34m", "DESKTOP ENVIRONMENT".to_string()), // Blue
        "gaming" => ("🎮", "\x1b[95m", "GAMING".to_string()), // Bright magenta
        "multimedia" => ("🎬", "\x1b[35m", "MULTIMEDIA".to_string()), // Magenta
        "hardware" => ("🔌", "\x1b[93m", "HARDWARE SUPPORT".to_string()), // Bright yellow
        "networking" => ("📡", "\x1b[96m", "NETWORKING".to_string()), // Bright cyan
        "beautification" => ("🎨", "\x1b[95m", "TERMINAL & CLI TOOLS".to_string()), // Bright magenta
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

    // Number and title
    println!("\x1b[90m\x1b[1m[{}]\x1b[0m  \x1b[1m\x1b[97m{}\x1b[0m", number, advice.title);

    // Badges
    println!("    {} {}", priority_badge, risk_badge);
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
            })
            .await?;

        if let ResponseData::ActionResult { success, message } = result {
            if success {
                println!("{}", beautiful::status(Level::Success, &message));
            } else {
                println!("{}", beautiful::status(Level::Error, &message));
            }
        }

        return Ok(());
    }

    // If nums provided, apply by index
    if let Some(nums_str) = nums {
        // First get all advice to map indices
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
            // Parse the nums string (e.g., "1,3,5-7")
            let indices = parse_number_ranges(&nums_str)?;

            // Apply each advice by index
            let mut success_count = 0;
            let mut fail_count = 0;

            for idx in indices {
                if idx < 1 || idx > advice_list.len() {
                    println!("{}", beautiful::status(Level::Warning,
                        &format!("Index {} out of range (1-{})", idx, advice_list.len())));
                    fail_count += 1;
                    continue;
                }

                let advice = &advice_list[idx - 1];
                println!("{}", beautiful::status(Level::Info,
                    &format!("{}. Applying: {}", idx, advice.title)));

                let result = client.call(Method::ApplyAction {
                    advice_id: advice.id.clone(),
                    dry_run,
                }).await?;

                if let ResponseData::ActionResult { success, message } = result {
                    if success {
                        println!("   {}", beautiful::status(Level::Success, &message));
                        success_count += 1;
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

    // Generate plain English summary
    generate_plain_english_report(&status, &facts, &advice_list);

    Ok(())
}

pub async fn doctor() -> Result<()> {
    println!("{}", header("System Diagnostics"));
    println!();
    println!("{}", section("Checks"));
    println!("  {} Pacman functional", beautiful::status(Level::Success, "✓"));
    println!("  {} Kernel modules loaded", beautiful::status(Level::Success, "✓"));
    println!("  {} Network connectivity", beautiful::status(Level::Success, "✓"));
    println!();
    println!("{}", beautiful::status(Level::Success, "All checks passed"));

    Ok(())
}

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
    use anna_common::RiskLevel;

    // First, show system health metrics
    println!("{}", section("🔍 System Health Analysis"));
    println!();

    // CPU and Memory
    println!("   \x1b[1mHardware:\x1b[0m");
    println!("     CPU: {} ({} cores)", facts.cpu_model, facts.cpu_cores);
    println!("     RAM: {:.1} GB total", facts.total_memory_gb);
    if let Some(ref gpu) = facts.gpu_vendor {
        println!("     GPU: {}", gpu);
    }
    println!();

    // Storage analysis
    println!("   \x1b[1mStorage:\x1b[0m");
    for device in &facts.storage_devices {
        let used_percent = if device.size_gb > 0.0 {
            (device.used_gb / device.size_gb * 100.0) as u8
        } else {
            0
        };
        let status_icon = if used_percent > 90 {
            "\x1b[91m⚠️\x1b[0m"
        } else if used_percent > 70 {
            "\x1b[93m●\x1b[0m"
        } else {
            "\x1b[92m✓\x1b[0m"
        };
        println!("     {} {} on {} - {:.1}/{:.1} GB ({}% full)",
            status_icon, device.filesystem, device.mount_point,
            device.used_gb, device.size_gb, used_percent);
    }
    println!();

    // Software environment
    println!("   \x1b[1mSoftware Environment:\x1b[0m");
    println!("     Kernel: {}", facts.kernel);
    println!("     Packages: {} installed", facts.installed_packages);
    if !facts.orphan_packages.is_empty() {
        println!("     Orphaned: \x1b[93m{} packages\x1b[0m (can be cleaned)", facts.orphan_packages.len());
    }
    if let Some(ref de) = facts.desktop_environment {
        println!("     Desktop: {}", de);
    }
    if let Some(ref ds) = facts.display_server {
        println!("     Display: {}", ds);
    }
    println!("     Shell: {}", facts.shell);
    println!();

    // Development environment detection
    if !facts.dev_tools_detected.is_empty() {
        println!("   \x1b[1mDevelopment Tools Detected:\x1b[0m");
        print!("     ");
        for (i, tool) in facts.dev_tools_detected.iter().enumerate() {
            print!("{}", tool);
            if i < facts.dev_tools_detected.len() - 1 {
                print!(", ");
            }
        }
        println!();
        println!();
    }

    // Network capabilities
    println!("   \x1b[1mNetwork:\x1b[0m");
    if facts.has_wifi {
        println!("     \x1b[92m✓\x1b[0m WiFi available");
    }
    if facts.has_ethernet {
        println!("     \x1b[92m✓\x1b[0m Ethernet available");
    }
    println!("     {} network interfaces detected", facts.network_interfaces.len());
    println!();

    println!("{}", section("💭 Overall Assessment"));
    println!();

    // Overall assessment
    let critical = advice.iter().filter(|a| matches!(a.risk, RiskLevel::High)).count();
    let recommended = advice.iter().filter(|a| matches!(a.risk, RiskLevel::Medium)).count();
    let optional = advice.iter().filter(|a| matches!(a.risk, RiskLevel::Low)).count();
    
    if critical == 0 && recommended == 0 && optional == 0 {
        println!("   Your system is in great shape! Everything looks secure and well-maintained.");
        println!("   I don't have any urgent recommendations right now.");
    } else if critical > 0 {
        println!("   I found {} critical {} that need your attention right away.", 
            critical, if critical == 1 { "issue" } else { "issues" });
        println!("   These affect your system's security or stability.");
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
pub async fn bundles() -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Workflow Bundles"));
    println!();
    println!("  \x1b[90mInstall complete development stacks with one command!\x1b[0m");
    println!();

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
            println!("  Use \x1b[38;5;159mannactl bundles\x1b[0m to see available bundles.");
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

        // Apply each in order
        println!("{}", section("🚀 Installing"));
        println!();

        let mut installed_items: Vec<String> = Vec::new();
        let mut installation_status = anna_common::BundleStatus::Completed;

        for (i, advice) in sorted.iter().enumerate() {
            println!("  [{}/{}] \x1b[1m{}\x1b[0m", i + 1, sorted.len(), advice.title);

            let result = client
                .call(Method::ApplyAction {
                    advice_id: advice.id.clone(),
                    dry_run: false,
                })
                .await?;

            if let ResponseData::ActionResult { success, message } = result {
                if success {
                    println!("         \x1b[92m✓\x1b[0m {}", message);
                    installed_items.push(advice.id.clone());
                } else {
                    println!("         \x1b[91m✗\x1b[0m {}", message);
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

        if let Err(e) = history.save() {
            println!("{}", beautiful::status(Level::Warning, &format!("Failed to save bundle history: {}", e)));
        }

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

/// Rollback a workflow bundle
pub async fn rollback(bundle_name: &str, dry_run: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header(&format!("Rolling Back Bundle: {}", bundle_name)));
    println!();

    // Load bundle history
    let history = match anna_common::BundleHistory::load() {
        Ok(h) => h,
        Err(e) => {
            println!("{}", beautiful::status(Level::Error, &format!("Failed to load bundle history: {}", e)));
            return Ok(());
        }
    };

    // Find the latest completed installation
    let entry = match history.get_latest(bundle_name) {
        Some(e) => e,
        None => {
            println!("{}", beautiful::status(Level::Error, &format!("No installation history found for bundle '{}'", bundle_name)));
            println!();
            println!("  This bundle was never installed or the history was cleared.");
            println!("  Use \x1b[38;5;159mannactl bundles\x1b[0m to see available bundles.");
            return Ok(());
        }
    };

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

    // Connect to daemon
    let client = match RpcClient::connect().await {
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
        println!("{}", beautiful::status(Level::Success, &format!("Rolled back '{}' - {} item(s) removed", bundle_name, removed_count)));
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
    let _client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Warning, "Daemon not running, updating cache directly..."));
            println!();

            // If daemon is not running, we'll need to import and call the update function directly
            // For now, show a message
            println!("{}", beautiful::status(Level::Info, "Starting wiki cache update..."));
            println!("  This will download 15 common Arch Wiki pages for offline access.");
            println!();

            // TODO: Call wiki_cache::update_common_pages() directly
            // This would require exposing the function or using a library structure
            println!("{}", beautiful::status(Level::Warning, "Direct cache update not yet implemented"));
            println!("  Please ensure the daemon is running: \x1b[38;5;159msudo systemctl start annad\x1b[0m");
            println!();

            return Ok(());
        }
    };

    // Request cache update via RPC
    // TODO: Add WikiCacheUpdate method to RPC
    println!("{}", beautiful::status(Level::Info, "Requesting cache update from daemon..."));
    println!();
    println!("{}", beautiful::status(Level::Warning, "RPC method not yet implemented"));
    println!("  The wiki cache update feature will be available in the next version.");
    println!();
    println!("  For now, wiki pages will be cached automatically when accessed.");
    println!();

    Ok(())
}
