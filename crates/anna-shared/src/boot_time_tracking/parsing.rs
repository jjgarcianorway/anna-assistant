//! systemd-analyze Parsing
//!
//! Parses systemd-analyze output to extract boot time information.

use super::types::BootRecord;
use std::collections::HashMap;

/// Parse boot time from systemd-analyze output
pub fn parse_systemd_analyze(output: &str) -> Option<BootRecord> {
    // Example: "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s"
    let mut kernel_time = 0.0;
    let mut userspace_time = 0.0;
    let mut total_time = 0.0;

    for line in output.lines() {
        if line.contains("Startup finished") {
            // Parse kernel time
            if let Some(kernel_match) = line.split("(kernel)").next() {
                if let Some(time_str) = kernel_match.split_whitespace().last() {
                    kernel_time = parse_time_value(time_str);
                }
            }
            // Parse userspace time
            if let Some(after_kernel) = line.split("(kernel)").nth(1) {
                if let Some(userspace_part) = after_kernel.split("(userspace)").next() {
                    if let Some(time_str) = userspace_part.trim().strip_prefix('+').and_then(|s| s.split_whitespace().next()) {
                        userspace_time = parse_time_value(time_str);
                    }
                }
            }
            // Parse total time
            if let Some(total_part) = line.split('=').nth(1) {
                if let Some(time_str) = total_part.split_whitespace().next() {
                    total_time = parse_time_value(time_str);
                }
            }
        }
    }

    if total_time > 0.0 {
        Some(BootRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            boot_time_secs: total_time,
            service_times: HashMap::new(),
            kernel_time_secs: kernel_time,
            userspace_time_secs: userspace_time,
            slow_services: Vec::new(),
        })
    } else {
        None
    }
}

/// Parse time value like "5.678s" or "567ms"
fn parse_time_value(s: &str) -> f64 {
    let s = s.trim();
    // Check ms first since "ms" ends with 's'
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else if let Some(mins) = s.strip_suffix("min") {
        mins.parse::<f64>().unwrap_or(0.0) * 60.0
    } else if let Some(secs) = s.strip_suffix('s') {
        secs.parse().unwrap_or(0.0)
    } else {
        s.parse().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_value() {
        assert!((parse_time_value("5.5s") - 5.5).abs() < 0.01);
        assert!((parse_time_value("500ms") - 0.5).abs() < 0.01);
        assert!((parse_time_value("1min") - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_systemd_analyze() {
        let output = "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s";
        let record = parse_systemd_analyze(output).unwrap();

        assert!((record.boot_time_secs - 8.023).abs() < 0.01);
        assert!((record.kernel_time_secs - 2.345).abs() < 0.01);
        assert!((record.userspace_time_secs - 5.678).abs() < 0.01);
    }
}
