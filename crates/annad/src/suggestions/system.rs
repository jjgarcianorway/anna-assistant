//! System-level suggestion checks.

use chrono::Utc;
use super::types::{Suggestion, SuggestionPriority};

/// Check if pacman cache is large
pub async fn check_pacman_cache() -> Option<Suggestion> {
    let output = std::process::Command::new("du")
        .args(["-sh", "/var/cache/pacman/pkg"])
        .output()
        .ok()?;

    let size_str = String::from_utf8_lossy(&output.stdout);
    let size = size_str.split_whitespace().next()?;

    // Parse size (e.g., "4.5G")
    let size_value: f32 = size.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.')
        .parse()
        .ok()?;

    let size_unit = size.chars().rev().next()?;

    let size_gb = match size_unit {
        'G' => size_value,
        'M' => size_value / 1024.0,
        'K' => size_value / (1024.0 * 1024.0),
        _ => return None,
    };

    if size_gb > 3.0 {
        Some(Suggestion {
            id: "pacman-cache-large".to_string(),
            priority: if size_gb > 5.0 {
                SuggestionPriority::High
            } else {
                SuggestionPriority::Medium
            },
            title: format!("Pacman cache is {:.1}GB", size_gb),
            description: format!(
                "Your package cache has grown to {:.1}GB. Cleaning it won't affect installed packages.",
                size_gb
            ),
            reasoning: "Large cache wastes disk space. Keeping last 3 versions is sufficient.".to_string(),
            action: Some("I can clean it for you, keeping the last 3 package versions. Just ask: 'clean pacman cache'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check for orphaned packages
pub async fn check_orphaned_packages() -> Option<Suggestion> {
    let output = std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let orphans = String::from_utf8_lossy(&output.stdout);
    let count = orphans.lines().filter(|l| !l.is_empty()).count();

    if count > 5 {
        // Get total size
        let size_output = std::process::Command::new("sh")
            .args(["-c", "pacman -Qdtq | xargs -r pacman -Qi | grep 'Installed Size' | awk '{sum+=$4} END {print sum}'"])
            .output()
            .ok()?;

        let size_kb: f32 = String::from_utf8_lossy(&size_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0.0);

        let size_mb = size_kb / 1024.0;

        Some(Suggestion {
            id: "orphaned-packages".to_string(),
            priority: if count > 20 {
                SuggestionPriority::Medium
            } else {
                SuggestionPriority::Low
            },
            title: format!("{} orphaned packages found", count),
            description: format!(
                "You have {} packages that were installed as dependencies but are no longer needed ({:.0}MB).",
                count, size_mb
            ),
            reasoning: "Orphaned packages waste disk space and can accumulate over time.".to_string(),
            action: Some("I can remove them safely. Just ask: 'remove orphaned packages'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check disk usage trends
pub async fn check_disk_trends() -> Option<Suggestion> {
    use crate::briefing::get_disk_usage_percentage;

    let current_usage = get_disk_usage_percentage();

    if current_usage > 85.0 {
        Some(Suggestion {
            id: "disk-usage-high".to_string(),
            priority: if current_usage > 95.0 {
                SuggestionPriority::Critical
            } else {
                SuggestionPriority::High
            },
            title: format!("Disk usage at {:.0}%", current_usage),
            description: "Your disk is filling up. This can cause system instability and prevent updates.".to_string(),
            reasoning: "Systems slow down significantly when disk exceeds 90% full.".to_string(),
            action: Some("I can help you find what's using space. Ask: 'what's using my disk space?'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if Telegram is configured
pub async fn check_telegram_setup() -> Option<Suggestion> {
    let telegram_config = std::path::Path::new("/etc/anna/telegram.env");

    if telegram_config.exists() {
        return None; // Already configured
    }

    Some(Suggestion {
        id: "telegram-not-configured".to_string(),
        priority: SuggestionPriority::Low,
        title: "Enable remote access via Telegram".to_string(),
        description: "You can control Anna from your phone and receive morning briefings with system health charts.".to_string(),
        reasoning: "Telegram provides convenient mobile access and proactive notifications.".to_string(),
        action: Some("Just ask: 'setup telegram bot'".to_string()),
        created_at: Utc::now().to_rfc3339(),
        shown_count: 0,
        dismissed: false,
    })
}
