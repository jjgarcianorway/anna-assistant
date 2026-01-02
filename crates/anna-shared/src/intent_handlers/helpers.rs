//! Helper functions for parsing probe data.

/// Extract a value from meminfo output (e.g., MemTotal, MemAvailable)
pub fn extract_meminfo_value(meminfo: &str, key: &str) -> Option<u64> {
    for line in meminfo.lines() {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
    }
    None
}

/// Extract total boot time from systemd-analyze output
pub fn extract_boot_total(boot_output: &str) -> Option<f64> {
    // Look for "= Xs" or "= X.Ys" pattern
    for line in boot_output.lines() {
        if line.contains("=") {
            // Find the total time after =
            if let Some(after_eq) = line.split('=').last() {
                let trimmed = after_eq.trim();
                // Extract number before 's'
                let num_str = trimmed.trim_end_matches('s').trim();
                if let Ok(secs) = num_str.parse::<f64>() {
                    return Some(secs);
                }
            }
        }
    }
    None
}
