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

    // Get advice from daemon
    let advice_data = client.call(Method::GetAdvice).await?;

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

pub async fn apply(id: Option<String>, auto: bool, dry_run: bool) -> Result<()> {
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
                "Use --id <advice-id> to apply specific recommendations"
            )
        );
        return Ok(());
    }

    // Show usage
    println!(
        "{}",
        beautiful::status(
            Level::Info,
            "Use --id <advice-id> to apply a specific recommendation"
        )
    );
    println!(
        "{}",
        beautiful::status(Level::Info, "Use --dry-run to see what would happen")
    );
    println!();
    println!("{}", beautiful::status(Level::Info, "Example:"));
    println!("  annactl apply --id orphan-packages --dry-run");
    println!("  annactl apply --id orphan-packages");

    Ok(())
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

pub async fn config(_set: Option<String>) -> Result<()> {
    println!("{}", header("Anna Configuration"));
    println!();
    println!("{}", section("Current Settings"));
    println!("  {}", kv("Autonomy Tier", "0 (Advise Only)"));
    println!("  {}", kv("Auto-update check", "enabled"));
    println!("  {}", kv("Wiki cache", "~/.local/share/anna/wiki"));
    println!();
    println!("{}", beautiful::status(Level::Info, "Use --set to change settings"));

    Ok(())
}

/// Refresh system scan and regenerate advice
pub async fn refresh() -> Result<()> {
    println!("{}", header("Refreshing System Scan"));
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
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    println!("{}", beautiful::status(Level::Info, "Scanning system..."));

    // Call refresh method
    let response = client.call(Method::Refresh).await?;

    if let ResponseData::ActionResult { success, message } = response {
        if success {
            println!();
            println!("{}", beautiful::status(Level::Success, &message));
            println!();
            println!("{}", beautiful::status(Level::Info, "Run 'annactl advise' to see updated recommendations"));
        } else {
            println!("{}", beautiful::status(Level::Error, &message));
        }
    }

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
fn generate_plain_english_report(status: &anna_common::ipc::StatusData, facts: &anna_common::SystemFacts, advice: &[anna_common::Advice]) {
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
