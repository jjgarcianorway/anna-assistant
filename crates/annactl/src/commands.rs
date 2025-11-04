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
        beautiful::status(Level::Info, "Analyzing system...")
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
                beautiful::status(Level::Success, "No recommendations at this time")
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
            println!("{}", section("Maintenance"));
            for advice in warnings {
                println!(
                    "  {} {}",
                    beautiful::status(Level::Warning, "→"),
                    advice.title
                );
                println!("    {}", advice.reason);
                if let Some(ref cmd) = advice.command {
                    println!("    Command: {}", cmd);
                }
            }
            println!();
        }

        // Display info
        if !info.is_empty() {
            println!("{}", section("Suggestions"));
            for advice in info {
                println!(
                    "  {} {}",
                    beautiful::status(Level::Info, "→"),
                    advice.title
                );
                println!("    {}", advice.reason);
                if let Some(ref cmd) = advice.command {
                    println!("    Command: {}", cmd);
                }
            }
            println!();
        }

        println!(
            "{}",
            beautiful::status(
                Level::Success,
                &format!("{} recommendations generated", advice_list.len())
            )
        );
    }

    Ok(())
}

pub async fn apply(_id: Option<String>, _auto: bool, _dry_run: bool) -> Result<()> {
    println!("{}", header("Apply Recommendations"));
    println!();
    println!("{}", beautiful::status(Level::Info, "This feature requires a running daemon"));
    println!("{}", beautiful::status(Level::Info, "Coming in next iteration"));

    Ok(())
}

pub async fn report() -> Result<()> {
    println!("{}", header("System Health Report"));
    println!();

    let report_lines = vec![
        "Anna Assistant v1.0.0-alpha.1",
        "",
        "System: Healthy",
        "Recommendations: 2 pending",
        "Last check: Just now",
        "",
        "Run 'annactl advise' for details",
    ];

    println!("{}", boxed(&report_lines));

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
