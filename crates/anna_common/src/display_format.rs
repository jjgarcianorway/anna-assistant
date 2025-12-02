//! Display Formatting Utilities v5.2.4
//!
//! Centralized formatting rules for all user-facing output.
//! All output must be ASCII only - no emojis, no Unicode decorations.
//!
//! Rules:
//! - Duration: <1s -> ms, 1s-1m -> X.Xs, 1m-1h -> Xm Ys, >1h -> Xh Ym
//! - Counts: Use K/M suffixes only when clearly helpful
//! - Times: Human-readable relative times ("3m ago", "2h ago")

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Format a duration in milliseconds to human-readable form
/// Rules:
/// - Under 1 second: show in ms (e.g. 120ms)
/// - From 1 second to 1 minute: show in seconds with one decimal (e.g. 4.2s)
/// - From 1 minute to 1 hour: show in minutes and seconds (e.g. 3m 15s)
/// - Over 1 hour: show in Hh Mm (e.g. 1h 34m)
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        let secs = ms as f64 / 1000.0;
        format!("{:.1}s", secs)
    } else if ms < 3_600_000 {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else {
        let hours = ms / 3_600_000;
        let mins = (ms % 3_600_000) / 60_000;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    }
}

/// Format a duration from seconds
pub fn format_duration_secs(secs: u64) -> String {
    format_duration_ms(secs * 1000)
}

/// Format a Duration struct
pub fn format_duration(d: Duration) -> String {
    format_duration_ms(d.as_millis() as u64)
}

/// Format a relative time ("3m ago", "2h ago", "just now")
pub fn format_time_ago(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if timestamp > now {
        return "just now".to_string();
    }

    let diff = now - timestamp;

    if diff < 60 {
        if diff <= 5 {
            "just now".to_string()
        } else {
            format!("{}s ago", diff)
        }
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{}m ago", mins)
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{}h ago", hours)
    } else {
        let days = diff / 86400;
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    }
}

/// Format a count with K/M suffix when appropriate
/// Only use suffixes for large numbers where exact count is not actionable
pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else if n >= 1_000 {
        // For 1K-10K, show exact number with comma
        format_with_commas(n)
    } else {
        format!("{}", n)
    }
}

/// Format a number with comma separators (e.g., 1,234)
pub fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a percentage (0-100)
pub fn format_percent(value: f64) -> String {
    if value >= 99.5 {
        "100%".to_string()
    } else if value < 1.0 && value > 0.0 {
        "<1%".to_string()
    } else {
        format!("{:.0}%", value)
    }
}

/// Format a ratio as percentage (e.g., 80/100 -> "80%")
pub fn format_ratio_percent(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        "n/a".to_string()
    } else {
        let pct = (numerator as f64 / denominator as f64) * 100.0;
        format_percent(pct)
    }
}

/// Format bytes to human-readable (KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Format an ETA based on progress and rate
/// Returns "n/a" if cannot estimate, otherwise "~Xm" or "~Xs"
pub fn format_eta(remaining: u64, rate_per_sec: f64) -> String {
    if rate_per_sec <= 0.0 || remaining == 0 {
        return "n/a".to_string();
    }

    let secs = (remaining as f64 / rate_per_sec) as u64;
    if secs < 60 {
        format!("~{}s", secs)
    } else if secs < 3600 {
        format!("~{}m", secs / 60)
    } else {
        format!("~{}h", secs / 3600)
    }
}

/// v7.29.0: DEPRECATED - prefer text_wrap functions for zero truncation
/// Truncate a string to max length, adding "..." if truncated
#[deprecated(since = "7.29.0", note = "use text_wrap module for zero truncation output")]
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

/// v7.29.0: Clip a string to max length without adding ellipsis
/// Use when display width is limited but full content is elsewhere
pub fn clip_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect()
    }
}

/// Format a timestamp as "YYYY-MM-DD HH:MM"
pub fn format_timestamp(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Format a timestamp as "YYYY-MM-DD HH:MM:SS"
pub fn format_timestamp_full(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Check if a string contains only ASCII characters
pub fn is_ascii_only(s: &str) -> bool {
    s.is_ascii()
}

/// Strip non-ASCII characters from a string
pub fn strip_non_ascii(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(50), "50ms");
        assert_eq!(format_duration_ms(999), "999ms");
        assert_eq!(format_duration_ms(1000), "1.0s");
        assert_eq!(format_duration_ms(4200), "4.2s");
        assert_eq!(format_duration_ms(59900), "59.9s");
        assert_eq!(format_duration_ms(60000), "1m");
        assert_eq!(format_duration_ms(195000), "3m 15s");
        assert_eq!(format_duration_ms(3600000), "1h");
        assert_eq!(format_duration_ms(5640000), "1h 34m");
    }

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1234), "1,234");
        assert_eq!(format_count(9999), "9,999");
        assert_eq!(format_count(10000), "10.0K");
        assert_eq!(format_count(90900), "90.9K");
        assert_eq!(format_count(1500000), "1.5M");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.0), "0%");
        assert_eq!(format_percent(0.5), "<1%");
        assert_eq!(format_percent(50.0), "50%");
        assert_eq!(format_percent(99.4), "99%");
        assert_eq!(format_percent(99.6), "100%");
        assert_eq!(format_percent(100.0), "100%");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.0KB");
        assert_eq!(format_bytes(8400000), "8.0MB");
        assert_eq!(format_bytes(1073741824), "1.0GB");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(0, 10.0), "n/a");
        assert_eq!(format_eta(100, 0.0), "n/a");
        assert_eq!(format_eta(30, 1.0), "~30s");
        assert_eq!(format_eta(180, 1.0), "~3m");
        assert_eq!(format_eta(7200, 1.0), "~2h");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 2), "hi");
    }

    #[test]
    fn test_is_ascii_only() {
        assert!(is_ascii_only("hello world"));
        assert!(is_ascii_only("test 123 !@#"));
        assert!(!is_ascii_only("hello 🌍"));
        assert!(!is_ascii_only("test ✓"));
    }

    #[test]
    fn test_strip_non_ascii() {
        assert_eq!(strip_non_ascii("hello 🌍 world"), "hello  world");
        assert_eq!(strip_non_ascii("test ✓ ok"), "test  ok");
        assert_eq!(strip_non_ascii("pure ascii"), "pure ascii");
    }
}
