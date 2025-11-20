//! Shared utility functions for commands

use anyhow::Result;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Check for updates and show notification banner (non-spammy, once per day)
pub async fn check_and_notify_updates() {
    // Cache file to track last check
    let cache_file = PathBuf::from("/tmp/anna-update-check");

    // Check if we already checked today
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(last_check) = content.trim().parse::<u64>() {
            // If checked within last 24 hours, skip
            if now - last_check < 86400 {
                return;
            }
        }
    }

    // IMMEDIATELY update cache to prevent spam on failures
    let _ = std::fs::write(&cache_file, now.to_string());

    // Check for updates (silently fail if network issue)
    if let Ok(update_info) = anna_common::updater::check_for_updates().await {
        if update_info.is_update_available {
            // Show update banner
            println!();
            println!("\x1b[38;5;226m╭─────────────────────────────────────────────────────────────╮\x1b[0m");
            println!("\x1b[38;5;226m│\x1b[0m  \x1b[1m📦 Update Available\x1b[0m: {} → {}  \x1b[38;5;226m│\x1b[0m",
                update_info.current_version, update_info.latest_version);
            println!("\x1b[38;5;226m│\x1b[0m  Run \x1b[38;5;159mannactl update --install\x1b[0m to upgrade                 \x1b[38;5;226m│\x1b[0m");
            println!("\x1b[38;5;226m╰─────────────────────────────────────────────────────────────╯\x1b[0m");
        }
    }
    // If check fails, we already updated cache, so won't spam
}

/// Wrap text at specified width with indentation
pub fn wrap_text(text: &str, width: usize, indent: &str) -> String {
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

/// Parse number ranges like "1,3,5-7" into a vector of indices
pub fn parse_number_ranges(input: &str) -> Result<Vec<usize>> {
    let mut result = Vec::new();

    for part in input.split(',') {
        let part = part.trim();

        if part.contains('-') {
            // Range like "5-7"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                anyhow::bail!("Invalid range format: {}", part);
            }

            let start: usize = range_parts[0].trim().parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", range_parts[0]))?;
            let end: usize = range_parts[1].trim().parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", range_parts[1]))?;

            if start > end {
                anyhow::bail!("Invalid range: {} (start > end)", part);
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

    Ok(result)
}
