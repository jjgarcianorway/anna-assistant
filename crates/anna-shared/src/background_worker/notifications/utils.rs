//! Notification utility functions.

use std::time::SystemTime;

/// Parse time string "HH:MM" to (hour, minute)
pub(super) fn parse_time(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h = parts[0].parse().ok()?;
    let m = parts[1].parse().ok()?;
    Some((h, m))
}

/// Minimal chrono-like time utilities
pub(super) mod chrono_lite {
    use std::time::SystemTime;

    pub fn current_hour_minute() -> (u32, u32) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Convert to local time (approximate - assumes UTC for simplicity)
        let secs_today = now % 86400;
        let hour = (secs_today / 3600) as u32;
        let minute = ((secs_today % 3600) / 60) as u32;
        (hour, minute)
    }
}

/// Get current unix timestamp
pub(super) fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiet_hours_parsing() {
        assert_eq!(parse_time("22:00"), Some((22, 0)));
        assert_eq!(parse_time("08:30"), Some((8, 30)));
        assert_eq!(parse_time("invalid"), None);
    }
}
