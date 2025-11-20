//! History command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

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
