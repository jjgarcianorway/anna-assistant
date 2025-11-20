//! Report command - generate system health report

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, Level, Priority};
use anyhow::Result;

use crate::rpc_client::RpcClient;
use super::utils::check_and_notify_updates;

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

    // Overall assessment - use Priority (not RiskLevel)
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
