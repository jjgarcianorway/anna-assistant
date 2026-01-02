//! Probe data extractors for fallback generation (v0.0.433).

/// Extract memory info from /proc/meminfo.
pub(crate) fn extract_memory_info(raw: &str) -> Option<String> {
    let mut mem_total: Option<u64> = None;
    let mut mem_available: Option<u64> = None;
    let mut mem_free: Option<u64> = None;

    for line in raw.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = extract_kb_value(line);
        } else if line.starts_with("MemAvailable:") {
            mem_available = extract_kb_value(line);
        } else if line.starts_with("MemFree:") {
            mem_free = extract_kb_value(line);
        }
    }

    let total = mem_total?;
    let available = mem_available.or(mem_free)?;

    let total_gib = total as f64 / 1024.0 / 1024.0;
    let available_gib = available as f64 / 1024.0 / 1024.0;
    let percent = (available as f64 / total as f64) * 100.0;

    Some(format!(
        "{:.1} GiB available out of {:.1} GiB total ({:.0}% free)",
        available_gib, total_gib, percent
    ))
}

/// Extract kB value from /proc/meminfo line.
fn extract_kb_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
}

/// Extract disk info from df output.
pub(crate) fn extract_disk_info(raw: &str) -> Option<String> {
    // Look for root filesystem
    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 && parts[5] == "/" {
            let used = parts[2];
            let avail = parts[3];
            let percent = parts[4];
            return Some(format!(
                "Root filesystem: {} used, {} available ({})",
                used, avail, percent
            ));
        }
    }

    // Fallback: just report first line
    let first_data = raw.lines().nth(1)?;
    let parts: Vec<&str> = first_data.split_whitespace().collect();
    if parts.len() >= 5 {
        Some(format!(
            "Disk: {} used, {} available ({})",
            parts.get(2).unwrap_or(&"?"),
            parts.get(3).unwrap_or(&"?"),
            parts.get(4).unwrap_or(&"?")
        ))
    } else {
        None
    }
}

/// Extract boot time from systemd-analyze.
pub(crate) fn extract_boot_time(raw: &str) -> Option<String> {
    // Look for the summary line
    for line in raw.lines() {
        if line.contains("reached after") || line.contains("Startup finished") {
            return Some(line.trim().to_string());
        }
    }

    // Try to extract time values
    if raw.contains("firmware") || raw.contains("kernel") {
        Some(raw.lines().next()?.trim().to_string())
    } else {
        None
    }
}

/// Extract failed services from systemctl --failed.
pub(crate) fn extract_failed_services(raw: &str) -> Option<String> {
    let mut failed_units = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        // Look for .service entries marked as failed
        if line.contains(".service") && line.contains("failed") {
            if let Some(unit) = line.split_whitespace().next() {
                failed_units.push(unit.to_string());
            }
        }
    }

    if failed_units.is_empty() {
        if raw.contains("0 loaded units listed") {
            Some("No failed units found.".to_string())
        } else {
            None
        }
    } else {
        Some(format!(
            "{} failed unit(s): {}",
            failed_units.len(),
            failed_units.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_extraction() {
        let raw = "MemTotal:       32896136 kB\n\
                   MemFree:         8234567 kB\n\
                   MemAvailable:   17825792 kB\n";

        let summary = extract_memory_info(raw).unwrap();
        assert!(summary.contains("GiB"));
        assert!(summary.contains("available"));
    }
}
