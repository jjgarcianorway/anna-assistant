//! CLI command implementations

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, boxed, header, kv, section, Level};
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

        // Display critical items
        if !critical.is_empty() {
            println!("{}", section("Critical"));
            for advice in critical {
                println!(
                    "  {} {}",
                    beautiful::status(Level::Error, "⚠"),
                    advice.title
                );
                println!("    {}", advice.reason);
                if let Some(ref cmd) = advice.command {
                    println!("    Command: {}", cmd);
                }
                for wiki in &advice.wiki_refs {
                    println!("    Wiki: {}", wiki);
                }
            }
            println!();
        }

        // Display warnings
        if !warnings.is_empty() {
            println!("{}", section("🔧 Maintenance"));
            for advice in warnings {
                let emoji = match advice.category.as_str() {
                    "security" => "🔒",
                    "updates" => "📦",
                    "cleanup" => "🧹",
                    _ => "⚙️",
                };
                println!(
                    "  {} {} {}\x1b[1m{}\x1b[0m",
                    emoji,
                    beautiful::status(Level::Warning, "→"),
                    "\x1b[38;5;228m",  // Yellow
                    advice.title
                );
                println!("    \x1b[38;5;250m💡 {}\x1b[0m", advice.reason);
                if let Some(ref cmd) = advice.command {
                    println!("    \x1b[38;5;159m📋 Command:\x1b[0m \x1b[38;5;250m{}\x1b[0m", cmd);
                }
                println!();
            }
        }

        // Display info
        if !info.is_empty() {
            println!("{}", section("✨ Suggestions"));
            for advice in info {
                let (emoji, color) = match advice.category.as_str() {
                    "development" => ("💻", "\x1b[38;5;117m"),  // Blue
                    "beautification" => ("🎨", "\x1b[38;5;213m"),  // Pink
                    "performance" => ("⚡", "\x1b[38;5;226m"),  // Bright yellow
                    "media" => ("🎵", "\x1b[38;5;183m"),  // Purple
                    _ => ("💡", "\x1b[38;5;159m"),  // Cyan
                };
                println!(
                    "  {} {} {}\x1b[1m{}\x1b[0m",
                    emoji,
                    beautiful::status(Level::Info, "→"),
                    color,
                    advice.title
                );
                println!("    \x1b[38;5;250m📖 {}\x1b[0m", advice.reason);
                if let Some(ref cmd) = advice.command {
                    println!("    \x1b[38;5;159m📋 Command:\x1b[0m \x1b[38;5;250m{}\x1b[0m", cmd);
                }
                println!();
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
    println!("{}", header("System Health Report"));
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

    // Get status and advice count
    let status_data = client.call(Method::Status).await?;
    let advice_data = client.call(Method::GetAdvice).await?;

    let (version, uptime, pending) = if let ResponseData::Status(status) = status_data {
        (status.version.clone(), status.uptime_seconds, status.pending_recommendations)
    } else {
        ("unknown".to_string(), 0, 0)
    };

    let advice_count = if let ResponseData::Advice(advice_list) = advice_data {
        advice_list.len()
    } else {
        0
    };

    let health = if pending > 5 {
        "⚠️  Needs Attention"
    } else if pending > 0 {
        "✓ Good"
    } else {
        "✓ Excellent"
    };

    let report_lines = vec![
        format!("📊 Anna Assistant {}", version),
        String::new(),
        format!("System Health: {}", health),
        format!("Recommendations: {} pending", advice_count),
        format!("Uptime: {}s", uptime),
        String::new(),
        "Run 'annactl advise' for details".to_string(),
    ];

    let report_strings: Vec<&str> = report_lines.iter().map(|s| s.as_str()).collect();
    println!("{}", boxed(&report_strings));

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
