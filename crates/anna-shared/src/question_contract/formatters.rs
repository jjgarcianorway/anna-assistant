//! Formatting utilities for answer values - v0.0.437.
//!
//! Helper functions for formatting data with units.

/// Format bytes to human readable.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in seconds to human readable.
pub fn format_duration(seconds: f64) -> String {
    if seconds >= 60.0 {
        let mins = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.1}s", mins as u64, secs)
    } else {
        format!("{:.2}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(15.5), "15.50s");
        assert_eq!(format_duration(75.3), "1m 15.3s");
    }
}
