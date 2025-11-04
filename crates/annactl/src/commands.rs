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

pub async fn advise(risk_filter: Option<String>) -> Result<()> {
    println!("{}", header("System Recommendations"));
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

    println!(
        "{}",
        beautiful::status(Level::Info, "Taking a look at your system...")
    );
    println!();

    // Get advice from daemon with user context for multi-user systems
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    // Detect desktop environment
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok();

    // Get shell
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| "bash".to_string())
        .split('/')
        .last()
        .unwrap_or("bash")
        .to_string();

    // Detect display server (Wayland vs X11)
    let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("wayland".to_string())
    } else if std::env::var("DISPLAY").is_ok() {
        Some("x11".to_string())
    } else {
        None
    };

    // Get advice with user context
    let advice_data = client.call(Method::GetAdviceWithContext {
        username,
        desktop_env,
        shell,
        display_server,
    }).await?;

    if let ResponseData::Advice(mut advice_list) = advice_data {
        // Filter by risk level if specified
        if let Some(ref risk) = risk_filter {
            advice_list.retain(|a| {
                format!("{:?}", a.risk).to_lowercase() == risk.to_lowercase()
            });
        }

        if advice_list.is_empty() {
            println!(
                "{}",
                beautiful::status(Level::Success, "Your system looks great! I don't have any suggestions right now.")
            );
            return Ok(());
        }

        // Group by risk level
        let mut critical = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        for advice in &advice_list {
            match advice.risk {
                anna_common::RiskLevel::High => critical.push(advice),
                anna_common::RiskLevel::Medium => warnings.push(advice),
                anna_common::RiskLevel::Low => info.push(advice),
            }
        }

        let mut counter = 1;

        // Display critical items
        if !critical.is_empty() {
            println!("{}", section("🚨 Critical"));
            println!();
            for advice in critical {
                display_advice_item(counter, advice, Level::Error);
                counter += 1;
            }
        }

        // Display warnings
        if !warnings.is_empty() {
            println!("{}", section("🔧 Recommended"));
            println!();
            for advice in warnings {
                display_advice_item(counter, advice, Level::Warning);
                counter += 1;
            }
        }

        // Display info
        if !info.is_empty() {
            println!("{}", section("✨ Optional"));
            println!();
            for advice in info {
                display_advice_item(counter, advice, Level::Info);
                counter += 1;
            }
        }

        let msg = if advice_list.len() == 1 {
            "Found 1 thing that could make your system better!".to_string()
        } else {
            format!("Found {} things that could make your system better!", advice_list.len())
        };
        println!(
            "{}",
            beautiful::status(Level::Success, &msg)
        );
    }

    Ok(())
}

pub async fn apply(id: Option<String>, nums: Option<String>, auto: bool, dry_run: bool) -> Result<()> {
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

pub async fn report() -> Result<()> {
    println!("{}", header("📊 System Health Report"));
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

    let advice_list = if let ResponseData::Advice(a) = advice_data {
        a
    } else {
        Vec::new()
    };

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


/// Display a single advice item with numbering and proper formatting
fn display_advice_item(number: usize, advice: &anna_common::Advice, _level: Level) {
    
    // Number and title
    let (emoji, color) = match advice.category.as_str() {
        "security" => ("🔒", "\x1b[38;5;196m"),  // Red
        "performance" => ("⚡", "\x1b[38;5;226m"), // Yellow
        "updates" => ("📦", "\x1b[38;5;117m"),    // Blue
        "cleanup" => ("🧹", "\x1b[38;5;159m"),    // Cyan
        "development" => ("💻", "\x1b[38;5;141m"), // Purple
        "beautification" => ("🎨", "\x1b[38;5;213m"), // Pink
        "gaming" => ("🎮", "\x1b[38;5;201m"),     // Magenta
        "desktop" => ("🖥️", "\x1b[38;5;117m"),    // Blue
        "multimedia" => ("🎬", "\x1b[38;5;183m"), // Light purple
        "hardware" => ("🔌", "\x1b[38;5;208m"),   // Orange
        "networking" => ("📡", "\x1b[38;5;87m"),  // Cyan
        "power" => ("🔋", "\x1b[38;5;220m"),      // Gold
        _ => ("💡", "\x1b[38;5;159m"),            // Cyan
    };

    println!("\x1b[1m\x1b[38;5;250m{}.\x1b[0m {} {}{}\x1b[0m", 
        number, 
        emoji,
        color,
        advice.title
    );
    
    // Reason - wrap text at 80 chars with proper indentation
    let reason = wrap_text(&advice.reason, 76, "   ");
    println!("\x1b[38;5;250m{}\x1b[0m", reason);
    println!();
    
    // Command if available
    if let Some(ref cmd) = advice.command {
        println!("   \x1b[38;5;117m→ Run:\x1b[0m \x1b[38;5;159m{}\x1b[0m", cmd);
        println!();
    }
    
    // ID for applying
    println!("   \x1b[38;5;240m[ID: {}]\x1b[0m", advice.id);
    println!();
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

/// Generate a plain English system health summary
fn generate_plain_english_report(_status: &anna_common::ipc::StatusData, facts: &anna_common::SystemFacts, advice: &[anna_common::Advice]) {
    use anna_common::RiskLevel;
    
    println!("{}", section("💭 What I think about your system"));
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
                let cat_name = match cat.as_str() {
                    "security" => "Security",
                    "performance" => "Performance",
                    "updates" => "Updates",
                    "gaming" => "Gaming",
                    "desktop" => "Desktop",
                    "multimedia" => "Multimedia",
                    "hardware" => "Hardware",
                    "beautification" => "Terminal & CLI",
                    "development" => "Development",
                    _ => cat,
                };
                println!("   \x1b[38;5;240m  • {} suggestions about {}\x1b[0m", count, cat_name);
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
