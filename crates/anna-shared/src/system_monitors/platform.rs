//! Platform-specific system information gathering.

/// Get disk space information for a given path
/// Returns (total bytes, available bytes)
pub fn get_disk_space(path: &str) -> (u64, u64) {
    // Read from /proc or use statvfs
    if let Ok(output) = std::process::Command::new("df")
        .args(["--output=size,avail", "-B1", path])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 2 {
                    let total = parts[0].parse().unwrap_or(0);
                    let avail = parts[1].parse().unwrap_or(0);
                    return (total, avail);
                }
            }
        }
    }
    (0, 0)
}

/// Get memory information from /proc/meminfo
/// Returns (total KB, available KB)
pub fn get_memory_info() -> (u64, u64) {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut available: u64 = 0;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_meminfo_value(line);
            } else if line.starts_with("MemAvailable:") {
                available = parse_meminfo_value(line);
            }
        }
        return (total, available);
    }
    (0, 0)
}

/// Parse a value from meminfo line
fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Get swap information from /proc/meminfo
/// Returns (total KB, used KB)
pub fn get_swap_info() -> (u64, u64) {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut free: u64 = 0;

        for line in content.lines() {
            if line.starts_with("SwapTotal:") {
                total = parse_meminfo_value(line);
            } else if line.starts_with("SwapFree:") {
                free = parse_meminfo_value(line);
            }
        }
        return (total, total - free);
    }
    (0, 0)
}

/// Get system load average
/// Returns load * 100 (for precision as u64)
pub fn get_load_average() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(load) = first.parse::<f64>() {
                return (load * 100.0) as u64;
            }
        }
    }
    0
}

/// Get number of CPU cores
pub fn get_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Count the number of failed systemd services
pub fn count_failed_services() -> u64 {
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain"])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            return text.lines().count() as u64;
        }
    }
    0
}

/// Check if a specific service is in failed state
pub fn is_service_failed(service: &str) -> bool {
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["is-failed", "--quiet", service])
        .output()
    {
        return output.status.code() == Some(0);
    }
    false
}
