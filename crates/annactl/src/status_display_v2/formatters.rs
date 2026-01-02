//! Formatting utilities for status display.

use chrono::{DateTime, Local, TimeZone, Utc};

/// Format uptime seconds into a human-readable string
pub fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

/// Format optional DateTime<Utc> to local time string
pub fn format_optional_dt(dt: Option<&DateTime<Utc>>) -> String {
    dt.map(|d| {
        let local: DateTime<Local> = d.with_timezone(&Local);
        local.format("%Y-%m-%d %H:%M:%S").to_string()
    })
    .unwrap_or_else(|| "-".to_string())
}

/// Format optional timestamp (u64) to local time string
pub fn format_optional_ts(ts: Option<u64>) -> String {
    ts.and_then(|t| Utc.timestamp_opt(t as i64, 0).single())
        .map(|d| {
            let local: DateTime<Local> = d.with_timezone(&Local);
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}
