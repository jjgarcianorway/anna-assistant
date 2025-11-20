//! Health command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

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
