//! Formatting helpers for UI output (v0.0.213).

use super::symbols;

/// Format a progress bar
pub fn progress_bar(progress: f32, width: usize) -> String {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}]",
        symbols::PROGRESS_FULL.repeat(filled),
        symbols::PROGRESS_EMPTY.repeat(empty)
    )
}

/// Format bytes as human readable
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human readable
pub fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{:02}:{:02}:{:02}", hours, mins, seconds % 60)
    } else if seconds >= 60 {
        let mins = seconds / 60;
        format!("{:02}:{:02}", mins, seconds % 60)
    } else {
        format!("00:00:{:02}", seconds)
    }
}
